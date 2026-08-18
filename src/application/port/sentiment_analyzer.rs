use crate::domain::sentiment::SentimentType;

#[derive(Debug)]
pub enum InferenceError {
    Overloaded,
    TimedOut,
    WorkerFailed,
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overloaded => formatter.write_str("Inference capacity is unavailable"),
            Self::TimedOut => formatter.write_str("Inference timed out"),
            Self::WorkerFailed => formatter.write_str("Inference worker failed"),
        }
    }
}

impl std::error::Error for InferenceError {}

#[async_trait::async_trait]
pub trait SentimentAnalyzer: Send + Sync {
    async fn analyze(&self, text: &str) -> Result<(SentimentType, f64), anyhow::Error>;
    async fn is_ready(&self) -> Result<(), anyhow::Error>;
}
