# MoreStickersManager-rs PRD

Last updated: 2026-05-14.

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
- Equicord moreStickers MSM subscription adapter for managed pack/group sync.
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
| Equicord plugin MSM adapter | Partially complete | MSM emits compatible static and dynamic pack/subscription payloads and now has configurable CORS for Discord/Equicord origins. A prepared upstream patch artifact covers MSM subscription UI, direct MSM fetches, dynamic sync fixes, and header-aware protected asset loading; applying/upstreaming and Equicord build/runtime verification remain. See `docs/integrations/equicord-morestickers-msm-adapter.md`. |
| Storage foundation | Implemented | SQLite and PostgreSQL migrations/repositories exist for tenants, users, packs, assets, PATs, Web sessions, product metadata, export jobs, Telegram publications, provider import jobs/configs, and portability helpers. Storage repository surfaces have backend-aware SQLite/PostgreSQL execution paths with shared tests, PostgreSQL CI service coverage, and user-facing deployment notes for both backends. |
| API/OpenAPI | Implemented | Health/readiness, OpenAPI, assets, pack CRUD/import/export, PATs, local/OIDC auth, tenant administration, export targets/jobs/recovery, provider import plan/job/config routes, Telegram publication history, product metadata, product membership, subscription links, and portability endpoints exist. |
| CLI | Implemented | Pack, PAT, PAT scope-policy discovery, export target/job/recovery, Telegram publication history, product metadata, product membership, tenant member, tenant settings, user status, role template, OIDC provider administration, provider import plan/job/config, and portable user export/import commands exist. |
| MCP | Implemented | Pack, PAT scope-policy discovery, export target/job/recovery, Telegram publication history, product metadata, product membership, tenant member, tenant settings, user status, role template, OIDC provider administration, provider import planning, provider import job tools, provider credential/config tools, and portable user export/import tools exist. `/mcp` is intentionally stateless JSON-RPC over POST, returns no-store responses, rejects SSE GET negotiation, and is documented in `docs/dev/mcp-transport-contract.md`. |
| Web UI | Implemented | Desktop/mobile shell, i18n, theme, PAT/local login with role-filtered scope discovery, OIDC login-start/callback controls, pack CRUD/import, provider import planning/job/config controls, product metadata create/list, product membership add/remove controls, tenant member/settings/user-status/role-template/OIDC-provider administration, export target/job/recovery UI, publication history, Telegram reconciliation controls, and portable user migration controls exist. |
| Provider ingestion | Implemented | Telegram fixtures, Telegram Bot API fetch/download execution, LINE fixtures, LINE product-page embedded metadata normalization, direct LINE asset internalization, provider job persistence/retry/events, planned-provider registry placeholders, and tenant-scoped provider credential/config storage plus API/OpenAPI, CLI, MCP, and Web redacted management exist. Signal, WhatsApp, Kakao, Band, OGQ, and Viber remain explicitly planned-only providers. |
| Export targets | Implemented | MoreStickers export and Telegram planning/publication/reconciliation targets exist with API/CLI/MCP/Web parity and failed-job recovery. Non-Telegram future remote targets dispatch through an injectable unsupported-target boundary so planned future adapters do not masquerade as implemented. |
| Media conversion | Implemented | Profiles, ffmpeg command plans, ffprobe command/report parsing, converter stdout/stderr/exit-code diagnostics, prepared-media cache reuse, export-job result visibility for output metadata, and target-specific validation exist. |
| Telegram publication | Implemented | `teloxide` boundary, publish, mutation, reconciliation planning, guarded execution, remote metadata fetch, mapping persistence, publication history, failed-job recovery, and Web/API/CLI/MCP controls exist for the current Telegram sticker-set contract. |
| Auth/RBAC | Implemented | PAT scopes, local auth, Web session cookie storage, first-start default tenant/admin bootstrap with console/log credentials, PAT lifecycle scope policy, API/CLI/MCP/Web scope-policy discovery, tenant member/settings/user-status/role-template administration, tenant-scoped local-registration enable/disable settings defaulting off, OIDC provider administration, OIDC login/callback with authorization-code exchange, verified ID-token/JWKS and userinfo handling, SSO-backed PAT/session issuance, SSO usage docs, and cross-tenant audit coverage exist. |
| Asset privacy/CDN | Implemented | Tenant public asset/CDN URL settings exist across API/OpenAPI, CLI, MCP, and Web. `MSM_PUBLIC_ASSET_URL` provides a system-wide fallback CDN base, and tenant settings take precedence. Protected pack exports and public pack/subscription payloads rewrite local sticker asset URLs to the selected CDN base when configured. Private pack/subscription reads accept owner PAT, matching subscription secret, or owner Web session. |
| Data portability | Implemented | Storage helpers plus API/OpenAPI, CLI, Web, and MCP portable user export/import surfaces exist, with cross-instance API compatibility coverage for moving user data between MSM instances. |
| CI/release | Implemented | CI, Docker publish, prerelease, release workflows, Dockerfile, and dev manager exist. |

