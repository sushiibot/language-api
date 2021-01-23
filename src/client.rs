use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use std::env;
use tracing_subscriber::EnvFilter;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: String = env::args().skip(1).collect::<Vec<_>>().join(" ");

    let addr = env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let mut client = GreeterClient::connect(addr).await?;

    let request = tonic::Request::new(HelloRequest {
        name: args,
    });

    tracing::info!("Request: {:#?}", request);

    let response = client.say_hello(request).await?;

    tracing::info!("Response: {:#?}", response);


    Ok(())
}
