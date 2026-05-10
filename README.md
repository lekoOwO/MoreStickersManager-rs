# MoreStickersManager-rs

MoreStickersManager-rs, abbreviated MSM, is a Rust self-hosted manager for MoreStickers-compatible sticker packs.

Current phase: subscription links and access model after product organization parity.

For a concise implemented-versus-planned feature map, see
`docs/status/implementation-matrix.md`.

## Compatibility Target

MSM preserves the `.stickerpack` JSON shape used by Equicord moreStickers and MoreStickersConverter. The compatibility source of truth is documented in `docs/dev/compatibility.md`.

## Development

Use the development manager to switch local env profiles and control the API/Web
dev processes:

```powershell
npm run dev:env -- init development
npm run dev:env -- use development
npm run dev:start
npm run dev:status
npm run dev:stop
```

The manager loads `.env.<name>` first and `.env.local` second for private local
overrides. PID files, stdout logs, stderr logs, and Windows wrapper files are
stored under `tmp/dev-manager/`. Tracked examples are provided as
`.env.development.example` and `.env.testing.example`.
The `development` profile enables dev bootstrap by default: after the API is
healthy, the manager creates or reuses the local dev account, creates a PAT,
writes `VITE_MSM_PAT` into a managed `.env.local` block, imports a small sample
pack, and only then starts the Web process so Vite receives a usable API token.
Set `MSM_DEV_BOOTSTRAP_ENABLED=false` to disable this behavior.
The Web dev process is launched through the local Vite binary, so the manager
works from both `npm run ...` and `pnpm run ...` entrypoints. On Windows,
services are started through hidden wrapper processes to avoid visible console
windows for Rust/Vite child processes.

