# MoreStickersManager-rs PRD

Last updated: 2026-05-10.

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
| CLI | Partially complete | Pack, PAT, PAT scope-policy discovery, export, Telegram publication history, product metadata, product membership, tenant member, tenant settings, user status, role template, and OIDC provider administration commands exist. |
| MCP | Partially complete | Pack, PAT scope-policy discovery, export, Telegram publication history, product metadata, product membership, tenant member, tenant settings, user status, role template, OIDC provider administration, and provider import planning tools exist. Session/SSE hardening remains incomplete. |
| Web UI | Partially complete | Desktop/mobile shell, i18n, theme, PAT/login with role-filtered scope discovery, OIDC login-start controls, pack CRUD/import, provider import planning, product metadata create/list, product membership add/remove controls, tenant member/settings/user-status/role-template/OIDC-provider administration, export target/job UI, publication history, and Telegram reconciliation controls exist. |
| Provider normalization | Partially complete | Telegram and LINE fixture normalization exist. Network fetch/download/internalization is not complete. |
| Export targets | Partially complete | MoreStickers target and Telegram planning/publication/reconciliation foundations exist. General remote target execution and future target support remain incomplete. |
| Media conversion | Partially complete | Profiles and ffmpeg command plans exist. ffprobe probing, richer execution diagnostics, and cache completion remain incomplete. |
| Telegram publication | Partially complete | `teloxide` boundary, publish, mutation, reconciliation planning, guarded execution, remote metadata fetch, and mapping persistence exist. Further operator polish and failure recovery remain. |
| Auth/RBAC | Partially complete | PAT scopes, local auth, Web session cookie storage, bootstrap admin, PAT lifecycle scope policy, API/CLI/MCP/Web scope-policy discovery, tenant member/settings/user-status/role-template administration, local-registration enable/disable tenant settings, and cross-tenant audit coverage exist. OIDC/SSO remains incomplete. |
| Asset privacy/CDN | Partially complete | URL resolver supports CDN preference conceptually. Private pack/subscription reads accept owner PAT, matching subscription secret, or owner Web session. Admin CDN config remains incomplete. |
| Data portability | Partially complete | Storage helpers exist. Full API/CLI/Web migration workflow is incomplete. |
| CI/release | Implemented | CI, Docker publish, prerelease, release workflows, Dockerfile, and dev manager exist. |

## Current Implementation Queue

Work these in order unless a higher-risk bug appears:

1. Wire executable provider import jobs for Telegram and LINE.

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

- [x] Local registration/login bootstrap.
- [x] Admin switches for enabling/disabling local registration.
  Progress: tenant settings now include `localRegistrationEnabled` across
  SQLite storage, API/OpenAPI DTOs, CLI, MCP, and Web tenant administration.
  Existing tenants can reject local registrations while existing accounts
  continue to log in; new-tenant bootstrap remains available.
- [x] OIDC provider configuration storage.
  Progress: SQLite migration, repository methods, and tenant admin API/OpenAPI
  routes now store per-tenant OIDC provider configs with issuer, redacted client
  credentials, scopes, enabled state, and registration policy. API, CLI, MCP, and Web
  list/upsert/delete/provider-management parity now exist.
- [ ] OIDC login/callback flow.
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
  now start login and complete callback requests from the dialog. Remaining work:
  user-facing SSO documentation.
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
- [ ] Telegram network fetch with asset download/internalization.
  Progress: `msm-providers` now exposes a testable Telegram remote fetch plan
  boundary for `getStickerSet` metadata and Telegram `getFile`/file download
  asset strategy. `msm-app` now has injected runtime metadata fetch and direct
  asset internalization helpers; Telegram-specific `getFile` execution and
  executable import workflow wiring remain. API can now create protected
  provider import fetch plans for Telegram sources, and CLI/MCP/Web can
  request/display those plans.
- [ ] LINE network fetch with asset download/internalization.
  Progress: `msm-providers` now exposes a testable LINE sticker-shop product
  fetch plan boundary and direct remote URL asset strategy. `msm-app` can execute
  planned metadata fetches through an injected runtime and download direct remote
  sticker assets into `LocalAssetStore` while rewriting pack image URLs. Parsing
  and executable import workflow wiring remain. API can now create protected
  provider import fetch plans for LINE sticker sources, and CLI/MCP/Web can
  request/display those plans.
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
- Tenant/RBAC administration: tenant member, tenant settings including local
  registration enable/disable, user disabled-status, and role template
  administration exist across API, CLI, MCP, and Web. Pack
  update/delete/export plus product metadata membership routes use RBAC policy
  helpers. Export target/job routes and Telegram publication reads also use
  tenant/pack RBAC helpers. Subscription-link management routes use
  pack/subscription-group/tenant RBAC helpers. PAT lifecycle endpoints now
  enforce `pat.manage` and role-allowed scopes across API, CLI, MCP, and Web.
  Cross-tenant audit tests exist; OIDC/SSO remains incomplete, with authorization-code exchange, discovery, signed ID-token validation, and userinfo fallback wired in the API; remaining gaps are Web OIDC callback completion UX and SSO-backed account documentation.

## Open Product Questions

Resolve these before implementing the related phase:

- Pack membership UI decision: the first complete surface lives in the
  Organize workspace as a dedicated membership console; a future pack-detail
  shortcut can be added only if it improves daily workflow density.
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
