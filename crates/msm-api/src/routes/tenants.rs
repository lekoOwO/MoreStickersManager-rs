use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::Permission;
use std::collections::BTreeSet;

use crate::{
    auth::require_pat,
    dto::{
        TenantMemberResponse, TenantRoleResponse, TenantSettingsResponse, TenantUserResponse,
        UpdateTenantSettingsRequest, UpdateTenantUserStatusRequest, UpsertTenantMemberRequest,
        UpsertTenantRoleRequest,
    },
    ApiError, ApiResult, ApiState,
};

#[utoipa::path(
    get,
    path = "/api/v1/tenants/{tenant_id}/members",
    tag = "tenants",
    params(("tenant_id" = String, Path, description = "Tenant ID")),
    responses(
        (status = 200, description = "Tenant members", body = [TenantMemberResponse]),
        (status = 403, description = "Not a tenant admin", body = crate::error::ApiErrorBody)
    )
)]
/// Lists members in a tenant.
///
/// # Errors
///
/// Returns an API error when the caller lacks tenant admin access or storage fails.
pub async fn list_tenant_members(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<Vec<TenantMemberResponse>>> {
    require_tenant_admin(
        &state,
        &headers,
        &tenant_id,
        Permission::TenantManageMembers,
    )
    .await?;
    let members = state.repository().list_tenant_members(&tenant_id).await?;
    Ok(Json(
        members
            .into_iter()
            .map(TenantMemberResponse::from)
            .collect(),
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/tenants/{tenant_id}/members/{user_id}",
    tag = "tenants",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("user_id" = String, Path, description = "User ID")
    ),
    request_body = UpsertTenantMemberRequest,
    responses(
        (status = 200, description = "Tenant member updated", body = TenantMemberResponse),
        (status = 400, description = "Invalid role", body = crate::error::ApiErrorBody),
        (status = 403, description = "Not a tenant admin", body = crate::error::ApiErrorBody)
    )
)]
/// Adds or updates a tenant member role.
///
/// # Errors
///
/// Returns an API error when the caller lacks tenant admin access, the role is invalid, or storage
/// fails.
pub async fn upsert_tenant_member(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, user_id)): Path<(String, String)>,
    Json(request): Json<UpsertTenantMemberRequest>,
) -> ApiResult<(StatusCode, Json<TenantMemberResponse>)> {
    require_tenant_admin(
        &state,
        &headers,
        &tenant_id,
        Permission::TenantManageMembers,
    )
    .await?;
    let role = normalize_role(&request.role)?;
    let member = state
        .repository()
        .upsert_tenant_member(&tenant_id, &user_id, role)
        .await?;

    Ok((StatusCode::OK, Json(TenantMemberResponse::from(member))))
}

#[utoipa::path(
    get,
    path = "/api/v1/tenants/{tenant_id}/settings",
    tag = "tenants",
    params(("tenant_id" = String, Path, description = "Tenant ID")),
    responses(
        (status = 200, description = "Tenant settings", body = TenantSettingsResponse),
        (status = 403, description = "Not a tenant admin", body = crate::error::ApiErrorBody),
        (status = 404, description = "Tenant not found", body = crate::error::ApiErrorBody)
    )
)]
/// Reads editable tenant settings.
///
/// # Errors
///
/// Returns an API error when the caller lacks tenant admin access, the tenant does not exist, or
/// storage fails.
pub async fn get_tenant_settings(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<TenantSettingsResponse>> {
    require_tenant_admin(
        &state,
        &headers,
        &tenant_id,
        Permission::TenantManageSettings,
    )
    .await?;
    let tenant = state
        .repository()
        .find_tenant(&tenant_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("tenant not found".to_owned()))?;

    Ok(Json(TenantSettingsResponse::from(tenant)))
}

#[utoipa::path(
    put,
    path = "/api/v1/tenants/{tenant_id}/settings",
    tag = "tenants",
    params(("tenant_id" = String, Path, description = "Tenant ID")),
    request_body = UpdateTenantSettingsRequest,
    responses(
        (status = 200, description = "Tenant settings updated", body = TenantSettingsResponse),
        (status = 400, description = "Invalid tenant settings", body = crate::error::ApiErrorBody),
        (status = 403, description = "Not a tenant admin", body = crate::error::ApiErrorBody),
        (status = 404, description = "Tenant not found", body = crate::error::ApiErrorBody)
    )
)]
/// Replaces editable tenant settings.
///
/// # Errors
///
/// Returns an API error when the caller lacks tenant admin access, input is invalid, the tenant
/// does not exist, or storage fails.
pub async fn update_tenant_settings(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
    Json(request): Json<UpdateTenantSettingsRequest>,
) -> ApiResult<(StatusCode, Json<TenantSettingsResponse>)> {
    require_tenant_admin(
        &state,
        &headers,
        &tenant_id,
        Permission::TenantManageSettings,
    )
    .await?;
    let name = normalize_tenant_name(&request.name)?;
    let public_asset_url = normalize_public_asset_url(request.public_asset_url.as_deref())?;
    let tenant = state
        .repository()
        .update_tenant_settings(
            &tenant_id,
            name,
            public_asset_url,
            request.local_registration_enabled,
        )
        .await?;

    Ok((StatusCode::OK, Json(TenantSettingsResponse::from(tenant))))
}