## Current Implementation Queue

1. Equicord moreStickers MSM adapter.
   Status: new scope, partially complete on the MSM side.
   Next slices:
   - [x] Document the upstream plugin adapter contract.
   - [x] Add MSM CORS configuration for Discord/Equicord browser origins.
   - [x] Prepare an Equicord plugin patch artifact for adding MSM subscriptions
     from URL. Artifact:
     `docs/integrations/patches/equicord-morestickers-msm-adapter.patch`.
   - [x] Apply the prepared Equicord plugin patch in a clean local plugin
     worktree and verify it builds.
   - [x] Verify plugin-side direct MSM fetch code path through TypeScript,
     targeted lint, and standalone build.
   - [x] Verify plugin-side dynamic pack-set sync code path for initial add,
     update, remove, manual refresh, and startup refresh through TypeScript,
     targeted lint, and standalone build.
   - [x] Verify plugin-side header-aware asset blob cache code path for
     protected MSM assets through TypeScript, targeted lint, and standalone
     build.
   - [x] Upstream the prepared Equicord plugin patch or otherwise make it
     available in the actual deployed plugin repository. PR:
     https://github.com/Equicord/Equicord/pull/1084.
   - [ ] Track PR review/merge/deployment status for the Equicord plugin patch
     (PR 1084 is open, its GitHub Actions `Test` check passed, and its body
     now includes MSM's GitHub link, feature summary, and MSC replacement
     recommendation).
   - [ ] Runtime-verify public and protected MSM pack/group subscriptions in
     Discord/Equicord.

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

- [x] Define final permission model for pack visibility, group visibility,
  subscription secrets, PAT access, and authenticated Web credentials.
- [x] Implement per-pack default subscription endpoint.
- [x] Implement user-created subscription group public/protected endpoints.
- [x] Ensure private packs do not leak assets through public groups.
- [x] Add moreStickers-compatible subscription payload contract.
- [x] Add persistent subscription secret/link rotation storage.
- [x] Enforce subscription secret access on public subscription endpoints.
- [x] Enforce owner PAT and owner Web-session access on private subscription
  endpoints.
- [x] Add API/OpenAPI controls for subscription link creation, listing,
  rotation, and revocation.
- [x] Add CLI controls for subscription link creation, listing, rotation, and
  revocation.
- [x] Add MCP controls for subscription link creation, listing, rotation, and
  revocation.
- [x] Add Web controls for subscription link creation, listing, rotation, and
  revocation.

### Phase C: Multi-Tenant Administration

- [x] Tenant admin API for users, roles, memberships, and tenant settings.
  Progress: member list/upsert API exists with CLI/MCP/Web parity; tenant
  settings read/update, user disabled-status update, and role template
  list/upsert APIs exist with CLI/MCP/Web parity.
- [x] Web tenant admin console.
- [x] Fine-grained RBAC checks for all resource-owning operations.
  Progress: pack update/delete/export routes now use the domain policy
  evaluator, and product metadata membership routes use shared RBAC helpers, so
  same-tenant admins can manage non-owned packs and metadata memberships while
  regular non-owners are denied. Export target management now requires tenant
  admin/custom-role authorization, export job create/read/event routes support
  same-tenant admin delegation, Telegram publication reads use pack RBAC, and
  subscription-link management routes support same-tenant admin delegation.
  Tenant tag create/list routes now require membership in the target tenant,
  preventing scoped PATs from creating or enumerating tags across tenants.
  Owner-scoped folder and subscription-group create/list routes now also
  require target-tenant membership instead of trusting `ownerUserId` alone.
  Pack import now requires target-tenant membership before writing imported
  packs. Owner PAT/Web-session reads of private assets, private pack refreshes,
  and private subscription groups now require the owner to remain a member of
  the resource tenant; subscription secrets remain explicit resource-sharing
  credentials. Pack listing now filters out packs whose owner is no longer a
  member of the pack's tenant. Subscription-link metadata listing now filters
  same-owner results by current owner membership in each token tenant.
  Route audit found no remaining API tenant/resource-owning route without a
  membership or policy gate.
