use axum::{Json, Router, response::IntoResponse, routing::get};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/api", get(api_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn api_handler() -> impl IntoResponse {
    let json_response = json!({
        "status": "success",
        "message": "Hello, World!"
    });
    Json(json_response)
}
