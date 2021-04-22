use lingua::Language::{English, French, German, Spanish, Turkish};
use lingua::LanguageDetectorBuilder;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use warp::Filter;

#[derive(Deserialize)]
pub struct Config {
    pub listen_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        Ok(cfg.try_into()?)
    }
}

#[derive(Debug)]
struct NotUtf8;
impl warp::reject::Reject for NotUtf8 {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = Config::from_env().expect("Failed to create config");

    let languages = vec![English, French, German, Spanish, Turkish];

    tracing::info!("Loading languages: {:?}", languages);
    let detector = Arc::new(LanguageDetectorBuilder::from_languages(&languages).build());
    tracing::info!("Finished loading language detector");

    let log = warp::log("language_api");

    let string_body = warp::body::bytes().and_then(|body: bytes::Bytes| async move {
        std::str::from_utf8(&body)
            .map(String::from)
            .map_err(|_e| warp::reject::custom(NotUtf8))
    });

    let detector_clone = detector.clone();
    let detect_language = warp::post()
        .and(warp::path("detect"))
        // Only accept bodies smaller than 16kb...
        .and(warp::body::content_length_limit(1024 * 16))
        .and(string_body)
        .map(move |text: String| {
            let language = detector_clone.detect_language_of(text);

            warp::reply::json(&language)
        })
        .with(log);

    let confidence_language = warp::post()
        .and(warp::path("confidence"))
        // Only accept bodies smaller than 16kb...
        .and(warp::body::content_length_limit(1024 * 16))
        .and(string_body)
        .map(move |text: String| {
            let confidences = detector.compute_language_confidence_values(text);

            warp::reply::json(&confidences)
        })
        .with(log);

    tracing::info!("Listening on {}", cfg.listen_addr);

    let routes = detect_language.or(confidence_language);

    warp::serve(routes).run(cfg.listen_addr).await;

    Ok(())
}