Run the current baseline checks:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run web:typecheck
npm run web:test
npm run web:build
```

GitHub Actions mirrors this baseline and the Web checks:

- `.github/workflows/ci.yml`: Rust, Web, and cross-platform service build checks.
- `.github/workflows/docker.yml`: publishes a multi-arch image to GHCR on `main` and `v*` tags.
- `.github/workflows/prerelease.yml`: publishes a moving `prerelease` release from `main`.
- `.github/workflows/release.yml`: publishes release binaries for `v*` tags.

## CLI Slice

The current CLI is an HTTP client for the API slice:

```powershell
cargo run -p msm-cli -- health
Invoke-WebRequest -UseBasicParsing http://127.0.0.1:3000/readyz
cargo run -p msm-cli -- packs list --user-id user_1
cargo run -p msm-cli -- packs import --tenant-id tenant_1 --owner-user-id user_1 --pack-id pack_1 --visibility private --file pack.stickerpack
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
cargo run -p msm-cli -- packs rename --pack-id pack_1 --title "Renamed Pack" --visibility public
cargo run -p msm-cli -- packs delete --pack-id pack_1
cargo run -p msm-cli -- pats create --id cli1 --user-id user_1 --name CLI --scope pack.read --scope asset.read
cargo run -p msm-cli -- pats list --user-id user_1
cargo run -p msm-cli -- pats revoke --token-id cli1
cargo run -p msm-cli -- tenants members list --tenant-id tenant_1
cargo run -p msm-cli -- tenants members set-role --tenant-id tenant_1 --user-id user_2 --role admin
cargo run -p msm-cli -- metadata folders create --id folder_1 --tenant-id tenant_1 --owner-user-id user_1 --name Favorites
cargo run -p msm-cli -- metadata folders list --tenant-id tenant_1 --owner-user-id user_1
cargo run -p msm-cli -- metadata folders packs add --folder-id folder_1 --pack-id pack_1 --sort-order 10
cargo run -p msm-cli -- metadata folders packs list --folder-id folder_1
cargo run -p msm-cli -- metadata folders packs remove --folder-id folder_1 --pack-id pack_1
cargo run -p msm-cli -- metadata tags create --id tag_1 --tenant-id tenant_1 --name cute
cargo run -p msm-cli -- metadata tags list --tenant-id tenant_1
cargo run -p msm-cli -- metadata pack-tags add --pack-id pack_1 --tag-id tag_1
cargo run -p msm-cli -- metadata pack-tags list --pack-id pack_1
cargo run -p msm-cli -- metadata pack-tags remove --pack-id pack_1 --tag-id tag_1
cargo run -p msm-cli -- metadata subscription-groups create --id sub_1 --tenant-id tenant_1 --owner-user-id user_1 --title Weekly --visibility private
cargo run -p msm-cli -- metadata subscription-groups list --tenant-id tenant_1 --owner-user-id user_1
cargo run -p msm-cli -- metadata subscription-groups packs add --subscription-group-id sub_1 --pack-id pack_1 --sort-order 20
cargo run -p msm-cli -- metadata subscription-groups packs list --subscription-group-id sub_1
cargo run -p msm-cli -- metadata subscription-groups packs remove --subscription-group-id sub_1 --pack-id pack_1
cargo run -p msm-cli -- exports kinds
cargo run -p msm-cli -- exports targets list --tenant-id tenant_1
cargo run -p msm-cli -- exports targets create --id target_telegram --tenant-id tenant_1 --kind telegram --name Telegram --config-json '{"botUsername":"msm_bot","botToken":"123:token"}'
cargo run -p msm-cli -- exports jobs create --id job_1 --tenant-id tenant_1 --source-pack-id pack_1 --target-id target_telegram --options-json '{"setNameSlug":"sample"}'
cargo run -p msm-cli -- exports jobs create --id job_reconcile --tenant-id tenant_1 --source-pack-id pack_1 --target-id target_telegram --telegram-live --telegram-reconcile-mode append-missing --execute-reconciliation --telegram-set-name-slug sample --telegram-default-emoji ok
cargo run -p msm-cli -- exports jobs get --job-id job_1
cargo run -p msm-cli -- exports jobs events --job-id job_1
```

Protected API commands can send a PAT with either:

```powershell
cargo run -p msm-cli -- --pat msm_pat_cli1_secret packs list --user-id user_1
$env:MSM_PAT="msm_pat_cli1_secret"
cargo run -p msm-cli -- packs list --user-id user_1
```

## Provider Slice

The `msm-providers` crate currently normalizes already-fetched provider JSON into
MoreStickers-compatible packs:

- Telegram sticker sets preserve `MoreStickers:Telegram:*` IDs and MSM self-hosted asset URLs.
- LINE sticker packs preserve `MoreStickers:Line:Pack:*` and `MoreStickers:Line:Sticker:*` IDs.
- LINE emoji packs preserve `MoreStickers:Line:Emoji-Pack:*` and `MoreStickers:Line-Emoji:*` IDs.

Remote provider fetch and asset download are intentionally separate future tasks.

## Export Pipeline

P24 started the export pipeline design for target-specific conversion and remote
publication. The current P33 focus is Telegram remote reconciliation usability:

- `msm-media`: partially implemented media kinds, Telegram static/video/thumbnail
  profiles, prepared output specs, conversion plan selection, and shell-free
  ffmpeg command planning. Media probing remains planned.
- `msm-exporters`: target registry for MoreStickers, Telegram, and future
  output targets. The base trait, capability metadata, request/plan types,
  duplicate-safe registry, concrete MoreStickers export target, Telegram sticker
  set planner, and Telegram reconciliation planner are implemented.
- `msm-telegram`: teloxide-based Telegram bot boundary with redacted token/config
  handling, Bot API URL configuration, mockable sticker set create/append,
  title/add/replace/delete mutation execution, and remote metadata fetches.

MSM can now plan Telegram sticker set names, size limits, create/append batches,
media profile selection, and teloxide `InputSticker` values without network
calls. Protected API/OpenAPI routes can list export capabilities, manage export
targets with redacted config responses, queue export jobs, and read job
status/events. The app worker can run MoreStickers serialization jobs, Telegram
dry-run planning jobs, Telegram publication jobs when options explicitly set
`"dryRun": false`, and guarded Telegram reconciliation mutation jobs. It can
optionally poll in the service process, write prepared media cache records
through the media executor boundary, publish prepared files through the teloxide
executor, fetch post-operation Telegram metadata, and refresh source sticker to
Telegram file mappings. Process-backed ffmpeg execution is available through
shell-free command plans.

Telegram jobs are dry-run by default. To create a Telegram sticker set, queue an
export job with options containing `"dryRun": false` or use the CLI
`--telegram-live` flag, and use a Telegram target config with `botToken`,
`botUsername`, and `ownerUserId`. Append-missing reconciliation can be queued
from Web controls, CLI flags, MCP named fields, or raw API/MCP job options.
Mirror-mode replace/delete requires `allowDestructiveReconciliation:true`.
OpenAPI documents the target-specific `TelegramExportJobOptions` schema behind
`CreateExportJobRequest.options` so API callers can discover the supported
Telegram fields without reading worker code.
Tests use injected publishers and do not contact Telegram.

## Web UI Slice

The current Web UI is a Vue/Vite frontend foundation with Shadcn Vue-compatible
local primitives and Tailwind CSS v4:

```powershell
npm run dev:start web
npm run dev:stop web
npm run web:dev
npm run web:typecheck
npm run web:test
npm run web:build
```

The Web UI includes a wide desktop workspace with Ant Design-inspired blue/gray
tokens, real workspace tabs, PAT, local login, and OIDC login-start dialogs, pack management,
pack import dialog, folder/tag/subscription-group metadata management, export
target settings, and a pack export wizard. API/OpenAPI, CLI, MCP, and Web now
expose folder-pack, pack-tag, and subscription-group pack membership links. It
can rename packs, change
public/private visibility, and delete packs when
`VITE_MSM_API_BASE_URL` is configured and the stored PAT has the required
scopes. It can also import a pasted MoreStickers `.stickerpack` JSON export
when the stored PAT has `import.run`.

Subscription payload endpoints are available at
`/api/public/packs/{pack_id}/subscription`,
`/api/public/packs/{pack_id}/stickerpack`, and
`/api/public/subscriptions/{subscription_group_id}`. Anonymous callers can read
public packs/groups; private payloads require an owner PAT or a matching
subscription access token. Private pack assets also accept an owner
`msm_session` Web session cookie from local login.

With the default development profile, `npm run dev:start` automatically points
the dashboard at `http://127.0.0.1:3000`, writes a development PAT to
`.env.local`, and imports a small sample sticker pack. To override those values
manually, set:

