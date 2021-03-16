use lingua::Language::{English, French, German, Spanish, Turkish};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use std::net::SocketAddr;
use std::env;
use tonic::{transport::Server, Request, Response, Status};
use tracing_subscriber::EnvFilter;

use language::language_service_server::{LanguageService, LanguageServiceServer};
use language::{LanguageReply, LanguageConfidenceReply, LanguageConfidence, LanguageRequest};

pub mod language {
    tonic::include_proto!("language");
}

pub struct Service {
    detector: LanguageDetector,
}

impl Service {
    pub fn new(detector: LanguageDetector) -> Self {
        Self { detector }
    }
}

#[tonic::async_trait]
impl LanguageService for Service {
    async fn detect_language(
        &self,
        request: Request<LanguageRequest>,
    ) -> Result<Response<LanguageReply>, Status> {
        tracing::info!("Got a request from {:?}", request.remote_addr());

        let detected_language: Option<Language> =
            self.detector.detect_language_of(request.into_inner().text);

        let reply = LanguageReply {
            language: format!("{:?}", detected_language),
        };
        Ok(Response::new(reply))
    }

    async fn language_confidence(
        &self,
        request: Request<LanguageRequest>,
    ) -> Result<Response<LanguageConfidenceReply>, Status> {
        tracing::info!("Got a request from {:?}", request.remote_addr());

        let languages: Vec<LanguageConfidence> = self.detector
            .compute_language_confidence_values(request.into_inner().text)
            .into_iter()
            .map(|x| {
                LanguageConfidence {
                    language: format!("{:?}", x.0),
                    confidence: x.1,
                }
            })
            .collect();

        let reply = LanguageConfidenceReply {
            languages,
        };

        Ok(Response::new(reply))
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
    let detector: LanguageDetector = LanguageDetectorBuilder::from_languages(&languages).build();
    tracing::info!("Finished loading language detector");

    let service = Service::new(detector);

    tracing::info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(LanguageServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
