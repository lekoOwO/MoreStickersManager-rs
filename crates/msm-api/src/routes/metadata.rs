use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::Permission;
use msm_storage::models::NewTag;

use crate::{
    auth::require_pat,
    dto::{
        CreateFolderRequest, CreateSubscriptionGroupRequest, CreateTagRequest, FolderResponse,
        ListFoldersQuery, ListSubscriptionGroupsQuery, ListTagsQuery, SubscriptionGroupResponse,
        TagResponse,
    },
    ApiResult, ApiState,
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
    let _pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
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
    let _pat = require_pat(&headers, &state, Permission::PackUpdate).await?;
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
    let groups = state
        .repository()
        .list_subscription_groups(&query.tenant_id, &query.owner_user_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(groups))
}
