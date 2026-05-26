use std::{path::Path, sync::Arc};

use axum::{Router, body::Body, routing::post};
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use sentiment_analyzer::{
    adapter::{
        inbound::rest::handler::predict_handler,
        outbound::{
            onnx::onnx_analyzer::OnnxAnalyzer,
            stub::{sentiment_analyzer::StubAnalyzer, stub_repository::StubRepository},
        },
    },
    app_state::AppState,
    application::{
        port::sentiment_analyzer::SentimentAnalyzer, service::sentiment_service::SentimentService,
    },
    domain::sentiment::SentimentType,
};

fn build_app(analyzer: Arc<dyn SentimentAnalyzer>) -> Router {
    let repo = Arc::new(StubRepository);
    let service = Arc::new(SentimentService::new(analyzer, repo));
    let state = AppState { service };
    Router::new()
        .route("/predict", post(predict_handler))
        .with_state(state)
}

async fn post_predict(app: Router, text: &str) -> (StatusCode, Value) {
    let body = json!({ "text": text });
    let request = Request::builder()
        .method("POST")
        .uri("/predict")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

/// Tests 1 & 2 use the real OnnxAnalyzer and require models/ to be present.
/// `flavor = "multi_thread"` is required because `block_in_place` is used inside the analyzer.

#[tokio::test(flavor = "multi_thread")]
async fn positive_text_returns_positive() {
    let analyzer: Arc<dyn SentimentAnalyzer> = Arc::new(
        OnnxAnalyzer::new(
            Path::new("models/model.onnx"),
            Path::new("models/tokenizer.json"),
        )
        .expect("Failed to load OnnxAnalyzer"),
    );
    let app = build_app(analyzer);

    let (status, json) = post_predict(app, "I absolutely love this, it is fantastic!").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["sentiment"], "Positive");
    assert!(json["probability"].as_f64().unwrap() > 0.55);
}

#[tokio::test(flavor = "multi_thread")]
async fn negative_text_returns_negative() {
    let analyzer: Arc<dyn SentimentAnalyzer> = Arc::new(
        OnnxAnalyzer::new(
            Path::new("models/model.onnx"),
            Path::new("models/tokenizer.json"),
        )
        .expect("Failed to load OnnxAnalyzer"),
    );
    let app = build_app(analyzer);

    let (status, json) = post_predict(
        app,
        "This is absolutely terrible, I hate everything about it.",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["sentiment"], "Negative");
    assert!(json["probability"].as_f64().unwrap() > 0.55);
}

/// Test 3 uses an inline stub to reliably exercise the Neutral path
/// without depending on model output probabilities.
struct NeutralStub;

#[async_trait::async_trait]
impl SentimentAnalyzer for NeutralStub {
    async fn analyze(&self, _text: &str) -> Result<(SentimentType, f64), anyhow::Error> {
        Ok((SentimentType::Neutral, 0.5))
    }
}

#[tokio::test]
async fn neutral_sentiment_returns_neutral() {
    let analyzer: Arc<dyn SentimentAnalyzer> = Arc::new(NeutralStub);
    let app = build_app(analyzer);

    let (status, json) = post_predict(app, "It was a thing that existed.").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["sentiment"], "Neutral");
    assert!((json["probability"].as_f64().unwrap() - 0.5).abs() < f64::EPSILON);
}

/// Sanity check: missing request body returns 422 Unprocessable Entity.
#[tokio::test]
async fn missing_body_returns_422() {
    let analyzer: Arc<dyn SentimentAnalyzer> = Arc::new(StubAnalyzer);
    let app = build_app(analyzer);

    let request = Request::builder()
        .method("POST")
        .uri("/predict")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
