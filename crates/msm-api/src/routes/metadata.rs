use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::{PackAction, Permission};
use msm_storage::models::{FolderRecord, NewTag, SubscriptionGroupRecord, TagRecord};

use crate::{
    auth::{require_pat, VerifiedPat},
    dto::{
        CreateFolderRequest, CreateSubscriptionGroupRequest, CreateTagRequest, FolderPackResponse,
        FolderResponse, ListFoldersQuery, ListSubscriptionGroupsQuery, ListTagsQuery,
        PackTagResponse, SubscriptionGroupPackResponse, SubscriptionGroupResponse, TagResponse,
        UpsertPackMembershipRequest,
    },
    rbac::{require_pack_access, require_tenant_permission, require_tenant_resource_access},
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    post,
    path = "/api/v1/folders",
    tag = "metadata",
    request_body = CreateFolderRequest,
    responses(
        (status = 201, description = "Folder created", body = FolderResponse),
        (status = 401, description = "Missing or invalid PAT", body = crate::error::ApiErrorBody),
        (status = 403, description = "Missing pack.update scope or user mismatch", body = crate::error::ApiErrorBody)
    )
)]
/// Creates a pack folder for the PAT user.
///
/// # Errors
///
/// Returns an API error when authorization fails or storage fails.
pub async fn create_folder(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateFolderRequest>,
) -> ApiResult<(StatusCode, Json<FolderResponse>)> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    pat.require_user(&request.owner_user_id)?;
    require_tenant_resource_access(
        &state,
        &pat,
        &request.tenant_id,
        &request.owner_user_id,
        Permission::PackUpdate,
        "PAT user cannot create folders in this tenant",
    )
    .await?;
    let folder = state
        .repository()
        .create_folder(
            &request.id,
            &request.tenant_id,
            &request.owner_user_id,
            &request.name,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(folder.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/folders",
    tag = "metadata",
    params(ListFoldersQuery),
    responses((status = 200, description = "Folders", body = Vec<FolderResponse>))
)]
/// Lists pack folders owned by the PAT user.
///
/// # Errors
///
/// Returns an API error when authorization fails or storage fails.
pub async fn list_folders(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListFoldersQuery>,
) -> ApiResult<Json<Vec<FolderResponse>>> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    pat.require_user(&query.owner_user_id)?;
    require_tenant_resource_access(
        &state,
        &pat,
        &query.tenant_id,
        &query.owner_user_id,
        Permission::PackUpdate,
        "PAT user cannot list folders in this tenant",
    )
    .await?;
    let folders = state
        .repository()
        .list_folders(&query.tenant_id, &query.owner_user_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(folders))
}

