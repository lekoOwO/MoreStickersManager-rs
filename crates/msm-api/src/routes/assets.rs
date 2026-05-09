use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderMap},
    response::{IntoResponse, Response},
};
use msm_domain::Permission;
use msm_storage::{
    models::{PackVisibility, SubscriptionAccessResourceType},
    AssetKey, StorageError,
};

use crate::{
    auth::{bearer_token, require_pat},
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    get,
    path = "/assets/packs/{pack_public_id}/{filename}",
    tag = "assets",
    params(
        ("pack_public_id" = String, Path, description = "Public pack ID"),
        ("filename" = String, Path, description = "Asset filename")
    ),
    responses(
        (status = 200, description = "Asset bytes"),
        (status = 404, description = "Asset not found", body = crate::error::ApiErrorBody)
    )
)]
/// Reads a local sticker asset.
///
/// # Errors
///
/// Returns an error when the asset key is invalid, the asset is missing, or storage fails.
pub async fn read_asset(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((pack_public_id, filename)): Path<(String, String)>,
) -> ApiResult<Response> {
    let key = AssetKey::new(pack_public_id, filename)
        .map_err(|_| ApiError::NotFound("Asset not found".to_owned()))?;
    require_asset_access(&state, &headers, key.pack_public_id()).await?;
    let filename = key.filename().to_owned();
    let bytes = state
        .asset_store()
        .read(&key)
        .await
        .map_err(|error| match error {
            StorageError::AssetNotFound { .. } => ApiError::NotFound("Asset not found".to_owned()),
            other => ApiError::from(other),
        })?;
    let content_type = mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string();

    Ok(([(CONTENT_TYPE, content_type)], bytes).into_response())
}

async fn require_asset_access(
    state: &ApiState,
    headers: &HeaderMap,
    pack_id: &str,
) -> ApiResult<()> {
    let Some(pack) = state.repository().find_sticker_pack_record(pack_id).await? else {
        return Ok(());
    };
    if pack.visibility == PackVisibility::Public {
        return Ok(());
    }
    if subscription_token_can_read_pack_asset(state, headers, pack_id).await? {
        return Ok(());
    }

    let pat = require_pat(headers, state, Permission::AssetRead).await?;
    pat.require_user(&pack.owner_user_id)
}

async fn subscription_token_can_read_pack_asset(
    state: &ApiState,
    headers: &HeaderMap,
    pack_id: &str,
) -> ApiResult<bool> {
    let token = match bearer_token(headers) {
        Ok(token) if token.starts_with("msm_sub_") => token,
        Ok(_) | Err(ApiError::Unauthorized(_)) => return Ok(false),
        Err(error) => return Err(error),
    };
    let record = state
        .repository()
        .verify_subscription_access_token(token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid subscription access token".to_owned()))?;

    match record.resource_type {
        SubscriptionAccessResourceType::Pack => Ok(record.resource_id == pack_id),
        SubscriptionAccessResourceType::SubscriptionGroup => {
            let pack_ids = state
                .repository()
                .list_subscription_pack_ids(&record.resource_id)
                .await?;
            Ok(pack_ids.iter().any(|candidate| candidate == pack_id))
        }
    }
}
