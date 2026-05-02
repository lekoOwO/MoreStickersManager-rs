use axum::{
    extract::{Path, State},
    http::header::CONTENT_TYPE,
    response::{IntoResponse, Response},
};
use msm_storage::{AssetKey, StorageError};

use crate::{ApiError, ApiResult, ApiState};

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
pub async fn read_asset(
    State(state): State<ApiState>,
    Path((pack_public_id, filename)): Path<(String, String)>,
) -> ApiResult<Response> {
    let key = AssetKey::new(pack_public_id, filename)
        .map_err(|_| ApiError::NotFound("Asset not found".to_owned()))?;
    let filename = key.filename().to_owned();
    let bytes = state.asset_store().read(&key).await.map_err(|error| match error {
        StorageError::AssetNotFound { .. } => ApiError::NotFound("Asset not found".to_owned()),
        other => ApiError::from(other),
    })?;
    let content_type = mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string();

    Ok(([(CONTENT_TYPE, content_type)], bytes).into_response())
}
