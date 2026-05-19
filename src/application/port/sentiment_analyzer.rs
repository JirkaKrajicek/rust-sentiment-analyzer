use crate::domain::sentiment::SentimentType;

#[async_trait::async_trait]
pub trait SentimentAnalyzer: Send + Sync {
    async fn analyze(&self, text: &str) -> Result<(SentimentType, f64), anyhow::Error>;
}
