use axum::{Router, routing::get};
use sentiment_analyzer::adapter::inbound::rest::handler::api_handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/api", get(api_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