```powershell
$env:VITE_MSM_API_BASE_URL="http://localhost:3000"
$env:VITE_MSM_USER_ID="user_1"
$env:VITE_MSM_PAT="<raw-pat>"
npm run web:dev
```

If `VITE_MSM_API_BASE_URL` is not set, the dashboard uses deterministic mock
data for local preview and tests. P16 adds browser-local PAT storage, pack API
Bearer forwarding, and a basic PAT create/list/revoke panel. The token is stored
in localStorage key `msm.pat`; local login also receives an HttpOnly
`msm_session` cookie from the API for Web-session protected reads.

P19 adds Web local register/login controls backed by
`/api/v1/auth/local/register` and `/api/v1/auth/local/login`. Successful login
stores the returned PAT through the same `msm.pat` browser-local path and sets a
server-verified `msm_session` cookie.

The auth dialog can also start OIDC login through
`GET /api/v1/auth/oidc/{tenant_id}/{provider_id}/login?redirectUri=...`, show
the returned provider authorization URL/state/nonce/expiry, persist pending
state/nonce, pre-fill callback fields from `/auth/oidc/callback?code=...&state=...`,
submit callback completion data, and store the returned PAT.

## Service Binary

Run the current all-in-one service:

```powershell
npm run dev:start api
npm run dev:stop api
npm run web:build
cargo run -p msm-app
```

Environment variables:

