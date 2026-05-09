# Roadmap

Last updated: 2026-05-09.

This file is the short handoff view. Use `current.md` for the detailed
chronological log, `implementation-matrix.md` for the feature truth table, and
`../PRD.md` for the living completion roadmap.

## Current Focus

Telegram export has moved past basic dry-run and publication. The P33
reconciliation usability slice is functionally covered across Web, API, CLI,
and MCP; the active handoff focus is product-data management surfaces.

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

## Immediate Plan

1. Add MCP/Web controls for subscription link creation, rotation,
   revocation, and visibility.
2. Extend subscription access policy enforcement beyond public/private/PAT to
   subscription secrets and Web session credentials.
3. Run targeted Rust/Web verification and update this roadmap, current status,
   implementation matrix, and checkpoints.

## Later Planned Work

- Pack and subscription-group access-management APIs beyond membership links.
- Pack and subscription-group public/private access-token model.
- User data export/import for instance migration.
- Provider download integrations beyond Telegram/LINE fixtures.
- Media probing through ffprobe and richer conversion diagnostics.
- MCP auth/session/SSE hardening.
- Multi-tenant RBAC, OIDC/SSO controls, and richer PAT management.

## Verification Expectations

For docs-only changes, `git diff --check` is sufficient. For CLI/MCP feature
work, run at least:

- `cargo fmt --all -- --check`
- `cargo test -p msm-cli -p msm-mcp --locked`
- `cargo clippy -p msm-cli -p msm-mcp --all-targets --locked -- -D warnings`
- `git diff --check`
