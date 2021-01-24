use futures::future::{self, Either, TryFutureExt};
use http::version::Version;
use hyper::{service::make_service_fn, Server};
use lingua::Language::{English, French, German, Spanish, Turkish};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tonic::{Request, Response as TonicResponse, Status};
use tower::Service;
use tracing_subscriber::EnvFilter;
use warp::{
    http::{Response, StatusCode},
    reject::Rejection,
    Filter,
};

use language::language_service_server::{LanguageService, LanguageServiceServer};
use language::{LanguageReply, LanguageRequest};

pub mod language {
    tonic::include_proto!("language");
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct TonicService {
    detector: Arc<LanguageDetector>,
}

impl TonicService {
    pub fn new(detector: Arc<LanguageDetector>) -> Self {
        Self { detector }
    }
}

#[tonic::async_trait]
impl LanguageService for TonicService {
    async fn detect_language(
        &self,
        request: Request<LanguageRequest>,
    ) -> Result<TonicResponse<LanguageReply>, Status> {
        tracing::info!("Got a request from {:?}", request.remote_addr());

        let detected_language: Option<Language> =
            self.detector.detect_language_of(request.into_inner().text);

        let reply = LanguageReply {
            language: format!("{:?}", detected_language),
        };
        Ok(TonicResponse::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let addr: SocketAddr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse::<SocketAddr>()
        .expect("Failed to parse LISTEN_ADDR as a SocketAddr");

    let languages = vec![English, French, German, Spanish, Turkish];

    tracing::info!("Loading languages: {:?}", languages);
    let detector = Arc::new(LanguageDetectorBuilder::from_languages(&languages).build());
    tracing::info!("Finished loading language detector");

    // Create gRPC service
    let service = TonicService::new(detector.clone());
    let tonic = LanguageServiceServer::new(service);

    // Create Warp HTTP server
    let detect =
        warp::path!("detectLanguage")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(get_text())
            .map(|_, text: String| {
                let detector = detector.clone();
                async move {
                    Response::builder().body(format!("{:?}", detector.detect_language_of(text)))
                }
            });

    let mut warp = warp::service(detect);

    tracing::info!("GreeterServer listening on {}", addr);

    Server::bind(&addr)
        .serve(make_service_fn(move |_| {
            let mut tonic = tonic.clone();
            future::ok::<_, Infallible>(tower::service_fn(
                move |req: hyper::Request<hyper::Body>| match req.version() {
                    Version::HTTP_11 | Version::HTTP_10 => Either::Left(
                        warp.call(req)
                            .map_ok(|res| res.map(EitherBody::Left))
                            .map_err(Error::from),
                    ),
                    Version::HTTP_2 => Either::Right(
                        tonic
                            .call(req)
                            .map_ok(|res| res.map(EitherBody::Right))
                            .map_err(Error::from),
                    ),
                    _ => unimplemented!(),
                },
            ))
        }))
        .await?;

    Ok(())
}

fn get_text() -> impl Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::query::<HashMap<String, String>>().and_then(|p: HashMap<String, String>| async move {
        if let Some(text) = p.get("text") {
            Ok(text.clone())
        } else {
            Err(warp::reject::custom(MissingText))
        }
    })
}

#[derive(Debug)]
struct MissingText;
impl warp::reject::Reject for MissingText {}

enum EitherBody<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> http_body::Body for EitherBody<A, B>
where
    A: http_body::Body + Send + Unpin,
    B: http_body::Body<Data = A::Data> + Send + Unpin,
    A::Error: Into<Error>,
    B::Error: Into<Error>,
{
    type Data = A::Data;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn is_end_stream(&self) -> bool {
        match self {
            EitherBody::Left(b) => b.is_end_stream(),
            EitherBody::Right(b) => b.is_end_stream(),
        }
    }

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.get_mut() {
            EitherBody::Left(b) => Pin::new(b).poll_data(cx).map(map_option_err),
            EitherBody::Right(b) => Pin::new(b).poll_data(cx).map(map_option_err),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        match self.get_mut() {
            EitherBody::Left(b) => Pin::new(b).poll_trailers(cx).map_err(Into::into),
            EitherBody::Right(b) => Pin::new(b).poll_trailers(cx).map_err(Into::into),
        }
    }
}

fn map_option_err<T, U: Into<Error>>(err: Option<Result<T, U>>) -> Option<Result<T, Error>> {
    err.map(|e| e.map_err(Into::into))
}
