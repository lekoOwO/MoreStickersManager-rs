# MoreStickersManager-rs

[繁體中文](README.zh-TW.md)

MoreStickersManager-rs (MSM) is a self-hosted sticker pack manager for
MoreStickers-compatible clients and export targets such as Telegram. It is a
Rust rewrite and expansion of the MoreStickersConverter companion tooling for
Equicord's moreStickers plugin.

MSM keeps the MoreStickers `.stickerpack` export contract stable while adding a
Web UI, HTTP API, CLI, MCP endpoint, provider import jobs, Telegram publication,
multi-tenant access control, and portable user data migration.

## Highlights

- **MoreStickers compatibility**: imports and exports the existing
  `.stickerpack` JSON shape.
- **Provider imports**: Telegram and LINE provider ingestion are implemented;
  Signal, WhatsApp, Kakao, Band, OGQ, and Viber are registered as planned
  provider families for future work.
- **Telegram export**: prepares media with ffmpeg/ffprobe, plans sticker-set
  creation/update flows, publishes through a teloxide-backed Telegram Bot API
  boundary, stores publication history, and supports guarded reconciliation.
- **Web UI**: responsive Vue UI with desktop/mobile layouts, dark/light theme,
  Traditional Chinese and English locales, pack management, exports,
  subscription links, tenant administration, and migration workflows.
- **API/OpenAPI**: axum API with utoipa OpenAPI output at `/openapi.json`.
- **CLI and MCP**: command-line and JSON-RPC MCP surfaces for automation.
- **Multi-tenant security**: tenant membership, admin/user roles, fine-grained
  permissions, PATs, local accounts, OIDC/SSO, and protected asset access.
- **Data portability**: export/import user data for migration between MSM
  instances.
- **Storage backends**: SQLite and PostgreSQL migrations/repositories.
- **Self-contained binary**: the Web UI can be embedded into the Rust service
  binary at build time.

## Repository layout

```text
apps/web/                  Vue + Tailwind CSS v4 + shadcn-vue Web UI
crates/msm-app/            All-in-one HTTP service and worker orchestration
crates/msm-api/            API routes and OpenAPI schema
crates/msm-cli/            CLI client
crates/msm-domain/         Sticker pack domain model and compatibility helpers
crates/msm-exporters/      Export target registry and Telegram planning
crates/msm-mcp/            MCP JSON-RPC endpoint and tools
crates/msm-media/          Media probing/conversion planning
crates/msm-providers/      Telegram/LINE provider normalization and fetch plans
crates/msm-storage/        SQLite/PostgreSQL storage layer
crates/msm-telegram/       teloxide-backed Telegram Bot API boundary
docs/                      User, developer, status, and release-readiness docs
examples/docker/           Docker Compose deployment example
```

## Quick start with Docker Compose

The easiest deployment path is the Compose example with PostgreSQL:

```bash
cp examples/docker/.env.example examples/docker/.env
# edit examples/docker/.env
docker compose --env-file examples/docker/.env -f examples/docker/docker-compose.yml up -d --build
curl -fsS http://localhost:3000/readyz
```

Open `http://localhost:3000` or the `MSM_EXTERNAL_URL` you configured.

For Authentik SSO, create an Authentik OAuth2/OpenID provider and set the
redirect URI to:

```text
${MSM_EXTERNAL_URL}/auth/oidc/callback
```

Then store the issuer URL, client ID, and client secret in
`examples/docker/.env`, bootstrap the first tenant admin, and register the OIDC
provider in MSM. See [`examples/docker/README.md`](examples/docker/README.md)
for the full walkthrough.

## Local development

Prerequisites:

- Rust stable toolchain
- Bun or Node.js/npm for Web development
- ffmpeg and ffprobe for media conversion flows
- Optional: PostgreSQL for backend parity testing

Install Web dependencies and run checks:

```bash
bun install --frozen-lockfile
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run web:typecheck
npm run web:test
npm run web:build
```

Run the all-in-one service locally:

