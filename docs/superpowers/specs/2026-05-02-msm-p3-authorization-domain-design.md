# MSM P3 Authorization Domain Design

Date: 2026-05-02
Phase: P3

## Purpose

P3 defines the provider-neutral authorization model used by future API, CLI, MCP, and Web UI layers. The goal is to make pack visibility, subscription group visibility, tenant membership, role permissions, PAT scopes, and anonymous public access explicit and testable before HTTP endpoints exist.

## Scope

In scope:
- Permission key catalog.
- Built-in roles: admin and user.
- Principal model: anonymous, user session, PAT, pack secret, subscription secret.
- Resource visibility model for sticker packs and subscription groups.
- Policy decision type with allow/deny reason.
- Pure policy evaluator in `msm-domain`.
- Unit tests for admin, regular owner/member, PAT, anonymous public, anonymous private, pack secret, and subscription secret access.
- Documentation for pack/subscription visibility interactions.

Out of scope:
- Password login.
- OIDC.
- PAT generation, hashing, persistence, and rotation.
- HTTP middleware.
- Audit log writes.
- UI permission rendering.

## Permission Catalog

Permission keys are stable strings. Storage and API layers may persist these strings directly.

Initial P3 keys:
- `system.configure`
- `tenant.manage_members`
- `tenant.view_audit_log`
- `pack.create`
- `pack.read`
- `pack.update`
- `pack.delete`
- `pack.manage_access`
- `asset.read`
- `subscription.create`
- `subscription.read`
- `subscription.update`
- `subscription.delete`
- `subscription.manage_access`
- `provider.import`
- `export.run`
- `import.run`
- `pat.manage`

Built-in admin role includes all permissions. Built-in user role includes:
- `pack.create`
- `pack.read`
- `pack.update` for owned packs
- `pack.delete` for owned packs
- `asset.read` for accessible packs
- `subscription.create`
- `subscription.read`
- `subscription.update` for owned subscription groups
- `subscription.delete` for owned subscription groups
- `provider.import`
- `export.run`
- `import.run`
- `pat.manage`

## Principal Model

`Principal` variants:
- `Anonymous`
- `User { user_id, tenant_id, role, permissions }`
- `PersonalAccessToken { user_id, tenant_id, scopes }`
- `PackSecret { pack_id }`
- `SubscriptionSecret { subscription_group_id }`

`Role` variants:
- `Admin`
- `User`
- `Custom(String)`

PAT scopes use the same strings as permission keys. A PAT never gains more permissions than its scopes. Later P10 can also intersect PAT scopes with the owning user's current role.

## Resource Model

Sticker pack resource:
- `id`
- `tenant_id`
- `owner_user_id`
- `visibility`: public/private
- `member_access`: owner only or tenant members

Subscription group resource:
- `id`
- `tenant_id`
- `owner_user_id`
- `visibility`: public/private
- `pack_ids`

Visibility:
- Public pack metadata and assets can be read anonymously.
- Private pack metadata and assets require direct authorization.
- Public subscription group metadata can be read anonymously.
- A public subscription group does not make private pack assets globally public.
- A subscription secret grants read access to the subscription group and included pack metadata/assets through that subscription context only.

## Policy Rules

Admin:
- Admin user principal allows every permission inside its tenant.

Owner:
- User owner may read/update/delete/manage access for owned packs and subscription groups.

Tenant member:
- User principal may read member-access packs in the same tenant.
- User principal may not read owner-only private packs owned by another user.

Anonymous:
- Anonymous may read public packs and public subscription groups.
- Anonymous may not read private packs or private subscription groups.

PAT:
- PAT allows an action only if the scope contains the required permission and the tenant matches.
- PAT resource ownership rules mirror regular user behavior for owner-only operations in P3 only when `user_id` matches the owner.

Pack secret:
- Pack secret grants `pack.read` and `asset.read` only for that pack.

Subscription secret:
- Subscription secret grants `subscription.read` for that group.
- Subscription secret grants `pack.read` and `asset.read` for packs included in that group when the request context names the subscription group.

## API Shape

P3 adds pure domain functions:

```rust
pub fn evaluate_pack_access(
    principal: &Principal,
    action: PackAction,
    resource: &PackResource,
    context: &AccessContext,
) -> PolicyDecision

pub fn evaluate_subscription_access(
    principal: &Principal,
    action: SubscriptionAction,
    resource: &SubscriptionResource,
    context: &AccessContext,
) -> PolicyDecision
```

`PolicyDecision` includes:
- `allowed: bool`
- `reason: PolicyReason`

Reasons are stable enough for tests and logs, but UI wording should be added later.

## Testing

P3 tests live in `crates/msm-domain/tests/authorization.rs`.

Required cases:
- admin can update any pack in tenant;
- admin cannot cross tenant;
- owner can update own private pack;
- tenant member can read member-access private pack;
- tenant member cannot read owner-only private pack;
- anonymous can read public pack;
- anonymous cannot read private pack;
- PAT with `asset.read` can read accessible asset;
- PAT without `asset.read` cannot read asset;
- pack secret reads only matching pack;
- public subscription does not expose private pack to anonymous asset read;
- subscription secret exposes included private pack through subscription context only.

## Documentation Updates

Update:
- `docs/dev/architecture.md` with policy module placement.
- `docs/dev/compatibility.md` only if export behavior changes, which P3 should not do.
- `docs/agents/project-map.md` with authorization module.
- `docs/agents/testing.md` with `cargo test -p msm-domain authorization`.
- `docs/status/current.md` and checkpoints.

## Design Decisions

1. Authorization is pure domain logic before middleware exists.
2. Public subscription groups do not implicitly publicize private pack assets.
3. Secrets are principal types, not magic bypass flags.
4. PAT scopes use the same stable permission key catalog as RBAC.
5. Cross-tenant access is denied even for admin unless future system-superadmin design explicitly adds it.
