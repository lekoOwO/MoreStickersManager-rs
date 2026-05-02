use axum::Json;

use crate::dto::HealthResponse;

#[utoipa::path(
    get,
    path = "/healthz",
    tag = "system",
    responses((status = 200, description = "Service is healthy", body = HealthResponse))
)]
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
