use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::DefaultBodyLimit,
    http::{Request, StatusCode},
    middleware,
    routing::{get, post},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use sentiment_analyzer::{
    adapter::{
        inbound::rest::handler::{
            delete_sentiment_handler, get_sentiment_handler, health_handler,
            list_sentiments_handler, predict_document_handler, predict_handler, readiness_handler,
        },
        inbound::rest::request_context::request_context,
        outbound::stub::{sentiment_analyzer::StubAnalyzer, stub_repository::StubRepository},
    },
    app_state::AppState,
    application::{
        port::sentiment_analyzer::SentimentAnalyzer, service::sentiment_service::SentimentService,
    },
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
    let state = AppState {
        service,
        document_max_characters: 1_000_000,
    };

    Router::new()
        .route(
            "/predict",
            post(predict_handler).layer(DefaultBodyLimit::max(limit)),
        )
        .route(
            "/predict/document",
            post(predict_document_handler).layer(DefaultBodyLimit::max(1024)),
        )
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/sentiments", get(list_sentiments_handler))
        .route(
            "/sentiments/{id}",
            get(get_sentiment_handler).delete(delete_sentiment_handler),
        )
        .with_state(state)
        .layer(middleware::from_fn(request_context))
}

async fn multipart_file(app: Router, file_name: &str, content: &[u8]) -> (StatusCode, Value) {
    let boundary = "document-boundary";
    let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n").into_bytes();
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/predict/document")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn document_prediction_accepts_utf8_text() {
    let (status, response) = multipart_file(build_app(), "note.txt", b"A short document").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["sentiment"], "Positive");
    assert_eq!(response["chunks"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn document_prediction_rejects_legacy_word_documents() {
    let (status, response) = multipart_file(build_app(), "old.doc", b"legacy binary").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response["code"], "invalid_document");
}

#[tokio::test]
async fn predict_rejects_empty_text() {
    let (status, response) = request(
        build_app(),
        "POST",
        "/predict",
        Body::from(json!({ "text": "   " }).to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response["code"], "empty_text");
    assert!(response["request_id"].as_str().is_some());
}

#[tokio::test]
async fn health_reports_the_process_as_healthy() {
    let (status, response) = request(build_app(), "GET", "/health", Body::empty()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["status"], "healthy");
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
