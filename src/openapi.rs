use utoipa::OpenApi;

use crate::adapter::inbound::rest::dto::{PredictRequest, PredictResponse, ReadinessResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::adapter::inbound::rest::handler::predict_handler,
        crate::adapter::inbound::rest::handler::readiness_handler,
        crate::adapter::inbound::rest::handler::list_sentiments_handler,
        crate::adapter::inbound::rest::handler::get_sentiment_handler,
        crate::adapter::inbound::rest::handler::delete_sentiment_handler
    ),
    components(schemas(PredictRequest, PredictResponse, ReadinessResponse)),
    tags((name = "sentiment", description = "Sentiment analysis endpoints")),
    info(
        title = "Sentiment Analyzer API",
        version = "1.0.0",
        description = "REST API for text sentiment analysis using ONNX models"
    )
)]
pub struct ApiDoc;
