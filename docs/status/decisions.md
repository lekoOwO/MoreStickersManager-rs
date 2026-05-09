# Decisions

## 2026-05-02: Phase-Based Delivery

MSM is implemented in small phases. P0/P1 builds foundation and compatibility only. Later phases require their own specs and plans.

## 2026-05-02: Domain Crate Boundary

`msm-domain` owns compatibility models and pure helper logic. It does not depend on API, database, provider SDK, or frontend crates.

## 2026-05-02: External Format Stability

MoreStickers-compatible JSON is the external contract. Internal data may become richer later, but exports must preserve compatibility.

## 2026-05-06: Provider And Export Target Separation

Providers are input-side normalizers. Export targets are output-side serializers
or remote publishers. Telegram can exist on both sides, but Telegram import must
stay in provider code while Telegram sticker set creation belongs to the planned
export pipeline.

## 2026-05-09: Pack, Subscription, And Asset Access Model

Public packs can be read anonymously through pack refresh, single-pack
subscription, and asset URLs. Private packs require one of:

- an owner PAT with the route-specific scope such as `pack.read` or
  `asset.read`;
- an owner Web session cookie issued by local/OIDC login;
- a matching pack subscription secret for read-only pack refresh,
  single-pack subscription, and asset reads;
- a subscription-group secret for read-only access to packs included in that
  group.

Public subscription groups can be read anonymously, but anonymous reads include
only public packs. Private subscription groups require an owner PAT with
`subscription.read`, an owner Web session cookie, or the matching group
subscription secret. Owner PAT/Web-session reads and group-secret reads include
private packs in the group.

Subscription secrets are bearer credentials for read/refresh only. They never
grant management actions, PAT creation, role administration, pack mutation, or
subscription-link rotation/revocation. PATs remain user-owned and scope-bound;
future tenant admin delegation must route through the domain RBAC evaluator
instead of weakening owner checks.

## 2026-05-09: Resource Ownership RBAC Delegation

Owner-only API checks should move through shared authorization helpers before
admin delegation is added. A valid PAT scope remains necessary for the route,
but non-owner access to tenant-owned resources is decided by tenant membership:
same-tenant admins may manage resources, custom roles may manage tenant
metadata resources when their role template contains the required permission,
and built-in regular users remain limited to resources they own unless a
specific future sharing model says otherwise.
