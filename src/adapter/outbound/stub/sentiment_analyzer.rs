use crate::{
    application::port::sentiment_analyzer::SentimentAnalyzer, domain::sentiment::SentimentType,
};

pub struct StubAnalyzer;

#[async_trait::async_trait]
impl SentimentAnalyzer for StubAnalyzer {
    async fn analyze(&self, _text: &str) -> Result<(SentimentType, f64), anyhow::Error> {
        Ok((SentimentType::Positive, 0.99))
    }
}
