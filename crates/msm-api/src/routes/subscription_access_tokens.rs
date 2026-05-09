use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::Permission;
use msm_storage::models::{
    StickerPackRecord, SubscriptionAccessResourceType, SubscriptionAccessTokenRecord,
    SubscriptionGroupRecord,
};

use crate::{
    auth::require_pat,
    dto::{
        CreateSubscriptionAccessTokenRequest, CreatedSubscriptionAccessTokenResponse,
        ListSubscriptionAccessTokensQuery, SubscriptionAccessResourceTypeDto,
        SubscriptionAccessTokenResponse,
    },
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    post,
    path = "/api/v1/subscription-access-tokens",
    tag = "subscription-access-tokens",
    request_body = CreateSubscriptionAccessTokenRequest,
    responses(
        (status = 201, description = "Subscription access token created", body = CreatedSubscriptionAccessTokenResponse),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "PAT cannot manage this resource link", body = crate::error::ApiErrorBody),
        (status = 404, description = "Resource not found", body = crate::error::ApiErrorBody)
    )
)]
/// Creates a pack or subscription-group link token and returns the raw secret once.
///
/// # Errors
///
/// Returns an API error when the caller lacks the resource manage-access scope or ownership.
pub async fn create_subscription_access_token(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateSubscriptionAccessTokenRequest>,
) -> ApiResult<(StatusCode, Json<CreatedSubscriptionAccessTokenResponse>)> {
    let (tenant_id, owner_user_id, resource_type) = authorize_create(
        &state,
        &headers,
        request.resource_type,
        &request.resource_id,
    )
    .await?;
    let created = state
        .repository()
        .create_subscription_access_token(
            &request.id,
            &tenant_id,
            &owner_user_id,
            resource_type,
            &request.resource_id,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedSubscriptionAccessTokenResponse::from(created)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/subscription-access-tokens",
    tag = "subscription-access-tokens",
    params(ListSubscriptionAccessTokensQuery),
    responses((status = 200, description = "Subscription access token metadata", body = Vec<SubscriptionAccessTokenResponse>))
)]
/// Lists subscription access token metadata for a user without raw secrets or hashes.
///
/// # Errors
///
/// Returns an API error when the caller lacks subscription access management rights.
pub async fn list_subscription_access_tokens(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListSubscriptionAccessTokensQuery>,
) -> ApiResult<Json<Vec<SubscriptionAccessTokenResponse>>> {
    let pat = require_pat(&headers, &state, Permission::SubscriptionManageAccess).await?;
    pat.require_user(&query.user_id)?;
    let tokens = state
        .repository()
        .list_subscription_access_tokens(&query.user_id)
        .await?
        .into_iter()
        .map(SubscriptionAccessTokenResponse::from)
        .collect();
    Ok(Json(tokens))
}

#[utoipa::path(
    patch,
    path = "/api/v1/subscription-access-tokens/{token_id}/rotate",
    tag = "subscription-access-tokens",
    params(("token_id" = String, Path, description = "Subscription access token ID")),
    responses(
        (status = 200, description = "Subscription access token rotated", body = CreatedSubscriptionAccessTokenResponse),
        (status = 404, description = "Subscription access token not found", body = crate::error::ApiErrorBody)
    )
)]
/// Rotates a subscription access token and returns the new raw secret once.
///
/// # Errors
///
/// Returns an API error when the token does not exist or the caller lacks ownership.
pub async fn rotate_subscription_access_token(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> ApiResult<Json<CreatedSubscriptionAccessTokenResponse>> {
    let record = load_subscription_access_token(&state, &token_id).await?;
    authorize_existing(&state, &headers, &record).await?;
    let rotated = state
        .repository()
        .rotate_subscription_access_token(&token_id)
        .await?;
    Ok(Json(CreatedSubscriptionAccessTokenResponse::from(rotated)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/subscription-access-tokens/{token_id}",
    tag = "subscription-access-tokens",
    params(("token_id" = String, Path, description = "Subscription access token ID")),
    responses((status = 204, description = "Subscription access token revoked"))
)]
/// Revokes a subscription access token.
///
/// # Errors
///
/// Returns an API error when the token does not exist or the caller lacks ownership.
pub async fn revoke_subscription_access_token(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> ApiResult<StatusCode> {
    let record = load_subscription_access_token(&state, &token_id).await?;
    authorize_existing(&state, &headers, &record).await?;
    state
        .repository()
        .revoke_subscription_access_token(&token_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn authorize_create(
    state: &ApiState,
    headers: &HeaderMap,
    resource_type: SubscriptionAccessResourceTypeDto,
    resource_id: &str,
) -> ApiResult<(String, String, SubscriptionAccessResourceType)> {
    match resource_type {
        SubscriptionAccessResourceTypeDto::Pack => {
            let pat = require_pat(headers, state, Permission::PackManageAccess).await?;
            let record = load_pack(state, resource_id).await?;
            pat.require_user(&record.owner_user_id)?;
            Ok((
                record.tenant_id,
                record.owner_user_id,
                SubscriptionAccessResourceType::Pack,
            ))
        }
        SubscriptionAccessResourceTypeDto::SubscriptionGroup => {
            let pat = require_pat(headers, state, Permission::SubscriptionManageAccess).await?;
            let record = load_subscription_group(state, resource_id).await?;
            pat.require_user(&record.owner_user_id)?;
            Ok((
                record.tenant_id,
                record.owner_user_id,
                SubscriptionAccessResourceType::SubscriptionGroup,
            ))
        }
    }
}

async fn authorize_existing(
    state: &ApiState,
    headers: &HeaderMap,
    record: &SubscriptionAccessTokenRecord,
) -> ApiResult<()> {
    let permission = match record.resource_type {
        SubscriptionAccessResourceType::Pack => Permission::PackManageAccess,
        SubscriptionAccessResourceType::SubscriptionGroup => Permission::SubscriptionManageAccess,
    };
    let pat = require_pat(headers, state, permission).await?;
    pat.require_user(&record.owner_user_id)
}

async fn load_subscription_access_token(
    state: &ApiState,
    token_id: &str,
) -> ApiResult<SubscriptionAccessTokenRecord> {
    state
        .repository()
        .find_subscription_access_token(token_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("subscription access token `{token_id}` not found"))
        })
}

async fn load_pack(state: &ApiState, pack_id: &str) -> ApiResult<StickerPackRecord> {
    state
        .repository()
        .find_sticker_pack_record(pack_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("pack `{pack_id}` not found")))
}

async fn load_subscription_group(
    state: &ApiState,
    subscription_group_id: &str,
) -> ApiResult<SubscriptionGroupRecord> {
    state
        .repository()
        .find_subscription_group_record(subscription_group_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "subscription group `{subscription_group_id}` not found"
            ))
        })
}
