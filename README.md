# MoreStickersManager-rs

[繁體中文](README.zh-TW.md)

**MoreStickersManager-rs (MSM)** is a self-hosted sticker library for people who
collect, manage, share, subscribe to, and publish sticker packs across
MoreStickers/Discord and Telegram.

Instead of keeping sticker exports in scattered files, MSM gives you one private
place to import packs, organize them, generate subscription links, control who
can access private assets, and publish selected packs to Telegram sticker sets.

## What you can do with MSM

### Manage sticker packs in one place

- Import existing MoreStickers `.stickerpack` exports.
- Import packs from supported providers such as Telegram and LINE.
- Create, rename, update, delete, and re-export sticker packs.
- Keep public packs openly accessible while protecting private packs behind
  login, PATs, or subscription secrets.
- Store sticker images on your own MSM instance and optionally publish asset
  URLs through a CDN such as Cloudflare.

### Organize packs for real daily use

- Put packs into folders.
- Add tags to packs.
- Build curated subscription groups from multiple packs.
- Manage everything from the Web UI instead of manually editing JSON files.

### Share auto-updating subscriptions

MSM can generate subscription-style endpoints for both individual sticker packs
and custom sticker-pack groups.

Use this when you want a client such as moreStickers to periodically refresh the
latest version of a pack or a curated collection:

- **Pack subscription**: every pack has its own default subscription-style
  payload so consumers can follow that pack directly.
- **Subscription groups**: create a group, add multiple packs, and share one
  subscription link for the entire collection.
- **Public subscriptions**: anyone with the link can read public payloads.
- **Protected subscriptions**: private pack/group payloads and assets require a
  matching subscription secret, a PAT, or an authenticated Web session.
- **Rotatable links**: subscription access tokens can be created, rotated, and
  revoked when links leak or membership changes.

### Publish to Telegram

MSM includes a Telegram export pipeline inspired by sticker-bot workflows:

- Prepare images/video for Telegram sticker requirements with ffmpeg/ffprobe.
- Create Telegram sticker sets through a bot token.
- Append missing stickers to an existing set.
- Reconcile MSM packs with Telegram sets using create-only, append-missing, or
  guarded mirror behavior.
- Keep publication history and per-sticker Telegram mapping records.
- Run exports from the Web UI, API, CLI, or MCP.

### Run it for yourself, a team, or a community

- Multi-tenant data model.
- Admin and regular-user roles.
- Fine-grained permissions for packs, assets, imports, exports, subscriptions,
  PATs, and tenant administration.
- Local accounts and OIDC/SSO login.
- Personal Access Tokens for API, CLI, and MCP clients.
- Export/import user data to migrate between MSM instances.

## Screens and workflows

The Web UI is designed as the primary management surface:

- Dashboard overview of packs and system state.
- Pack management with import, rename, visibility, delete, and export actions.
- Provider import workspace for Telegram/LINE.
- Organize workspace for folders, tags, subscription groups, and memberships.
- Export workspace for MoreStickers and Telegram jobs.
- Tenant admin workspace for members, roles, settings, local registration, and
  OIDC providers.
- Migration workspace for portable user export/import.
- Desktop and mobile layouts, dark/light mode, English and Traditional Chinese.

## Quick start with Docker Compose

The Compose example runs MSM with PostgreSQL:

```bash
cp examples/docker/.env.example examples/docker/.env
# edit examples/docker/.env
docker compose --env-file examples/docker/.env -f examples/docker/docker-compose.yml up -d --build
curl -fsS http://localhost:3000/readyz
```

Open `http://localhost:3000` or your configured `_MSM_EXTERNAL_URL`.

On the first empty-database start, MSM creates the default tenant/admin and
prints the admin password in the `bootstrap_admin_created` log event. Full
deployment notes, including Authentik SSO setup and first-admin bootstrap, are
in [`examples/docker/README.md`](examples/docker/README.md).

## Authentik / OIDC SSO

To connect Authentik:

1. Create an Authentik OAuth2/OpenID Provider and Application for MSM.
2. Set the allowed redirect URI to:

   ```text
   ${_MSM_EXTERNAL_URL}/auth/oidc/callback
   ```

