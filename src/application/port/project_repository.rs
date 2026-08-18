use chrono::{DateTime, Utc};
use domain::sentiment::{DocumentDetails, Sentiment, SentimentType};
use uuid::Uuid;

use crate::domain;

#[async_trait::async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn insert(
        &self,
        input_text: &str,
        sentiment: SentimentType,
        probability: f64,
    ) -> Result<Sentiment, anyhow::Error>;
    async fn insert_document(
        &self,
        input_text: &str,
        sentiment: SentimentType,
        probability: f64,
        details: DocumentDetails,
    ) -> Result<Sentiment, anyhow::Error>;
    async fn get_sentiment(&self, prompt_id: Uuid) -> Result<Option<Sentiment>, anyhow::Error>;
    async fn delete(&self, prompt_id: Uuid) -> Result<bool, anyhow::Error>;
    async fn delete_created_before(&self, cutoff: DateTime<Utc>) -> Result<u64, anyhow::Error>;
    async fn list(&self) -> Result<Vec<Sentiment>, anyhow::Error>;
}