- [x] PAT creation policy and scope templates by role.
  Progress: PAT create/list/revoke routes now require a `pat.manage` Bearer
  PAT for the same user, PAT creation and local login reject scopes outside the
  user's built-in role, tenant admin role, or custom role-template permissions,
  dev bootstrap obtains PATs through local login, API/OpenAPI exposes
  `GET /api/v1/pats/scope-policy`, CLI exposes
  `msm pats scope-policy --user-id ...`, MCP exposes
  `msm.get_pat_scope_policy`, and Web PAT/local-login dialogs filter selectable
  scope cards from the live scope-policy endpoint when a `pat.manage` PAT is
  available.
- [x] Audit tests for cross-tenant isolation.
  Progress: API audit coverage now verifies that an admin PAT scoped for one
  tenant is rejected when attempting to manage another tenant's pack metadata,
  export target, subscription access token, Telegram publication history, PAT
  list, and tenant settings.

### Phase D: Auth Providers

- [x] First-start default tenant/admin bootstrap.
  Progress: service startup now creates a `default` tenant and `admin`
  local account only when the database has no tenants and no users. The
  generated or configured bootstrap password is emitted through the structured
  `bootstrap_admin_created` console/log event. The bootstrapped tenant keeps
  `localRegistrationEnabled=false`.
- [x] Local registration/login.
  Progress: local register now requires an existing tenant with
  `localRegistrationEnabled=true`, creates a normal `user` membership only,
  and no longer creates tenants or allows self-assigned `admin` membership.
  Login still verifies Argon2 credentials, returns a PAT, and creates a Web
  session cookie.
- [x] Admin switches for enabling/disabling local registration.
  Progress: tenant settings now include `localRegistrationEnabled` across
  SQLite storage, API/OpenAPI DTOs, CLI, MCP, and Web tenant administration.
  Existing tenants can reject local registrations while existing accounts
  continue to log in. New tenants default to closed registration, and the
  switch is managed through API/CLI/MCP/Web tenant settings instead of runtime
  env.
- [x] OIDC provider configuration storage.
  Progress: SQLite migration, repository methods, and tenant admin API/OpenAPI
  routes now store per-tenant OIDC provider configs with issuer, redacted client
  credentials, scopes, enabled state, and registration policy. API, CLI, MCP, and Web
  list/upsert/delete/provider-management parity now exist.
- [x] OIDC login/callback flow.
  Progress: API now starts OIDC login by creating one-time hashed state and
  nonce tokens and building provider authorization URLs. The callback endpoint
  verifies state/nonce, validates trusted issuer and audience claims, then
  consumes state, links or creates tenant users when provider registration is
  enabled, issues PATs, and creates Web sessions for already-validated provider
  claims. Callback requests may now include an authorization code; the API
  exchanges it against the provider token endpoint using the stored redirect URI
  before creating the session. Discovery document parsing now validates issuer and required endpoint URLs,
  and callback authorization-code exchange uses the discovered token endpoint
  through an injectable discovery fetcher. JWKS parsing/signature-key selection, unverified ID-token header/claim
  parsing, issuer/audience/nonce/expiration claim validation, and RS256
  JWKS-backed signature verification now exist. Callback authorization-code
  completion now verifies ID-token signatures/claims and uses validated subject,
  email, and display-name claims for user-link creation when an ID token is
  returned. Userinfo response parsing now validates subject and normalizes display-name
  fallback. Callback completion now fetches userinfo when a verified ID token
  omits email/name, validates userinfo subject against the ID-token subject, and
  uses validated userinfo profile claims for user/link creation. Web auth can
  now start login and complete callback requests from the dialog. User-facing SSO documentation now exists in `docs/user/oidc-sso.md` and the user README.
- [x] Web SSO login controls.
  Progress: Web tenant administration now lists, creates/updates, and deletes
  OIDC providers. The Web auth dialog can start OIDC login, call the live
  login-start endpoint, and show the provider authorization URL with state,
  nonce, and expiry details. It can also submit authorization code/state/nonce
  plus fallback claims to the callback endpoint, store the returned PAT, and
  reuse the existing role-filtered scope picker defaults. The Web app persists
  pending state/nonce in browser storage and pre-fills callback fields when the
  provider redirects back to `/auth/oidc/callback?code=...&state=...`.
