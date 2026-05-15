use uuid::Uuid;

pub enum SentimentType {
    Positive,
    Negative,
    Neutral,
}

pub struct Sentiment {
    pub prompt_id: Uuid,
    pub sentiment: SentimentType,
    pub probability: f64,
}
