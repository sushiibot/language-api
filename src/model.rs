use lingua::Language;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct DetectQuery {
    pub text: String,
}

#[derive(Deserialize, Serialize)]
pub struct DetectResponse(pub Option<Language>);

#[derive(Deserialize, Serialize)]
pub struct ConfidenceResponse(pub Vec<(Language, f64)>);
