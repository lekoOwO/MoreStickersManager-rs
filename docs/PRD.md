# MoreStickersManager-rs PRD

Last updated: 2026-05-09.

This is the living product requirements document for MoreStickersManager-rs
(MSM). Keep it accurate until the project can be considered complete. When
scope, implementation status, or acceptance criteria changes, update this file
in the same change set.

## Product Summary

MSM is a self-hosted Rust application for importing, managing, organizing,
exporting, and publishing sticker packs. It must preserve the MoreStickers
`.stickerpack` export format while adding Web UI, API, CLI, and MCP management
surfaces.

Primary integrations:

- Equicord moreStickers-compatible import/export.
- Telegram import/provider normalization and Telegram sticker set publication
  through `teloxide`.
- LINE provider normalization.
- Future provider and export-target support for Signal, WhatsApp, Kakao, Band,
  OGQ, Viber, and other sticker ecosystems.

## Non-Negotiable Requirements

- Preserve MoreStickers `.stickerpack` JSON compatibility for import/export.
- Use Rust for backend crates and service binary.
- Serve API/OpenAPI with `utoipa` documentation.
- Provide complete feature parity across Web UI, API, CLI, and MCP before
  marking a feature complete.
- Use Shadcn Vue-compatible primitives and Tailwind CSS v4 for Web UI.
- Embed built Web dist into the Rust binary.
- Keep Provider and Export Target abstractions modular.
- Use `teloxide` for Telegram bot interactions.
- Support self-hosted assets and optional system-wide public asset/CDN URL.
- Support multi-tenant usage with RBAC, at minimum admin and regular user
  roles.
- Support local accounts, OIDC/SSO later, and PATs for API/CLI/MCP access.
- Support SQLite now and PostgreSQL before project completion.
- Maintain Git-based development, normal human commit messages, and author
  `Leko <leko@leko.moe>`.
- Do not include tool or assistant authorship in commits, branches, PRs, or
  editable release text.

## User Goals

- A user can import existing MoreStickers exports and manage packs in MSM.
- A user can create, rename, delete, organize, tag, and publish sticker packs.
- A user can decide whether packs and subscription groups are public or
  protected.
- A user can create subscription links for a pack or a group of packs so
  moreStickers can periodically update from MSM.
- An admin can configure tenants, roles, auth methods, PAT policy, asset/CDN
  URLs, and provider/export target credentials.
- A user can export all personal data and import it into another MSM instance
  without manual database surgery.
- External automation can use API, CLI, or MCP with equivalent capability.

## Current Status

Status meanings:

- `Implemented`: current scoped contract is usable and tested.
- `Partially complete`: foundation exists, but parity, hardening, or required
  completion scope remains.
- `Not implemented`: no usable production surface exists yet.

| Area | Status | Notes |
| --- | --- | --- |
| MoreStickers compatibility | Implemented | Domain models preserve `.stickerpack` shape and provider ID conventions. |
| Storage foundation | Partially complete | SQLite migrations and repositories exist for tenants, users, packs, assets, PATs, Web sessions, product metadata, export jobs, Telegram publications, and portability helpers. PostgreSQL remains incomplete. |
| API/OpenAPI | Partially complete | Health, OpenAPI, assets, pack CRUD/import/export, PATs, local auth, tenant member administration, export jobs, Telegram publication history, product metadata, and product membership endpoints exist. |
| CLI | Partially complete | Pack, PAT, export, Telegram publication history, product metadata, product membership, and tenant member administration commands exist. |
| MCP | Partially complete | Pack, export, Telegram publication history, product metadata, product membership, and tenant member administration tools exist. Session/SSE hardening remains incomplete. |
| Web UI | Partially complete | Desktop/mobile shell, i18n, theme, PAT/login, pack CRUD/import, product metadata create/list, product membership add/remove controls, export target/job UI, publication history, and Telegram reconciliation controls exist. |
| Provider normalization | Partially complete | Telegram and LINE fixture normalization exist. Network fetch/download/internalization is not complete. |
| Export targets | Partially complete | MoreStickers target and Telegram planning/publication/reconciliation foundations exist. General remote target execution and future target support remain incomplete. |
| Media conversion | Partially complete | Profiles and ffmpeg command plans exist. ffprobe probing, richer execution diagnostics, and cache completion remain incomplete. |
| Telegram publication | Partially complete | `teloxide` boundary, publish, mutation, reconciliation planning, guarded execution, remote metadata fetch, and mapping persistence exist. Further operator polish and failure recovery remain. |
| Auth/RBAC | Partially complete | PAT scopes, local auth, Web session cookie storage, bootstrap admin, and tenant member list/upsert exist in API/CLI/MCP. Full tenant settings, role management, Web admin parity, and OIDC/SSO remain incomplete. |
| Asset privacy/CDN | Partially complete | URL resolver supports CDN preference conceptually. Private pack asset reads require owner PAT, matching subscription secret, or owner Web session. Admin CDN config remains incomplete. |
| Data portability | Partially complete | Storage helpers exist. Full API/CLI/Web migration workflow is incomplete. |
| CI/release | Implemented | CI, Docker publish, prerelease, release workflows, Dockerfile, and dev manager exist. |

