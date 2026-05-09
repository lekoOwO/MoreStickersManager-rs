# Architecture

MSM is built as a Rust workspace. The domain crate owns MoreStickers compatibility types and provider-neutral logic. Later crates add storage, API, CLI, MCP, providers, and the final app binary.

## Crate Boundaries

- `msm-domain`: compatibility models, pure ID helpers, pure URL resolution, and import/export helpers.
- `msm-domain::authz`: pure authorization policy evaluation for pack and subscription access.
- `msm-storage`: database repositories, asset storage, and export job persistence, added across P2/P12/P18/P21/Task 3.
- `msm-api`: HTTP API and OpenAPI, added in P4 and extended with export target/job routes in Task 8.
- `msm-cli`: command-line client, added in P5 and extended with export target/job commands in Task 10.
- `msm-mcp`: MCP JSON-RPC endpoint and tool execution, added in P11 and extended with export target/job and Telegram publication history tools.
- `msm-providers`: provider registry and provider-specific normalization into `MoreStickers` packs, added in P6.
- `msm-media`: media profile and command planning foundation added in P25; media probing remains planned.
- `msm-exporters`: export target trait, registry, MoreStickers export adapter, Telegram export planner, and Telegram reconciliation policy planner added in Tasks 4-7/P33; future output targets remain planned.
- `msm-telegram`: teloxide-based Telegram bot boundary with redacted token/config handling, Bot API URL configuration, mockable sticker set create/append execution, and mockable sticker set mutation execution.
- `msm-app`: runnable service composition binary, added in P9 and extended with export worker execution, prepared media conversion, and Telegram publication.
- `apps/web`: Vue/Vite Web UI foundation with Shadcn Vue-compatible primitives and Tailwind CSS v4, added in P7 and extended with export target/job workflow controls and Telegram publication result display.

## Dependency Rule

`msm-domain` must not depend on Axum, SQLx, provider SDKs, frontend code, or runtime-specific infrastructure.

Authorization policies stay in `msm-domain` so API, CLI, MCP, and Web UI assumptions can share the same rules.

`msm-domain::Permission` also owns stable PAT scope keys such as `pack.read`,
`asset.read`, and `pat.manage`.

## Provider Boundary

`msm-providers` converts provider-specific payloads into `msm-domain::StickerPack`.
It must keep output IDs and `.stickerpack` field names compatible with upstream
moreStickers conventions. Network fetching and asset downloading are not part of
the P6 provider boundary; they should be added behind explicit provider
capabilities so API, CLI, MCP, and Web UI can expose the same feature set.

Providers are input adapters only. Telegram can be both an input provider and an
output destination, but those roles must stay separate: Telegram import belongs
to `msm-providers`, while Telegram sticker set creation belongs to the exporter
pipeline.

## Export Target Boundary

Export targets publish or serialize a canonical MSM sticker pack. The existing
MoreStickers `.stickerpack` output is an export target, and Telegram sticker set
creation is the first planned remote publication target.

Planned exporter work is split into:

- `msm-media`: target-neutral media kinds, output profiles, conversion plans, shell-free converter command plans, and planned probing/converter execution.
- `msm-exporters`: target traits, capability metadata, export plans, target registry, MoreStickers serialization target, Telegram sticker set planner, and pure Telegram reconciliation policy planner.
- `msm-telegram`: `teloxide::Bot` construction/configuration plus sticker set create/append/update/delete methods behind a mockable trait.

Task 8 exposes export target and queued job records through protected API routes.
Those routes do not execute conversions or remote publication; Task 9 worker
execution owns that state transition.

The worker can run MoreStickers serialization jobs, Telegram dry-run planning
jobs, and Telegram publication jobs from queued storage records. It writes
prepared media cache records through an injected media executor and can be
started as an optional polling loop. The process-backed prepared media executor
runs shell-free ffmpeg command plans and returns output metadata. Telegram
publication remains opt-in: job options default to `"dryRun": true`, and remote
publication only runs when options explicitly set `"dryRun": false` and the
target config includes `botToken`, `botUsername`, and `ownerUserId`. Retryable
worker failures requeue jobs with bounded `attempt_count`, `max_attempts`, and
`next_attempt_at` metadata before terminal failure.

