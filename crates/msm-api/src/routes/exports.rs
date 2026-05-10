use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::{PackAction, Permission};
use msm_exporters::{ExportCapabilities, ExportTarget, ExportTargetKind, MoreStickersExportTarget};
use msm_storage::models::{
    ExportJobEventRecord, ExportJobRecord, ExportTargetRecord, TelegramPublicationRecord,
};
use msm_storage::models::{NewExportJob, NewExportJobEvent, NewExportTarget};

use crate::{
    auth::require_pat,
    dto::{
        CreateExportJobRequest, CreateExportTargetRequest, ExportJobEventResponse,
        ExportJobResponse, ExportTargetKindResponse, ExportTargetResponse, ListExportTargetsQuery,
        ListTelegramPublicationsQuery, TelegramPublicationResponse, UpdateExportTargetRequest,
    },
    rbac::{require_pack_access, require_tenant_permission, require_tenant_resource_access},
    ApiError, ApiResult, ApiState,
};

const REDACTED_SECRET: &str = "<redacted>";

/// Lists supported export target kinds and their capabilities.
///
/// # Errors
///
/// Returns an API error when the PAT is missing required scope.
#[utoipa::path(
    get,
    path = "/api/v1/export-target-kinds",
    tag = "exports",
    responses(
        (status = 200, description = "Export target capabilities", body = [ExportTargetKindResponse]),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.read scope", body = crate::error::ApiErrorBody)
    )
)]
pub async fn list_target_kinds(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ExportTargetKindResponse>>> {
    let _pat = require_pat(&headers, &state, Permission::ExportRead).await?;
    Ok(Json(export_target_capabilities()))
}

/// Lists configured export targets for one tenant.
///
/// # Errors
///
/// Returns an API error when the PAT is missing required scope or storage fails.
#[utoipa::path(
    get,
    path = "/api/v1/export-targets",
    tag = "exports",
    params(ListExportTargetsQuery),
    responses(
        (status = 200, description = "Configured export targets", body = [ExportTargetResponse]),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.read scope", body = crate::error::ApiErrorBody)
    )
)]
pub async fn list_targets(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListExportTargetsQuery>,
) -> ApiResult<Json<Vec<ExportTargetResponse>>> {
    let pat = require_pat(&headers, &state, Permission::ExportRead).await?;
    require_tenant_permission(
        &state,
        &pat,
        &query.tenant_id,
        Permission::ExportRead,
        true,
        "PAT user cannot access export targets for this tenant",
    )
    .await?;
    let targets = state
        .repository()
        .list_export_targets(&query.tenant_id)
        .await?;

    targets
        .into_iter()
        .map(export_target_response)
        .collect::<ApiResult<Vec<_>>>()
        .map(Json)
}

/// Creates a configured export target.
///
/// # Errors
///
/// Returns an API error when the PAT lacks manage scope, the target kind is unknown, config
/// serialization fails, or storage fails.
#[utoipa::path(
    post,
    path = "/api/v1/export-targets",
    tag = "exports",
    request_body = CreateExportTargetRequest,
    responses(
        (status = 201, description = "Export target created", body = ExportTargetResponse),
        (status = 400, description = "Invalid target request", body = crate::error::ApiErrorBody),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.target.manage scope", body = crate::error::ApiErrorBody)
    )
)]
pub async fn create_target(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateExportTargetRequest>,
) -> ApiResult<(StatusCode, Json<ExportTargetResponse>)> {
    let pat = require_pat(&headers, &state, Permission::ExportTargetManage).await?;
    require_tenant_permission(
        &state,
        &pat,
        &request.tenant_id,
        Permission::ExportTargetManage,
        false,
        "PAT user cannot manage export targets for this tenant",
    )
    .await?;
    validate_target_kind(&request.kind)?;
    let config_json = serde_json::to_string(&request.config)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let target = state
        .repository()
        .create_export_target(NewExportTarget {
            id: &request.id,
            tenant_id: &request.tenant_id,
            kind: &request.kind,
            name: &request.name,
            config_json: &config_json,
            is_enabled: request.is_enabled,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(export_target_response(target)?)))
}

