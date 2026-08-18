use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::SentimentType"]
#[DbValueStyle = "PascalCase"]
pub enum SentimentType {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    pub prompt_id: Uuid,
    pub sentiment: SentimentType,
    pub probability: f64,
    pub document_details: Option<DocumentDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSentiment {
    pub index: usize,
    pub sentiment: SentimentType,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDetails {
    pub aggregation: String,
    pub chunks: Vec<ChunkSentiment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentSentiment {
    pub result: Sentiment,
    pub aggregation: &'static str,
    pub chunks: Vec<ChunkSentiment>,
}
