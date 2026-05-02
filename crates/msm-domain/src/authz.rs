use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    SystemConfigure,
    TenantManageMembers,
    TenantViewAuditLog,
    PackCreate,
    PackRead,
    PackUpdate,
    PackDelete,
    PackManageAccess,
    AssetRead,
    SubscriptionCreate,
    SubscriptionRead,
    SubscriptionUpdate,
    SubscriptionDelete,
    SubscriptionManageAccess,
    ProviderImport,
    ExportRun,
    ImportRun,
    PatManage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Principal {
    Anonymous,
    User {
        user_id: String,
        tenant_id: String,
        role: Role,
        permissions: BTreeSet<Permission>,
    },
    PersonalAccessToken {
        user_id: String,
        tenant_id: String,
        scopes: BTreeSet<Permission>,
    },
    PackSecret {
        pack_id: String,
    },
    SubscriptionSecret {
        subscription_group_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberAccess {
    OwnerOnly,
    TenantMembers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackAction {
    Read,
    ReadAsset,
    Update,
    Delete,
    ManageAccess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubscriptionAction {
    Read,
    Update,
    Delete,
    ManageAccess,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackResource {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub visibility: Visibility,
    pub member_access: MemberAccess,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionResource {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub visibility: Visibility,
    pub pack_ids: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccessContext {
    pub subscription_group_id: Option<String>,
    pub subscription_pack_ids: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: PolicyReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyReason {
    AllowedByAdmin,
    AllowedByOwner,
    AllowedByTenantMember,
    AllowedByPublicVisibility,
    AllowedByPatScope,
    AllowedByPackSecret,
    AllowedBySubscriptionSecret,
    DeniedCrossTenant,
    DeniedMissingPermission,
    DeniedPrivateResource,
    DeniedOwnerOnly,
    DeniedSecretMismatch,
}

impl PolicyDecision {
    #[must_use]
    pub fn allow(reason: PolicyReason) -> Self {
        Self {
            allowed: true,
            reason,
        }
    }

    #[must_use]
    pub fn deny(reason: PolicyReason) -> Self {
        Self {
            allowed: false,
            reason,
        }
    }
}

#[must_use]
pub fn evaluate_pack_access(
    principal: &Principal,
    action: PackAction,
    resource: &PackResource,
    context: &AccessContext,
) -> PolicyDecision {
    let required = pack_permission(action);

    match principal {
        Principal::Anonymous => evaluate_anonymous_pack(action, resource),
        Principal::PackSecret { pack_id } => {
            if pack_id == &resource.id && matches!(action, PackAction::Read | PackAction::ReadAsset) {
                PolicyDecision::allow(PolicyReason::AllowedByPackSecret)
            } else {
                PolicyDecision::deny(PolicyReason::DeniedSecretMismatch)
            }
        }
        Principal::SubscriptionSecret {
            subscription_group_id,
        } => {
            let context_matches = context.subscription_group_id.as_ref() == Some(subscription_group_id)
                && context.subscription_pack_ids.contains(&resource.id);
            if context_matches && matches!(action, PackAction::Read | PackAction::ReadAsset) {
                PolicyDecision::allow(PolicyReason::AllowedBySubscriptionSecret)
            } else {
                PolicyDecision::deny(PolicyReason::DeniedSecretMismatch)
            }
        }
        Principal::User {
            user_id,
            tenant_id,
            role,
            permissions,
        } => evaluate_user_pack(
            user_id,
            tenant_id,
            role,
            permissions,
            required,
            action,
            resource,
        ),
        Principal::PersonalAccessToken {
            user_id,
            tenant_id,
            scopes,
        } => evaluate_pat_pack(user_id, tenant_id, scopes, required, action, resource),
    }
}

#[must_use]
pub fn evaluate_subscription_access(
    principal: &Principal,
    action: SubscriptionAction,
    resource: &SubscriptionResource,
    _context: &AccessContext,
) -> PolicyDecision {
    let required = subscription_permission(action);

    match principal {
        Principal::Anonymous => {
            if action == SubscriptionAction::Read && resource.visibility == Visibility::Public {
                PolicyDecision::allow(PolicyReason::AllowedByPublicVisibility)
            } else {
                PolicyDecision::deny(PolicyReason::DeniedPrivateResource)
            }
        }
        Principal::SubscriptionSecret {
            subscription_group_id,
        } => {
            if action == SubscriptionAction::Read && subscription_group_id == &resource.id {
                PolicyDecision::allow(PolicyReason::AllowedBySubscriptionSecret)
            } else {
                PolicyDecision::deny(PolicyReason::DeniedSecretMismatch)
            }
        }
        Principal::PackSecret { .. } => PolicyDecision::deny(PolicyReason::DeniedSecretMismatch),
        Principal::User {
            user_id,
            tenant_id,
            role,
            permissions,
        } => evaluate_user_subscription(user_id, tenant_id, role, permissions, required, action, resource),
        Principal::PersonalAccessToken {
            user_id,
            tenant_id,
            scopes,
        } => evaluate_pat_subscription(user_id, tenant_id, scopes, required, action, resource),
    }
}

fn evaluate_anonymous_pack(action: PackAction, resource: &PackResource) -> PolicyDecision {
    if matches!(action, PackAction::Read | PackAction::ReadAsset)
        && resource.visibility == Visibility::Public
    {
        PolicyDecision::allow(PolicyReason::AllowedByPublicVisibility)
    } else {
        PolicyDecision::deny(PolicyReason::DeniedPrivateResource)
    }
}

fn evaluate_user_pack(
    user_id: &str,
    tenant_id: &str,
    role: &Role,
    permissions: &BTreeSet<Permission>,
    required: Permission,
    action: PackAction,
    resource: &PackResource,
) -> PolicyDecision {
    if tenant_id != resource.tenant_id {
        return PolicyDecision::deny(PolicyReason::DeniedCrossTenant);
    }

    if role == &Role::Admin {
        return PolicyDecision::allow(PolicyReason::AllowedByAdmin);
    }

    if !role_allows(role, permissions, &required) {
        return PolicyDecision::deny(PolicyReason::DeniedMissingPermission);
    }

    if user_id == resource.owner_user_id {
        return PolicyDecision::allow(PolicyReason::AllowedByOwner);
    }

    if matches!(action, PackAction::Read | PackAction::ReadAsset)
        && resource.member_access == MemberAccess::TenantMembers
    {
        return PolicyDecision::allow(PolicyReason::AllowedByTenantMember);
    }

    PolicyDecision::deny(PolicyReason::DeniedOwnerOnly)
}

fn evaluate_pat_pack(
    user_id: &str,
    tenant_id: &str,
    scopes: &BTreeSet<Permission>,
    required: Permission,
    action: PackAction,
    resource: &PackResource,
) -> PolicyDecision {
    if tenant_id != resource.tenant_id {
        return PolicyDecision::deny(PolicyReason::DeniedCrossTenant);
    }

    if !scopes.contains(&required) {
        return PolicyDecision::deny(PolicyReason::DeniedMissingPermission);
    }

    if user_id == resource.owner_user_id
        || (matches!(action, PackAction::Read | PackAction::ReadAsset)
            && resource.member_access == MemberAccess::TenantMembers)
    {
        PolicyDecision::allow(PolicyReason::AllowedByPatScope)
    } else {
        PolicyDecision::deny(PolicyReason::DeniedOwnerOnly)
    }
}

fn evaluate_user_subscription(
    user_id: &str,
    tenant_id: &str,
    role: &Role,
    permissions: &BTreeSet<Permission>,
    required: Permission,
    action: SubscriptionAction,
    resource: &SubscriptionResource,
) -> PolicyDecision {
    if tenant_id != resource.tenant_id {
        return PolicyDecision::deny(PolicyReason::DeniedCrossTenant);
    }

    if role == &Role::Admin {
        return PolicyDecision::allow(PolicyReason::AllowedByAdmin);
    }

    if !role_allows(role, permissions, &required) {
        return PolicyDecision::deny(PolicyReason::DeniedMissingPermission);
    }

    if user_id == resource.owner_user_id {
        return PolicyDecision::allow(PolicyReason::AllowedByOwner);
    }

    if action == SubscriptionAction::Read && resource.visibility == Visibility::Public {
        PolicyDecision::allow(PolicyReason::AllowedByPublicVisibility)
    } else {
        PolicyDecision::deny(PolicyReason::DeniedOwnerOnly)
    }
}

fn evaluate_pat_subscription(
    user_id: &str,
    tenant_id: &str,
    scopes: &BTreeSet<Permission>,
    required: Permission,
    action: SubscriptionAction,
    resource: &SubscriptionResource,
) -> PolicyDecision {
    if tenant_id != resource.tenant_id {
        return PolicyDecision::deny(PolicyReason::DeniedCrossTenant);
    }

    if !scopes.contains(&required) {
        return PolicyDecision::deny(PolicyReason::DeniedMissingPermission);
    }

    if user_id == resource.owner_user_id
        || (action == SubscriptionAction::Read && resource.visibility == Visibility::Public)
    {
        PolicyDecision::allow(PolicyReason::AllowedByPatScope)
    } else {
        PolicyDecision::deny(PolicyReason::DeniedOwnerOnly)
    }
}

fn role_allows(role: &Role, custom_permissions: &BTreeSet<Permission>, required: &Permission) -> bool {
    match role {
        Role::Admin => true,
        Role::User => built_in_user_permissions().contains(required),
        Role::Custom(_) => custom_permissions.contains(required),
    }
}

fn pack_permission(action: PackAction) -> Permission {
    match action {
        PackAction::Read => Permission::PackRead,
        PackAction::ReadAsset => Permission::AssetRead,
        PackAction::Update => Permission::PackUpdate,
        PackAction::Delete => Permission::PackDelete,
        PackAction::ManageAccess => Permission::PackManageAccess,
    }
}

fn subscription_permission(action: SubscriptionAction) -> Permission {
    match action {
        SubscriptionAction::Read => Permission::SubscriptionRead,
        SubscriptionAction::Update => Permission::SubscriptionUpdate,
        SubscriptionAction::Delete => Permission::SubscriptionDelete,
        SubscriptionAction::ManageAccess => Permission::SubscriptionManageAccess,
    }
}

fn built_in_user_permissions() -> BTreeSet<Permission> {
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
        Permission::ExportRun,
        Permission::ImportRun,
        Permission::PatManage,
    ]
    .into_iter()
    .collect()
}
