use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};
use sentiment_analyzer::{
    adapter::{
        inbound::rest::handler::{
            delete_sentiment_handler, get_sentiment_handler, health_handler,
            list_sentiments_handler, predict_document_handler, predict_handler, readiness_handler,
        },
        inbound::rest::request_context::request_context,
        outbound::{onnx::onnx_analyzer::OnnxAnalyzer, postgres::postgres_store::PostgresStore},
    },
    app_state::AppState,
    application::port::project_repository::ProjectRepository,
    application::service::sentiment_service::SentimentService,
    config::AppConfig,
    openapi::ApiDoc,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let config = AppConfig::from_env()?;
    let repo = Arc::new(PostgresStore::new(&config.database)?);
    if config.database.run_migrations() {
        repo.run_migrations().await?;
    }
    remove_expired_results(&repo, config.retention.result_retention_days).await?;
    spawn_retention_job(repo.clone(), config.retention.result_retention_days);
    let analyzer = Arc::new(OnnxAnalyzer::new(
        &config.model.model_path,
        &config.model.tokenizer_path,
        config.inference.max_tokens,
        config.inference.queue_timeout,
        config.inference.execution_timeout,
    )?);
    let service = Arc::new(SentimentService::new(analyzer, repo));
    let state = AppState {
        service,
        document_max_characters: config.inference.document_max_characters,
        document_extraction: Arc::new(tokio::sync::Semaphore::new(
            config.inference.document_extraction_concurrency,
        )),
        document_extraction_queue_timeout: config.inference.document_extraction_queue_timeout,
        document_extraction_timeout: config.inference.document_extraction_timeout,
    };

    let app = Router::new()
        .route(
            "/predict",
            post(predict_handler).layer(DefaultBodyLimit::max(
                config.inference.predict_max_body_bytes,
            )),
        )
        .route(
            "/predict/document",
            post(predict_document_handler).layer(DefaultBodyLimit::max(
                config.inference.document_max_body_bytes,
            )),
        )
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/sentiments", get(list_sentiments_handler))
        .route(
            "/sentiments/{id}",
            get(get_sentiment_handler).delete(delete_sentiment_handler),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(middleware::from_fn(request_context));

    let listener = tokio::net::TcpListener::bind(config.server.bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn remove_expired_results(repo: &PostgresStore, retention_days: u32) -> anyhow::Result<()> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(retention_days));
    let deleted = repo.delete_created_before(cutoff).await?;
    if deleted > 0 {
        eprintln!("level=info event=expired_sentiments_deleted count={deleted}");
    }
    Ok(())
}

fn spawn_retention_job(repo: Arc<PostgresStore>, retention_days: u32) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 60));
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(error) = remove_expired_results(&repo, retention_days).await {
                eprintln!("level=error event=expired_sentiments_cleanup_failed error={error}");
            }
        }
    });
}
