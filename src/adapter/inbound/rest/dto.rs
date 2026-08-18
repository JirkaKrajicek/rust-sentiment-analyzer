use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PredictRequest {
    /// The text to analyze
    pub text: String,
}

#[derive(Debug, ToSchema)]
pub struct DocumentUploadRequest {
    /// A UTF-8 `.txt` or `.docx` file to analyze.
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PredictResponse {
    /// Identifier of the persisted prediction
    pub id: Uuid,
    /// Predicted sentiment label (Positive or Negative)
    pub sentiment: String,
    /// Confidence probability of the prediction
    pub probability: f64,
    /// Present only for results created from a document upload.
    pub document_details: Option<DocumentDetailsResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadinessResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: &'static str,
    pub request_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentPredictResponse {
    pub id: Uuid,
    pub sentiment: String,
    pub probability: f64,
    pub aggregation: String,
    pub chunks: Vec<DocumentChunkResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentDetailsResponse {
    pub aggregation: String,
    pub chunks: Vec<DocumentChunkResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentChunkResponse {
    pub index: usize,
    pub sentiment: String,
    pub probability: f64,
}
