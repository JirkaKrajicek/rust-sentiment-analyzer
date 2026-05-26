use std::sync::Arc;

use axum::{Router, routing::post};
use sentiment_analyzer::{
    adapter::{
        inbound::rest::handler::predict_handler,
        outbound::{onnx::onnx_analyzer::OnnxAnalyzer, postgres::postgres_store::PostgresStore},
    },
    app_state::AppState,
    application::service::sentiment_service::SentimentService,
    config::DbConfig,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_config = DbConfig::from_env();
    let repo = Arc::new(PostgresStore::new(&db_config)?);
    let analyzer = Arc::new(OnnxAnalyzer::new(
        std::path::Path::new("models/model.onnx"),
        std::path::Path::new("models/tokenizer.json"),
    )?);
    let service = Arc::new(SentimentService::new(analyzer, repo));
    let state = AppState { service };

    let app = Router::new()
        .route("/predict", post(predict_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
