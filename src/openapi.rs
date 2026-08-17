use utoipa::OpenApi;

use crate::adapter::inbound::rest::dto::{PredictRequest, PredictResponse};

#[derive(OpenApi)]
#[openapi(
    paths(crate::adapter::inbound::rest::handler::predict_handler),
    components(schemas(PredictRequest, PredictResponse)),
    tags((name = "sentiment", description = "Sentiment analysis endpoints")),
    info(
        title = "Sentiment Analyzer API",
        version = "1.0.0",
        description = "REST API for text sentiment analysis using ONNX models"
    )
)]
pub struct ApiDoc;