## Current Implementation Queue

Work these in order unless a higher-risk bug appears:

1. Add Web parity for tenant member administration.
2. Extend tenant administration APIs to tenant settings, role templates, and
   user status controls.

Each queue item must update this section when completed or reordered.

## Completion Roadmap

Update these checkboxes in place as work lands. A phase is complete only after
tests and docs are updated.

### Phase A: Product Organization Parity

- [x] Storage for folders, tags, subscription groups.
- [x] API/OpenAPI create/list for folders, tags, subscription groups.
- [x] CLI create/list for folders, tags, subscription groups.
- [x] MCP create/list tools for folders, tags, subscription groups.
- [x] Web create/list management for folders, tags, subscription groups.
- [x] Storage for folder-pack, pack-tag, and subscription-group pack links.
- [x] API/OpenAPI add/list/remove for product membership links.
- [x] CLI add/list/remove for product membership links.
- [x] MCP add/list/remove tools for product membership links.
- [x] Web add/list/remove controls for product membership links.

### Phase B: Subscription Links And Access Model

- [ ] Define final permission model for pack visibility, group visibility,
  subscription secrets, PAT access, and authenticated Web credentials.
- [x] Implement per-pack default subscription endpoint.
- [x] Implement user-created subscription group public/protected endpoints.
- [x] Ensure private packs do not leak assets through public groups.
- [x] Add moreStickers-compatible subscription payload contract.
- [x] Add persistent subscription secret/link rotation storage.
- [x] Enforce subscription secret access on public subscription endpoints.
- [x] Add API/OpenAPI controls for subscription link creation, listing,
  rotation, and revocation.
- [x] Add CLI controls for subscription link creation, listing, rotation, and
  revocation.
- [x] Add MCP controls for subscription link creation, listing, rotation, and
  revocation.
- [x] Add Web controls for subscription link creation, listing, rotation, and
  revocation.

### Phase C: Multi-Tenant Administration

- [ ] Tenant admin API for users, roles, memberships, and tenant settings.
  Progress: member list/upsert API exists with CLI/MCP parity; users, roles,
  Web controls, and settings remain.
- [ ] Web tenant admin console.
- [ ] Fine-grained RBAC checks for all resource-owning operations.
- [ ] PAT creation policy and scope templates by role.
- [ ] Audit tests for cross-tenant isolation.

### Phase D: Auth Providers

- [x] Local registration/login bootstrap.
- [ ] Admin switches for enabling/disabling local registration.
- [ ] OIDC provider configuration storage.
- [ ] OIDC login/callback flow.
- [ ] Web SSO login controls.
- [ ] CLI/MCP documentation for PAT usage with SSO-backed accounts.

### Phase E: Provider Ingestion

- [x] Provider trait and Telegram/LINE fixture normalization.
- [ ] Telegram network fetch with asset download/internalization.
- [ ] LINE network fetch with asset download/internalization.
- [ ] Provider credential/config UI and API.
- [ ] Provider job progress and retry model.
- [ ] Placeholder registry entries for Signal, WhatsApp, Kakao, Band, OGQ,
  Viber without pretending they are implemented.

### Phase F: Media Conversion

- [x] Media profiles and command planning.
- [ ] ffprobe probing.
- [ ] ffmpeg execution hardening and diagnostics.
- [ ] Prepared media cache completion.
- [ ] Web/API/CLI/MCP visibility into conversion errors and output metadata.
- [ ] Target-specific validation for Telegram and future export targets.

### Phase G: Export And Publication Targets

- [x] MoreStickers target serialization.
- [x] Telegram create/append publication foundation.
- [x] Telegram reconciliation planning and guarded mutation execution.
- [ ] Export-target execution abstraction for non-Telegram remote targets.
- [ ] Web/API/CLI/MCP target parity for all implemented targets.
- [ ] Recovery tools for failed or partially-applied remote publication jobs.

### Phase H: Asset Privacy And CDN

- [ ] Tenant/system setting for public asset URL.
- [ ] Admin UI/API for CDN URL configuration.
- [x] Private asset authorization through pack subscription secret,
  subscription-group secret, or owner PAT.
- [x] Private asset authorization through Web session credentials.
- [x] Tests proving private images cannot be fetched anonymously.

### Phase I: Data Portability

- [ ] API export/import endpoints for user data.
- [ ] CLI export/import commands.
- [ ] Web migration flow.
- [ ] Compatibility tests for moving between MSM instances.

### Phase J: PostgreSQL Support

- [ ] PostgreSQL migrations.
- [ ] Repository abstraction verified against SQLite and PostgreSQL.
- [ ] CI matrix for both database backends.
- [ ] Deployment docs for both backends.

### Phase K: Production Hardening

- [ ] MCP session/auth/SSE hardening.
- [ ] Rate limits and request size limits for upload/import routes.
- [ ] Structured logs and operator-facing health diagnostics.
- [ ] Backup/restore guidance.
- [ ] Security review of token storage, secret redaction, and asset access.

