use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SentimentType {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    pub prompt_id: Uuid,
    pub sentiment: SentimentType,
    pub probability: f64,
}
