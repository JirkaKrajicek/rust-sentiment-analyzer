use std::{sync::Arc, time::Duration};

use tokio::sync::Semaphore;

use crate::application::service::sentiment_service::SentimentService;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<SentimentService>,
    pub document_max_characters: usize,
    pub document_extraction: Arc<Semaphore>,
    pub document_extraction_queue_timeout: Duration,
    pub document_extraction_timeout: Duration,
}