/// Updates a configured export target.
///
/// # Errors
///
/// Returns an API error when the PAT lacks manage scope, the target does not exist, config
/// serialization fails, or storage fails.
#[utoipa::path(
    patch,
    path = "/api/v1/export-targets/{target_id}",
    tag = "exports",
    request_body = UpdateExportTargetRequest,
    params(("target_id" = String, Path, description = "Export target ID")),
    responses(
        (status = 200, description = "Export target updated", body = ExportTargetResponse),
        (status = 400, description = "Invalid target request", body = crate::error::ApiErrorBody),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.target.manage scope", body = crate::error::ApiErrorBody),
        (status = 404, description = "Export target not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn update_target(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(target_id): Path<String>,
    Json(request): Json<UpdateExportTargetRequest>,
) -> ApiResult<Json<ExportTargetResponse>> {
    let pat = require_pat(&headers, &state, Permission::ExportTargetManage).await?;
    let existing = state
        .repository()
        .find_export_target(&target_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("export target not found: {target_id}")))?;
    require_tenant_permission(
        &state,
        &pat,
        &existing.tenant_id,
        Permission::ExportTargetManage,
        false,
        "PAT user cannot manage export targets for this tenant",
    )
    .await?;
    validate_target_kind(&existing.kind)?;
    let config_json = serde_json::to_string(&request.config)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let updated = state
        .repository()
        .update_export_target(&target_id, &request.name, &config_json, request.is_enabled)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("export target not found: {target_id}")))?;

    export_target_response(updated).map(Json)
}

/// Deletes a configured export target.
///
/// # Errors
///
/// Returns an API error when the PAT lacks manage scope, the target does not exist, or storage
/// fails.
#[utoipa::path(
    delete,
    path = "/api/v1/export-targets/{target_id}",
    tag = "exports",
    params(("target_id" = String, Path, description = "Export target ID")),
    responses(
        (status = 204, description = "Export target deleted"),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.target.manage scope", body = crate::error::ApiErrorBody),
        (status = 404, description = "Export target not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn delete_target(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(target_id): Path<String>,
) -> ApiResult<StatusCode> {
    let pat = require_pat(&headers, &state, Permission::ExportTargetManage).await?;
    let existing = state
        .repository()
        .find_export_target(&target_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("export target not found: {target_id}")))?;
    require_tenant_permission(
        &state,
        &pat,
        &existing.tenant_id,
        Permission::ExportTargetManage,
        false,
        "PAT user cannot manage export targets for this tenant",
    )
    .await?;
    let deleted = state.repository().delete_export_target(&target_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(format!(
            "export target not found: {target_id}"
        )))
    }
}

/// Queues an export job.
///
/// # Errors
///
/// Returns an API error when the PAT lacks run scope, the source pack/target is invalid, the PAT
/// user does not own the pack, request serialization fails, or storage fails.
#[utoipa::path(
    post,
    path = "/api/v1/export-jobs",
    tag = "exports",
    request_body = CreateExportJobRequest,
    responses(
        (status = 201, description = "Export job queued", body = ExportJobResponse),
        (status = 400, description = "Invalid export job request", body = crate::error::ApiErrorBody),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.run scope or pack ownership", body = crate::error::ApiErrorBody),
        (status = 404, description = "Pack or target not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn create_job(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateExportJobRequest>,
) -> ApiResult<(StatusCode, Json<ExportJobResponse>)> {
    let pat = require_pat(&headers, &state, Permission::ExportRun).await?;
    let pack = require_pack_access(&state, &pat, PackAction::Read, &request.source_pack_id).await?;
    if pack.tenant_id != request.tenant_id {
        return Err(ApiError::BadRequest(
            "source pack tenant does not match request tenant".to_owned(),
        ));
    }

    let target = state
        .repository()
        .find_export_target(&request.target_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("export target not found: {}", request.target_id))
        })?;
    if target.tenant_id != request.tenant_id {
        return Err(ApiError::BadRequest(
            "export target tenant does not match request tenant".to_owned(),
        ));
    }
    if !target.is_enabled {
        return Err(ApiError::BadRequest("export target is disabled".to_owned()));
    }

    let request_json =
        serde_json::to_string(&request).map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let job = state
        .repository()
        .create_export_job(NewExportJob {
            id: &request.id,
            tenant_id: &request.tenant_id,
            owner_user_id: &pat.user_id,
            source_pack_id: &request.source_pack_id,
            target_id: &request.target_id,
            request_json: &request_json,
            max_attempts: 3,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(export_job_response(job)?)))
}

