use lingua::Language::{English, French, German, Spanish, Turkish};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use std::net::SocketAddr;
use std::env;
use tonic::{transport::Server, Request, Response, Status};
use tracing_subscriber::EnvFilter;

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

pub struct MyGreeter {
    detector: LanguageDetector,
}

impl MyGreeter {
    pub fn new(detector: LanguageDetector) -> Self {
        Self { detector }
    }
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        tracing::info!("Got a request from {:?}", request.remote_addr());

        let detected_language: Option<Language> =
            self.detector.detect_language_of(request.into_inner().name);

        let reply = hello_world::HelloReply {
            message: format!("{:?}", detected_language),
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

    let greeter = MyGreeter::new(detector);

    tracing::info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