#[utoipa::path(
    put,
    path = "/api/v1/tenants/{tenant_id}/users/{user_id}/status",
    tag = "tenants",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("user_id" = String, Path, description = "User ID")
    ),
    request_body = UpdateTenantUserStatusRequest,
    responses(
        (status = 200, description = "Tenant user status updated", body = TenantUserResponse),
        (status = 403, description = "Not a tenant admin", body = crate::error::ApiErrorBody),
        (status = 404, description = "Tenant user not found", body = crate::error::ApiErrorBody)
    )
)]
/// Enables or disables a user that belongs to the tenant.
///
/// # Errors
///
/// Returns an API error when the caller lacks tenant admin access, the target user is not in the
/// tenant, or storage fails.
pub async fn update_tenant_user_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, user_id)): Path<(String, String)>,
    Json(request): Json<UpdateTenantUserStatusRequest>,
) -> ApiResult<(StatusCode, Json<TenantUserResponse>)> {
    require_tenant_admin(&state, &headers, &tenant_id, Permission::TenantManageUsers).await?;
    state
        .repository()
        .find_tenant_member(&tenant_id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("tenant user not found".to_owned()))?;
    let user = state
        .repository()
        .set_user_disabled(&user_id, request.is_disabled)
        .await?;

    Ok((StatusCode::OK, Json(TenantUserResponse::from(user))))
}

#[utoipa::path(
    get,
    path = "/api/v1/tenants/{tenant_id}/roles",
    tag = "tenants",
    params(("tenant_id" = String, Path, description = "Tenant ID")),
    responses(
        (status = 200, description = "Tenant role templates", body = [TenantRoleResponse]),
        (status = 403, description = "Not a tenant admin", body = crate::error::ApiErrorBody)
    )
)]
/// Lists tenant role templates.
///
/// # Errors
///
/// Returns an API error when the caller lacks tenant admin access or storage fails.
pub async fn list_tenant_roles(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<Vec<TenantRoleResponse>>> {
    require_tenant_admin(&state, &headers, &tenant_id, Permission::TenantManageRoles).await?;
    let roles = state.repository().list_role_templates(&tenant_id).await?;
    Ok(Json(
        roles.into_iter().map(TenantRoleResponse::from).collect(),
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/tenants/{tenant_id}/roles/{role_id}",
    tag = "tenants",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("role_id" = String, Path, description = "Role template ID")
    ),
    request_body = UpsertTenantRoleRequest,
    responses(
        (status = 200, description = "Tenant role template upserted", body = TenantRoleResponse),
        (status = 400, description = "Invalid role template", body = crate::error::ApiErrorBody),
        (status = 403, description = "Not a tenant admin", body = crate::error::ApiErrorBody)
    )
)]
/// Adds or updates a tenant role template.
///
/// # Errors
///
/// Returns an API error when the caller lacks tenant admin access, input is invalid, or storage
/// fails.
pub async fn upsert_tenant_role(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, role_id)): Path<(String, String)>,
    Json(request): Json<UpsertTenantRoleRequest>,
) -> ApiResult<(StatusCode, Json<TenantRoleResponse>)> {
    require_tenant_admin(&state, &headers, &tenant_id, Permission::TenantManageRoles).await?;
    let name = normalize_role_template_name(&request.name)?;
    let permissions = normalize_permissions(&request.permissions)?;
    let role = state
        .repository()
        .upsert_role_template(&role_id, &tenant_id, name, &permissions)
        .await?;

    Ok((StatusCode::OK, Json(TenantRoleResponse::from(role))))
}

async fn require_tenant_admin(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &str,
    required: Permission,
) -> ApiResult<()> {
    let pat = require_pat(headers, state, required).await?;
    let member = state
        .repository()
        .find_tenant_member(tenant_id, &pat.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("tenant admin membership required".to_owned()))?;
    if member.role == "admin" {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "tenant admin membership required".to_owned(),
        ))
    }
}

fn normalize_tenant_name(name: &str) -> ApiResult<&str> {
    let name = name.trim();
    if name.is_empty() {
        Err(ApiError::BadRequest(
            "tenant name must not be empty".to_owned(),
        ))
    } else {
        Ok(name)
    }
}

fn normalize_public_asset_url(value: Option<&str>) -> ApiResult<Option<&str>> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.starts_with("https://") || value.starts_with("http://") {
                Ok(value)
            } else {
                Err(ApiError::BadRequest(
                    "public asset URL must start with http:// or https://".to_owned(),
                ))
            }
        })
        .transpose()
}

fn normalize_role_template_name(name: &str) -> ApiResult<&str> {
    let name = name.trim();
    if name.is_empty() {
        Err(ApiError::BadRequest(
            "role template name must not be empty".to_owned(),
        ))
    } else {
        Ok(name)
    }
}

fn normalize_permissions(permissions: &[String]) -> ApiResult<BTreeSet<Permission>> {
    permissions
        .iter()
        .map(|key| {
            Permission::from_key(key)
                .ok_or_else(|| ApiError::BadRequest(format!("unknown permission `{key}`")))
        })
        .collect()
}

fn normalize_role(role: &str) -> ApiResult<&'static str> {
    match role {
        "admin" => Ok("admin"),
        "user" => Ok("user"),
        _ => Err(ApiError::BadRequest(
            "tenant member role must be `admin` or `user`".to_owned(),
        )),
    }
}