- `MSM_BIND_ADDR`: bind address, default `127.0.0.1:3000`.
- `MSM_DATABASE_URL`: database URL, default `sqlite:data/msm.sqlite3`.
- `MSM_ASSET_DIR`: local asset directory, default `data/assets`.
- `MSM_WEB_DIST_DIR`: Web UI dist directory, default `apps/web/dist`.
- `MSM_REQUEST_BODY_LIMIT_BYTES`: maximum request body size accepted by the API/app router before JSON upload/import parsing, default `10485760` (10 MiB).
- `MSM_IMPORT_RATE_LIMIT_REQUESTS`: requests allowed per identity for import-like routes in one window, default `60`.
- `MSM_IMPORT_RATE_LIMIT_WINDOW_SECS`: import-like route rate-limit window length in seconds, default `60`.
- `MSM_FFMPEG_PATH`: ffmpeg path for future export conversion execution, default `ffmpeg`.
- `MSM_FFPROBE_PATH`: ffprobe path for future export probing execution, default `ffprobe`.
- `MSM_PREPARED_MEDIA_DIR`: prepared media output directory, default `data/prepared-media`.
- `MSM_EXPORT_MAX_CONCURRENT_JOBS`: future export worker concurrency, default `1`.
- `MSM_EXPORT_WORKER_ENABLED`: set to `true` to spawn the export worker polling loop, default `false`.
- `MSM_EXPORT_WORKER_POLL_INTERVAL_MS`: export worker poll interval, default `5000`.
- `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON`: optional JSON array of export targets to create/update at startup.


Structured operator logs and readiness diagnostics:

- `msm-app` writes JSON lines to stderr for `service_starting`, `service_listening`, and `http_request` events. HTTP request logs include method, path, status, and elapsed milliseconds, and intentionally omit query strings and credentials.
- `GET /readyz` returns dependency diagnostics for the database and local asset store. It returns `200` when all components are ready and `503` when any component is degraded.
Backup and restore: see `docs/user/backup-restore-runbook.md` for SQLite/PostgreSQL database backups, asset/prepared-media backups, deployment secrets, restore verification, and cross-instance portable-data migration boundaries.

Security review: see `docs/status/security-review.md` for token/session/subscription storage, secret redaction, private asset access, CDN, logging, and residual hardening notes.

Database backends:

- SQLite: use `sqlite:data/msm.sqlite3` or another `sqlite:<path>` URL. Ensure
  the parent directory is writable by the service user.
- PostgreSQL: use a PostgreSQL URL such as
  `postgres://msm:password@postgres:5432/msm`. The service selects the
  PostgreSQL migration set automatically and runs migrations at startup. In CI,
  `MSM_TEST_POSTGRES_URL` enables the optional PostgreSQL repository test legs.

When `apps/web/dist` exists before `cargo build -p msm-app`, P10 embeds that
dist into the binary. If dist is missing, the binary embeds a small placeholder
page so clean Rust builds still work. At runtime `MSM_WEB_DIST_DIR` remains a
disk override for development.

## Docker

Build the container image from the repository root:

```powershell
docker build -t morestickersmanager-rs .
```

The runtime image listens on `0.0.0.0:3000` and stores SQLite/assets under
`/data` by default.

## MCP Slice

`msm-app` exposes the MCP endpoint at `/mcp`. It supports JSON-RPC
`initialize`, `ping`, `tools/list`, and `tools/call` with these tools:

- `msm.list_sticker_packs`
- `msm.export_sticker_pack`
- `msm.import_sticker_pack`
- `msm.update_sticker_pack`
- `msm.delete_sticker_pack`
- `msm.list_folders`
- `msm.create_folder`
- `msm.list_folder_packs`
- `msm.add_pack_to_folder`
- `msm.remove_pack_from_folder`
- `msm.list_tags`
- `msm.create_tag`
- `msm.list_pack_tags`
- `msm.add_tag_to_pack`
- `msm.remove_tag_from_pack`
- `msm.list_subscription_groups`
- `msm.create_subscription_group`
- `msm.list_subscription_group_packs`
- `msm.add_pack_to_subscription_group`
- `msm.remove_pack_from_subscription_group`
- `msm.create_subscription_link`
- `msm.list_subscription_links`
- `msm.rotate_subscription_link`
- `msm.revoke_subscription_link`
- `msm.list_tenant_members`
- `msm.set_tenant_member_role`
- `msm.list_export_target_kinds`
- `msm.list_export_targets`
- `msm.create_export_target`
- `msm.create_export_job`
- `msm.get_export_job`
- `msm.list_export_job_events`
- `msm.list_telegram_publications`
- `msm.get_telegram_publication`

