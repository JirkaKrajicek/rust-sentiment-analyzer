use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::DefaultBodyLimit,
    http::{Request, StatusCode},
    routing::{get, post},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use sentiment_analyzer::{
    adapter::{
        inbound::rest::handler::{
            delete_sentiment_handler, get_sentiment_handler, list_sentiments_handler,
            predict_handler, readiness_handler,
        },
        outbound::stub::{sentiment_analyzer::StubAnalyzer, stub_repository::StubRepository},
    },
    app_state::AppState,
    application::{
        port::sentiment_analyzer::SentimentAnalyzer, service::sentiment_service::SentimentService,
    },
    domain::sentiment::SentimentType,
};

fn build_app() -> Router {
    build_app_with_predict_body_limit(64 * 1024)
}

fn build_app_with_predict_body_limit(limit: usize) -> Router {
    build_app_with_analyzer(Arc::new(StubAnalyzer), limit)
}

fn build_app_with_analyzer(analyzer: Arc<dyn SentimentAnalyzer>, limit: usize) -> Router {
    let repository = Arc::new(StubRepository::default());
    let service = Arc::new(SentimentService::new(analyzer, repository));
    let state = AppState { service };

    Router::new()
        .route(
            "/predict",
            post(predict_handler).layer(DefaultBodyLimit::max(limit)),
        )
        .route("/ready", get(readiness_handler))
        .route("/sentiments", get(list_sentiments_handler))
        .route(
            "/sentiments/{id}",
            get(get_sentiment_handler).delete(delete_sentiment_handler),
        )
        .with_state(state)
}

struct UnreadyAnalyzer;

#[async_trait::async_trait]
impl SentimentAnalyzer for UnreadyAnalyzer {
    async fn analyze(&self, _text: &str) -> Result<(SentimentType, f64), anyhow::Error> {
        anyhow::bail!("Analyzer is unavailable")
    }

    async fn is_ready(&self) -> Result<(), anyhow::Error> {
        anyhow::bail!("Analyzer is unavailable")
    }
}

#[tokio::test]
async fn predict_rejects_empty_text() {
    let (status, _) = request(
        build_app(),
        "POST",
        "/predict",
        Body::from(json!({ "text": "   " }).to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn predict_rejects_a_body_larger_than_its_route_limit() {
    let (status, _) = request(
        build_app_with_predict_body_limit(16),
        "POST",
        "/predict",
        Body::from(json!({ "text": "larger than sixteen bytes" }).to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn readiness_reports_the_stub_analyzer_as_ready() {
    let (status, response) = request(build_app(), "GET", "/ready", Body::empty()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["status"], "ready");
}

#[tokio::test]
async fn readiness_reports_an_unready_analyzer_as_unavailable() {
    let app = build_app_with_analyzer(Arc::new(UnreadyAnalyzer), 64 * 1024);
    let (status, _) = request(app, "GET", "/ready", Body::empty()).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

async fn request(app: Router, method: &str, uri: &str, body: Body) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, json)
}

#[tokio::test]
async fn predict_persists_and_returns_a_result_id() {
    let (status, response) = request(
        build_app(),
        "POST",
        "/predict",
        Body::from(json!({ "text": "A very good day" }).to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["sentiment"], "Positive");
    assert_eq!(response["probability"], 0.99);
    assert!(response["id"].as_str().is_some());
}

#[tokio::test]
async fn prediction_can_be_listed_retrieved_and_deleted() {
    let app = build_app();
    let (_, created) = request(
        app.clone(),
        "POST",
        "/predict",
        Body::from(json!({ "text": "Remember this result" }).to_string()),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (list_status, list) = request(app.clone(), "GET", "/sentiments", Body::empty()).await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], id);

    let (get_status, retrieved) = request(
        app.clone(),
        "GET",
        &format!("/sentiments/{id}"),
        Body::empty(),
    )
    .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(retrieved["id"], id);

    let (delete_status, _) = request(
        app.clone(),
        "DELETE",
        &format!("/sentiments/{id}"),
        Body::empty(),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (missing_status, _) =
        request(app, "GET", &format!("/sentiments/{id}"), Body::empty()).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
}
