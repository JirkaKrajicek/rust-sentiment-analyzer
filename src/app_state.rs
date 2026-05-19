use std::sync::Arc;

use crate::application::service::sentiment_service::SentimentService;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<SentimentService>,
}
