use std::collections::BTreeMap;

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use msm_domain::{
    build_dynamic_subscription_payload, subscription_bearer_headers, Permission,
    SubscriptionPackInput, SubscriptionPayloadInput,
};
use msm_storage::models::{
    PackVisibility, StickerPackRecord, SubscriptionAccessResourceType, SubscriptionGroupRecord,
};

use crate::{
    auth::{bearer_token, optional_web_session, require_pat},
    rbac::require_tenant_resource_access,
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    get,
    path = "/api/public/packs/{pack_id}/stickerpack",
    tag = "subscriptions",
    params(("pack_id" = String, Path, description = "Internal pack ID")),
    responses(
        (status = 200, description = "MoreStickers-compatible sticker pack", body = serde_json::Value),
        (status = 401, description = "Private pack requires a valid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Pack is private and no valid credential was provided", body = crate::error::ApiErrorBody),
        (status = 404, description = "Pack not found", body = crate::error::ApiErrorBody)
    )
)]
/// Reads a pack refresh payload for dynamic `MoreStickers` subscriptions.
///
/// # Errors
///
/// Returns an API error when the pack is not public and the caller lacks an owner PAT.
pub async fn public_pack_stickerpack(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(pack_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let record = load_pack_record(&state, &pack_id).await?;
    require_pack_subscription_access(&state, &headers, &record).await?;
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
    get,
    path = "/api/public/packs/{pack_id}/subscription",
    tag = "subscriptions",
    params(("pack_id" = String, Path, description = "Internal pack ID")),
    responses(
        (status = 200, description = "Single-pack MoreStickers dynamic subscription", body = serde_json::Value),
        (status = 401, description = "Private pack requires a valid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "PAT cannot read this private pack", body = crate::error::ApiErrorBody),
        (status = 404, description = "Pack not found", body = crate::error::ApiErrorBody)
    )
)]
/// Reads a default single-pack dynamic `MoreStickers` subscription payload.
///
/// # Errors
///
/// Returns an API error when the pack is not public and the caller lacks an owner PAT.
pub async fn public_pack_subscription(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(pack_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let record = load_pack_record(&state, &pack_id).await?;
    let access = require_pack_subscription_access(&state, &headers, &record).await?;
    let base_url = public_base_url(&headers);
    let pack = state
        .repository()
        .find_sticker_pack(&pack_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pack not found".to_owned()))?;
    let payload = build_dynamic_subscription_payload(SubscriptionPayloadInput {
        id: pack_id.clone(),
        version: Some("1".to_owned()),
        title: Some(pack.title.clone()),
        author: pack.author.clone(),
        refresh_url: format!("{base_url}/api/public/packs/{pack_id}/subscription"),
        auth_headers: access.auth_headers,
        packs: vec![SubscriptionPackInput {
            pack,
            refresh_url: format!("{base_url}/api/public/packs/{pack_id}/stickerpack"),
        }],
    });

    serde_json::to_value(payload)
        .map(Json)
        .map_err(|error| ApiError::Internal(error.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/public/subscriptions/{subscription_group_id}",
    tag = "subscriptions",
    params(("subscription_group_id" = String, Path, description = "Subscription group ID")),
    responses(
        (status = 200, description = "MoreStickers dynamic subscription pack set", body = serde_json::Value),
        (status = 401, description = "Private subscription group requires a valid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Subscription group is private and no valid credential was provided", body = crate::error::ApiErrorBody),
        (status = 404, description = "Subscription group not found", body = crate::error::ApiErrorBody)
    )
)]
/// Reads a dynamic `MoreStickers` subscription group payload.
///
/// # Errors
///
/// Returns an API error when the group is not public and the caller lacks an owner PAT.
pub async fn public_subscription_group(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(subscription_group_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let group = load_subscription_group(&state, &subscription_group_id).await?;
    let access = require_subscription_access(&state, &headers, &group).await?;
    let base_url = public_base_url(&headers);
    let pack_ids = state
        .repository()
        .list_subscription_pack_ids(&subscription_group_id)
        .await?;
    let mut packs = Vec::new();

    for pack_id in pack_ids {
        let record = load_pack_record(&state, &pack_id).await?;
        if access.include_private || record.visibility == PackVisibility::Public {
            let pack = state
                .repository()
                .find_sticker_pack(&pack_id)
                .await?
                .ok_or_else(|| ApiError::NotFound(format!("pack `{pack_id}` not found")))?;
            packs.push(SubscriptionPackInput {
                pack,
                refresh_url: format!("{base_url}/api/public/packs/{pack_id}/stickerpack"),
            });
        }
    }

    let payload = build_dynamic_subscription_payload(SubscriptionPayloadInput {
        id: group.id,
        version: Some("1".to_owned()),
        title: Some(group.title),
        author: None,
        refresh_url: format!("{base_url}/api/public/subscriptions/{subscription_group_id}"),
        auth_headers: access.auth_headers,
        packs,
    });

    serde_json::to_value(payload)
        .map(Json)
        .map_err(|error| ApiError::Internal(error.to_string()))
}

async fn load_pack_record(state: &ApiState, pack_id: &str) -> ApiResult<StickerPackRecord> {
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

struct PackSubscriptionAccess {
    auth_headers: Option<BTreeMap<String, String>>,
}

struct SubscriptionGroupAccess {
    include_private: bool,
    auth_headers: Option<BTreeMap<String, String>>,
}

async fn require_pack_subscription_access(
    state: &ApiState,
    headers: &HeaderMap,
    pack: &StickerPackRecord,
) -> ApiResult<PackSubscriptionAccess> {
    if pack.visibility == PackVisibility::Public {
        return Ok(PackSubscriptionAccess { auth_headers: None });
    }

    if let Some(auth_headers) = require_subscription_token_access(
        state,
        headers,
        SubscriptionAccessResourceType::Pack,
        &pack.id,
    )
    .await?
    {
        return Ok(PackSubscriptionAccess {
            auth_headers: Some(auth_headers),
        });
    }

    if web_session_can_read_owned_resource(state, headers, &pack.tenant_id, &pack.owner_user_id)
        .await?
    {
        return Ok(PackSubscriptionAccess { auth_headers: None });
    }

    let pat = require_pat(headers, state, Permission::PackRead).await?;
    require_tenant_resource_access(
        state,
        &pat,
        &pack.tenant_id,
        &pack.owner_user_id,
        Permission::PackRead,
        "PAT user cannot read private packs in this tenant",
    )
    .await?;
    Ok(PackSubscriptionAccess { auth_headers: None })
}

async fn require_subscription_access(
    state: &ApiState,
    headers: &HeaderMap,
    group: &SubscriptionGroupRecord,
) -> ApiResult<SubscriptionGroupAccess> {
    if let Some(auth_headers) = require_subscription_token_access(
        state,
        headers,
        SubscriptionAccessResourceType::SubscriptionGroup,
        &group.id,
    )
    .await?
    {
        return Ok(SubscriptionGroupAccess {
            include_private: true,
            auth_headers: Some(auth_headers),
        });
    }

    if web_session_can_read_owned_resource(state, headers, &group.tenant_id, &group.owner_user_id)
        .await?
    {
        return Ok(SubscriptionGroupAccess {
            include_private: true,
            auth_headers: None,
        });
    }

    if pat_can_read_owned_subscription(state, headers, group).await? {
        return Ok(SubscriptionGroupAccess {
            include_private: true,
            auth_headers: None,
        });
    }

    if group.visibility == PackVisibility::Public {
        return Ok(SubscriptionGroupAccess {
            include_private: false,
            auth_headers: None,
        });
    }

    let pat = require_pat(headers, state, Permission::SubscriptionRead).await?;
    require_tenant_resource_access(
        state,
        &pat,
        &group.tenant_id,
        &group.owner_user_id,
        Permission::SubscriptionRead,
        "PAT user cannot read private subscription groups in this tenant",
    )
    .await?;
    Ok(SubscriptionGroupAccess {
        include_private: true,
        auth_headers: None,
    })
}

async fn require_subscription_token_access(
    state: &ApiState,
    headers: &HeaderMap,
    resource_type: SubscriptionAccessResourceType,
    resource_id: &str,
) -> ApiResult<Option<BTreeMap<String, String>>> {
    let token = match bearer_token(headers) {
        Ok(token) if token.starts_with("msm_sub_") => token,
        Ok(_) | Err(ApiError::Unauthorized(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    let record = state
        .repository()
        .verify_subscription_access_token(token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid subscription access token".to_owned()))?;

    if record.resource_type != resource_type || record.resource_id != resource_id {
        return Err(ApiError::Forbidden(
            "subscription access token resource mismatch".to_owned(),
        ));
    }

    Ok(Some(subscription_bearer_headers(token)))
}

async fn web_session_can_read_owned_resource(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &str,
    owner_user_id: &str,
) -> ApiResult<bool> {
    let Some(session) = optional_web_session(headers, state).await? else {
        return Ok(false);
    };
    session.require_user(owner_user_id)?;
    user_has_tenant_membership(state, tenant_id, owner_user_id).await
}

async fn pat_can_read_owned_subscription(
    state: &ApiState,
    headers: &HeaderMap,
    group: &SubscriptionGroupRecord,
) -> ApiResult<bool> {
    let token = match bearer_token(headers) {
        Ok(token) if !token.starts_with("msm_sub_") => token,
        Ok(_) | Err(ApiError::Unauthorized(_)) => return Ok(false),
        Err(error) => return Err(error),
    };
    let Some(record) = state
        .repository()
        .verify_personal_access_token(token)
        .await?
    else {
        return Ok(false);
    };

    if record.user_id != group.owner_user_id
        || !record.scopes.contains(&Permission::SubscriptionRead)
    {
        return Ok(false);
    }

    user_has_tenant_membership(state, &group.tenant_id, &record.user_id).await
}

async fn user_has_tenant_membership(
    state: &ApiState,
    tenant_id: &str,
    user_id: &str,
) -> ApiResult<bool> {
    Ok(state
        .repository()
        .find_tenant_member(tenant_id, user_id)
        .await?
        .is_some())
}

fn public_base_url(headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("http");
    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("127.0.0.1:3000");
    format!("{scheme}://{}", host.trim_end_matches('/'))
}
