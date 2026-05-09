use std::collections::BTreeSet;

use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use msm_domain::Permission;

use crate::{
    dto::{
        CreatedPersonalAccessTokenResponse, LocalUserResponse, LoginLocalUserRequest,
        RegisterLocalUserRequest,
    },
    rbac::require_user_pat_scopes_allowed,
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
    let tenant = if let Some(tenant_id) = request
        .tenant_id
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        let tenant = state.repository().find_tenant(tenant_id).await?;
        if tenant
            .as_ref()
            .is_some_and(|tenant| !tenant.local_registration_enabled)
        {
            return Err(ApiError::Forbidden(
                "local registration is disabled for this tenant".to_owned(),
            ));
        }
        Some((tenant_id.to_owned(), tenant))
    } else {
        None
    };

    let user = state
        .repository()
        .create_local_user_with_password(
            &request.id,
            &request.email,
            &request.display_name,
            &request.password,
        )
        .await?;
    if let Some((tenant_id, tenant)) = tenant {
        let tenant_name = request.tenant_name.as_deref().unwrap_or(&tenant_id);
        let tenant_role = request.tenant_role.as_deref().unwrap_or("admin");
        if tenant.is_none() {
            state
                .repository()
                .create_tenant(&tenant_id, tenant_name)
                .await?;
        }
        state
            .repository()
            .add_tenant_member(&tenant_id, &user.id, tenant_role)
            .await?;
    }

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
) -> ApiResult<impl IntoResponse> {
    let user = state
        .repository()
        .verify_local_user_password(&request.email, &request.password)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid local credentials".to_owned()))?;
    let scopes = parse_scopes(&request.scopes)?;
    require_user_pat_scopes_allowed(&state, &user.id, &scopes).await?;
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
    let session = state
        .repository()
        .create_web_session(&request.token_id, &user.id, request.expires_at.as_deref())
        .await?;
    let cookie = HeaderValue::from_str(&format!(
        "msm_session={}; Path=/; HttpOnly; SameSite=Lax",
        session.token
    ))
    .map_err(|_| ApiError::Internal("failed to build Web session cookie".to_owned()))?;

    Ok((
        [(SET_COOKIE, cookie)],
        Json(CreatedPersonalAccessTokenResponse::from(created)),
    ))
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