- [x] CLI/MCP documentation for PAT usage with SSO-backed accounts.
  Progress: CLI, MCP, and Web tenant administration now manage OIDC providers;
  user docs now explain provider administration, OIDC login/callback endpoints,
  Web callback behavior, SSO-returned PAT reuse, CLI `MSM_PAT`, MCP Bearer usage,
  and role-capped scope selection.

### Phase E: Provider Ingestion

- [x] Provider trait and Telegram/LINE fixture normalization.
- [x] Telegram network fetch with asset download/internalization.
  Progress: `msm-providers` now exposes a testable Telegram remote fetch plan
  boundary for `getStickerSet` metadata and Telegram `getFile`/file download
  asset strategy. `msm-app` now has injected runtime metadata fetch helpers and
  a provider import worker foundation. The tested worker resolves Telegram sticker
  `fileId` values through `getFile`, downloads Bot API file URLs into the local
  asset store, rewrites pack image URLs to MSM-hosted assets, and upserts the
  imported pack. API can create protected provider import fetch plans and queue
  provider import jobs for Telegram sources. CLI/MCP/Web can request/display
  those plans and create/read provider import jobs/events.
- [x] LINE network fetch with asset download/internalization.
  Progress: `msm-providers` now exposes a testable LINE sticker-shop product
  fetch plan boundary and direct remote URL asset strategy. `msm-app` can execute
  planned metadata fetches through an injected runtime and download direct remote
  sticker assets into `LocalAssetStore` while rewriting pack image URLs. Tested worker execution accepts fixture-schema JSON and LINE product pages with
  embedded metadata, then internalizes direct remote assets. API can create protected
  provider import fetch plans and queue provider import jobs for LINE sticker sources.
  CLI/MCP/Web can request/display those plans, and CLI/MCP/Web can create/read
  provider import jobs and list job events.
- [x] Provider credential/config UI and API.
  Progress: SQLite storage plus API/OpenAPI, CLI, MCP, and Web surfaces now manage tenant-scoped provider configs through `GET /api/v1/provider-configs?tenantId=...`, `PUT /api/v1/provider-configs/{config_id}`, and `DELETE /api/v1/provider-configs/{config_id}`. CLI exposes `msm providers configs list/upsert/delete` with human/JSON output; MCP exposes `msm.list_provider_configs`, `msm.upsert_provider_config`, and `msm.delete_provider_config`; Web exposes a provider credential panel with list/upsert/delete controls and redacted JSON display. Responses recursively redact token/secret fields while retaining raw secrets in storage. Management requires `provider.import` plus tenant admin/custom-role authorization; same-tenant users with `provider.import` may list configs but cannot modify them. Provider import worker now consumes enabled configs for base URLs and Telegram bot tokens.
- [x] Provider job progress and retry model.
  Progress: SQLite/API job persistence exists; `msm-app` has a tested
  queued-job worker foundation with running/succeeded/failed/retry transitions
  for LINE direct-asset imports. `MSM_PROVIDER_IMPORT_WORKER_ENABLED`,
  `MSM_PROVIDER_IMPORT_WORKER_POLL_INTERVAL_MS`,
  `MSM_PROVIDER_IMPORT_RETRY_BACKOFF_MS`, and `MSM_PUBLIC_ASSET_BASE_URL` wire
  the worker into service startup. CLI, MCP, and Web job create/get/events controls now exist.
- [x] Placeholder registry entries for Signal, WhatsApp, Kakao, Band, OGQ,
  Viber without pretending they are implemented.
  Progress: `msm-providers` registry exposes planned metadata for Signal, WhatsApp, Kakao, Band, OGQ, and Viber with `ProviderStatus::Planned` and tests that keep them distinct from implemented Telegram/LINE providers.

### Phase F: Media Conversion

- [x] Media profiles and command planning.
- [x] ffprobe probing.
  Progress: `msm-media` exposes shell-free `MediaProbeCommand`/`MediaProbeToolchain` plus `MediaProbeReport::from_ffprobe_json` for static image, animated image, and video classification with dimensions, duration, size, and codec facts.
