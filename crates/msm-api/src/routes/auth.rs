use std::collections::BTreeSet;

use axum::{
    extract::{Path, Query, State},
    http::{header::SET_COOKIE, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use msm_domain::Permission;
use url::Url;

use crate::{
    dto::{
        CompleteOidcLoginRequest, CreatedPersonalAccessTokenResponse, LocalUserResponse,
        LoginLocalUserRequest, OidcLoginStartResponse, RegisterLocalUserRequest,
        StartOidcLoginQuery,
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
    create_pat_and_session_response(
        &state,
        &user.id,
        &request.token_id,
        &request.token_name,
        &scopes,
        request.expires_at.as_deref(),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/oidc/{tenant_id}/{provider_id}/login",
    tag = "auth",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("provider_id" = String, Path, description = "OIDC provider ID"),
        StartOidcLoginQuery
    ),
    responses(
        (status = 200, description = "OIDC authorization URL", body = OidcLoginStartResponse),
        (status = 404, description = "OIDC provider not found", body = crate::error::ApiErrorBody)
    )
)]
/// Starts an OIDC login by creating a one-time state and returning the provider authorization URL.
///
/// # Errors
///
/// Returns not found for unknown/disabled providers or bad request for invalid URLs.
pub async fn start_oidc_login(
    State(state): State<ApiState>,
    Path((tenant_id, provider_id)): Path<(String, String)>,
    Query(query): Query<StartOidcLoginQuery>,
) -> ApiResult<Json<OidcLoginStartResponse>> {
    let provider = state
        .repository()
        .find_oidc_provider_config(&tenant_id, &provider_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("OIDC provider not found".to_owned()))?;
    if !provider.is_enabled {
        return Err(ApiError::NotFound("OIDC provider not found".to_owned()));
    }
    Url::parse(&query.redirect_uri)
        .map_err(|error| ApiError::BadRequest(format!("invalid redirect URI: {error}")))?;
    let expires_at = (Utc::now() + Duration::minutes(10)).to_rfc3339();
    let created = state
        .repository()
        .create_oidc_login_state(&tenant_id, &provider_id, &query.redirect_uri, &expires_at)
        .await?;
    let authorization_url = build_authorization_url(
        &provider.issuer_url,
        &provider.client_id,
        &query.redirect_uri,
        &created.state,
        &created.nonce,
        provider.scopes.iter().map(String::as_str),
    )?;

    Ok(Json(OidcLoginStartResponse {
        tenant_id,
        provider_id,
        authorization_url,
        state: created.state,
        nonce: created.nonce,
        expires_at,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/oidc/callback",
    tag = "auth",
    request_body = CompleteOidcLoginRequest,
    responses(
        (status = 200, description = "OIDC login completed", body = CreatedPersonalAccessTokenResponse),
        (status = 401, description = "Invalid OIDC state", body = crate::error::ApiErrorBody),
        (status = 403, description = "OIDC registration is disabled", body = crate::error::ApiErrorBody)
    )
)]
/// Completes an OIDC login for already-validated provider claims.
///
/// # Errors
///
/// Returns unauthorized for invalid state, forbidden when registration is disabled, or storage
/// errors.
pub async fn complete_oidc_login(
    State(state): State<ApiState>,
    Json(request): Json<CompleteOidcLoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let login_state = state
        .repository()
        .verify_oidc_login_state(&request.state, &request.nonce)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid OIDC state".to_owned()))?;
    let provider = state
        .repository()
        .find_oidc_provider_config(&login_state.tenant_id, &login_state.provider_id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("OIDC provider not found".to_owned()))?;
    if !provider.is_enabled {
        return Err(ApiError::Unauthorized("OIDC provider not found".to_owned()));
    }
    validate_oidc_claims(&provider.issuer_url, &provider.client_id, &request)?;
    state
        .repository()
        .consume_oidc_login_state(&request.state, &request.nonce)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid OIDC state".to_owned()))?;
    let user_id = if let Some(link) = state
        .repository()
        .find_oidc_user_link(
            &login_state.tenant_id,
            &login_state.provider_id,
            &request.provider_subject,
        )
        .await?
    {
        link.user_id
    } else {
        if !provider.allow_registration {
            return Err(ApiError::Forbidden(
                "OIDC registration is disabled for this provider".to_owned(),
            ));
        }
        let user_id = oidc_user_id(
            &login_state.tenant_id,
            &login_state.provider_id,
            &request.provider_subject,
        );
        state
            .repository()
            .create_user(&user_id, &request.email, &request.display_name)
            .await?;
        state
            .repository()
            .add_tenant_member(&login_state.tenant_id, &user_id, "user")
            .await?;
        user_id
    };
    state
        .repository()
        .upsert_oidc_user_link(
            &login_state.tenant_id,
            &login_state.provider_id,
            &request.provider_subject,
            &user_id,
            &request.email,
            &request.display_name,
        )
        .await?;
    let scopes = parse_scopes(&request.scopes)?;
    require_user_pat_scopes_allowed(&state, &user_id, &scopes).await?;
    create_pat_and_session_response(
        &state,
        &user_id,
        &request.token_id,
        &request.token_name,
        &scopes,
        request.expires_at.as_deref(),
    )
    .await
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

async fn create_pat_and_session_response(
    state: &ApiState,
    user_id: &str,
    token_id: &str,
    token_name: &str,
    scopes: &BTreeSet<Permission>,
    expires_at: Option<&str>,
) -> ApiResult<impl IntoResponse> {
    let created = state
        .repository()
        .create_personal_access_token(token_id, user_id, token_name, scopes, expires_at)
        .await?;
    let session = state
        .repository()
        .create_web_session(token_id, user_id, expires_at)
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

fn build_authorization_url<'a>(
    issuer_url: &str,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    nonce: &str,
    scopes: impl Iterator<Item = &'a str>,
) -> ApiResult<String> {
    let mut url = Url::parse(issuer_url)
        .map_err(|error| ApiError::BadRequest(format!("invalid issuer URL: {error}")))?;
    let authorize_path = format!("{}/authorize", url.path().trim_end_matches('/'));
    url.set_path(&authorize_path);
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", &scopes.collect::<Vec<_>>().join(" "))
        .append_pair("state", state)
        .append_pair("nonce", nonce);
    Ok(url.to_string())
}

fn validate_oidc_claims(
    expected_issuer: &str,
    expected_audience: &str,
    request: &CompleteOidcLoginRequest,
) -> ApiResult<()> {
    if normalize_url_claim(&request.issuer) != normalize_url_claim(expected_issuer) {
        return Err(ApiError::Unauthorized(
            "OIDC issuer claim mismatch".to_owned(),
        ));
    }
    if request.audience != expected_audience {
        return Err(ApiError::Unauthorized(
            "OIDC audience claim mismatch".to_owned(),
        ));
    }
    Ok(())
}

fn normalize_url_claim(value: &str) -> String {
    value.trim_end_matches('/').to_owned()
}

fn oidc_user_id(tenant_id: &str, provider_id: &str, provider_subject: &str) -> String {
    let mut id = format!("oidc-{tenant_id}-{provider_id}-{provider_subject}");
    id.retain(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    id
}
