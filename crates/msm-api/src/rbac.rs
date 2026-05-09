use std::collections::BTreeSet;

use msm_domain::{
    evaluate_pack_access, evaluate_subscription_access, AccessContext, MemberAccess, PackAction,
    PackResource, Permission, PolicyReason, Principal, Role, SubscriptionAction,
    SubscriptionResource, Visibility,
};
use msm_storage::models::{PackVisibility, StickerPackRecord, SubscriptionGroupRecord};

use crate::{auth::VerifiedPat, ApiError, ApiResult, ApiState};

pub async fn require_pack_access(
    state: &ApiState,
    pat: &VerifiedPat,
    action: PackAction,
    pack_id: &str,
) -> ApiResult<StickerPackRecord> {
    let record = state
        .repository()
        .find_sticker_pack_record(pack_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pack not found".to_owned()))?;
    let (role, permissions) =
        tenant_role_permissions(state, &record.tenant_id, &pat.user_id).await?;
    let principal = Principal::User {
        user_id: pat.user_id.clone(),
        tenant_id: record.tenant_id.clone(),
        role,
        permissions,
    };
    let decision = evaluate_pack_access(
        &principal,
        action,
        &PackResource {
            id: record.id.clone(),
            tenant_id: record.tenant_id.clone(),
            owner_user_id: record.owner_user_id.clone(),
            visibility: pack_visibility(&record.visibility),
            member_access: MemberAccess::OwnerOnly,
        },
        &AccessContext::default(),
    );

    if decision.allowed {
        Ok(record)
    } else if decision.reason == PolicyReason::DeniedCrossTenant {
        Err(ApiError::NotFound("Pack not found".to_owned()))
    } else {
        Err(ApiError::Forbidden(format!(
            "pack access denied: {:?}",
            decision.reason
        )))
    }
}

pub async fn require_tenant_resource_access(
    state: &ApiState,
    pat: &VerifiedPat,
    tenant_id: &str,
    owner_user_id: &str,
    required: Permission,
    denied_message: &'static str,
) -> ApiResult<()> {
    if pat.user_id == owner_user_id {
        return Ok(());
    }

    require_tenant_permission(state, pat, tenant_id, required, false, denied_message).await
}

pub async fn require_subscription_group_access(
    state: &ApiState,
    pat: &VerifiedPat,
    action: SubscriptionAction,
    subscription_group_id: &str,
) -> ApiResult<SubscriptionGroupRecord> {
    let record = state
        .repository()
        .find_subscription_group_record(subscription_group_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "subscription group `{subscription_group_id}` not found"
            ))
        })?;
    let (role, permissions) =
        tenant_role_permissions(state, &record.tenant_id, &pat.user_id).await?;
    let principal = Principal::User {
        user_id: pat.user_id.clone(),
        tenant_id: record.tenant_id.clone(),
        role,
        permissions,
    };
    let decision = evaluate_subscription_access(
        &principal,
        action,
        &SubscriptionResource {
            id: record.id.clone(),
            tenant_id: record.tenant_id.clone(),
            owner_user_id: record.owner_user_id.clone(),
            visibility: pack_visibility(&record.visibility),
            pack_ids: BTreeSet::new(),
        },
        &AccessContext::default(),
    );

    if decision.allowed {
        Ok(record)
    } else if decision.reason == PolicyReason::DeniedCrossTenant {
        Err(ApiError::NotFound(format!(
            "subscription group `{subscription_group_id}` not found"
        )))
    } else {
        Err(ApiError::Forbidden(format!(
            "subscription group access denied: {:?}",
            decision.reason
        )))
    }
}

pub async fn require_tenant_permission(
    state: &ApiState,
    pat: &VerifiedPat,
    tenant_id: &str,
    required: Permission,
    allow_regular_user: bool,
    denied_message: &'static str,
) -> ApiResult<()> {
    let (role, permissions) = tenant_role_permissions(state, tenant_id, &pat.user_id).await?;
    if role_allows_tenant_permission(&role, &permissions, required, allow_regular_user) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(denied_message.to_owned()))
    }
}

pub async fn require_user_pat_scopes_allowed(
    state: &ApiState,
    user_id: &str,
    requested: &BTreeSet<Permission>,
) -> ApiResult<()> {
    let allowed = allowed_pat_scopes_for_user(state, user_id).await?;
    let denied = requested
        .iter()
        .find(|permission| !allowed.contains(permission));
    if let Some(permission) = denied {
        Err(ApiError::Forbidden(format!(
            "PAT scope `{}` is not allowed for this user's role",
            permission.as_key()
        )))
    } else {
        Ok(())
    }
}

async fn tenant_role_permissions(
    state: &ApiState,
    tenant_id: &str,
    user_id: &str,
) -> ApiResult<(Role, BTreeSet<Permission>)> {
    let Some(member) = state
        .repository()
        .find_tenant_member(tenant_id, user_id)
        .await?
    else {
        return Ok((Role::Custom("missing".to_owned()), BTreeSet::new()));
    };

    match member.role.as_str() {
        "admin" => Ok((Role::Admin, BTreeSet::new())),
        "user" => Ok((Role::User, BTreeSet::new())),
        role_id => {
            let permissions = state
                .repository()
                .find_role_template(tenant_id, role_id)
                .await?
                .map(|role| role.permissions)
                .unwrap_or_default();
            Ok((Role::Custom(role_id.to_owned()), permissions))
        }
    }
}

pub async fn allowed_pat_scopes_for_user(
    state: &ApiState,
    user_id: &str,
) -> ApiResult<BTreeSet<Permission>> {
    let mut allowed = built_in_user_pat_permissions();
    let memberships = state.repository().list_user_tenant_members(user_id).await?;

    for member in memberships {
        match member.role.as_str() {
            "admin" => allowed.extend(tenant_admin_pat_permissions()),
            "user" => {}
            role_id => {
                if let Some(role) = state
                    .repository()
                    .find_role_template(&member.tenant_id, role_id)
                    .await?
                {
                    allowed.extend(role.permissions);
                }
            }
        }
    }

    Ok(allowed)
}

fn built_in_user_pat_permissions() -> BTreeSet<Permission> {
    [
        Permission::PackCreate,
        Permission::PackRead,
        Permission::PackUpdate,
        Permission::PackDelete,
        Permission::AssetRead,
        Permission::SubscriptionCreate,
        Permission::SubscriptionRead,
        Permission::SubscriptionUpdate,
        Permission::SubscriptionDelete,
        Permission::ProviderImport,
        Permission::ExportRead,
        Permission::ExportRun,
        Permission::ExportTargetManage,
        Permission::ImportRun,
        Permission::PatManage,
    ]
    .into_iter()
    .collect()
}

fn tenant_admin_pat_permissions() -> BTreeSet<Permission> {
    [
        Permission::TenantManageMembers,
        Permission::TenantManageSettings,
        Permission::TenantManageUsers,
        Permission::TenantManageRoles,
        Permission::TenantViewAuditLog,
        Permission::PackManageAccess,
        Permission::SubscriptionManageAccess,
    ]
    .into_iter()
    .collect()
}

fn role_allows_tenant_permission(
    role: &Role,
    permissions: &BTreeSet<Permission>,
    required: Permission,
    allow_regular_user: bool,
) -> bool {
    match role {
        Role::Admin => true,
        Role::User => allow_regular_user,
        Role::Custom(_) => permissions.contains(&required),
    }
}

fn pack_visibility(visibility: &PackVisibility) -> Visibility {
    match visibility {
        PackVisibility::Public => Visibility::Public,
        PackVisibility::Private => Visibility::Private,
    }
}
