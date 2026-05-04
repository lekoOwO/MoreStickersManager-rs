use std::collections::BTreeSet;

use axum::{extract::State, http::StatusCode, Json};
use msm_domain::Permission;

use crate::{
    dto::{
        CreatedPersonalAccessTokenResponse, LocalUserResponse, LoginLocalUserRequest,
        RegisterLocalUserRequest,
    },
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    post,
    path = "/api/v1/auth/local/register",
    tag = "auth",
    request_body = RegisterLocalUserRequest,
    responses(
        (status = 201, description = "Local user registered", body = LocalUserResponse),
        (status = 400, description = "Invalid registration", body = crate::error::ApiErrorBody)
    )
)]
/// Registers a local user with a password credential.
///
/// # Errors
///
/// Returns an error when storage rejects the user or password credential.
pub async fn register_local_user(
    State(state): State<ApiState>,
    Json(request): Json<RegisterLocalUserRequest>,
) -> ApiResult<(StatusCode, Json<LocalUserResponse>)> {
    let user = state
        .repository()
        .create_local_user_with_password(
            &request.id,
            &request.email,
            &request.display_name,
            &request.password,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(LocalUserResponse::from(user))))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/local/login",
    tag = "auth",
    request_body = LoginLocalUserRequest,
    responses(
        (status = 200, description = "Login succeeded", body = CreatedPersonalAccessTokenResponse),
        (status = 401, description = "Invalid credentials", body = crate::error::ApiErrorBody)
    )
)]
/// Verifies local credentials and returns a newly created PAT.
///
/// # Errors
///
/// Returns unauthorized for invalid credentials, bad request for invalid scopes, or storage errors.
pub async fn login_local_user(
    State(state): State<ApiState>,
    Json(request): Json<LoginLocalUserRequest>,
) -> ApiResult<Json<CreatedPersonalAccessTokenResponse>> {
    let user = state
        .repository()
        .verify_local_user_password(&request.email, &request.password)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid local credentials".to_owned()))?;
    let scopes = parse_scopes(&request.scopes)?;
    let created = state
        .repository()
        .create_personal_access_token(
            &request.token_id,
            &user.id,
            &request.token_name,
            &scopes,
            request.expires_at.as_deref(),
        )
        .await?;

    Ok(Json(CreatedPersonalAccessTokenResponse::from(created)))
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
