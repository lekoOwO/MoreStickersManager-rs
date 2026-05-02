use std::collections::BTreeSet;

use msm_domain::{
    evaluate_pack_access, evaluate_subscription_access, AccessContext, MemberAccess, PackAction,
    PackResource, Permission, PolicyReason, Principal, Role, SubscriptionAction,
    SubscriptionResource, Visibility,
};

#[test]
fn admin_can_update_any_pack_in_tenant() {
    let decision = evaluate_pack_access(
        &admin("tenant_1"),
        PackAction::Update,
        &private_owner_only_pack("owner_2", "tenant_1"),
        &AccessContext::default(),
    );

    assert!(decision.allowed);
    assert_eq!(decision.reason, PolicyReason::AllowedByAdmin);
}

#[test]
fn admin_cannot_cross_tenant() {
    let decision = evaluate_pack_access(
        &admin("tenant_2"),
        PackAction::Update,
        &private_owner_only_pack("owner_1", "tenant_1"),
        &AccessContext::default(),
    );

    assert!(!decision.allowed);
    assert_eq!(decision.reason, PolicyReason::DeniedCrossTenant);
}

#[test]
fn owner_can_update_own_private_pack() {
    let decision = evaluate_pack_access(
        &user("owner_1", "tenant_1"),
        PackAction::Update,
        &private_owner_only_pack("owner_1", "tenant_1"),
        &AccessContext::default(),
    );

    assert!(decision.allowed);
    assert_eq!(decision.reason, PolicyReason::AllowedByOwner);
}

#[test]
fn tenant_member_can_read_member_access_private_pack() {
    let mut pack = private_owner_only_pack("owner_1", "tenant_1");
    pack.member_access = MemberAccess::TenantMembers;

    let decision = evaluate_pack_access(
        &user("member_1", "tenant_1"),
        PackAction::Read,
        &pack,
        &AccessContext::default(),
    );

    assert!(decision.allowed);
    assert_eq!(decision.reason, PolicyReason::AllowedByTenantMember);
}

#[test]
fn tenant_member_cannot_read_owner_only_private_pack() {
    let decision = evaluate_pack_access(
        &user("member_1", "tenant_1"),
        PackAction::Read,
        &private_owner_only_pack("owner_1", "tenant_1"),
        &AccessContext::default(),
    );

    assert!(!decision.allowed);
    assert_eq!(decision.reason, PolicyReason::DeniedOwnerOnly);
}

#[test]
fn anonymous_can_read_public_pack() {
    let mut pack = private_owner_only_pack("owner_1", "tenant_1");
    pack.visibility = Visibility::Public;

    let decision = evaluate_pack_access(
        &Principal::Anonymous,
        PackAction::Read,
        &pack,
        &AccessContext::default(),
    );

    assert!(decision.allowed);
    assert_eq!(decision.reason, PolicyReason::AllowedByPublicVisibility);
}

#[test]
fn anonymous_cannot_read_private_pack() {
    let decision = evaluate_pack_access(
        &Principal::Anonymous,
        PackAction::Read,
        &private_owner_only_pack("owner_1", "tenant_1"),
        &AccessContext::default(),
    );

    assert!(!decision.allowed);
    assert_eq!(decision.reason, PolicyReason::DeniedPrivateResource);
}

#[test]
fn pat_with_asset_read_can_read_accessible_asset() {
    let mut pack = private_owner_only_pack("owner_1", "tenant_1");
    pack.member_access = MemberAccess::TenantMembers;
    let principal = Principal::PersonalAccessToken {
        user_id: "member_1".to_owned(),
        tenant_id: "tenant_1".to_owned(),
        scopes: BTreeSet::from([Permission::AssetRead]),
    };

    let decision =
        evaluate_pack_access(&principal, PackAction::ReadAsset, &pack, &AccessContext::default());

    assert!(decision.allowed);
    assert_eq!(decision.reason, PolicyReason::AllowedByPatScope);
}

#[test]
fn pat_without_asset_read_cannot_read_asset() {
    let principal = Principal::PersonalAccessToken {
        user_id: "owner_1".to_owned(),
        tenant_id: "tenant_1".to_owned(),
        scopes: BTreeSet::from([Permission::PackRead]),
    };

    let decision = evaluate_pack_access(
        &principal,
        PackAction::ReadAsset,
        &private_owner_only_pack("owner_1", "tenant_1"),
        &AccessContext::default(),
    );

    assert!(!decision.allowed);
    assert_eq!(decision.reason, PolicyReason::DeniedMissingPermission);
}

