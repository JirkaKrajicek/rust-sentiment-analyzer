use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PredictRequest {
    /// The text to analyze
    pub text: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PredictResponse {
    /// Identifier of the persisted prediction
    pub id: Uuid,
    /// Predicted sentiment label (Positive, Negative, Neutral)
    pub sentiment: String,
    /// Confidence probability of the prediction
    pub probability: f64,
}
