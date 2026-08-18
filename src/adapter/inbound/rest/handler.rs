use axum::{
    Json,
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    adapter::inbound::rest::{
        dto::{
            DocumentChunkResponse, DocumentDetailsResponse, DocumentPredictResponse, ErrorResponse,
            HealthResponse, PredictRequest, PredictResponse, ReadinessResponse,
        },
        request_context::RequestId,
    },
    app_state::AppState,
    application::port::sentiment_analyzer::InferenceError,
};

#[utoipa::path(
    post,
    path = "/predict",
    request_body = PredictRequest,
    responses(
        (status = 200, description = "Sentiment prediction result", body = PredictResponse),
        (status = 413, description = "Request body exceeds the configured limit", body = ErrorResponse),
        (status = 422, description = "Text is empty", body = ErrorResponse),
        (status = 429, description = "Inference capacity is unavailable", body = ErrorResponse),
        (status = 504, description = "Inference timed out", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "sentiment"
)]
pub async fn predict_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(body): Json<PredictRequest>,
) -> Result<Json<PredictResponse>, ApiError> {
    if body.text.trim().is_empty() {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "empty_text",
            "Text must not be empty",
            request_id,
        ));
    }
    let sentiment = state
        .service
        .predict(body.text)
        .await
        .map_err(|error| predict_error(error, request_id.clone()))?;

    Ok(Json(to_response(sentiment)))
}

#[utoipa::path(
    post,
    path = "/predict/document",
    request_body(content = String, content_type = "multipart/form-data"),
    responses((status = 200, body = DocumentPredictResponse), (status = 422, body = ErrorResponse), (status = 500, body = ErrorResponse)),
    tag = "sentiment"
)]
pub async fn predict_document_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    mut multipart: Multipart,
) -> Result<Json<DocumentPredictResponse>, ApiError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::internal(error.into(), request_id.clone()))?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "missing_file",
                "A file field is required",
                request_id.clone(),
            )
        })?;
    if field.name() != Some("file") {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "missing_file",
            "A file field is required",
            request_id,
        ));
    }
    let file_name = field.file_name().unwrap_or_default().to_owned();
    let bytes = field
        .bytes()
        .await
        .map_err(|error| ApiError::internal(error.into(), request_id.clone()))?;
    let text = crate::adapter::inbound::rest::document::extract_text(
        &file_name,
        &bytes,
        state.document_max_characters,
    )
    .map_err(|_| {
        ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_document",
            "The document could not be processed",
            request_id.clone(),
        )
    })?;
    let document = state
        .service
        .predict_document(text)
        .await
        .map_err(|error| predict_error(error, request_id))?;
    Ok(Json(DocumentPredictResponse {
        id: document.result.prompt_id,
        sentiment: format!("{:?}", document.result.sentiment),
        probability: document.result.probability,
        aggregation: document.aggregation.to_string(),
        chunks: document
            .chunks
            .into_iter()
            .map(|chunk| DocumentChunkResponse {
                index: chunk.index,
                sentiment: format!("{:?}", chunk.sentiment),
                probability: chunk.probability,
            })
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Service process is healthy", body = HealthResponse)),
    tag = "sentiment"
)]
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "healthy" })
}

#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "Model is ready", body = ReadinessResponse),
        (status = 503, description = "Model is not ready", body = ErrorResponse)
    ),
    tag = "sentiment"
)]
pub async fn readiness_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<ReadinessResponse>, ApiError> {
    state
        .service
        .is_ready()
        .await
        .map_err(|error| ApiError::internal(error, request_id))?;
    Ok(Json(ReadinessResponse { status: "ready" }))
}

#[utoipa::path(
    get,
    path = "/sentiments",
    responses((status = 200, description = "Persisted sentiment predictions", body = [PredictResponse]), (status = 500, description = "Internal server error", body = ErrorResponse)),
    tag = "sentiment"
)]
pub async fn list_sentiments_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<Vec<PredictResponse>>, ApiError> {
    let sentiments = state
        .service
        .list()
        .await
        .map_err(|error| ApiError::internal(error, request_id))?;
    Ok(Json(sentiments.into_iter().map(to_response).collect()))
}

#[utoipa::path(
    get,
    path = "/sentiments/{id}",
    params(("id" = Uuid, Path, description = "Prediction identifier")),
    responses((status = 200, description = "Persisted sentiment prediction", body = PredictResponse), (status = 404, description = "Prediction not found", body = ErrorResponse), (status = 500, description = "Internal server error", body = ErrorResponse)),
    tag = "sentiment"
)]
pub async fn get_sentiment_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(id): Path<Uuid>,
) -> Result<Json<PredictResponse>, ApiError> {
    let sentiment = state
        .service
        .get(id)
        .await
        .map_err(|error| ApiError::internal(error, request_id.clone()))?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "sentiment_not_found",
                "Sentiment prediction was not found",
                request_id,
            )
        })?;
    Ok(Json(to_response(sentiment)))
}

#[utoipa::path(
    delete,
    path = "/sentiments/{id}",
    params(("id" = Uuid, Path, description = "Prediction identifier")),
    responses((status = 204, description = "Prediction deleted"), (status = 404, description = "Prediction not found", body = ErrorResponse), (status = 500, description = "Internal server error", body = ErrorResponse)),
    tag = "sentiment"
)]
pub async fn delete_sentiment_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let deleted = state
        .service
        .delete(id)
        .await
        .map_err(|error| ApiError::internal(error, request_id.clone()))?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "sentiment_not_found",
            "Sentiment prediction was not found",
            request_id,
        ))
    }
}

pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    request_id: RequestId,
}

impl ApiError {
    fn new(
        status: StatusCode,
        code: &'static str,
        message: &'static str,
        request_id: RequestId,
    ) -> Self {
        Self {
            status,
            code,
            message,
            request_id,
        }
    }

    fn internal(_error: anyhow::Error, request_id: RequestId) -> Self {
        eprintln!(
            "level=error event=internal_error request_id={}",
            request_id.0
        );
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "An internal error occurred",
            request_id,
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                code: self.code,
                message: self.message,
                request_id: self.request_id.0,
            }),
        )
            .into_response()
    }
}

fn to_response(sentiment: crate::domain::sentiment::Sentiment) -> PredictResponse {
    PredictResponse {
        id: sentiment.prompt_id,
        sentiment: format!("{:?}", sentiment.sentiment),
        probability: sentiment.probability,
        document_details: sentiment
            .document_details
            .map(|details| DocumentDetailsResponse {
                aggregation: details.aggregation,
                chunks: details
                    .chunks
                    .into_iter()
                    .map(|chunk| DocumentChunkResponse {
                        index: chunk.index,
                        sentiment: format!("{:?}", chunk.sentiment),
                        probability: chunk.probability,
                    })
                    .collect(),
            }),
    }
}

fn predict_error(error: anyhow::Error, request_id: RequestId) -> ApiError {
    match error.downcast_ref::<InferenceError>() {
        Some(InferenceError::Overloaded) => ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "inference_overloaded",
            "Inference capacity is unavailable",
            request_id,
        ),
        Some(InferenceError::TimedOut) => ApiError::new(
            StatusCode::GATEWAY_TIMEOUT,
            "inference_timed_out",
            "Inference timed out",
            request_id,
        ),
        _ => ApiError::internal(error, request_id),
    }
}