#[utoipa::path(
    get,
    path = "/api/v1/folders/{folder_id}/packs",
    tag = "metadata",
    params(("folder_id" = String, Path, description = "Folder ID")),
    responses((status = 200, description = "Pack IDs in folder", body = Vec<String>))
)]
/// Lists pack IDs assigned to a folder owned by the PAT user.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, or storage fails.
pub async fn list_folder_pack_ids(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(folder_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    let _folder = require_folder_access(&state, &folder_id, &pat, Permission::PackUpdate).await?;
    let pack_ids = state.repository().list_folder_pack_ids(&folder_id).await?;
    Ok(Json(pack_ids))
}

#[utoipa::path(
    put,
    path = "/api/v1/folders/{folder_id}/packs/{pack_id}",
    tag = "metadata",
    params(
        ("folder_id" = String, Path, description = "Folder ID"),
        ("pack_id" = String, Path, description = "Sticker pack ID")
    ),
    request_body = UpsertPackMembershipRequest,
    responses((status = 200, description = "Folder-pack membership", body = FolderPackResponse))
)]
/// Adds or updates a pack assignment in a folder.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, tenants differ, or
/// storage fails.
pub async fn add_pack_to_folder(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((folder_id, pack_id)): Path<(String, String)>,
    Json(request): Json<UpsertPackMembershipRequest>,
) -> ApiResult<Json<FolderPackResponse>> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    let folder = require_folder_access(&state, &folder_id, &pat, Permission::PackUpdate).await?;
    let pack = require_pack_access(&state, &pat, PackAction::Update, &pack_id).await?;
    require_same_tenant(&folder.tenant_id, &pack.tenant_id)?;
    let link = state
        .repository()
        .add_pack_to_folder(&folder_id, &pack_id, request.sort_order)
        .await?;
    Ok(Json(link.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/folders/{folder_id}/packs/{pack_id}",
    tag = "metadata",
    params(
        ("folder_id" = String, Path, description = "Folder ID"),
        ("pack_id" = String, Path, description = "Sticker pack ID")
    ),
    responses((status = 204, description = "Folder-pack membership removed"))
)]
/// Removes a pack assignment from a folder.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, tenants differ,
/// storage fails, or the membership does not exist.
pub async fn remove_pack_from_folder(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((folder_id, pack_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    let folder = require_folder_access(&state, &folder_id, &pat, Permission::PackUpdate).await?;
    let pack = require_pack_access(&state, &pat, PackAction::Update, &pack_id).await?;
    require_same_tenant(&folder.tenant_id, &pack.tenant_id)?;
    let removed = state
        .repository()
        .remove_pack_from_folder(&folder_id, &pack_id)
        .await?;
    require_removed(removed)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/tags",
    tag = "metadata",
    request_body = CreateTagRequest,
    responses((status = 201, description = "Tag created", body = TagResponse))
)]
/// Creates a tenant tag.
///
/// # Errors
///
/// Returns an API error when authorization fails or storage fails.
pub async fn create_tag(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateTagRequest>,
) -> ApiResult<(StatusCode, Json<TagResponse>)> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    require_tenant_permission(
        &state,
        &pat,
        &request.tenant_id,
        Permission::PackUpdate,
        true,
        "PAT user cannot manage tags in this tenant",
    )
    .await?;
    let tag = state
        .repository()
        .create_tag(NewTag {
            id: &request.id,
            tenant_id: &request.tenant_id,
            name: &request.name,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(tag.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/tags",
    tag = "metadata",
    params(ListTagsQuery),
    responses((status = 200, description = "Tags", body = Vec<TagResponse>))
)]
/// Lists tags in one tenant.
///
/// # Errors
///
/// Returns an API error when authorization fails or storage fails.
pub async fn list_tags(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListTagsQuery>,
) -> ApiResult<Json<Vec<TagResponse>>> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    require_tenant_permission(
        &state,
        &pat,
        &query.tenant_id,
        Permission::PackUpdate,
        true,
        "PAT user cannot list tags in this tenant",
    )
    .await?;
    let tags = state
        .repository()
        .list_tags(&query.tenant_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(tags))
}

#[utoipa::path(
    get,
    path = "/api/v1/packs/{pack_id}/tags",
    tag = "metadata",
    params(("pack_id" = String, Path, description = "Sticker pack ID")),
    responses((status = 200, description = "Tag IDs assigned to pack", body = Vec<String>))
)]
/// Lists tag IDs assigned to a pack owned by the PAT user.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, or storage fails.
pub async fn list_pack_tag_ids(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(pack_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    let _pack = require_pack_access(&state, &pat, PackAction::Update, &pack_id).await?;
    let tag_ids = state.repository().list_pack_tag_ids(&pack_id).await?;
    Ok(Json(tag_ids))
}

#[utoipa::path(
    put,
    path = "/api/v1/packs/{pack_id}/tags/{tag_id}",
    tag = "metadata",
    params(
        ("pack_id" = String, Path, description = "Sticker pack ID"),
        ("tag_id" = String, Path, description = "Tag ID")
    ),
    responses((status = 200, description = "Pack-tag membership", body = PackTagResponse))
)]
/// Assigns a tag to a pack.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, tenants differ, or
/// storage fails.
pub async fn add_tag_to_pack(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((pack_id, tag_id)): Path<(String, String)>,
) -> ApiResult<Json<PackTagResponse>> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    let pack = require_pack_access(&state, &pat, PackAction::Update, &pack_id).await?;
    let tag = require_tag(&state, &tag_id).await?;
    require_same_tenant(&tag.tenant_id, &pack.tenant_id)?;
    let link = state
        .repository()
        .add_tag_to_pack(&pack_id, &tag_id)
        .await?;
    Ok(Json(link.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/packs/{pack_id}/tags/{tag_id}",
    tag = "metadata",
    params(
        ("pack_id" = String, Path, description = "Sticker pack ID"),
        ("tag_id" = String, Path, description = "Tag ID")
    ),
    responses((status = 204, description = "Pack-tag membership removed"))
)]
/// Removes a tag from a pack.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, storage fails, or
/// the membership does not exist.
pub async fn remove_tag_from_pack(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((pack_id, tag_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
    let _pack = require_pack_access(&state, &pat, PackAction::Update, &pack_id).await?;
    let removed = state
        .repository()
        .remove_tag_from_pack(&pack_id, &tag_id)
        .await?;
    require_removed(removed)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/subscription-groups",
    tag = "metadata",
    request_body = CreateSubscriptionGroupRequest,
    responses((status = 201, description = "Subscription group created", body = SubscriptionGroupResponse))
)]
/// Creates a subscription group for the PAT user.
///
/// # Errors
///
/// Returns an API error when authorization fails or storage fails.
pub async fn create_subscription_group(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateSubscriptionGroupRequest>,
) -> ApiResult<(StatusCode, Json<SubscriptionGroupResponse>)> {
    let pat = require_pat(&headers, &state, Permission::SubscriptionCreate).await?;
    pat.require_user(&request.owner_user_id)?;
    require_tenant_resource_access(
        &state,
        &pat,
        &request.tenant_id,
        &request.owner_user_id,
        Permission::SubscriptionCreate,
        "PAT user cannot create subscription groups in this tenant",
    )
    .await?;
    let group = state
        .repository()
        .create_subscription_group(
            &request.id,
            &request.tenant_id,
            &request.owner_user_id,
            &request.title,
            request.visibility.into(),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(group.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/subscription-groups",
    tag = "metadata",
    params(ListSubscriptionGroupsQuery),
    responses((status = 200, description = "Subscription groups", body = Vec<SubscriptionGroupResponse>))
)]
/// Lists subscription groups owned by the PAT user.
///
/// # Errors
///
/// Returns an API error when authorization fails or storage fails.
pub async fn list_subscription_groups(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListSubscriptionGroupsQuery>,
) -> ApiResult<Json<Vec<SubscriptionGroupResponse>>> {
    let pat = require_pat(&headers, &state, Permission::SubscriptionRead).await?;
    pat.require_user(&query.owner_user_id)?;
    require_tenant_resource_access(
        &state,
        &pat,
        &query.tenant_id,
        &query.owner_user_id,
        Permission::SubscriptionRead,
        "PAT user cannot list subscription groups in this tenant",
    )
    .await?;
    let groups = state
        .repository()
        .list_subscription_groups(&query.tenant_id, &query.owner_user_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(groups))
}

#[utoipa::path(
    get,
    path = "/api/v1/subscription-groups/{subscription_group_id}/packs",
    tag = "metadata",
    params(("subscription_group_id" = String, Path, description = "Subscription group ID")),
    responses((status = 200, description = "Pack IDs in subscription group", body = Vec<String>))
)]
/// Lists pack IDs in a subscription group owned by the PAT user.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, or storage fails.
pub async fn list_subscription_group_pack_ids(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(subscription_group_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    let pat = require_pat(&headers, &state, Permission::SubscriptionRead).await?;
    let _group = require_subscription_group_access(
        &state,
        &subscription_group_id,
        &pat,
        Permission::SubscriptionRead,
    )
    .await?;
    let pack_ids = state
        .repository()
        .list_subscription_pack_ids(&subscription_group_id)
        .await?;
    Ok(Json(pack_ids))
}

#[utoipa::path(
    put,
    path = "/api/v1/subscription-groups/{subscription_group_id}/packs/{pack_id}",
    tag = "metadata",
    params(
        ("subscription_group_id" = String, Path, description = "Subscription group ID"),
        ("pack_id" = String, Path, description = "Sticker pack ID")
    ),
    request_body = UpsertPackMembershipRequest,
    responses((status = 200, description = "Subscription group-pack membership", body = SubscriptionGroupPackResponse))
)]
/// Adds or updates a pack assignment in a subscription group.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, tenants differ, or
/// storage fails.
pub async fn add_pack_to_subscription_group(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((subscription_group_id, pack_id)): Path<(String, String)>,
    Json(request): Json<UpsertPackMembershipRequest>,
) -> ApiResult<Json<SubscriptionGroupPackResponse>> {
    let pat = require_pat(&headers, &state, Permission::SubscriptionCreate).await?;
    let group = require_subscription_group_access(
        &state,
        &subscription_group_id,
        &pat,
        Permission::SubscriptionCreate,
    )
    .await?;
    let pack = require_pack_access(&state, &pat, PackAction::Update, &pack_id).await?;
    require_same_tenant(&group.tenant_id, &pack.tenant_id)?;
    let link = state
        .repository()
        .add_pack_to_subscription_group(&subscription_group_id, &pack_id, request.sort_order)
        .await?;
    Ok(Json(link.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/subscription-groups/{subscription_group_id}/packs/{pack_id}",
    tag = "metadata",
    params(
        ("subscription_group_id" = String, Path, description = "Subscription group ID"),
        ("pack_id" = String, Path, description = "Sticker pack ID")
    ),
    responses((status = 204, description = "Subscription group-pack membership removed"))
)]
/// Removes a pack assignment from a subscription group.
///
/// # Errors
///
/// Returns an API error when authorization fails, ownership validation fails, tenants differ,
/// storage fails, or the membership does not exist.
pub async fn remove_pack_from_subscription_group(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((subscription_group_id, pack_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let pat = require_pat(&headers, &state, Permission::SubscriptionCreate).await?;
    let group = require_subscription_group_access(
        &state,
        &subscription_group_id,
        &pat,
        Permission::SubscriptionCreate,
    )
    .await?;
    let pack = require_pack_access(&state, &pat, PackAction::Update, &pack_id).await?;
    require_same_tenant(&group.tenant_id, &pack.tenant_id)?;
    let removed = state
        .repository()
        .remove_pack_from_subscription_group(&subscription_group_id, &pack_id)
        .await?;
    require_removed(removed)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn require_folder_access(
    state: &ApiState,
    folder_id: &str,
    pat: &VerifiedPat,
    required: Permission,
) -> ApiResult<FolderRecord> {
    let folder = state
        .repository()
        .find_folder_record(folder_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("folder `{folder_id}` not found")))?;
    require_tenant_resource_access(
        state,
        pat,
        &folder.tenant_id,
        &folder.owner_user_id,
        required,
        "PAT user cannot access folder",
    )
    .await?;
    Ok(folder)
}

async fn require_tag(state: &ApiState, tag_id: &str) -> ApiResult<TagRecord> {
    state
        .repository()
        .find_tag_record(tag_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("tag `{tag_id}` not found")))
}

async fn require_subscription_group_access(
    state: &ApiState,
    subscription_group_id: &str,
    pat: &VerifiedPat,
    required: Permission,
) -> ApiResult<SubscriptionGroupRecord> {
    let group = state
        .repository()
        .find_subscription_group_record(subscription_group_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "subscription group `{subscription_group_id}` not found"
            ))
        })?;
    require_tenant_resource_access(
        state,
        pat,
        &group.tenant_id,
        &group.owner_user_id,
        required,
        "PAT user cannot access subscription group",
    )
    .await?;
    Ok(group)
}

fn require_same_tenant(left: &str, right: &str) -> ApiResult<()> {
    if left == right {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "membership resources must belong to the same tenant".to_owned(),
        ))
    }
}

fn require_removed(removed: bool) -> ApiResult<()> {
    if removed {
        Ok(())
    } else {
        Err(ApiError::NotFound("membership not found".to_owned()))
    }
}
