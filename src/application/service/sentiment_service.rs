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
    _repo: Arc<dyn ProjectRepository>,
}

impl SentimentService {
    pub fn new(analyzer: Arc<dyn SentimentAnalyzer>, _repo: Arc<dyn ProjectRepository>) -> Self {
        Self { analyzer, _repo }
    }

    pub async fn predict(&self, text: String) -> Result<Sentiment, anyhow::Error> {
        let (sentiment_type, probability) = self.analyzer.analyze(&text).await?;
        let result = Sentiment {
            prompt_id: Uuid::new_v4(),
            sentiment: sentiment_type,
            probability,
        };
        Ok(result)
    }
}
