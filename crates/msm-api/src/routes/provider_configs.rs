use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::Permission;
use msm_storage::models::{NewProviderConfig, ProviderConfigRecord};

use crate::{
    auth::require_pat,
    dto::{ListProviderConfigsQuery, ProviderConfigResponse, UpsertProviderConfigRequest},
    rbac::require_tenant_permission,
    ApiError, ApiResult, ApiState,
};

const REDACTED_SECRET: &str = "<redacted>";

/// Lists tenant-scoped provider import configurations.
///
/// # Errors
///
/// Returns an API error when the PAT lacks provider import scope or tenant access.
#[utoipa::path(
    get,
    path = "/api/v1/provider-configs",
    tag = "provider-configs",
    params(ListProviderConfigsQuery),
    responses(
        (status = 200, description = "Configured provider import credentials", body = [ProviderConfigResponse]),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing provider.import scope or tenant role", body = crate::error::ApiErrorBody)
    )
)]
pub async fn list_provider_configs(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListProviderConfigsQuery>,
) -> ApiResult<Json<Vec<ProviderConfigResponse>>> {
    let pat = require_pat(&headers, &state, Permission::ProviderImport).await?;
    require_tenant_permission(
        &state,
        &pat,
        &query.tenant_id,
        Permission::ProviderImport,
        true,
        "PAT user cannot access provider configs for this tenant",
    )
    .await?;

    state
        .repository()
        .list_provider_configs(&query.tenant_id)
        .await?
        .into_iter()
        .map(provider_config_response)
        .collect::<ApiResult<Vec<_>>>()
        .map(Json)
}

/// Creates or replaces a tenant-scoped provider import configuration.
///
/// # Errors
///
/// Returns an API error when the PAT lacks provider import scope and tenant
/// admin/custom-role access, config serialization fails, or storage fails.
#[utoipa::path(
    put,
    path = "/api/v1/provider-configs/{config_id}",
    tag = "provider-configs",
    params(("config_id" = String, Path, description = "Provider config ID")),
    request_body = UpsertProviderConfigRequest,
    responses(
        (status = 200, description = "Provider config upserted", body = ProviderConfigResponse),
        (status = 400, description = "Invalid provider config", body = crate::error::ApiErrorBody),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing provider.import scope or tenant admin role", body = crate::error::ApiErrorBody)
    )
)]
pub async fn upsert_provider_config(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(config_id): Path<String>,
    Json(request): Json<UpsertProviderConfigRequest>,
) -> ApiResult<Json<ProviderConfigResponse>> {
    let pat = require_pat(&headers, &state, Permission::ProviderImport).await?;
    require_tenant_permission(
        &state,
        &pat,
        &request.tenant_id,
        Permission::ProviderImport,
        false,
        "PAT user cannot manage provider configs for this tenant",
    )
    .await?;
    validate_provider_id(&request.provider_id)?;
    let config_json = serde_json::to_string(&request.config)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    let config = state
        .repository()
        .upsert_provider_config(NewProviderConfig {
            id: &config_id,
            tenant_id: &request.tenant_id,
            provider_id: &request.provider_id,
            name: &request.name,
            config_json: &config_json,
            is_enabled: request.is_enabled,
        })
        .await?;

    provider_config_response(config).map(Json)
}

/// Deletes one tenant-scoped provider import configuration.
///
/// # Errors
///
/// Returns an API error when the config is missing, the PAT lacks tenant
/// admin/custom-role access, or storage fails.
#[utoipa::path(
    delete,
    path = "/api/v1/provider-configs/{config_id}",
    tag = "provider-configs",
    params(("config_id" = String, Path, description = "Provider config ID")),
    responses(
        (status = 204, description = "Provider config deleted"),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing provider.import scope or tenant admin role", body = crate::error::ApiErrorBody),
        (status = 404, description = "Provider config not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn delete_provider_config(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(config_id): Path<String>,
) -> ApiResult<StatusCode> {
    let pat = require_pat(&headers, &state, Permission::ProviderImport).await?;
    let existing = state
        .repository()
        .find_provider_config(&config_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("provider config not found: {config_id}")))?;
    require_tenant_permission(
        &state,
        &pat,
        &existing.tenant_id,
        Permission::ProviderImport,
        false,
        "PAT user cannot manage provider configs for this tenant",
    )
    .await?;

    if state
        .repository()
        .delete_provider_config(&config_id)
        .await?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(format!(
            "provider config not found: {config_id}"
        )))
    }
}

fn validate_provider_id(provider_id: &str) -> ApiResult<()> {
    if matches!(provider_id, "telegram" | "line-stickers") {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "unsupported provider config source: {provider_id}"
        )))
    }
}

fn provider_config_response(record: ProviderConfigRecord) -> ApiResult<ProviderConfigResponse> {
    Ok(ProviderConfigResponse {
        id: record.id,
        tenant_id: record.tenant_id,
        provider_id: record.provider_id,
        name: record.name,
        config: redacted_config(&record.config_json)?,
        is_enabled: record.is_enabled,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

fn redacted_config(config_json: &str) -> ApiResult<serde_json::Value> {
    let mut config = parse_json_value(config_json)?;
    redact_secrets(&mut config);
    Ok(config)
}

fn parse_json_value(value: &str) -> ApiResult<serde_json::Value> {
    serde_json::from_str(value).map_err(|error| ApiError::Internal(error.to_string()))
}

fn redact_secrets(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let key = key.to_ascii_lowercase();
                if key.contains("token") || key.contains("secret") {
                    *value = serde_json::Value::String(REDACTED_SECRET.to_owned());
                } else {
                    redact_secrets(value);
                }
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(redact_secrets),
        _ => {}
    }
}
