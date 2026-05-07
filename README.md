# MoreStickersManager-rs

MoreStickersManager-rs, abbreviated MSM, is a Rust self-hosted manager for MoreStickers-compatible sticker packs.

Current phase: P25 export worker foundation.

For a concise implemented-versus-planned feature map, see
`docs/status/implementation-matrix.md`.

## Compatibility Target

MSM preserves the `.stickerpack` JSON shape used by Equicord moreStickers and MoreStickersConverter. The compatibility source of truth is documented in `docs/dev/compatibility.md`.

## Development

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
cargo run -p msm-cli -- packs list --user-id user_1
cargo run -p msm-cli -- packs import --tenant-id tenant_1 --owner-user-id user_1 --pack-id pack_1 --visibility private --file pack.stickerpack
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
cargo run -p msm-cli -- packs rename --pack-id pack_1 --title "Renamed Pack" --visibility public
cargo run -p msm-cli -- packs delete --pack-id pack_1
cargo run -p msm-cli -- pats create --id cli1 --user-id user_1 --name CLI --scope pack.read --scope asset.read
cargo run -p msm-cli -- pats list --user-id user_1
cargo run -p msm-cli -- pats revoke --token-id cli1
cargo run -p msm-cli -- exports kinds
cargo run -p msm-cli -- exports targets list --tenant-id tenant_1
cargo run -p msm-cli -- exports targets create --id target_telegram --tenant-id tenant_1 --kind telegram --name Telegram --config-json '{"botUsername":"msm_bot","botToken":"123:token"}'
cargo run -p msm-cli -- exports jobs create --id job_1 --tenant-id tenant_1 --source-pack-id pack_1 --target-id target_telegram --options-json '{"setNameSlug":"sample"}'
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

## Export Pipeline Planning

P24 documents the planned export pipeline for target-specific conversion and
remote publication:

- `msm-media`: partially implemented media kinds, Telegram static/video/thumbnail
  profiles, prepared output specs, conversion plan selection, and shell-free
  ffmpeg command planning. Media probing, converter execution, and prepared
  output caching remain planned.
- `msm-exporters`: target registry for MoreStickers, Telegram, and future
  output targets. The base trait, capability metadata, request/plan types,
  duplicate-safe registry, concrete MoreStickers export target, and Telegram
  sticker set planner are implemented.
- `msm-telegram`: teloxide-based Telegram bot boundary with redacted token/config
  handling and Bot API URL configuration.

Telegram sticker set creation is not implemented yet. MSM can now plan Telegram
sticker set names, size limits, create/append batches, media profile selection,
and teloxide `InputSticker` values without network calls. Protected API/OpenAPI
routes can list export capabilities, manage export targets with redacted config
responses, queue export jobs, and read job status/events. The app worker can run
MoreStickers serialization jobs and Telegram dry-run planning jobs from queued
records, optionally poll in the service process, and write prepared media cache
records through the media executor boundary. Process-backed ffmpeg execution is
available through shell-free command plans. The CLI can list target kinds,
create/list export targets, create export jobs, and read job status/events.
MCP tools can list target kinds, list/create export targets, create export jobs,
and read job status/events. The Web dashboard can configure export targets,
queue export jobs, and show job status/events. Telegram upload/set creation is still planned in
`docs/superpowers/plans/2026-05-06-msm-telegram-export-pipeline.md`.

## Web UI Slice

The current Web UI is a Vue/Vite frontend foundation with Shadcn Vue-compatible
local primitives and Tailwind CSS v4:

```powershell
npm run web:dev
npm run web:typecheck
npm run web:test
npm run web:build
```

The Web UI includes dashboard, PAT, and local register/login bootstrap slices.
The dashboard can rename packs, change public/private visibility, and delete
packs when `VITE_MSM_API_BASE_URL` is configured and the stored PAT has the
required scopes. It can also import a pasted MoreStickers `.stickerpack` JSON
export when the stored PAT has `import.run`. The dashboard now includes export
target settings and a pack export wizard for queuing export jobs and reading
job progress.

To connect the dashboard to the current P4 API list route, set:

```powershell
$env:VITE_MSM_API_BASE_URL="http://localhost:3000"
$env:VITE_MSM_USER_ID="user_1"
$env:VITE_MSM_PAT="msm_pat_cli1_secret"
npm run web:dev
```

If `VITE_MSM_API_BASE_URL` is not set, the dashboard uses deterministic mock
data for local preview and tests. P16 adds browser-local PAT storage, pack API
Bearer forwarding, and a basic PAT create/list/revoke panel. The token is stored
in localStorage key `msm.pat`; this is a bootstrap UX, not a replacement for
future login/session storage.

