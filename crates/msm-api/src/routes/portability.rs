use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::Permission;
use msm_storage::portability::{export_user_data, import_user_data, PortableUserExport};

use crate::{
    auth::require_pat,
    dto::{ExportUserDataQuery, ImportUserDataRequest},
    rate_limit::enforce_import_rate_limit,
    rbac::require_tenant_resource_access,
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    get,
    path = "/api/v1/portable/user-export",
    tag = "portable",
    params(ExportUserDataQuery),
    responses(
        (status = 200, description = "Portable user export", body = serde_json::Value),
        (status = 403, description = "PAT cannot export this user", body = crate::error::ApiErrorBody)
    )
)]
/// Exports all portable data for one user.
///
/// # Errors
///
/// Returns an error when authorization fails, storage fails, or the export cannot be serialized.
pub async fn export_user(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ExportUserDataQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let pat = require_pat(&headers, &state, Permission::PackRead).await?;
    pat.require_user(&query.user_id)?;
    let export = export_user_data(state.repository(), &query.user_id).await?;
    serde_json::to_value(export)
        .map(Json)
        .map_err(|error| ApiError::Internal(error.to_string()))
}

#[utoipa::path(
    post,
    path = "/api/v1/portable/user-import",
    tag = "portable",
    request_body = ImportUserDataRequest,
    responses(
        (status = 201, description = "Portable user import completed"),
        (status = 400, description = "Invalid portable user export", body = crate::error::ApiErrorBody),
        (status = 403, description = "PAT cannot import into this tenant", body = crate::error::ApiErrorBody)
    )
)]
/// Imports portable user data into an existing tenant.
///
/// # Errors
///
/// Returns an error when authorization fails, the export payload is invalid, or storage fails.
pub async fn import_user(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<ImportUserDataRequest>,
) -> ApiResult<StatusCode> {
    enforce_import_rate_limit(&headers, &state)?;
    let export: PortableUserExport = serde_json::from_value(request.export)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let pat = require_pat(&headers, &state, Permission::ImportRun).await?;
    pat.require_user(&export.user.id)?;
    require_tenant_resource_access(
        &state,
        &pat,
        &request.tenant_id,
        &export.user.id,
        Permission::ImportRun,
        "PAT user cannot import portable data into this tenant",
    )
    .await?;

    import_user_data(state.repository(), &request.tenant_id, &export).await?;
    Ok(StatusCode::CREATED)
}
