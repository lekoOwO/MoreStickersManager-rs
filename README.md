# MoreStickersManager-rs

MoreStickersManager-rs, abbreviated MSM, is a Rust self-hosted manager for MoreStickers-compatible sticker packs.

Current phase: P14 CLI PAT commands.

## Compatibility Target

MSM preserves the `.stickerpack` JSON shape used by Equicord moreStickers and MoreStickersConverter. The compatibility source of truth is documented in `docs/dev/compatibility.md`.

## Development

Run the current baseline checks:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Before the Rust workspace exists, use:

```powershell
git status --short
```

## CLI Slice

The current CLI is an HTTP client for the API slice:

```powershell
cargo run -p msm-cli -- health
cargo run -p msm-cli -- packs list --user-id user_1
cargo run -p msm-cli -- packs import --tenant-id tenant_1 --owner-user-id user_1 --pack-id pack_1 --visibility private --file pack.stickerpack
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
cargo run -p msm-cli -- pats create --id cli1 --user-id user_1 --name CLI --scope pack.read --scope asset.read
cargo run -p msm-cli -- pats list --user-id user_1
cargo run -p msm-cli -- pats revoke --token-id cli1
```

## Provider Slice

The `msm-providers` crate currently normalizes already-fetched provider JSON into
MoreStickers-compatible packs:

- Telegram sticker sets preserve `MoreStickers:Telegram:*` IDs and MSM self-hosted asset URLs.
- LINE sticker packs preserve `MoreStickers:Line:Pack:*` and `MoreStickers:Line:Sticker:*` IDs.
- LINE emoji packs preserve `MoreStickers:Line:Emoji-Pack:*` and `MoreStickers:Line-Emoji:*` IDs.

Remote provider fetch and asset download are intentionally separate future tasks.

## Web UI Slice

The current Web UI is a Vue/Vite frontend foundation with Shadcn Vue-compatible
local primitives and Tailwind CSS v4:

```powershell
npm run web:dev
npm run web:typecheck
npm run web:test
npm run web:build
```

P7 uses mock sticker-pack data only. Backend API integration, authentication,
CRUD, and binary embedding are later phases.

To connect the dashboard to the current P4 API list route, set:

```powershell
$env:VITE_MSM_API_BASE_URL="http://localhost:3000"
$env:VITE_MSM_USER_ID="user_1"
npm run web:dev
```

If `VITE_MSM_API_BASE_URL` is not set, the dashboard uses deterministic mock
data for local preview and tests.

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

When `apps/web/dist` exists before `cargo build -p msm-app`, P10 embeds that
dist into the binary. If dist is missing, the binary embeds a small placeholder
page so clean Rust builds still work. At runtime `MSM_WEB_DIST_DIR` remains a
disk override for development.

## MCP Slice

`msm-app` exposes the initial MCP endpoint at `/mcp`. P11 supports JSON-RPC
`initialize`, `ping`, `tools/list`, and `tools/call` with these tools:

- `msm.list_sticker_packs`
- `msm.export_sticker_pack`
- `msm.import_sticker_pack`

This first MCP slice returns `application/json` responses and does not yet
implement SSE streams, session management, or PAT/RBAC enforcement.

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
the raw token once; list responses never include token hashes. PAT/RBAC
enforcement on existing API, CLI, and MCP operations remains a later integration
phase.

## Project Docs

- `docs/dev/architecture.md`: architecture and crate boundaries.
- `docs/dev/compatibility.md`: sticker pack format compatibility.
- `docs/dev/providers.md`: provider normalization status.
- `docs/user/README.md`: user-facing documentation index.
- `docs/agents/README.md`: agent handoff entrypoint.
- `docs/status/current.md`: current development state.