P19 adds Web local register/login controls backed by
`/api/v1/auth/local/register` and `/api/v1/auth/local/login`. Successful login
stores the returned PAT through the same `msm.pat` browser-local path.

## Service Binary

Run the current all-in-one service:

```powershell
npm run web:build
cargo run -p msm-app
```

Environment variables:

- `MSM_BIND_ADDR`: bind address, default `127.0.0.1:3000`.
- `MSM_DATABASE_URL`: database URL, default `sqlite:data/msm.sqlite3`.
- `MSM_ASSET_DIR`: local asset directory, default `data/assets`.
- `MSM_WEB_DIST_DIR`: Web UI dist directory, default `apps/web/dist`.
- `MSM_FFMPEG_PATH`: ffmpeg path for future export conversion execution, default `ffmpeg`.
- `MSM_FFPROBE_PATH`: ffprobe path for future export probing execution, default `ffprobe`.
- `MSM_PREPARED_MEDIA_DIR`: prepared media output directory, default `data/prepared-media`.
- `MSM_EXPORT_MAX_CONCURRENT_JOBS`: future export worker concurrency, default `1`.
- `MSM_EXPORT_WORKER_ENABLED`: set to `true` to spawn the export worker polling loop, default `false`.
- `MSM_EXPORT_WORKER_POLL_INTERVAL_MS`: export worker poll interval, default `5000`.
- `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON`: optional JSON array of export targets to create/update at startup.

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

`msm-app` exposes the initial MCP endpoint at `/mcp`. P11 supports JSON-RPC
`initialize`, `ping`, `tools/list`, and `tools/call` with these tools:

- `msm.list_sticker_packs`
- `msm.export_sticker_pack`
- `msm.import_sticker_pack`
- `msm.update_sticker_pack`
- `msm.delete_sticker_pack`
- `msm.list_export_target_kinds`
- `msm.list_export_targets`
- `msm.create_export_target`
- `msm.create_export_job`
- `msm.get_export_job`
- `msm.list_export_job_events`

This first MCP slice returns `application/json` responses and does not yet
implement SSE streams or session management. MCP `tools/call` pack and export
operations use Bearer PAT enforcement.

## PAT Foundation

P12 adds Personal Access Token storage lifecycle support. Tokens use:

```text
msm_pat_<token_id>_<random_secret>
```

Only `sha256(random_secret)` is stored. Permission scopes use stable keys such
as `pack.read`, `asset.read`, and `pat.manage`. API/CLI/MCP enforcement is a
later auth integration phase.

P13 exposes PAT lifecycle APIs:

- `POST /api/v1/pats`
- `GET /api/v1/pats?userId=...`
- `DELETE /api/v1/pats/{token_id}`

Create responses include the raw token. List responses intentionally omit raw
tokens and token hashes.

P14 exposes those PAT lifecycle operations through the CLI. CLI create prints
the raw token once; list responses never include token hashes.

P15 enforces Bearer PAT scopes on pack API routes and MCP `tools/call`:

- `pack.read`: list/export sticker packs.
- `import.run`: import sticker packs.
- `pack.update`: rename sticker packs and update visibility.
- `pack.delete`: delete sticker packs.
- `export.read`: list/read export target kinds, targets, jobs, and job events.
- `export.run`: create export jobs.
- `export.target.manage`: create export targets.

API `healthz`, OpenAPI, PAT lifecycle endpoints, MCP `initialize`, MCP `ping`,
and MCP `tools/list` remain public in this bootstrap slice. Asset privacy and
OIDC/local-login backed admin enforcement are later phases.

P18 adds local password bootstrap APIs:

- `POST /api/v1/auth/local/register`
- `POST /api/v1/auth/local/login`

Passwords are stored as Argon2 PHC hashes. Login returns a newly created PAT
using the same response shape as PAT creation.

P20 lets local registration optionally bootstrap a tenant admin by passing
`tenantId`, optional `tenantName`, and optional `tenantRole` fields. The role
defaults to `admin`.

## Project Docs

- `docs/dev/architecture.md`: architecture and crate boundaries.
- `docs/dev/compatibility.md`: sticker pack format compatibility.
- `docs/dev/providers.md`: provider normalization status.
- `docs/status/implementation-matrix.md`: implemented-versus-planned feature status.
- `docs/user/README.md`: user-facing documentation index.
- `docs/agents/README.md`: agent handoff entrypoint.
- `docs/status/current.md`: current development state.
