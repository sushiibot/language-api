use lingua::Language;
use reqwest::Client;

pub mod error;

use error::Error;

#[derive(Clone, Debug)]
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

    pub async fn detect_language(&self, text: &str) -> Result<Option<Language>, Error> {
        self.client
            .post(format!("{}/detect", self.endpoint))
            .body(text.to_string())
            .send()
            .await?
            .json::<Option<Language>>()
            .await
            .map_err(From::from)
    }

    pub async fn detect_language_confidences(
        &self,
        text: &str,
    ) -> Result<Vec<(Language, f64)>, Error> {
        self.client
            .post(format!("{}/confidence", self.endpoint))
            .body(text.to_string())
            .send()
            .await?
            .json::<Vec<(Language, f64)>>()
            .await
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

        let detected_language = language_client
            .detect_language("bisous et l'étreinte")
            .await
            .expect("Detect language");

        assert_eq!(detected_language, Some(Language::French));
    }

    #[tokio::test]
    async fn detects_language_confidences() {
        let client = Client::new();
        let language_client = LanguageApiClient::new(client, "http://localhost:8080");

        let detected_language = language_client
            .detect_language_confidences("bisous et l'étreinte")
            .await
            .expect("Detect language confidence");

        assert!(detected_language.iter().any(|l| l.0 == Language::French));
    }
}
