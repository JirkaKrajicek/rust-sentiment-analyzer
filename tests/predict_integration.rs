use std::sync::Arc;

use axum::{
    Router,
    body::Body,
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
            predict_handler,
        },
        outbound::stub::{sentiment_analyzer::StubAnalyzer, stub_repository::StubRepository},
    },
    app_state::AppState,
    application::service::sentiment_service::SentimentService,
};

fn build_app() -> Router {
    let analyzer = Arc::new(StubAnalyzer);
    let repository = Arc::new(StubRepository::default());
    let service = Arc::new(SentimentService::new(analyzer, repository));
    let state = AppState { service };

    Router::new()
        .route("/predict", post(predict_handler))
        .route("/sentiments", get(list_sentiments_handler))
        .route(
            "/sentiments/{id}",
            get(get_sentiment_handler).delete(delete_sentiment_handler),
        )
        .with_state(state)
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
