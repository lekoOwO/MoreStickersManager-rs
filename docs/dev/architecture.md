# Architecture

MSM is built as a Rust workspace. The domain crate owns MoreStickers compatibility types and provider-neutral logic. Later crates add storage, API, CLI, MCP, providers, and the final app binary.

## Crate Boundaries

- `msm-domain`: compatibility models, pure ID helpers, pure URL resolution, and import/export helpers.
- `msm-domain::authz`: pure authorization policy evaluation for pack and subscription access.
- `msm-storage`: database repositories, asset storage, and export job persistence, added across P2/P12/P18/P21/Task 3.
- `msm-api`: HTTP API and OpenAPI, added in P4.
- `msm-cli`: command-line client, added in P5.
- `msm-mcp`: MCP JSON-RPC endpoint and tool execution, added in P11.
- `msm-providers`: provider registry and provider-specific normalization into `MoreStickers` packs, added in P6.
- `msm-media`: media profile and command planning foundation added in P25; media probing, converter execution, and prepared output caching remain planned.
- `msm-exporters`: export target trait, registry, MoreStickers export adapter, and Telegram export planner added in Tasks 4-7; remote execution and future output targets remain planned.
- `msm-telegram`: teloxide-based Telegram bot boundary with redacted token/config handling and Bot API URL configuration.
- `msm-app`: runnable service composition binary, added in P9.
- `apps/web`: Vue/Vite Web UI foundation with Shadcn Vue-compatible primitives and Tailwind CSS v4, added in P7.

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
- `msm-exporters`: target traits, capability metadata, export plans, target registry, MoreStickers serialization target, and Telegram sticker set planner.
- `msm-telegram`: `teloxide::Bot` construction and configuration; sticker upload, set creation, and set append should use teloxide requester methods in later worker/exporter phases.

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
execution for the current pack operations. It reuses `msm-api::ApiState` so the
service binary can mount `/mcp` next to the HTTP API. P11 intentionally supports
JSON `POST` request/response only; Streamable HTTP SSE, session management, and
PAT/RBAC enforcement belong to later auth and transport hardening phases.

## PAT Boundary

P12 implements PAT lifecycle persistence in `msm-storage`. Raw tokens are only
returned at creation time. The database stores token IDs, SHA-256 token secret
hashes, scope keys, expiry timestamps, and revocation timestamps. API/CLI/MCP
middleware must use the repository verification method rather than reading token
hashes directly.
