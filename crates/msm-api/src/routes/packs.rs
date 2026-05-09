use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};

use msm_domain::{PackAction, Permission, StickerPack};

use crate::{
    auth::require_pat,
    dto::{ImportPackRequest, ListPacksQuery, UpdatePackRequest},
    rbac::require_pack_access,
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    post,
    path = "/api/v1/packs/import",
    tag = "packs",
    request_body = ImportPackRequest,
    responses(
        (status = 201, description = "Sticker pack imported"),
        (status = 400, description = "Invalid sticker pack", body = crate::error::ApiErrorBody),
        (status = 500, description = "Storage failure", body = crate::error::ApiErrorBody)
    )
)]
/// Imports a `MoreStickers` sticker pack into storage.
///
/// # Errors
///
/// Returns an error when the request pack JSON is invalid or storage fails.
pub async fn import_pack(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<ImportPackRequest>,
) -> ApiResult<StatusCode> {
    let pat = require_pat(&headers, &state, Permission::ImportRun).await?;
    pat.require_user(&request.owner_user_id)?;

    let pack: StickerPack = serde_json::from_value(request.pack)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    state
        .repository()
        .upsert_sticker_pack(
            &request.pack_id,
            &request.tenant_id,
            &request.owner_user_id,
            request.visibility.into(),
            None,
            &pack,
        )
        .await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "/api/v1/packs",
    tag = "packs",
    params(ListPacksQuery),
    responses((status = 200, description = "Owned sticker packs", body = Vec<serde_json::Value>))
)]
/// Lists sticker packs owned by a user.
///
/// # Errors
///
/// Returns an error when storage fails or a stored pack cannot be serialized.
pub async fn list_packs(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListPacksQuery>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let pat = require_pat(&headers, &state, Permission::PackRead).await?;
    pat.require_user(&query.user_id)?;

    let packs = state
        .repository()
        .list_user_sticker_packs(&query.user_id)
        .await?;
    packs
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map(Json)
        .map_err(|error| ApiError::Internal(error.to_string()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/packs/{pack_id}",
    tag = "packs",
    params(("pack_id" = String, Path, description = "Internal pack ID")),
    request_body = UpdatePackRequest,
    responses(
        (status = 200, description = "Sticker pack updated", body = serde_json::Value),
        (status = 403, description = "PAT cannot update this pack", body = crate::error::ApiErrorBody),
        (status = 404, description = "Pack not found", body = crate::error::ApiErrorBody)
    )
)]
/// Updates basic metadata for one owned sticker pack.
///
/// # Errors
///
/// Returns an error when authorization fails, the pack does not exist, or storage fails.
pub async fn update_pack(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(pack_id): Path<String>,
    Json(request): Json<UpdatePackRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    let record = require_pack_access(&state, &pat, PackAction::Update, &pack_id).await?;
    let updated = state
        .repository()
        .update_sticker_pack_metadata(
            &pack_id,
            &record.owner_user_id,
            &request.title,
            request.visibility.into(),
        )
        .await?;
    if !updated {
        return Err(ApiError::NotFound("Pack not found".to_owned()));
    }

    let pack = state
        .repository()
        .find_sticker_pack(&pack_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pack not found".to_owned()))?;
    serde_json::to_value(pack)
        .map(Json)
        .map_err(|error| ApiError::Internal(error.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/packs/{pack_id}",
    tag = "packs",
    params(("pack_id" = String, Path, description = "Internal pack ID")),
    responses(
        (status = 204, description = "Sticker pack deleted"),
        (status = 403, description = "PAT cannot delete this pack", body = crate::error::ApiErrorBody),
        (status = 404, description = "Pack not found", body = crate::error::ApiErrorBody)
    )
)]
/// Deletes one owned sticker pack.
///
/// # Errors
///
/// Returns an error when authorization fails, the pack does not exist, or storage fails.
pub async fn delete_pack(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(pack_id): Path<String>,
) -> ApiResult<StatusCode> {
    let pat = require_pat(&headers, &state, Permission::PackDelete).await?;
    let record = require_pack_access(&state, &pat, PackAction::Delete, &pack_id).await?;
    let deleted = state
        .repository()
        .delete_sticker_pack(&pack_id, &record.owner_user_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound("Pack not found".to_owned()))
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/packs/{pack_id}/stickerpack",
    tag = "packs",
    params(("pack_id" = String, Path, description = "Internal pack ID")),
    responses(
        (status = 200, description = "MoreStickers-compatible sticker pack", body = serde_json::Value),
        (status = 404, description = "Pack not found", body = crate::error::ApiErrorBody)
    )
)]
/// Exports one stored pack as a `MoreStickers` sticker pack JSON payload.
///
/// # Errors
///
/// Returns an error when the pack does not exist, storage fails, or the pack cannot be serialized.
pub async fn export_pack(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(pack_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let pat = require_pat(&headers, &state, Permission::PackRead).await?;
    require_pack_access(&state, &pat, PackAction::Read, &pack_id).await?;
    let pack = state
        .repository()
        .find_sticker_pack(&pack_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pack not found".to_owned()))?;
    serde_json::to_value(pack)
        .map(Json)
        .map_err(|error| ApiError::Internal(error.to_string()))
}
