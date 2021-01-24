use language::language_service_client::LanguageServiceClient;
use language::LanguageRequest;
use std::env;
use tracing_subscriber::EnvFilter;

pub mod language {
    tonic::include_proto!("language");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: String = env::args().skip(1).collect::<Vec<_>>().join(" ");

    let addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let mut client = LanguageServiceClient::connect(addr).await?;

    let request = tonic::Request::new(LanguageRequest { text: args });

    tracing::info!("Request: {:#?}", request);

    let response = client.detect_language(request).await?;

    tracing::info!("Response: {:#?}", response);

    Ok(())
}