3. Copy the issuer URL, client ID, and client secret into your deployment env.
4. Log in as the first-start bootstrap admin or another tenant admin.
5. Add the OIDC provider from the Web UI Tenant admin page, CLI, API, or MCP.

See [`examples/docker/README.md`](examples/docker/README.md) and
[`docs/user/oidc-sso.md`](docs/user/oidc-sso.md).

## MoreStickers compatibility

MSM preserves the existing MoreStickers `.stickerpack` export shape so packs can
continue to be consumed by clients that understand the current moreStickers
format. The project also supports dynamic pack and group subscription payloads
for clients that refresh from a URL.

Compatibility notes live in [`docs/dev/compatibility.md`](docs/dev/compatibility.md).

## Supported surfaces

The same product features are exposed through multiple surfaces:

| Surface | What it is for |
| --- | --- |
| Web UI | Daily pack management, subscriptions, exports, tenant administration. |
| HTTP API | Integration with scripts, services, reverse proxies, or custom clients. |
| OpenAPI | Machine-readable API schema at `/openapi.json`. |
| CLI | Terminal workflows and automation. |
| MCP | Tool access for MCP-capable clients through `/mcp`. |

## Import and export targets

Currently implemented:

- MoreStickers import/export.
- Telegram provider import.
- LINE provider import.
- Telegram sticker-set export and guarded reconciliation.

Planned provider families registered for future work:

- Signal
- WhatsApp
- Kakao
- Band
- OGQ
- Viber

## Deployment notes

Common runtime settings:

| Variable | Purpose |
| --- | --- |
| `MSM_BIND_ADDR` | Service bind address. |
| `MSM_DATABASE_URL` | SQLite or PostgreSQL database URL. |
| `MSM_ASSET_DIR` | Local sticker asset storage. |
| `MSM_PREPARED_MEDIA_DIR` | Prepared Telegram media output/cache. |
| `MSM_PUBLIC_ASSET_URL` | Optional system-wide CDN/public asset URL. |
| `MSM_CORS_ALLOWED_ORIGINS` | Optional comma-separated browser/plugin origins allowed to call MSM directly, for example Discord/Equicord origins. |
| `MSM_EXPORT_WORKER_ENABLED` | Enable export job polling. |
| `MSM_PROVIDER_IMPORT_WORKER_ENABLED` | Enable provider import job polling. |
| `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON` | Optional startup export target bootstrap. |

The Docker image includes ffmpeg/ffprobe for media conversion. Use PostgreSQL for
multi-user deployments; SQLite is useful for small or single-user instances.

Backup/restore guidance is in
[`docs/user/backup-restore-runbook.md`](docs/user/backup-restore-runbook.md).

## Local development

Prerequisites:

- Rust stable toolchain
- Node.js 24 with npm
- ffmpeg and ffprobe
- Optional PostgreSQL for backend parity tests

Useful checks:

```bash
npm ci
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run web:typecheck
npm run web:test
npm run web:build
```

Run locally:

```bash
npm run web:build
cargo run -p msm-app
```

Default endpoints:

- `GET /healthz`
- `GET /readyz`
- `GET /openapi.json`
- `POST /mcp`

## Documentation

- [`examples/docker/README.md`](examples/docker/README.md): Docker Compose and
  Authentik setup
- [`docs/user/README.md`](docs/user/README.md): detailed user guide and API/CLI
  examples
- [`docs/user/oidc-sso.md`](docs/user/oidc-sso.md): SSO guide
- [`docs/user/backup-restore-runbook.md`](docs/user/backup-restore-runbook.md):
  backup and restore
- [`docs/dev/compatibility.md`](docs/dev/compatibility.md): MoreStickers format
  compatibility
- [`docs/dev/mcp-transport-contract.md`](docs/dev/mcp-transport-contract.md):
  MCP transport behavior
- [`docs/status/completion-audit.md`](docs/status/completion-audit.md):
  release-readiness audit

## Project status

The current PRD contract is complete. Release-readiness verification is recorded
in [`docs/status/completion-audit.md`](docs/status/completion-audit.md). New
product scope should be tracked as a new PRD revision before implementation.

## License

This project is licensed under the GNU General Public License v3.0 or later (`GPL-3.0-or-later`).
