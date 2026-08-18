use std::{collections::HashMap, sync::Mutex};

use uuid::Uuid;

use crate::{
    application::port::project_repository::ProjectRepository,
    domain::sentiment::{DocumentDetails, Sentiment, SentimentType},
};

#[derive(Default)]
pub struct StubRepository {
    results: Mutex<HashMap<Uuid, Sentiment>>,
}

#[async_trait::async_trait]
impl ProjectRepository for StubRepository {
    async fn insert(
        &self,
        _input_text: &str,
        sentiment: SentimentType,
        probability: f64,
    ) -> Result<Sentiment, anyhow::Error> {
        let result = Sentiment {
            prompt_id: Uuid::new_v4(),
            sentiment,
            probability,
            document_details: None,
        };
        self.results
            .lock()
            .map_err(|_| anyhow::anyhow!("Stub repository lock was poisoned"))?
            .insert(result.prompt_id, result.clone());
        Ok(result)
    }

    async fn insert_document(
        &self,
        input_text: &str,
        sentiment: SentimentType,
        probability: f64,
        details: DocumentDetails,
    ) -> Result<Sentiment, anyhow::Error> {
        let mut result = self.insert(input_text, sentiment, probability).await?;
        result.document_details = Some(details);
        self.results
            .lock()
            .map_err(|_| anyhow::anyhow!("Stub repository lock was poisoned"))?
            .insert(result.prompt_id, result.clone());
        Ok(result)
    }

    async fn get_sentiment(&self, prompt_id: Uuid) -> Result<Option<Sentiment>, anyhow::Error> {
        Ok(self
            .results
            .lock()
            .map_err(|_| anyhow::anyhow!("Stub repository lock was poisoned"))?
            .get(&prompt_id)
            .cloned())
    }

    async fn delete(&self, prompt_id: Uuid) -> Result<bool, anyhow::Error> {
        Ok(self
            .results
            .lock()
            .map_err(|_| anyhow::anyhow!("Stub repository lock was poisoned"))?
            .remove(&prompt_id)
            .is_some())
    }

    async fn list(&self) -> Result<Vec<Sentiment>, anyhow::Error> {
        Ok(self
            .results
            .lock()
            .map_err(|_| anyhow::anyhow!("Stub repository lock was poisoned"))?
            .values()
            .cloned()
            .collect())
    }
}