The MCP endpoint is intentionally stateless JSON-RPC over POST. It returns
`application/json` responses with `Cache-Control: no-store`, rejects SSE GET
negotiation with structured JSON, and does not issue MCP session IDs. Protected
MCP `tools/call` operations use Bearer PAT enforcement. See
`docs/dev/mcp-transport-contract.md` for the supported transport/security
contract.

## PAT Foundation

P12 adds Personal Access Token storage lifecycle support. Tokens use:

```text
msm_pat_<token_id>_<random_secret>
```

Only `sha256(random_secret)` is stored. Permission scopes use stable keys such
as `pack.read`, `asset.read`, `tenant.manage_members`, and `pat.manage`.
Protected API/CLI/MCP surfaces enforce Bearer PAT scopes.

P13 exposes PAT lifecycle APIs:

- `POST /api/v1/pats`
- `GET /api/v1/pats?userId=...`
- `DELETE /api/v1/pats/{token_id}`

Create responses include the raw token. List responses intentionally omit raw
tokens and token hashes.

P14 exposes those PAT lifecycle operations through the CLI. CLI create prints
the raw token once; list responses never include token hashes.

Bearer PAT scopes are enforced on protected API routes and MCP `tools/call`:

- `pack.read`: list/export sticker packs.
- `import.run`: import sticker packs.
- `pack.update`: rename sticker packs and update visibility.
- `pack.delete`: delete sticker packs.
- `export.read`: list/read export target kinds, targets, jobs, and job events.
- `export.run`: create export jobs.
- `export.target.manage`: create export targets.
- `tenant.manage_members`: list and update tenant member roles.
- `tenant.manage_settings`: read and update tenant settings such as name and
  public asset URL.
- `tenant.manage_users`: enable or disable tenant users.
- `tenant.manage_roles`: list and update tenant role templates.

API `healthz`, OpenAPI, PAT lifecycle endpoints, MCP `initialize`, MCP `ping`,
and MCP `tools/list` remain public in this bootstrap slice. OIDC-backed admin
enforcement is a later phase.

P18 adds local password bootstrap APIs:

- `POST /api/v1/auth/local/register`
- `POST /api/v1/auth/local/login`

Passwords are stored as Argon2 PHC hashes. Login returns a newly created PAT
using the same response shape as PAT creation and sets an HttpOnly
`msm_session` cookie for Web-session access.

P20 lets local registration optionally bootstrap a tenant admin by passing
`tenantId`, optional `tenantName`, and optional `tenantRole` fields. The role
defaults to `admin`.

Tenant member administration currently has API/OpenAPI, CLI, MCP, and Web
support:

- `GET /api/v1/tenants/{tenant_id}/members`
- `PUT /api/v1/tenants/{tenant_id}/members/{user_id}`
- `msm tenants members list --tenant-id <tenant_id>`
- `msm tenants members set-role --tenant-id <tenant_id> --user-id <user_id> --role <admin|user>`
- `msm.list_tenant_members`
- `msm.set_tenant_member_role`

These management surfaces require a Bearer PAT with `tenant.manage_members`,
and the PAT user must be an `admin` member of the target tenant. The Web UI
exposes the same member list and role assignment workflow from the Tenant admin
workspace.

Tenant settings administration currently has API/OpenAPI, CLI, MCP, and Web
support:

