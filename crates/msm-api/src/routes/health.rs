use axum::{extract::State, http::StatusCode, Json};

use crate::{
    dto::{HealthComponentResponse, HealthDiagnosticsResponse, HealthResponse},
    ApiState,
};

#[utoipa::path(
    get,
    path = "/healthz",
    tag = "system",
    responses((status = 200, description = "Service is healthy", body = HealthResponse))
)]
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[utoipa::path(
    get,
    path = "/readyz",
    tag = "system",
    responses(
        (status = 200, description = "Service dependencies are ready", body = HealthDiagnosticsResponse),
        (status = 503, description = "One or more dependencies are not ready", body = HealthDiagnosticsResponse)
    )
)]
pub async fn readyz(
    State(state): State<ApiState>,
) -> (StatusCode, Json<HealthDiagnosticsResponse>) {
    let mut components = Vec::new();

    components.push(match state.repository().check().await {
        Ok(()) => HealthComponentResponse {
            name: "database",
            status: "ok",
            message: "database query succeeded".to_owned(),
        },
        Err(error) => HealthComponentResponse {
            name: "database",
            status: "error",
            message: error.to_string(),
        },
    });

    components.push(match state.asset_store().check().await {
        Ok(()) => HealthComponentResponse {
            name: "assetStore",
            status: "ok",
            message: "asset directory is available".to_owned(),
        },
        Err(error) => HealthComponentResponse {
            name: "assetStore",
            status: "error",
            message: error.to_string(),
        },
    });

    let status = if components.iter().all(|component| component.status == "ok") {
        "ok"
    } else {
        "degraded"
    };
    let http_status = if status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        http_status,
        Json(HealthDiagnosticsResponse { status, components }),
    )
}
