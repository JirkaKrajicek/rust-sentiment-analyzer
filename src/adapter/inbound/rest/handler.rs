use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    adapter::inbound::rest::dto::{PredictRequest, PredictResponse},
    app_state::AppState,
};

#[utoipa::path(
    post,
    path = "/predict",
    request_body = PredictRequest,
    responses(
        (status = 200, description = "Sentiment prediction result", body = PredictResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "sentiment"
)]
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
        id: sentiment.prompt_id,
        sentiment: format!("{:?}", sentiment.sentiment),
        probability: sentiment.probability,
    };

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/sentiments",
    responses((status = 200, description = "Persisted sentiment predictions", body = [PredictResponse])),
    tag = "sentiment"
)]
pub async fn list_sentiments_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<PredictResponse>>, StatusCode> {
    let sentiments = state
        .service
        .list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(sentiments.into_iter().map(to_response).collect()))
}

#[utoipa::path(
    get,
    path = "/sentiments/{id}",
    params(("id" = Uuid, Path, description = "Prediction identifier")),
    responses((status = 200, description = "Persisted sentiment prediction", body = PredictResponse), (status = 404, description = "Prediction not found")),
    tag = "sentiment"
)]
pub async fn get_sentiment_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PredictResponse>, StatusCode> {
    let sentiment = state
        .service
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(to_response(sentiment)))
}

#[utoipa::path(
    delete,
    path = "/sentiments/{id}",
    params(("id" = Uuid, Path, description = "Prediction identifier")),
    responses((status = 204, description = "Prediction deleted"), (status = 404, description = "Prediction not found")),
    tag = "sentiment"
)]
pub async fn delete_sentiment_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state
        .service
        .delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn to_response(sentiment: crate::domain::sentiment::Sentiment) -> PredictResponse {
    PredictResponse {
        id: sentiment.prompt_id,
        sentiment: format!("{:?}", sentiment.sentiment),
        probability: sentiment.probability,
    }
}
