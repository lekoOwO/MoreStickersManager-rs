# Roadmap

Last updated: 2026-05-10.

This file is the short handoff view. Use `current.md` for the detailed
chronological log, `implementation-matrix.md` for the feature truth table, and
`../PRD.md` for the living completion roadmap.

## Current Focus

Telegram export has moved past basic dry-run and publication. The P33
reconciliation usability slice is functionally covered across Web, API, CLI,
and MCP; the active handoff focus is tenant/RBAC administration.

Web, CLI, and MCP now expose dry-run, reconciliation mode,
execute-reconciliation, and destructive mirror guard controls without requiring
hand-written worker JSON. OpenAPI now documents the target-specific
`TelegramExportJobOptions` schema behind the generic export job options object.

## Recently Completed

- Web desktop/mobile UX was reworked into a native-feeling desktop rail plus a
  separate compact mobile layout.
- Development startup can bootstrap a local account, PAT, and sample pack so
  the Web UI runs against the live API by default in the development profile.
- Telegram publication jobs can run through teloxide when `dryRun:false` is
  explicitly requested and the target contains bot credentials.
- Telegram publication and reconciliation mutation jobs persist durable
  publication records and refresh MSM source sticker to Telegram file mappings.
- Telegram reconciliation can derive remote state from fetched Telegram metadata
  plus stored mappings when callers omit `remoteSet`.
- Mirror-mode destructive replace/delete remains guarded by
  `allowDestructiveReconciliation:true`.
- CLI export job creation has Telegram-specific flags for live publication,
  reconciliation mode, execution, set-name slug, default emoji, and destructive
  mirror opt-in.
- MCP `msm.create_export_job` has Telegram-specific named fields for the same
  reconciliation options while preserving raw `options` for advanced callers.
- API/OpenAPI exposes a typed `TelegramExportJobOptions` schema for the generic
  export job options object.
- Destructive mirror operation now has a user-facing runbook.
- Product-data storage repository primitives now exist for folders, tags, and
  subscription group metadata.
- Product-data API routes and OpenAPI schemas now exist for folder, tag, and
  subscription group create/list workflows.
- Product-data CLI create/list commands now exist for folders, tags, and
  subscription groups.
- Product-data MCP create/list tools now exist for folders, tags, and
  subscription groups.
- Product-data Web management UI now exists for folders, tags, and subscription
  groups in the Organize workspace.
- Product-data storage primitives now exist for folder-pack, pack-tag, and
  subscription-group pack membership links.
- Product-data API/OpenAPI routes now exist for folder-pack, pack-tag, and
  subscription-group pack membership links.
- Product-data CLI add/list/remove commands now exist for folder-pack, pack-tag,
  and subscription-group pack membership links.
- Product-data MCP add/list/remove tools now exist for folder-pack, pack-tag,
  and subscription-group pack membership links.
- Product-data Web controls now exist in the Organize workspace for
  folder-pack, pack-tag, and subscription-group pack membership links.
- Domain subscription payload helpers now build MoreStickers dynamic pack-set
  metadata for public and protected subscription links.
- Public API subscription endpoints now expose per-pack dynamic subscription
  payloads, pack refresh payloads, and subscription-group dynamic payloads.
- Public subscription-group payloads filter out private packs for anonymous
  callers; owner PAT access can read private packs and private groups.
- Storage now persists subscription access tokens for pack and
  subscription-group links, with create, verify, rotate, list, and revoke
  repository methods.
- Public subscription endpoints now accept matching `msm_sub_*` subscription
  access tokens for private pack refreshes and private subscription groups,
  and protected dynamic payloads include refresh `Authorization` headers only
  for subscription-token access.
- API/OpenAPI controls now create, list, rotate, and revoke subscription
  access tokens for pack default links and subscription-group links with
  resource-specific manage-access PAT checks.
- CLI `subscription-links` commands now create, list, rotate, and revoke
  subscription access tokens, including one-time secret output for create and
  rotate.
- MCP `msm.create_subscription_link`, `msm.list_subscription_links`,
  `msm.rotate_subscription_link`, and `msm.revoke_subscription_link` tools now
  expose the same link management controls with raw secrets returned only for
  create/rotate.
- Web Organize now lists subscription links and can create, rotate, and revoke
  links with one-time secret display after create/rotate.
- Private asset reads now require authorization when an asset path maps to a
  private pack. Owner `asset.read` PATs, pack subscription tokens, and
  subscription-group tokens containing the pack can read the asset; anonymous
  callers cannot.
- Local login now creates a hashed `msm_session` Web session cookie, and owner
  Web sessions can read private assets for their packs without sending a PAT.
- Tenant member administration now has storage helpers plus protected
  API/OpenAPI list/upsert routes guarded by `tenant.manage_members` and an
  admin tenant membership check.
- CLI `msm tenants members list` and `msm tenants members set-role` now expose
  tenant member administration.
- MCP `msm.list_tenant_members` and `msm.set_tenant_member_role` now expose the
  same tenant member administration contract.