Telegram remote reconciliation is modeled before execution. The exporter layer
can plan create-only, append-missing, and mirror policies from known remote
sticker set state. Planned operations cover set creation, title update, sticker
keep/add/replace, and remote-only sticker deletion. `msm-telegram` can execute
ordered mutation sequences for title update, add, replace, and delete through a
mockable teloxide-backed trait. Worker dry-run jobs can accept supplied remote
state and return reconciliation operation/mutation summaries. Non-dry-run
append-missing reconciliation can execute mutations when `executeReconciliation`
is explicitly enabled. Mirror-mode replace/delete also requires
`allowDestructiveReconciliation` before mutation execution. Remote state
retrieval exists in the Telegram boundary through a mockable `getStickerSet`
adapter, and the worker can derive `TelegramRemoteSet` by combining fetched
Telegram metadata with stored MSM source sticker to Telegram file mappings when
callers omit `remoteSet`.

Storage now has a `telegram_sticker_mappings` table for MSM source sticker IDs,
Telegram file IDs, file unique IDs, and positions per publication/target/set.
The table is the durable bridge used for fetched-state reconciliation.
Successful non-dry-run publication jobs and reconciliation mutation jobs refresh
it from post-operation `getStickerSet` results.

Startup export targets can be bootstrapped from `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON`.
Task 10 exposes the same target/job operations through CLI and MCP. Task 11 adds
Web export target settings, Telegram token validation, export job queueing, job
event display, completed sticker set URL display, and Web reconciliation
controls. CLI/MCP named reconciliation affordances and the OpenAPI
`TelegramExportJobOptions` schema now document the same worker options. Use
`docs/user/telegram-reconciliation-runbook.md` for destructive mirror operator
flow.

Worker tests keep Telegram network access behind injected fake publishers. Local
and CI verification must not call Telegram; live publication requires an
operator-created target and an explicitly non-dry-run queued job.

No export target may mutate MoreStickers-compatible pack JSON as a side effect of
publishing. Target-specific prepared media should be cached separately from the
canonical pack.

## Frontend Boundary

`apps/web` is an npm workspace package. It owns the browser UI, local UI
primitives, i18n labels, theme preferences, and mock frontend client used by P7.
The frontend must keep API access behind small client modules so later OpenAPI
or handwritten HTTP clients can replace mock data without rewriting dashboard
components. `apps/web/dist` is a build artifact and must remain ignored until a
Rust embedding phase copies or embeds it intentionally.

Export Web UI code follows the same boundary: `apps/web/src/lib/exportApi.ts`
contains HTTP calls, while `ExportTargetPanel`, `PackExportWizard`, and
`ExportJobTimeline` receive injectable clients for tests. The UI may queue and
inspect jobs, but it must not duplicate worker conversion or Telegram publishing
logic.

## Service Boundary

`msm-app` is the composition crate. It reads runtime configuration from
environment variables, runs storage migrations, creates the local asset store,
mounts the API router, and serves Web UI assets.

P10 embeds Web assets into the binary through an `msm-app` build script. If
`apps/web/dist/index.html` exists during compilation, that real dist is embedded.
Otherwise a committed placeholder is embedded so clean Rust-only builds remain
valid. Runtime serving is disk-first (`MSM_WEB_DIST_DIR`) and embedded-second.

## MCP Boundary

`msm-mcp` owns MCP JSON-RPC request/response shapes, tool metadata, and tool
execution for pack and export operations. It reuses `msm-api::ApiState` so the
service binary can mount `/mcp` next to the HTTP API. P11 intentionally supports
JSON `POST` request/response only; Streamable HTTP SSE and session management
belong to later transport hardening phases. Pack and export tool calls currently
enforce Bearer PAT scopes.

## PAT Boundary

P12 implements PAT lifecycle persistence in `msm-storage`. Raw tokens are only
returned at creation time. The database stores token IDs, SHA-256 token secret
hashes, scope keys, expiry timestamps, and revocation timestamps. API/CLI/MCP
middleware must use the repository verification method rather than reading token
hashes directly. PAT lifecycle API routes require a same-user Bearer PAT with
`pat.manage`; local login and PAT creation reject requested scopes outside the
user's built-in user permissions, tenant-admin permissions, or custom
role-template permissions.

Local Web sessions use the same hash-only secret storage pattern in the
`web_sessions` table. `/api/v1/auth/local/login` preserves the PAT response body
for API/Web bootstrap compatibility and also sets an HttpOnly `msm_session`
cookie. Asset authorization accepts that cookie only when the session user owns
the private pack being read.

Tenant member administration currently lives in `tenant_members`. The API
requires both a `tenant.manage_members` PAT scope and an `admin` membership in
the target tenant before listing or upserting member roles. This is the first
tenant-admin slice. Role templates, tenant settings, local-registration
enable/disable, tenant user status, and UI/CLI/MCP parity are implemented for
the current tenant administration surface; OIDC/SSO provider administration is
still future work.
