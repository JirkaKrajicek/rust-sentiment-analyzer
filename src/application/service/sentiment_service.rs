use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::port::{
        project_repository::ProjectRepository, sentiment_analyzer::SentimentAnalyzer,
    },
    domain::sentiment::Sentiment,
};

pub struct SentimentService {
    analyzer: Arc<dyn SentimentAnalyzer>,
    repo: Arc<dyn ProjectRepository>,
}

impl SentimentService {
    pub fn new(analyzer: Arc<dyn SentimentAnalyzer>, repo: Arc<dyn ProjectRepository>) -> Self {
        Self { analyzer, repo }
    }

    pub async fn predict(&self, text: String) -> Result<Sentiment, anyhow::Error> {
        let (sentiment_type, probability) = self.analyzer.analyze(&text).await?;
        self.repo.insert(&text, sentiment_type, probability).await
    }

    pub async fn get(&self, prompt_id: Uuid) -> Result<Option<Sentiment>, anyhow::Error> {
        self.repo.get_sentiment(prompt_id).await
    }

    pub async fn list(&self) -> Result<Vec<Sentiment>, anyhow::Error> {
        self.repo.list().await
    }

    pub async fn delete(&self, prompt_id: Uuid) -> Result<bool, anyhow::Error> {
        self.repo.delete(prompt_id).await
    }

    pub async fn is_ready(&self) -> Result<(), anyhow::Error> {
        self.analyzer.is_ready().await
    }
}
