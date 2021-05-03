use lingua::Language;
use lingua::LanguageDetectorBuilder;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use warp::Filter;

#[derive(Deserialize)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub languages: String,
    pub minimum_relative_distance: Option<f64>,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::new();

        cfg.set_default("listen_addr", "0.0.0.0:8080")?;
        cfg.merge(config::Environment::new())?;
        Ok(cfg.try_into()?)
    }

    pub fn parse_languages(&self) -> Result<Vec<Language>, serde_json::Error> {
        // Somewhat hacky way to parse
        let s = self
            .languages
            .to_uppercase()
            .split(",")
            .map(|s| format!("\"{}\"", s.trim()))
            .collect::<Vec<_>>()
            .join(",");

        serde_json::from_str(&format!("[{}]", s))
    }
}

#[derive(Debug)]
struct NotUtf8;
impl warp::reject::Reject for NotUtf8 {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .init();

    let cfg = Config::from_env().expect("Failed to create config");

    // let languages = vec![English, French, German, Spanish, Turkish];
    let languages = cfg.parse_languages()?;

    tracing::info!("Loading languages: {:?}", languages);
    tracing::info!("Using minimum relative distance of {}", cfg.minimum_relative_distance.unwrap_or(0.0));

    let detector = Arc::new(
        LanguageDetectorBuilder::from_languages(&languages)
            .with_minimum_relative_distance(cfg.minimum_relative_distance.unwrap_or(0.0))
            .with_preloaded_language_models()
            .build(),
    );

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
