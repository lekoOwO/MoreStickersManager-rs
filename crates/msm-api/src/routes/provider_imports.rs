use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    Json,
};

use msm_domain::Permission;
use msm_providers::{
    line_sticker_pack_fetch_plan, telegram_sticker_set_fetch_plan, ProviderAssetDownloadStrategy,
    ProviderRemoteFetchPlan,
};
use msm_storage::models::{
    NewProviderImportJob, NewProviderImportJobEvent, ProviderImportJobEventRecord,
    ProviderImportJobRecord,
};

use crate::{
    auth::require_pat,
    dto::{
        CreateProviderImportJobRequest, CreateProviderImportPlanRequest,
        ProviderHttpHeaderResponse, ProviderHttpRequestPlanResponse,
        ProviderImportJobEventResponse, ProviderImportJobResponse, ProviderImportPlanResponse,
    },
    rate_limit::enforce_import_rate_limit,
    rbac::require_tenant_resource_access,
    ApiError, ApiResult, ApiState,
};
use axum::extract::State;

#[utoipa::path(
    post,
    path = "/api/v1/provider-imports/plan",
    tag = "provider-imports",
    request_body = CreateProviderImportPlanRequest,
    responses(
        (status = 200, description = "Provider import fetch plan", body = ProviderImportPlanResponse),
        (status = 400, description = "Unsupported provider or invalid remote ID", body = crate::error::ApiErrorBody),
        (status = 403, description = "PAT cannot import providers into this tenant", body = crate::error::ApiErrorBody)
    )
)]
/// Creates a provider import fetch plan for runtime execution.
///
/// # Errors
///
/// Returns an error when authorization fails, the provider is unsupported, or
/// the requested provider/base URL/remote ID cannot produce a safe fetch plan.
pub async fn create_provider_import_plan(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateProviderImportPlanRequest>,
) -> ApiResult<Json<ProviderImportPlanResponse>> {
    enforce_import_rate_limit(&headers, &state)?;
    let pat = require_pat(&headers, &state, Permission::ProviderImport).await?;
    pat.require_user(&request.owner_user_id)?;
    require_tenant_resource_access(
        &state,
        &pat,
        &request.tenant_id,
        &request.owner_user_id,
        Permission::ProviderImport,
        "PAT user cannot import provider packs into this tenant",
    )
    .await?;

    let plan = match request.provider_id.as_str() {
        "telegram" => telegram_sticker_set_fetch_plan(
            request
                .base_url
                .as_deref()
                .unwrap_or("https://api.telegram.org"),
            &request.remote_id,
        ),
        "line-stickers" => line_sticker_pack_fetch_plan(
            request
                .base_url
                .as_deref()
                .unwrap_or("https://store.line.me"),
            &request.remote_id,
        ),
        other => {
            return Err(ApiError::BadRequest(format!(
                "unsupported provider import source: {other}"
            )));
        }
    }
    .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    Ok(Json(ProviderImportPlanResponse::from(plan)))
}

