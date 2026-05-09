use std::collections::BTreeSet;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::Permission;

use crate::{
    auth::require_pat,
    dto::{
        CreatePersonalAccessTokenRequest, CreatedPersonalAccessTokenResponse,
        ListPersonalAccessTokensQuery, PersonalAccessTokenResponse,
    },
    rbac::require_user_pat_scopes_allowed,
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    post,
    path = "/api/v1/pats",
    tag = "pats",
    request_body = CreatePersonalAccessTokenRequest,
    responses(
        (status = 201, description = "Personal Access Token created", body = CreatedPersonalAccessTokenResponse),
        (status = 400, description = "Invalid PAT request", body = crate::error::ApiErrorBody),
        (status = 500, description = "Storage failure", body = crate::error::ApiErrorBody)
    )
)]
/// Creates a Personal Access Token and returns the raw token once.
///
/// # Errors
///
/// Returns an error when scope keys are invalid or storage fails.
pub async fn create_pat(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreatePersonalAccessTokenRequest>,
) -> ApiResult<(StatusCode, Json<CreatedPersonalAccessTokenResponse>)> {
    let pat = require_pat(&headers, &state, Permission::PatManage).await?;
    pat.require_user(&request.user_id)?;
    let scopes = parse_scopes(&request.scopes)?;
    require_user_pat_scopes_allowed(&state, &request.user_id, &scopes).await?;
    let created = state
        .repository()
        .create_personal_access_token(
            &request.id,
            &request.user_id,
            &request.name,
            &scopes,
            request.expires_at.as_deref(),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedPersonalAccessTokenResponse::from(created)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/pats",
    tag = "pats",
    params(ListPersonalAccessTokensQuery),
    responses((status = 200, description = "Personal Access Tokens", body = Vec<PersonalAccessTokenResponse>))
)]
/// Lists Personal Access Tokens for a user without token secrets or hashes.
///
/// # Errors
///
/// Returns an error when storage fails.
pub async fn list_pats(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListPersonalAccessTokensQuery>,
) -> ApiResult<Json<Vec<PersonalAccessTokenResponse>>> {
    let pat = require_pat(&headers, &state, Permission::PatManage).await?;
    pat.require_user(&query.user_id)?;
    let tokens = state
        .repository()
        .list_personal_access_tokens(&query.user_id)
        .await?
        .into_iter()
        .map(PersonalAccessTokenResponse::from)
        .collect();
    Ok(Json(tokens))
}

#[utoipa::path(
    delete,
    path = "/api/v1/pats/{token_id}",
    tag = "pats",
    params(("token_id" = String, Path, description = "Personal Access Token ID")),
    responses((status = 204, description = "Personal Access Token revoked"))
)]
/// Revokes a Personal Access Token by ID.
///
/// # Errors
///
/// Returns an error when storage fails.
pub async fn revoke_pat(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> ApiResult<StatusCode> {
    let pat = require_pat(&headers, &state, Permission::PatManage).await?;
    let record = state
        .repository()
        .find_personal_access_token(&token_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("PAT `{token_id}` not found")))?;
    pat.require_user(&record.user_id)?;
    state
        .repository()
        .revoke_personal_access_token(&token_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn parse_scopes(scopes: &[String]) -> ApiResult<BTreeSet<Permission>> {
    scopes
        .iter()
        .map(|scope| {
            Permission::from_key(scope)
                .ok_or_else(|| ApiError::BadRequest(format!("unknown PAT scope `{scope}`")))
        })
        .collect()
}
