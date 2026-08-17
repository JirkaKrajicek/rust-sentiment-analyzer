use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use sentiment_analyzer::{
    adapter::{
        inbound::rest::handler::{
            delete_sentiment_handler, get_sentiment_handler, list_sentiments_handler,
            predict_handler,
        },
        outbound::{onnx::onnx_analyzer::OnnxAnalyzer, postgres::postgres_store::PostgresStore},
    },
    app_state::AppState,
    application::service::sentiment_service::SentimentService,
    config::DbConfig,
    openapi::ApiDoc,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let db_config = DbConfig::from_env();
    let repo = Arc::new(PostgresStore::new(&db_config)?);
    if db_config.run_migrations() {
        repo.run_migrations().await?;
    }
    let analyzer = Arc::new(OnnxAnalyzer::new(
        std::path::Path::new("models/model.onnx"),
        std::path::Path::new("models/tokenizer.json"),
    )?);
    let service = Arc::new(SentimentService::new(analyzer, repo));
    let state = AppState { service };

    let app = Router::new()
        .route("/predict", post(predict_handler))
        .route("/sentiments", get(list_sentiments_handler))
        .route(
            "/sentiments/{id}",
            get(get_sentiment_handler).delete(delete_sentiment_handler),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
