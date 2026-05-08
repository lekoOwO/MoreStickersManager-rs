# Roadmap

Last updated: 2026-05-09.

This file is the short handoff view. Use `current.md` for the detailed
chronological log and `implementation-matrix.md` for the feature truth table.

## Current Focus

Telegram export has moved past basic dry-run and publication. The active focus
is now reconciliation usability and parity across Web, API, CLI, and MCP.

Web, CLI, and MCP now expose dry-run, reconciliation mode,
execute-reconciliation, and destructive mirror guard controls without requiring
hand-written worker JSON. Direct API callers can queue the same jobs through the
generic export job options object.

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

## Immediate Plan

1. Add API/OpenAPI examples or typed option documentation for Telegram
   reconciliation job options.
2. Continue hardening P33 remote target sync by covering destructive mirror
   execution examples and operator runbooks.
3. Start the next product-data slice: folder, tag, subscription-group, and pack
   access-management APIs.
4. Run targeted Rust/Web verification and update this roadmap, current status,
   implementation matrix, and checkpoints.

## Later Planned Work

- Folder, tag, subscription-group, and pack access-management APIs.
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