- [x] ffmpeg execution hardening and diagnostics.
  Progress: process-backed conversion already used shell-free commands, timeout handling, and exit-status validation; it now captures converter stdout, stderr, and exit code into `ConversionCommandOutput` and returns those diagnostics through `PreparedMediaOutput` for downstream persistence/surface work.
- [x] Prepared media cache completion.
  Progress: export worker now checks `prepared_media_assets` by source asset hash and target profile before invoking the media executor. Cache hits are reused in Telegram dry-run/publication/reconciliation result summaries without re-running conversion.
- [x] Web/API/CLI/MCP visibility into conversion errors and output metadata.
  Progress: export job result JSON now includes prepared media output metadata plus converter stdout, stderr, and exit code for newly converted assets. Existing API/CLI/MCP/Web export-job read surfaces consume the shared job result payload, so this metadata is visible without adding a new route.
- [x] Target-specific validation for Telegram and future export targets.
  Progress: `PreparedMediaSpec::validate_probe_report` validates ffprobe facts against target profile canvas size, max file size, and max duration. The validation is generic over `StickerTargetProfile`, so Telegram profiles and future target profiles share the same enforcement path.

### Phase G: Export And Publication Targets

- [x] MoreStickers target serialization.
- [x] Telegram create/append publication foundation.
- [x] Telegram reconciliation planning and guarded mutation execution.
- [x] Export-target execution abstraction for non-Telegram remote targets.
  Progress: `RemoteExportTargetExecutor` now receives target/job/config/pack snapshots for future remote targets and returns target-neutral execution summaries. The default executor safely reports unsupported target kinds until concrete target adapters are implemented.
- [x] Web/API/CLI/MCP target parity for all implemented targets.
  Progress: implemented export target/job operations are available across API/OpenAPI, CLI, MCP, and Web, including target kind/list/create/update/delete, job create/get/events/requeue, and Telegram publication list/get surfaces.
- [x] Recovery tools for failed or partially-applied remote publication jobs.
  Progress: API/OpenAPI, CLI, MCP, and Web can requeue failed or cancelled export jobs for operator recovery. Requeue resets attempt/error state, records a recovery event for API/MCP direct recovery, and refreshes visible job timelines in Web.

### Phase H: Asset Privacy And CDN

- [x] Tenant setting for public asset URL across API/OpenAPI, CLI, MCP, and Web.
- [x] System-wide default setting for public asset URL through `MSM_PUBLIC_ASSET_URL`.
- [x] Admin UI/API for tenant CDN URL configuration.
- [x] Sticker pack and subscription payloads rewrite local asset URLs to the tenant CDN base when configured.
- [x] Private asset authorization through pack subscription secret,
  subscription-group secret, or owner PAT.
- [x] Private asset authorization through Web session credentials.
- [x] Tests proving private images cannot be fetched anonymously.

### Phase I: Data Portability

- [x] API export/import endpoints for user data.
- [x] CLI export/import commands.
- [x] Web migration flow.
- [x] MCP export/import tools.
- [x] Compatibility tests for moving between MSM instances.

### Phase J: PostgreSQL Support

- [x] PostgreSQL migrations.
  Progress: backend-specific `migrations/sqlite` and `migrations/postgres` sets exist; PostgreSQL migrations use boolean columns where repository models expect booleans, and `DbPool::run_migrations` selects the correct migrator per backend. Live PostgreSQL repository coverage remains in the next checkbox.
- [x] Repository abstraction verified against SQLite and PostgreSQL.
  Progress: tenant/user/tenant-member core identity operations, sticker-pack upsert/find/list/read-record operations, folder create/list/find operations, tag create/list/find operations, subscription-group create/list/find operations, folder-pack, pack-tag, and subscription-group-pack membership add/list operations, personal-access-token create/list/find/verify/revoke operations, Web-session create/verify/revoke operations, local-credential create/read/verify operations, OIDC provider/state/user-link operations, subscription access-token create/list/find/verify/rotate/revoke operations, export-target create/list/find/update/delete operations, export-job create/find/queue/retry/status/recovery/event operations, prepared-media upsert/find operations, and Telegram publication/mapping upsert/find/list operations, provider config upsert/list/find/delete operations, provider import-job create/find/due/status/retry/failure/event operations, metadata membership removal operations, portable user export/import helpers, tenant admin helper operations (tenant settings updates, user disabled-status updates, tenant member upsert, role template upsert/list/find), metadata rename/delete operations, and sticker-pack metadata update/delete/list-accessible helpers now have backend-aware SQLite/PostgreSQL SQL paths and shared contract tests. The PostgreSQL legs run when `MSM_TEST_POSTGRES_URL` is configured; CI now starts PostgreSQL and sets `MSM_TEST_POSTGRES_URL`; README/user docs describe SQLite and PostgreSQL deployment URLs and migration behavior.