- `GET /api/v1/tenants/{tenant_id}/settings`
- `PUT /api/v1/tenants/{tenant_id}/settings`
- `msm tenants settings get --tenant-id <tenant_id>`
- `msm tenants settings update --tenant-id <tenant_id> --name <name> [--public-asset-url <url>]`
- `msm.get_tenant_settings`
- `msm.update_tenant_settings`

These routes require a Bearer PAT with `tenant.manage_settings`, and the PAT
user must be an `admin` member of the target tenant. The Tenant admin Web
workspace exposes tenant name and public asset/CDN URL controls.

Tenant user status administration currently has API/OpenAPI, CLI, MCP, and Web
support:

- `PUT /api/v1/tenants/{tenant_id}/users/{user_id}/status`
- `msm tenants users set-status --tenant-id <tenant_id> --user-id <user_id> --disabled`
- `msm.set_tenant_user_status`

This route requires a Bearer PAT with `tenant.manage_users`, the PAT user must
be an `admin` member of the target tenant, and the target user must also belong
to that tenant. The Tenant admin Web workspace exposes enable/disable controls.

OIDC provider administration currently has API/OpenAPI, CLI, MCP, and Web tenant admin support:

- `GET /api/v1/tenants/{tenant_id}/oidc-providers`
- `PUT /api/v1/tenants/{tenant_id}/oidc-providers/{provider_id}`
- `DELETE /api/v1/tenants/{tenant_id}/oidc-providers/{provider_id}`
- `msm tenants oidc-providers list --tenant-id <tenant_id>`
- `msm tenants oidc-providers upsert --tenant-id <tenant_id> --provider-id <provider_id> --display-name <name> --issuer-url <issuer_url> --client-id <client_id> --client-secret <client_secret> --scope openid [--scope email] [--disabled] [--deny-registration]`
- `msm tenants oidc-providers delete --tenant-id <tenant_id> --provider-id <provider_id>`
- `msm.list_oidc_providers`
- `msm.upsert_oidc_provider`
- `msm.delete_oidc_provider`

Provider responses redact `clientSecret`; update calls replace it with the
submitted secret. These routes require `tenant.manage_settings` and an admin
tenant membership. End-user Web SSO login-start and callback completion controls exist, including authorization-code exchange, verified ID-token/JWKS handling, userinfo fallback, PAT/session issuance, and `docs/user/oidc-sso.md` usage guidance.

Tenant role template administration currently has API/OpenAPI, CLI, MCP, and
Web support:

- `GET /api/v1/tenants/{tenant_id}/roles`
- `PUT /api/v1/tenants/{tenant_id}/roles/{role_id}`
- `msm tenants roles list --tenant-id <tenant_id>`
- `msm tenants roles upsert --tenant-id <tenant_id> --role-id <role_id> --name <name> --permission <permission_key>`
- `msm.list_tenant_roles`
- `msm.upsert_tenant_role`

These routes require a Bearer PAT with `tenant.manage_roles`, and the PAT user
must be an `admin` member of the target tenant. The Tenant admin Web workspace
uses selectable permission keys for role templates.

The finalized public/private pack, subscription group, subscription secret,
PAT, and Web-session read-access model is recorded in
`docs/status/decisions.md`. Private pack refresh and subscription endpoints
accept owner PATs, matching subscription secrets, or an owner `msm_session`
cookie; anonymous reads of public subscription groups still omit private packs.
Pack update, delete, and export API routes now use the domain RBAC policy
evaluator, so same-tenant admins with matching PAT scopes can manage
non-owned packs while regular non-owners are denied.

## Project Docs

- `docs/PRD.md`: living product requirements, status, roadmap, and completion checklist.
- `docs/dev/architecture.md`: architecture and crate boundaries.
- `docs/dev/compatibility.md`: sticker pack format compatibility.
- `docs/dev/providers.md`: provider normalization status.
- `docs/status/implementation-matrix.md`: compact implemented-versus-planned feature status.
- `docs/user/README.md`: user-facing documentation index.
- `docs/agents/README.md`: minimal agent handoff entrypoint.
- `docs/status/current.md`: current development state.
