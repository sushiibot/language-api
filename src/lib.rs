use reqwest::Client;
use lingua::Language;

mod error;
mod model;
use error::Error;
use model::{DetectQuery, DetectResponse, ConfidenceResponse};

pub struct LanguageApiClient {
    client: Client,
    endpoint: String,
}

impl LanguageApiClient {
    pub fn new(client: Client, endpoint: &str) -> Self {
        Self {
            client,
            endpoint: endpoint.trim_end_matches("/").to_string(),
        }
    }

    async fn detect_language(&self, text: &str) -> Result<Option<Language>, Error> {
        self.client.post(format!("{}/detect", self.endpoint))
            .json(&DetectQuery {
                text: text.to_string(),
            })
            .send()
            .await?
            .json::<DetectResponse>()
            .await
            .map(|l| l.0)
            .map_err(From::from)
    }

    async fn detect_language_confidences(&self, text: &str) -> Result<Vec<(Language, f64)>, Error> {
        self.client.post(format!("{}/confidence", self.endpoint))
            .json(&DetectQuery {
                text: text.to_string(),
            })
            .send()
            .await?
            .json::<ConfidenceResponse>()
            .await
            .map(|l| l.0)
            .map_err(From::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingua::Language;

    #[tokio::test]
    async fn detects_language() {
        let client = Client::new();
        let language_client = LanguageApiClient::new(client, "http://localhost:8080");

        let detected_language = language_client.detect_language("bisous et l'étreinte")
            .await
            .expect("Detect language");

        assert_eq!(detected_language, Some(Language::French));
    }

    #[tokio::test]
    async fn detects_language_confidences() {
        let client = Client::new();
        let language_client = LanguageApiClient::new(client, "http://localhost:8080");

        let detected_language = language_client.detect_language_confidences("bisous et l'étreinte")
            .await
            .expect("Detect language confidence");

        assert!(detected_language.iter().any(|l| l.0 == Language::French));
    }
}