```bash
npm run web:build
cargo run -p msm-app
```

By default the service listens on `127.0.0.1:3000`, uses
`sqlite:data/msm.sqlite3`, serves local assets from `data/assets`, and exposes:

- `GET /healthz`
- `GET /readyz`
- `GET /openapi.json`
- `POST /mcp`

## Configuration

Common environment variables:

| Variable | Default | Description |
| --- | --- | --- |
| `MSM_BIND_ADDR` | `127.0.0.1:3000` | Service bind address. |
| `MSM_DATABASE_URL` | `sqlite:data/msm.sqlite3` | `sqlite:<path>` or PostgreSQL URL. |
| `MSM_ASSET_DIR` | `data/assets` | Local source asset directory. |
| `MSM_PREPARED_MEDIA_DIR` | `data/prepared-media` | Converted media cache/output directory. |
| `MSM_WEB_DIST_DIR` | `apps/web/dist` | Optional runtime Web dist override. |
| `MSM_PUBLIC_ASSET_URL` | unset | System-wide CDN/public asset URL fallback. |
| `MSM_PUBLIC_ASSET_BASE_URL` | derived from bind addr | Public base used by provider import workers. |
| `MSM_REQUEST_BODY_LIMIT_BYTES` | `10485760` | API request body cap. |
| `MSM_IMPORT_RATE_LIMIT_REQUESTS` | `60` | Per-identity import-like request limit. |
| `MSM_IMPORT_RATE_LIMIT_WINDOW_SECS` | `60` | Rate-limit window length. |
| `MSM_FFMPEG_PATH` | `ffmpeg` | ffmpeg executable path. |
| `MSM_FFPROBE_PATH` | `ffprobe` | ffprobe executable path. |
| `MSM_EXPORT_WORKER_ENABLED` | `false` | Enable export worker polling. |
| `MSM_PROVIDER_IMPORT_WORKER_ENABLED` | `false` | Enable provider import worker polling. |
| `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON` | unset | Optional startup export-target bootstrap JSON. |

## Authentication and SSO

MSM supports local account registration/login and tenant-scoped OIDC providers.
A successful local or OIDC login returns a PAT and sets an HttpOnly
`msm_session` cookie for Web-session-protected reads.

OIDC provider administration is available through Web Tenant admin, API, CLI,
and MCP. User-facing SSO guidance is in [`docs/user/oidc-sso.md`](docs/user/oidc-sso.md).

## CLI

The CLI binary is named `msm`:

```bash
cargo run -p msm-cli -- --help
cargo run -p msm-cli -- --base-url http://127.0.0.1:3000 --pat "$MSM_PAT" packs list --user-id user_1
```

## MCP

The MCP endpoint is stateless JSON-RPC over HTTP POST at `/mcp`. Public
metadata methods include `initialize`, `ping`, and `tools/list`. Protected tool
calls require an HTTP `Authorization: Bearer msm_pat_...` header.

See [`docs/dev/mcp-transport-contract.md`](docs/dev/mcp-transport-contract.md)
for the transport contract.

## Documentation

- [`docs/user/README.md`](docs/user/README.md): user guide and API/CLI examples
- [`docs/user/oidc-sso.md`](docs/user/oidc-sso.md): OIDC/SSO guide
- [`docs/user/backup-restore-runbook.md`](docs/user/backup-restore-runbook.md): backup and restore
- [`docs/dev/architecture.md`](docs/dev/architecture.md): architecture notes
- [`docs/dev/compatibility.md`](docs/dev/compatibility.md): MoreStickers compatibility
- [`docs/status/completion-audit.md`](docs/status/completion-audit.md): release-readiness audit
- [`docs/PRD.md`](docs/PRD.md): product requirements and completion status

## Current status

The current PRD contract is complete and release-readiness verification is
recorded in [`docs/status/completion-audit.md`](docs/status/completion-audit.md).
Future product scope should be tracked as a new PRD revision before
implementation.

## License

This workspace declares `MIT OR Apache-2.0` in `Cargo.toml`.
