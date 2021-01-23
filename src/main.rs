use tonic::{transport::Server, Request, Response, Status};
use tracing_subscriber::EnvFilter;

use lingua::Language::{English, French, German, Spanish};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};

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

    let addr = "[::1]:50051".parse().unwrap();

    let languages = vec![English, French, German, Spanish];
    let detector: LanguageDetector = LanguageDetectorBuilder::from_languages(&languages).build();

    let greeter = MyGreeter::new(detector);

    tracing::info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
