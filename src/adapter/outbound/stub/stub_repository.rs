use uuid::Uuid;

use crate::{
    application::port::project_repository::ProjectRepository,
    domain::sentiment::{Sentiment, SentimentType},
};

pub struct StubRepository;

#[async_trait::async_trait]
impl ProjectRepository for StubRepository {
    async fn insert(&self, _prompt: String) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn get_sentiment(&self, prompt_id: Uuid) -> Result<Sentiment, anyhow::Error> {
        Ok(Sentiment {
            prompt_id,
            sentiment: SentimentType::Neutral,
            probability: 0.0,
        })
    }

    async fn delete(&self, _prompt_id: Uuid) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Sentiment>, anyhow::Error> {
        Ok(vec![])
    }
}
