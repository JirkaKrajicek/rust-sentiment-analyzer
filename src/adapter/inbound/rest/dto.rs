use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PredictRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct PredictResponse {
    pub sentiment: String,
    pub probability: f64,
}