/// Reads one export job.
///
/// # Errors
///
/// Returns an API error when the PAT lacks read scope, the job does not exist, the PAT user does not
/// own the job, stored JSON is invalid, or storage fails.
#[utoipa::path(
    get,
    path = "/api/v1/export-jobs/{job_id}",
    tag = "exports",
    params(("job_id" = String, Path, description = "Export job ID")),
    responses(
        (status = 200, description = "Export job", body = ExportJobResponse),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.read scope or job ownership", body = crate::error::ApiErrorBody),
        (status = 404, description = "Export job not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn get_job(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> ApiResult<Json<ExportJobResponse>> {
    let pat = require_pat(&headers, &state, Permission::ExportRead).await?;
    let job = load_accessible_job(&state, &job_id, &pat).await?;
    export_job_response(job).map(Json)
}

/// Requeues a failed or cancelled export job for operator recovery.
///
/// # Errors
///
/// Returns an API error when the PAT lacks run scope, the job does not exist, the PAT user does not
/// have run access to the job owner resource, the job is not recoverable, or storage fails.
#[utoipa::path(
    post,
    path = "/api/v1/export-jobs/{job_id}/requeue",
    tag = "exports",
    params(("job_id" = String, Path, description = "Export job ID")),
    responses(
        (status = 200, description = "Export job requeued", body = ExportJobResponse),
        (status = 400, description = "Export job is not in a recoverable state", body = crate::error::ApiErrorBody),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.run scope or job ownership", body = crate::error::ApiErrorBody),
        (status = 404, description = "Export job not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn requeue_job(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> ApiResult<Json<ExportJobResponse>> {
    let pat = require_pat(&headers, &state, Permission::ExportRun).await?;
    let job = load_job_for_permission(&state, &job_id, &pat, Permission::ExportRun).await?;
    let requeued = state
        .repository()
        .requeue_export_job_for_recovery(&job.id)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(
                "export job must be failed or cancelled before it can be requeued".to_owned(),
            )
        })?;
    append_requeue_event(&state, &job.id).await?;
    export_job_response(requeued).map(Json)
}

/// Lists ordered events for one export job.
///
/// # Errors
///
/// Returns an API error when the PAT lacks read scope, the job does not exist, the PAT user does not
/// own the job, stored event metadata is invalid, or storage fails.
#[utoipa::path(
    get,
    path = "/api/v1/export-jobs/{job_id}/events",
    tag = "exports",
    params(("job_id" = String, Path, description = "Export job ID")),
    responses(
        (status = 200, description = "Export job events", body = [ExportJobEventResponse]),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.read scope or job ownership", body = crate::error::ApiErrorBody),
        (status = 404, description = "Export job not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn list_job_events(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> ApiResult<Json<Vec<ExportJobEventResponse>>> {
    let pat = require_pat(&headers, &state, Permission::ExportRead).await?;
    let _job = load_accessible_job(&state, &job_id, &pat).await?;
    let events = state.repository().list_export_job_events(&job_id).await?;

    events
        .into_iter()
        .map(export_job_event_response)
        .collect::<ApiResult<Vec<_>>>()
        .map(Json)
}

/// Lists Telegram publications for one source pack.
///
/// # Errors
///
/// Returns an API error when the PAT lacks read scope, the pack does not exist, the PAT user does
/// not own the pack, or storage fails.
#[utoipa::path(
    get,
    path = "/api/v1/telegram-publications",
    tag = "exports",
    params(ListTelegramPublicationsQuery),
    responses(
        (status = 200, description = "Telegram publications for a pack", body = [TelegramPublicationResponse]),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.read scope or pack ownership", body = crate::error::ApiErrorBody),
        (status = 404, description = "Pack not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn list_telegram_publications(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListTelegramPublicationsQuery>,
) -> ApiResult<Json<Vec<TelegramPublicationResponse>>> {
    let pat = require_pat(&headers, &state, Permission::ExportRead).await?;
    let _pack = require_pack_access(&state, &pat, PackAction::Read, &query.pack_id).await?;
    let publications = state
        .repository()
        .list_telegram_publications_for_pack(&query.pack_id)
        .await?;

    Ok(Json(
        publications
            .into_iter()
            .map(telegram_publication_response)
            .collect(),
    ))
}

/// Reads one Telegram publication by ID.
///
/// # Errors
///
/// Returns an API error when the PAT lacks read scope, the publication does not exist, the PAT user
/// does not own the source pack, or storage fails.
#[utoipa::path(
    get,
    path = "/api/v1/telegram-publications/{publication_id}",
    tag = "exports",
    params(("publication_id" = String, Path, description = "Telegram publication ID")),
    responses(
        (status = 200, description = "Telegram publication", body = TelegramPublicationResponse),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing export.read scope or pack ownership", body = crate::error::ApiErrorBody),
        (status = 404, description = "Telegram publication not found", body = crate::error::ApiErrorBody)
    )
)]
pub async fn get_telegram_publication(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(publication_id): Path<String>,
) -> ApiResult<Json<TelegramPublicationResponse>> {
    let pat = require_pat(&headers, &state, Permission::ExportRead).await?;
    let publication = state
        .repository()
        .find_telegram_publication(&publication_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Telegram publication not found: {publication_id}"))
        })?;
    let _pack = require_pack_access(&state, &pat, PackAction::Read, &publication.pack_id).await?;

    Ok(Json(telegram_publication_response(publication)))
}