- [x] CI matrix for both database backends.
- [x] Deployment docs for both backends.

### Phase K: Production Hardening

- [x] MCP session/auth/SSE hardening.
  Progress: `/mcp` intentionally supports stateless JSON-RPC over POST, returns `Cache-Control: no-store`, rejects SSE GET negotiation with a structured JSON response, and documents the supported session/auth/proxy contract in `docs/dev/mcp-transport-contract.md`. Protected `tools/call` operations continue to use Bearer PAT scope enforcement and shared tenant/RBAC checks. Stateful MCP sessions or SSE resumability are out of the current contract and require a future PRD item before implementation.
- [x] Rate limits and request size limits for upload/import routes.
  Progress: API/app routers now apply a configurable `MSM_REQUEST_BODY_LIMIT_BYTES` request body cap (default 10 MiB) before JSON import handling. Pack import, portable user import, provider import planning, and provider import job creation also pass through an in-memory per-identity rate limiter configured by `MSM_IMPORT_RATE_LIMIT_REQUESTS` and `MSM_IMPORT_RATE_LIMIT_WINDOW_SECS`.
- [x] Structured logs and operator-facing health diagnostics.
  Progress: `msm-app` emits JSON lines for service startup/listening and HTTP request summaries without logging query strings or credentials. `/readyz` returns operator diagnostics for database query readiness and local asset-store directory readiness, and OpenAPI documents the endpoint.
- [x] Backup/restore guidance.
  Progress: `docs/user/backup-restore-runbook.md` documents complete instance backups for SQLite/PostgreSQL, assets, prepared media, secrets/config, restore steps, cross-instance portable-data migration boundaries, readiness verification, and restore drills.
- [x] Security review of token storage, secret redaction, and asset access.
  Progress: `docs/status/security-review.md` records current controls, evidence, residual risks, and follow-up hardening candidates for PAT/session/subscription/OIDC token storage, local password hashing, secret redaction, private asset authorization, CDN interactions, and log hygiene.

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
- Telegram export/reconciliation: API, CLI, MCP, and Web controls exist, including publication history, reconciliation options, and failed-job recovery.
- User data migration: storage helpers plus API/OpenAPI, CLI, Web, and MCP
  workflows exist, and cross-instance API compatibility coverage closes Phase I.
- Tenant/RBAC administration: tenant member, tenant settings including local
  registration enable/disable, user disabled-status, and role template
  administration exist across API, CLI, MCP, and Web. Pack
  update/delete/export plus product metadata membership routes use RBAC policy
  helpers. Export target/job routes and Telegram publication reads also use
  tenant/pack RBAC helpers. Subscription-link management routes use
  pack/subscription-group/tenant RBAC helpers. PAT lifecycle endpoints now
  enforce `pat.manage` and role-allowed scopes across API, CLI, MCP, and Web.
  Cross-tenant audit tests exist; OIDC/SSO login-start, callback completion, authorization-code exchange, discovery, signed ID-token validation, userinfo fallback, Web callback UX, SSO-backed PAT/session issuance, and SSO usage documentation are complete for the current contract.

## Open Product Questions

Resolve these before implementing the related phase:

- Pack membership UI decision: the first complete surface lives in the
  Organize workspace as a dedicated membership console; a future pack-detail
  shortcut can be added only if it improves daily workflow density.
- PostgreSQL strategy: use SQLx compile-time checked queries per backend, query
  builder abstraction, or repository trait with backend-specific
  implementations?
- Provider credentials decision: provider import configs are tenant-scoped records because provider imports create tenant resources and should be managed by tenant admins/custom roles. Per-user or per-job credential overrides may be added later only when a concrete sharing model requires them.

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

Completion audit: passed on 2026-05-11 for the current PRD contract. Evidence is
recorded in `docs/status/completion-audit.md`; Docker image execution remains
CI-bound because Docker CLI is not installed in the local Windows environment,
while the Docker publish workflow and Dockerfile were statically reviewed.

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
