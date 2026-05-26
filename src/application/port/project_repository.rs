use domain::sentiment::Sentiment;
use uuid::Uuid;

use crate::domain;

#[async_trait::async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn insert(&self, prompt: String) -> Result<(), anyhow::Error>;
    async fn get_sentiment(&self, prompt_id: Uuid) -> Result<Sentiment, anyhow::Error>;
    async fn delete(&self, prompt_id: Uuid) -> Result<(), anyhow::Error>;
    async fn list(&self) -> Result<Vec<Sentiment>, anyhow::Error>;
}
