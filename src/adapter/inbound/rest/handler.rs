use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn api_handler() -> impl IntoResponse {
    let json_response = json!({
        "status": "ok",
        "message": "Hello, World!"
    });
    Json(json_response)
}
