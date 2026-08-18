use std::{
    io::{Cursor, Write},
    sync::Arc,
    time::Duration,
};

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
use tokio::sync::Semaphore;
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
    domain::sentiment::SentimentType,
};

fn build_app() -> Router {
    build_app_with_predict_body_limit(64 * 1024)
}

fn build_app_with_predict_body_limit(limit: usize) -> Router {
    build_app_with_analyzer(Arc::new(StubAnalyzer), limit)
}

fn build_app_with_analyzer(analyzer: Arc<dyn SentimentAnalyzer>, limit: usize) -> Router {
    build_app_with_limits(analyzer, limit, 1_000_000)
}

fn build_app_with_document_character_limit(limit: usize) -> Router {
    build_app_with_limits(Arc::new(StubAnalyzer), 64 * 1024, limit)
}

fn build_app_with_limits(
    analyzer: Arc<dyn SentimentAnalyzer>,
    predict_limit: usize,
    document_max_characters: usize,
) -> Router {
    build_app_with_extraction_control(
        analyzer,
        predict_limit,
        document_max_characters,
        1,
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .0
}

fn build_app_with_extraction_control(
    analyzer: Arc<dyn SentimentAnalyzer>,
    predict_limit: usize,
    document_max_characters: usize,
    extraction_concurrency: usize,
    extraction_queue_timeout: Duration,
    extraction_timeout: Duration,
) -> (Router, Arc<Semaphore>) {
    let repository = Arc::new(StubRepository::default());
    let service = Arc::new(SentimentService::new(analyzer, repository));
    let document_extraction = Arc::new(Semaphore::new(extraction_concurrency));
    let state = AppState {
        service,
        document_max_characters,
        document_extraction: document_extraction.clone(),
        document_extraction_queue_timeout: extraction_queue_timeout,
        document_extraction_timeout: extraction_timeout,
    };

    let app = Router::new()
        .route(
            "/predict",
            post(predict_handler).layer(DefaultBodyLimit::max(predict_limit)),
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
        .layer(middleware::from_fn(request_context));

    (app, document_extraction)
}

struct ChunkingAnalyzer;

#[async_trait::async_trait]
impl SentimentAnalyzer for ChunkingAnalyzer {
    async fn analyze(&self, text: &str) -> Result<(SentimentType, f64), anyhow::Error> {
        if text == "negative" {
            Ok((SentimentType::Negative, 0.8))
        } else {
            Ok((SentimentType::Positive, 0.9))
        }
    }

    fn chunk_text(&self, _text: &str) -> Result<Vec<String>, anyhow::Error> {
        Ok(vec!["positive".to_string(), "negative".to_string()])
    }

    async fn is_ready(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
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
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn document_prediction_accepts_utf8_text() {
    let (status, response) = multipart_file(build_app(), "note.txt", b"A short document").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["sentiment"], "Positive");
    assert_eq!(response["chunks"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn document_prediction_extracts_text_from_docx() {
    let cursor = Cursor::new(Vec::new());
    let mut archive = zip::ZipWriter::new(cursor);
    archive
        .start_file(
            "word/document.xml",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    archive
        .write_all(b"<w:document><w:body><w:p><w:r><w:t>Hello from DOCX</w:t></w:r></w:p></w:body></w:document>")
        .unwrap();
    let document = archive.finish().unwrap().into_inner();

    let (status, response) = multipart_file(build_app(), "note.docx", &document).await;

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
async fn document_prediction_rejects_malformed_docx() {
    let (status, response) = multipart_file(build_app(), "broken.docx", b"PK not a ZIP").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response["code"], "invalid_document");
}

#[tokio::test]
async fn document_prediction_rejects_a_body_larger_than_its_route_limit() {
    let oversized_document = vec![b'x'; 2_000];
    let (status, response) = multipart_file(build_app(), "large.txt", &oversized_document).await;

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(response["code"], "payload_too_large");
}

#[tokio::test]
async fn document_prediction_rejects_non_utf8_text() {
    let (status, response) = multipart_file(build_app(), "broken.txt", &[0xff, 0xfe]).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response["code"], "invalid_document");
}

#[tokio::test]
async fn document_prediction_rejects_text_larger_than_the_extracted_text_limit() {
    let (status, response) = multipart_file(
        build_app_with_document_character_limit(5),
        "long.txt",
        b"six chars",
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response["code"], "invalid_document");
}

#[tokio::test]
async fn document_prediction_rejects_when_extraction_capacity_is_full() {
    let (app, extraction) = build_app_with_extraction_control(
        Arc::new(StubAnalyzer),
        64 * 1024,
        1_000_000,
        1,
        Duration::from_millis(5),
        Duration::from_secs(1),
    );
    let _permit = extraction.acquire_owned().await.unwrap();

    let (status, response) = multipart_file(app, "note.txt", b"A short document").await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response["code"], "document_extraction_overloaded");
}

#[tokio::test]
async fn document_prediction_returns_each_chunk_and_aggregates_their_scores() {
    let app = build_app_with_analyzer(Arc::new(ChunkingAnalyzer), 64 * 1024);
    let (status, response) = multipart_file(app, "document.txt", b"any text").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["aggregation"], "mean_signed_confidence");
    assert_eq!(response["sentiment"], "Positive");
    assert_eq!(response["chunks"].as_array().unwrap().len(), 2);
    assert_eq!(response["chunks"][1]["sentiment"], "Negative");
}

#[tokio::test]
async fn document_prediction_details_are_available_after_retrieval() {
    let app = build_app_with_analyzer(Arc::new(ChunkingAnalyzer), 64 * 1024);
    let (status, created) = multipart_file(app.clone(), "document.txt", b"any text").await;
    assert_eq!(status, StatusCode::OK);
    let id = created["id"].as_str().unwrap();

    let (status, retrieved) =
        request(app, "GET", &format!("/sentiments/{id}"), Body::empty()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        retrieved["document_details"]["aggregation"],
        "mean_signed_confidence"
    );
    assert_eq!(
        retrieved["document_details"]["chunks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        retrieved["document_details"]["chunks"][1]["sentiment"],
        "Negative"
    );
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
async fn successful_requests_keep_the_client_request_id() {
    let response = build_app()
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("x-request-id", "client-request-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-request-id"], "client-request-id");
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
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
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