## Surface Parity Rule

Each user-facing capability must eventually exist in:

- API/OpenAPI.
- CLI.
- MCP.
- Web UI.

Temporary single-surface slices are allowed during development, but this PRD
must record remaining surfaces in the roadmap until parity is reached.

Current parity gaps:

- Product membership links: API, CLI, MCP, and Web controls exist.
- Product metadata create/list: API, CLI, MCP, and Web exist.
- Telegram export/reconciliation: API, CLI, MCP, and Web controls exist, but
  operator recovery polish remains.
- User data migration: storage helpers exist; API, CLI, and Web workflows are
  missing.
- Tenant/RBAC administration: tenant member storage and API/CLI/MCP list/upsert
  exist; Web controls, tenant settings, role templates, and user status controls
  are incomplete.

## Open Product Questions

Resolve these before implementing the related phase:

- Subscription access decision: matching pack subscription tokens and
  subscription-group tokens grant read access to private pack assets and
  refresh/subscription endpoints; they do not grant management access.
- Pack membership UI decision: the first complete surface lives in the
  Organize workspace as a dedicated membership console; a future pack-detail
  shortcut can be added only if it improves daily workflow density.
- RBAC granularity: should owner-only checks remain direct `owner_user_id`
  comparisons, or should all checks move through a tenant role/permission
  evaluator before adding admin delegation?
- PostgreSQL strategy: use SQLx compile-time checked queries per backend, query
  builder abstraction, or repository trait with backend-specific
  implementations?
- Provider credentials: store per tenant, per user, or per export/import target?

When a question is answered, replace it with the decision or move the decision
to `docs/status/decisions.md`.

## Slice Definition Of Done

For any implementation slice:

- Add or update tests before implementation where behavior changes.
- Update API/OpenAPI, CLI, MCP, and Web parity status in this PRD.
- Update `docs/status/current.md` with exact verification commands.
- Append one concise checkpoint to `docs/status/checkpoints.md`.
- Keep user docs current when commands, routes, or UI behavior changes.
- Commit with author `Leko <leko@leko.moe>` and a normal human commit message.

## Architecture Map

- `crates/msm-domain`: compatibility models, provider IDs, authorization policy.
- `crates/msm-storage`: database repositories, migrations, local asset storage,
  PAT/local credential storage, export persistence, portability helpers.
- `crates/msm-api`: Axum routes and `utoipa` OpenAPI.
- `crates/msm-cli`: HTTP client and command surface.
- `crates/msm-mcp`: JSON-RPC MCP endpoint and tools.
- `crates/msm-providers`: provider normalization.
- `crates/msm-media`: media kind/profile/conversion planning.
- `crates/msm-exporters`: target traits, planning, MoreStickers and Telegram
  target logic.
- `crates/msm-telegram`: `teloxide` boundary.
- `crates/msm-app`: service composition, worker, static Web serving.
- `apps/web`: Vue/Vite/Tailwind/Shadcn Vue-compatible Web client.
- `scripts/dev-manager.mjs`: local API/Web environment manager.

Do not add provider-specific behavior into `msm-domain` unless it is a stable
identifier or pure compatibility rule.

## Verification Policy

Use targeted tests during TDD, then full relevant verification before each
commit. For docs-only changes, `git diff --check` is sufficient.

Common commands:

```powershell
cargo fmt --all -- --check
cargo test -p msm-domain --locked
cargo test -p msm-storage --locked
cargo test -p msm-api --locked
cargo test -p msm-cli --locked
cargo test -p msm-mcp --locked
cargo test -p msm-app --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run web:typecheck
npm run web:test
npm run web:build
npm run web:e2e
git diff --check
```

E2E must use the installed Microsoft Edge setup and must not install Chromium
unless explicitly requested.

## Documentation Policy

Active documentation is intentionally small:

- `docs/PRD.md`: product requirements, status, roadmap, acceptance criteria.
- `docs/agents/README.md`: minimal agent handoff protocol.
- `docs/status/current.md`: latest session state and verification log.
- `docs/status/checkpoints.md`: chronological development log.
- `docs/status/implementation-matrix.md`: compact feature truth table.
- `docs/dev/*`: developer architecture and compatibility references.
- `docs/user/*`: user-facing usage docs and runbooks.

Do not create new long-lived phase plan/spec files for routine work. If a
large feature needs a temporary implementation plan, either add a short section
to this PRD or create a short-lived branch-local note and remove it before
merge after the PRD/checkpoints are updated.

## Completion Definition

MSM can be considered complete when:

- All roadmap phases above are checked.
- Web/API/CLI/MCP parity exists for all implemented product features.
- SQLite and PostgreSQL pass the same repository/API test expectations.
- MoreStickers compatibility fixtures still pass.
- Private asset and subscription access rules are enforced by tests.
- User data can migrate between MSM instances through documented workflows.
- Release workflows produce usable binaries and Docker images.
- User, developer, and agent docs accurately describe the final system without
  relying on historical phase notes.
