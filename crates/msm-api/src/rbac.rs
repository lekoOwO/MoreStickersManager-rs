use std::collections::BTreeSet;

use msm_domain::{
    evaluate_pack_access, AccessContext, MemberAccess, PackAction, PackResource, Permission,
    PolicyReason, Principal, Role, Visibility,
};
use msm_storage::models::{PackVisibility, StickerPackRecord};

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

    let (role, permissions) = tenant_role_permissions(state, tenant_id, &pat.user_id).await?;
    if role_allows_tenant_resource(&role, &permissions, required) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(denied_message.to_owned()))
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

fn role_allows_tenant_resource(
    role: &Role,
    permissions: &BTreeSet<Permission>,
    required: Permission,
) -> bool {
    match role {
        Role::Admin => true,
        Role::User => false,
        Role::Custom(_) => permissions.contains(&required),
    }
}

fn pack_visibility(visibility: &PackVisibility) -> Visibility {
    match visibility {
        PackVisibility::Public => Visibility::Public,
        PackVisibility::Private => Visibility::Private,
    }
}
