use axum::Json;
use utoipa::OpenApi;

use crate::{dto::HealthResponse, error::ApiErrorBody, routes::health};

#[derive(OpenApi)]
#[openapi(
    paths(health::healthz),
    components(schemas(HealthResponse, ApiErrorBody)),
    tags((name = "system", description = "System endpoints"))
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