#[test]
fn pack_secret_reads_only_matching_pack() {
    let principal = Principal::PackSecret {
        pack_id: "pack_1".to_owned(),
    };

    let allowed = evaluate_pack_access(
        &principal,
        PackAction::ReadAsset,
        &private_pack_with_id("pack_1"),
        &AccessContext::default(),
    );
    let denied = evaluate_pack_access(
        &principal,
        PackAction::ReadAsset,
        &private_pack_with_id("pack_2"),
        &AccessContext::default(),
    );

    assert!(allowed.allowed);
    assert!(!denied.allowed);
}

#[test]
fn public_subscription_does_not_expose_private_pack_to_anonymous_asset_read() {
    let subscription = public_subscription();
    let mut context = AccessContext {
        subscription_group_id: Some(subscription.id.clone()),
        subscription_pack_ids: subscription.pack_ids.clone(),
    };
    context.subscription_pack_ids.insert("pack_1".to_owned());

    let decision = evaluate_pack_access(
        &Principal::Anonymous,
        PackAction::ReadAsset,
        &private_pack_with_id("pack_1"),
        &context,
    );

    assert!(!decision.allowed);
    assert_eq!(decision.reason, PolicyReason::DeniedPrivateResource);
}

#[test]
fn subscription_secret_exposes_included_private_pack_through_context_only() {
    let subscription = public_subscription();
    let principal = Principal::SubscriptionSecret {
        subscription_group_id: subscription.id.clone(),
    };
    let context = AccessContext {
        subscription_group_id: Some(subscription.id.clone()),
        subscription_pack_ids: BTreeSet::from(["pack_1".to_owned()]),
    };

    let allowed = evaluate_pack_access(
        &principal,
        PackAction::ReadAsset,
        &private_pack_with_id("pack_1"),
        &context,
    );
    let denied = evaluate_pack_access(
        &principal,
        PackAction::ReadAsset,
        &private_pack_with_id("pack_2"),
        &context,
    );

    assert!(allowed.allowed);
    assert_eq!(allowed.reason, PolicyReason::AllowedBySubscriptionSecret);
    assert!(!denied.allowed);
}

#[test]
fn subscription_secret_reads_matching_subscription() {
    let subscription = public_subscription();
    let principal = Principal::SubscriptionSecret {
        subscription_group_id: subscription.id.clone(),
    };

    let decision = evaluate_subscription_access(
        &principal,
        SubscriptionAction::Read,
        &subscription,
        &AccessContext::default(),
    );

    assert!(decision.allowed);
    assert_eq!(decision.reason, PolicyReason::AllowedBySubscriptionSecret);
}

fn admin(tenant_id: &str) -> Principal {
    Principal::User {
        user_id: "admin_1".to_owned(),
        tenant_id: tenant_id.to_owned(),
        role: Role::Admin,
        permissions: BTreeSet::new(),
    }
}

fn user(user_id: &str, tenant_id: &str) -> Principal {
    Principal::User {
        user_id: user_id.to_owned(),
        tenant_id: tenant_id.to_owned(),
        role: Role::User,
        permissions: BTreeSet::new(),
    }
}

fn private_owner_only_pack(owner_user_id: &str, tenant_id: &str) -> PackResource {
    PackResource {
        id: "pack_1".to_owned(),
        tenant_id: tenant_id.to_owned(),
        owner_user_id: owner_user_id.to_owned(),
        visibility: Visibility::Private,
        member_access: MemberAccess::OwnerOnly,
    }
}

fn private_pack_with_id(id: &str) -> PackResource {
    PackResource {
        id: id.to_owned(),
        tenant_id: "tenant_1".to_owned(),
        owner_user_id: "owner_1".to_owned(),
        visibility: Visibility::Private,
        member_access: MemberAccess::OwnerOnly,
    }
}

fn public_subscription() -> SubscriptionResource {
    SubscriptionResource {
        id: "sub_1".to_owned(),
        tenant_id: "tenant_1".to_owned(),
        owner_user_id: "owner_1".to_owned(),
        visibility: Visibility::Public,
        pack_ids: BTreeSet::from(["pack_1".to_owned()]),
    }
}