- Web now has a Tenant admin workspace for listing members and assigning
  `admin` or `user` roles.
- Tenant settings now have storage helpers plus protected API/OpenAPI read and
  replace routes guarded by `tenant.manage_settings` and an admin tenant
  membership check.
- User status controls now have storage helpers plus a protected API/OpenAPI
  route guarded by `tenant.manage_users` and an admin tenant membership check.
- Role templates now have storage helpers plus protected API/OpenAPI list and
  upsert routes guarded by `tenant.manage_roles` and an admin tenant membership
  check.
- CLI now exposes tenant settings get/update, tenant user disabled-status
  updates, and tenant role template list/upsert.
- MCP now exposes tenant settings get/update, tenant user disabled-status
  updates, and tenant role template list/upsert.
- Web now exposes tenant settings, public asset URL, tenant user enable/disable,
  and tenant role template list/upsert controls, with selectable permission
  keys and tenant administration PAT scopes.
- Pack/subscription/asset read access is now finalized in
  `docs/status/decisions.md`, including the rule that owner Web sessions should
  read private pack refresh and subscription endpoints.
- Private pack refresh/subscription endpoints now accept owner Web sessions,
  and owner PAT reads of public subscription groups include owned private packs.
- Pack update/delete/export routes now use the domain policy evaluator, giving
  same-tenant admins scoped non-owner access while denying regular non-owners.
- Product metadata membership routes now use shared RBAC helpers, giving
  same-tenant admins scoped non-owner access to folder-pack, pack-tag, and
  subscription-group pack links while denying regular non-owners.
- Export target management now requires tenant admin/custom-role authorization,
  export job create/read/event routes support same-tenant admin delegation, and
  Telegram publication reads use pack RBAC.
- Subscription access token create/list/rotate/revoke routes now use
  pack/subscription-group/tenant RBAC and support same-tenant admin delegation.
- PAT create/list/revoke routes now require a same-user `pat.manage` Bearer
  PAT, and PAT creation/local login reject scopes outside the user's
  role-allowed scope template.
- Fine-grained RBAC audit for tenant/resource-owning API routes is closed in
  the PRD.
- Tenant settings now include a local-registration enable/disable switch across
  storage, API/OpenAPI, CLI, MCP, and Web, and disabled existing tenants reject
  new local registrations.
- OIDC provider configuration storage now exists for per-tenant issuer, client,
  scope, enabled-state, and registration-policy settings. API OIDC start/state and trusted callback completion now exist with nonce plus issuer/audience checks; authorization-code exchange and signed claim validation are next.
  role-allowed PAT scopes for the current user.
- CLI now exposes `msm pats scope-policy --user-id ...` with human/JSON output
  backed by the same protected API endpoint.
- MCP now exposes `msm.get_pat_scope_policy` with the same `pat.manage`
  same-user policy.
- Web PAT/local-login dialogs now load the same scope policy and filter
  selectable scope cards when a `pat.manage` PAT is available.
- Cross-tenant API audit coverage now verifies tenant-scoped admin PATs cannot
  cross into another tenant's pack, export, subscription-link, publication,
  PAT, or tenant-settings operations.
- Tenant tag create/list routes now require target-tenant membership and the
  matching `pack.update` role permission, closing a cross-tenant metadata gap
  found during the RBAC route audit.
- Owner-scoped folder and subscription-group create/list routes now require
  target-tenant membership even when the PAT user matches `ownerUserId`.
- Pack import now requires membership in the target tenant before storing an
  imported `.stickerpack`.
- Owner PAT/Web-session reads of private assets, private pack refreshes, and
  private subscription groups now require owner membership in the resource
  tenant; subscription secrets remain explicit sharing credentials.
- Pack listing now filters out packs whose owner is no longer a member of the
  pack tenant.
- Same-owner subscription-link metadata listing now filters out token records
  whose owner is no longer a member of the token tenant.
- Fine-grained RBAC audit for tenant/resource-owning API routes is closed in
  the PRD.
- Tenant settings now include a local-registration enable/disable switch across
  storage, API/OpenAPI, CLI, MCP, and Web, and disabled existing tenants reject
  new local registrations.

## Immediate Plan

1. Complete non-API OIDC/SSO admin and client surfaces, including Web login controls and CLI/MCP documentation.

## Later Planned Work

- Pack and subscription-group access-management APIs beyond membership links.
- Pack and subscription-group public/private access-token model.
- User data export/import for instance migration.
- Provider download integrations beyond Telegram/LINE fixtures.
- Media probing through ffprobe and richer conversion diagnostics.
- MCP auth/session/SSE hardening.
- Broader multi-tenant RBAC checks, OIDC/SSO controls, and richer PAT
  management.

## Verification Expectations

For docs-only changes, `git diff --check` is sufficient. For Rust feature
work, run the relevant package tests/clippy with `TMP` and `TEMP` pointed at
`D:\Temp`. For Web feature work, run at least:

- `npm run web:typecheck`
- `npm run web:test`
- `npm run web:build`
- `git diff --check`