#[utoipa::path(
    post,
    path = "/api/v1/provider-import-jobs",
    tag = "provider-imports",
    request_body = CreateProviderImportJobRequest,
    responses(
        (status = 201, description = "Provider import job queued", body = ProviderImportJobResponse),
        (status = 400, description = "Unsupported provider or invalid remote ID", body = crate::error::ApiErrorBody),
        (status = 403, description = "PAT cannot import providers into this tenant", body = crate::error::ApiErrorBody)
    )
)]
/// Queues a provider import job for worker execution.
///
/// # Errors
///
/// Returns an error when authorization fails, the provider is unsupported, or
/// the request cannot be persisted.
pub async fn create_provider_import_job(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateProviderImportJobRequest>,
) -> ApiResult<(StatusCode, Json<ProviderImportJobResponse>)> {
    enforce_import_rate_limit(&headers, &state)?;
    let pat =
        authorize_provider_import(&state, &headers, &request.tenant_id, &request.owner_user_id)
            .await?;
    let plan = provider_import_plan(
        &request.provider_id,
        &request.remote_id,
        request.base_url.as_deref(),
    )?;
    let request_json = serde_json::to_string(&serde_json::json!({
        "providerId": request.provider_id,
        "remoteId": request.remote_id,
        "targetPackId": request.target_pack_id,
        "baseUrl": request.base_url,
        "plan": ProviderImportPlanResponse::from(plan),
    }))
    .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    let job = state
        .repository()
        .create_provider_import_job(NewProviderImportJob {
            id: &request.id,
            tenant_id: &request.tenant_id,
            owner_user_id: &pat.user_id,
            provider_id: &request.provider_id,
            remote_id: &request.remote_id,
            target_pack_id: request.target_pack_id.as_deref(),
            request_json: &request_json,
            max_attempts: 3,
        })
        .await?;
    state
        .repository()
        .append_provider_import_job_event(NewProviderImportJobEvent {
            job_id: &job.id,
            sequence: 1,
            level: "info",
            stage: "queued",
            message: "Provider import job queued.",
            metadata_json: "{}",
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(provider_import_job_response(job)?),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/provider-import-jobs/{job_id}",
    tag = "provider-imports",
    params(("job_id" = String, Path, description = "Provider import job ID")),
    responses(
        (status = 200, description = "Provider import job", body = ProviderImportJobResponse),
        (status = 404, description = "Provider import job was not found", body = crate::error::ApiErrorBody)
    )
)]
/// Reads a provider import job.
///
/// # Errors
///
/// Returns an error when the job is missing or authorization fails.
pub async fn get_provider_import_job(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> ApiResult<Json<ProviderImportJobResponse>> {
    let job = load_provider_import_job(&state, &headers, &job_id).await?;
    Ok(Json(provider_import_job_response(job)?))
}

#[utoipa::path(
    get,
    path = "/api/v1/provider-import-jobs/{job_id}/events",
    tag = "provider-imports",
    params(("job_id" = String, Path, description = "Provider import job ID")),
    responses((status = 200, description = "Provider import job events", body = [ProviderImportJobEventResponse]))
)]
/// Lists provider import job events.
///
/// # Errors
///
/// Returns an error when the job is missing, authorization fails, or event
/// metadata cannot be parsed.
pub async fn list_provider_import_job_events(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> ApiResult<Json<Vec<ProviderImportJobEventResponse>>> {
    let _job = load_provider_import_job(&state, &headers, &job_id).await?;
    let events = state
        .repository()
        .list_provider_import_job_events(&job_id)
        .await?
        .into_iter()
        .map(provider_import_job_event_response)
        .collect::<ApiResult<Vec<_>>>()?;
    Ok(Json(events))
}

impl From<ProviderRemoteFetchPlan> for ProviderImportPlanResponse {
    fn from(plan: ProviderRemoteFetchPlan) -> Self {
        Self {
            provider_id: plan.provider_id,
            remote_id: plan.remote_id,
            metadata_request: ProviderHttpRequestPlanResponse {
                method: plan.metadata_request.method,
                url: plan.metadata_request.url,
                redacted_headers: plan
                    .metadata_request
                    .redacted_headers
                    .into_iter()
                    .map(|(name, value)| ProviderHttpHeaderResponse { name, value })
                    .collect(),
            },
            asset_strategy: match plan.asset_strategy {
                ProviderAssetDownloadStrategy::TelegramBotFileApi => "telegramBotFileApi",
                ProviderAssetDownloadStrategy::DirectRemoteUrls => "directRemoteUrls",
            }
            .to_owned(),
        }
    }
}

async fn authorize_provider_import(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &str,
    owner_user_id: &str,
) -> ApiResult<crate::auth::VerifiedPat> {
    let pat = require_pat(headers, state, Permission::ProviderImport).await?;
    pat.require_user(owner_user_id)?;
    require_tenant_resource_access(
        state,
        &pat,
        tenant_id,
        owner_user_id,
        Permission::ProviderImport,
        "PAT user cannot import provider packs into this tenant",
    )
    .await?;
    Ok(pat)
}

fn provider_import_plan(
    provider_id: &str,
    remote_id: &str,
    base_url: Option<&str>,
) -> ApiResult<ProviderRemoteFetchPlan> {
    match provider_id {
        "telegram" => telegram_sticker_set_fetch_plan(
            base_url.unwrap_or("https://api.telegram.org"),
            remote_id,
        ),
        "line-stickers" => {
            line_sticker_pack_fetch_plan(base_url.unwrap_or("https://store.line.me"), remote_id)
        }
        other => {
            return Err(ApiError::BadRequest(format!(
                "unsupported provider import source: {other}"
            )));
        }
    }
    .map_err(|error| ApiError::BadRequest(error.to_string()))
}

async fn load_provider_import_job(
    state: &ApiState,
    headers: &HeaderMap,
    job_id: &str,
) -> ApiResult<ProviderImportJobRecord> {
    let pat = require_pat(headers, state, Permission::ProviderImport).await?;
    let job = state
        .repository()
        .find_provider_import_job(job_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Provider import job `{job_id}` not found")))?;
    pat.require_user(&job.owner_user_id)?;
    require_tenant_resource_access(
        state,
        &pat,
        &job.tenant_id,
        &job.owner_user_id,
        Permission::ProviderImport,
        "PAT user cannot read provider import jobs in this tenant",
    )
    .await?;
    Ok(job)
}

fn provider_import_job_response(
    record: ProviderImportJobRecord,
) -> ApiResult<ProviderImportJobResponse> {
    Ok(ProviderImportJobResponse {
        id: record.id,
        tenant_id: record.tenant_id,
        owner_user_id: record.owner_user_id,
        provider_id: record.provider_id,
        remote_id: record.remote_id,
        target_pack_id: record.target_pack_id,
        status: record.status.as_str().to_owned(),
        request: serde_json::from_str(&record.request_json)
            .map_err(|error| ApiError::Internal(error.to_string()))?,
        result: record
            .result_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| ApiError::Internal(error.to_string()))?,
        error_summary: record.error_summary,
        attempt_count: record.attempt_count,
        max_attempts: record.max_attempts,
        next_attempt_at: record.next_attempt_at,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

fn provider_import_job_event_response(
    record: ProviderImportJobEventRecord,
) -> ApiResult<ProviderImportJobEventResponse> {
    Ok(ProviderImportJobEventResponse {
        job_id: record.job_id,
        sequence: record.sequence,
        level: record.level,
        stage: record.stage,
        message: record.message,
        metadata: serde_json::from_str(&record.metadata_json)
            .map_err(|error| ApiError::Internal(error.to_string()))?,
        created_at: record.created_at,
    })
}
