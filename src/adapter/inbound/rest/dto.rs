use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PredictRequest {
    /// The text to analyze
    pub text: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PredictResponse {
    /// Predicted sentiment label (Positive, Negative, Neutral)
    pub sentiment: String,
    /// Confidence probability of the prediction
    pub probability: f64,
}
