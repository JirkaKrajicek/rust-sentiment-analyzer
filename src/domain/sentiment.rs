use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::SentimentType"]
#[DbValueStyle = "PascalCase"]
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
