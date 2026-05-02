use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use msm_domain::StickerPack;

use crate::{
    dto::{ImportPackRequest, ListPacksQuery},
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
    Json(request): Json<ImportPackRequest>,
) -> ApiResult<StatusCode> {
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
    Query(query): Query<ListPacksQuery>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
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
    Path(pack_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let pack = state
        .repository()
        .find_sticker_pack(&pack_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pack not found".to_owned()))?;
    serde_json::to_value(pack)
        .map(Json)
        .map_err(|error| ApiError::Internal(error.to_string()))
}
