use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    adapter::inbound::rest::dto::{PredictRequest, PredictResponse},
    app_state::AppState,
};

pub async fn predict_handler(
    State(state): State<AppState>,
    Json(body): Json<PredictRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let sentiment = state
        .service
        .predict(body.text)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = PredictResponse {
        sentiment: format!("{:?}", sentiment.sentiment),
        probability: sentiment.probability,
    };

    Ok(Json(response))
}