fn export_target_capabilities() -> Vec<ExportTargetKindResponse> {
    let mut capabilities = vec![
        MoreStickersExportTarget.capabilities(),
        ExportCapabilities {
            kind: ExportTargetKind::new("telegram"),
            display_name: "Telegram".to_owned(),
            supports_remote_publication: true,
            supports_media_conversion: true,
            requires_credentials: true,
        },
    ];
    capabilities.sort_by(|left, right| left.kind.cmp(&right.kind));
    capabilities
        .into_iter()
        .map(|capability| ExportTargetKindResponse {
            kind: capability.kind.as_str().to_owned(),
            display_name: capability.display_name,
            supports_remote_publication: capability.supports_remote_publication,
            supports_media_conversion: capability.supports_media_conversion,
            requires_credentials: capability.requires_credentials,
        })
        .collect()
}

fn validate_target_kind(kind: &str) -> ApiResult<()> {
    if export_target_capabilities()
        .iter()
        .any(|capability| capability.kind == kind)
    {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "unknown export target kind: {kind}"
        )))
    }
}

fn export_target_response(record: ExportTargetRecord) -> ApiResult<ExportTargetResponse> {
    Ok(ExportTargetResponse {
        id: record.id,
        tenant_id: record.tenant_id,
        kind: record.kind,
        name: record.name,
        config: redacted_config(&record.config_json)?,
        is_enabled: record.is_enabled,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

fn export_job_response(record: ExportJobRecord) -> ApiResult<ExportJobResponse> {
    Ok(ExportJobResponse {
        id: record.id,
        tenant_id: record.tenant_id,
        owner_user_id: record.owner_user_id,
        source_pack_id: record.source_pack_id,
        target_id: record.target_id,
        status: record.status.as_str().to_owned(),
        request: parse_json_value(&record.request_json)?,
        result: record
            .result_json
            .as_deref()
            .map(parse_json_value)
            .transpose()?,
        error_summary: record.error_summary,
        attempt_count: record.attempt_count,
        max_attempts: record.max_attempts,
        next_attempt_at: record.next_attempt_at,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

fn export_job_event_response(record: ExportJobEventRecord) -> ApiResult<ExportJobEventResponse> {
    Ok(ExportJobEventResponse {
        job_id: record.job_id,
        sequence: record.sequence,
        level: record.level,
        stage: record.stage,
        message: record.message,
        metadata: parse_json_value(&record.metadata_json)?,
        created_at: record.created_at,
    })
}

fn telegram_publication_response(record: TelegramPublicationRecord) -> TelegramPublicationResponse {
    TelegramPublicationResponse {
        id: record.id,
        pack_id: record.pack_id,
        target_id: record.target_id,
        job_id: record.job_id,
        sticker_set_name: record.sticker_set_name,
        sticker_set_url: record.sticker_set_url,
        sticker_count: record.sticker_count,
        sticker_type: record.sticker_type,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

async fn load_accessible_job(
    state: &ApiState,
    job_id: &str,
    pat: &crate::auth::VerifiedPat,
) -> ApiResult<ExportJobRecord> {
    let job = state
        .repository()
        .find_export_job(job_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("export job not found: {job_id}")))?;
    require_tenant_resource_access(
        state,
        pat,
        &job.tenant_id,
        &job.owner_user_id,
        Permission::ExportRead,
        "PAT user cannot access this export job",
    )
    .await?;
    Ok(job)
}

async fn load_job_for_permission(
    state: &ApiState,
    job_id: &str,
    pat: &crate::auth::VerifiedPat,
    permission: Permission,
) -> ApiResult<ExportJobRecord> {
    let job = state
        .repository()
        .find_export_job(job_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("export job not found: {job_id}")))?;
    require_tenant_resource_access(
        state,
        pat,
        &job.tenant_id,
        &job.owner_user_id,
        permission,
        "PAT user cannot access this export job",
    )
    .await?;
    Ok(job)
}

async fn append_requeue_event(state: &ApiState, job_id: &str) -> ApiResult<()> {
    let event_count = state
        .repository()
        .list_export_job_events(job_id)
        .await?
        .len();
    let sequence = i64::try_from(event_count)
        .map_err(|_| ApiError::BadRequest("export job event sequence overflow".to_owned()))?
        + 1;
    state
        .repository()
        .append_export_job_event(NewExportJobEvent {
            job_id,
            sequence,
            level: "info",
            stage: "requeued",
            message: "export job requeued for recovery",
            metadata_json: "{}",
        })
        .await?;
    Ok(())
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
