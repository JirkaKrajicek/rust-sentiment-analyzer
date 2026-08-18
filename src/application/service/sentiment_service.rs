use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::port::{
        project_repository::ProjectRepository, sentiment_analyzer::SentimentAnalyzer,
    },
    domain::sentiment::{ChunkSentiment, DocumentSentiment, Sentiment, SentimentType},
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

    pub async fn predict_document(&self, text: String) -> Result<DocumentSentiment, anyhow::Error> {
        let chunks = self.analyzer.chunk_text(&text)?;
        if chunks.is_empty() {
            anyhow::bail!("Document did not contain analyzable text");
        }
        let mut results = Vec::with_capacity(chunks.len());
        for (index, chunk) in chunks.iter().enumerate() {
            let (sentiment, probability) = self.analyzer.analyze(chunk).await?;
            results.push(ChunkSentiment { index, sentiment, probability });
        }
        let score = results.iter().map(|chunk| match chunk.sentiment {
            SentimentType::Positive => chunk.probability,
            SentimentType::Negative => -chunk.probability,
        }).sum::<f64>() / results.len() as f64;
        let (sentiment, probability) = if score >= 0.0 {
            (SentimentType::Positive, score)
        } else {
            (SentimentType::Negative, -score)
        };
        let result = self.repo.insert(&text, sentiment, probability).await?;
        Ok(DocumentSentiment { result, aggregation: "mean_signed_confidence", chunks: results })
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
