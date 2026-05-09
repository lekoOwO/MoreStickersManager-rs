use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_domain::Permission;

use crate::{
    auth::require_pat,
    dto::{TenantMemberResponse, UpsertTenantMemberRequest},
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
    require_tenant_admin(&state, &headers, &tenant_id).await?;
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
    require_tenant_admin(&state, &headers, &tenant_id).await?;
    let role = normalize_role(&request.role)?;
    let member = state
        .repository()
        .upsert_tenant_member(&tenant_id, &user_id, role)
        .await?;

    Ok((StatusCode::OK, Json(TenantMemberResponse::from(member))))
}

async fn require_tenant_admin(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &str,
) -> ApiResult<()> {
    let pat = require_pat(headers, state, Permission::TenantManageMembers).await?;
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

fn normalize_role(role: &str) -> ApiResult<&'static str> {
    match role {
        "admin" => Ok("admin"),
        "user" => Ok("user"),
        _ => Err(ApiError::BadRequest(
            "tenant member role must be `admin` or `user`".to_owned(),
        )),
    }
}
