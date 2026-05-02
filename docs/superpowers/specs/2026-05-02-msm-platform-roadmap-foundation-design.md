# MSM Platform Roadmap and Foundation Design

Date: 2026-05-02
Project: MoreStickersManager-rs
Short name: MSM

## Purpose

MSM is a Rust rewrite and expansion of MoreStickersConverter. It must preserve the sticker pack export format consumed by Equicord moreStickers while adding a self-hosted multi-tenant management platform with Web UI, API, CLI, MCP endpoint, provider integrations, RBAC, SSO, PATs, subscription groups, and portable import/export.

This document defines the full staged roadmap and expands P0/P1 to implementation-ready design depth. Later phases get their own specs before implementation.

## Upstream Compatibility Facts

The compatibility target is the MoreStickers sticker pack JSON shape used by Equicord moreStickers and MoreStickersConverter.

Primary upstream references:
- MoreStickersConverter: https://github.com/lekoOwO/MoreStickersConverter
- Equicord moreStickers: https://github.com/Equicord/Equicord/tree/main/src/equicordplugins/moreStickers
- Equicord `types.ts`: `Sticker`, `StickerPackMeta`, `DynamicStickerPackMeta`, `StickerPack`, `DynamicPackSetMeta`
- MoreStickersConverter `mcStickerPack.ts`: base `.stickerpack` shape
- MoreStickersConverter `telegramStickers.ts`: Telegram ID and image URL conventions
- Equicord `lineStickers.ts` and `lineEmojis.ts`: LINE provider ID conventions

The preserved export shape is:

```json
{
  "id": "MoreStickers:Telegram:Pack:pack_name",
  "title": "Pack title",
  "author": {
    "name": "Author",
    "url": "https://example.com"
  },
  "logo": {
    "id": "MoreStickers:Telegram:Sticker:pack_name:file_unique_id",
    "image": "https://public.example/sticker/telegram/pack_name/file_unique_id.webp",
    "title": "😀",
    "stickerPackId": "MoreStickers:Telegram:Pack:pack_name",
    "filename": "file_unique_id.webp",
    "isAnimated": false
  },
  "stickers": []
}
```

Supported dynamic subscription metadata must remain compatible with Equicord's `DynamicStickerPackMeta` and `DynamicPackSetMeta`:

```json
{
  "id": "subscription-id",
  "version": "1",
  "title": "Subscription title",
  "author": {
    "name": "Owner",
    "url": "https://example.com"
  },
  "packs": [
    {
      "id": "MoreStickers:Telegram:Pack:pack_name",
      "title": "Pack title",
      "logo": {},
      "dynamic": {
        "version": "1",
        "refreshUrl": "https://public.example/api/public/packs/pack-id/stickerpack",
        "authHeaders": {
          "Authorization": "Bearer token"
        }
      }
    }
  ],
  "refreshUrl": "https://public.example/api/public/subscriptions/subscription-id",
  "authHeaders": {
    "Authorization": "Bearer token"
  }
}
```

MSM may store richer internal data, but exported JSON must be byte-stable where tests require it and schema-compatible everywhere.

## Roadmap

### P0 Project Foundation

Goal: create a maintainable Rust project baseline and documentation system before feature work.

Deliverables:
- Rust workspace layout for backend, domain core, CLI, MCP, and web asset embedding.
- Frontend workspace placeholder for Vue/Shadcn Vue/Tailwind v4 without building management UI yet.
- `.gitignore`, license decision placeholder, README, environment examples.
- Git author instructions documented as `Leko <leko@leko.moe>`.
- Developer docs under `docs/dev/`.
- User docs under `docs/user/`.
- Agent docs under `docs/agents/` using progressive disclosure.
- Status docs under `docs/status/` with current phase, last verified command, next steps, blockers, decisions.
- CI baseline for format, lint, test, and docs link checks.

Exit criteria:
- A new developer can run one documented command to validate the baseline.
- The repository can be resumed after context loss by reading `docs/status/current.md` and `docs/agents/README.md`.
- No generated or dependency directories are tracked.

### P1 Format Compatibility Core

Goal: implement exact MoreStickers export/import compatibility before storage, UI, or auth complexity.

Deliverables:
- Rust `msm-domain` crate defining `Sticker`, `StickerPackMeta`, `DynamicStickerPackMeta`, `StickerPack`, `DynamicPackSetMeta`.
- Serde import/export for `.stickerpack`.
- Golden fixtures based on Telegram, LINE sticker, LINE emoji, and dynamic pack set examples.
- Roundtrip tests proving unknown-compatible optional fields do not break import.
- URL builder tests proving public asset URL and system-wide CDN asset URL behavior.
- Provider ID convention helpers for Telegram and LINE.

Exit criteria:
- `cargo test -p msm-domain` passes.
- Golden fixtures export to the expected JSON field names and optional field behavior.
- Public/CDN URL substitution is deterministic and documented.

### P2 Storage and Asset Core

Goal: persist packs, stickers, users, and assets behind portable storage interfaces.

Deliverables:
- SQLx-backed repositories supporting SQLite and PostgreSQL.
- Migrations for tenants, users, roles, permissions, sticker packs, stickers, folders, tags, subscriptions, PATs, settings, and audit log.
- Asset store abstraction with local filesystem implementation.
- Asset addressing independent from provider source.
- Public asset URL resolver using pack URL first and system CDN URL when configured.

Exit criteria:
- SQLite and PostgreSQL integration tests pass.
- Export/import can migrate one user's data between two MSM instances.
- Asset path traversal and unauthorized access tests pass.

### P3 Domain Model and Authorization

Goal: encode the multi-tenant domain and permission model before exposing large APIs.

Deliverables:
- Tenant membership model.
- Built-in admin and regular user roles.
- Fine-grained permissions for pack CRUD, asset read, subscription manage, member access, PAT manage, system config, provider import, export/import.
- Policy evaluation library shared by API, CLI, Web UI assumptions, and MCP.
- Public/private interaction model for packs and subscription groups.

Exit criteria:
- Policy tests cover admin, owner, member, PAT, public anonymous, private anonymous, and subscription-token access.
- Pack visibility and subscription visibility interactions are documented with examples.

### P4 Backend API and OpenAPI

Goal: provide the authoritative HTTP API and OpenAPI contract.

Deliverables:
- Axum API server.
- utoipa OpenAPI generation.
- API routes for auth, users, tenants, packs, stickers, folders, tags, subscription groups, import/export, assets, system settings, provider jobs, PATs.
- Static file serving for embedded frontend dist.
- API contract tests.

Exit criteria:
- OpenAPI JSON is generated in CI.
- API integration tests prove core flows through HTTP.
- Authenticated and public asset endpoints enforce the P3 policy model.

### P5 CLI Client

Goal: expose all core operations through a scriptable CLI.

Deliverables:
- `msm` CLI using the same OpenAPI-compatible request/response models.
- Auth commands for login/PAT configuration.
- Pack CRUD, import/export, subscription group management, provider import, admin settings.
- Human and JSON output modes.

Exit criteria:
- CLI smoke tests cover all major command families.
- CLI can import a `.stickerpack`, export it, and preserve compatibility.

### P6 Provider Interface and Initial Providers

Goal: make Telegram and LINE first-class providers without baking either into the core.

Deliverables:
- Provider trait with discovery, fetch, normalize, asset download, and capability metadata.
- Provider job model with retry, cancellation, and progress.
- Telegram provider matching MoreStickersConverter behavior.
- LINE sticker and LINE emoji providers matching Equicord conversion behavior.
- Placeholder capability registry entries for Signal, WhatsApp, Kakao, Band, OGQ, Viber.

Exit criteria:
- Provider tests use fixture responses, not live provider networks.
- Telegram/LINE outputs match P1 compatibility expectations.

### P7 Web UI Shell

Goal: establish the frontend foundation without mixing it with domain policy design.

Deliverables:
- Vue app with Shadcn Vue and Tailwind CSS v4.
- RWD layout shell.
- i18n setup.
- Light/dark mode.
- Auth screens and app navigation.
- API client generated or typed from OpenAPI.

Exit criteria:
- Web UI builds and embeds into Rust binary.
- Accessibility and responsive smoke checks pass.

### P8 Web UI Management

Goal: implement user-facing management flows.

Deliverables:
- Sticker pack list/detail/create/edit/delete.
- Rename, member availability controls, folders, tags.
- Subscription group create/edit/delete and subscription URLs.
- Public/private controls with clear permission explanations.
- Import/export UX.
- Provider import UX with job progress.

Exit criteria:
- UI E2E tests cover primary flows on desktop and mobile viewport sizes.
- Web UI can perform the same core operations as CLI/API.

### P9 MCP Endpoint

Goal: allow tool-driven clients to manage MSM through MCP.

Deliverables:
- MCP endpoint authenticated by PAT.
- Tool coverage aligned with API/CLI.
- Schemas for pack operations, subscriptions, provider import, export/import, and system queries.
- Tests for PAT scope enforcement.

Exit criteria:
- MCP integration tests cover tool schema validation and auth failures.
- MCP can manage a pack end-to-end without Web UI involvement.

### P10 Enterprise Hardening

Goal: tighten security, operability, and compliance boundaries.

Deliverables:
- OIDC/SSO login.
- Local registration enable/disable setting.
- PAT scopes, expiration, rotation, and revocation.
- Audit logs.
- Rate limits.
- Security headers and CORS policy.
- Backup/restore validation.

Exit criteria:
- Threat-model-driven tests cover auth bypass, PAT misuse, private asset leakage, tenant isolation, and subscription token misuse.
- OIDC can be disabled without breaking local accounts, and local registration can be disabled without breaking existing users.

### P11 Packaging and Release

Goal: ship MSM as a self-contained service.

Deliverables:
- Frontend dist embedded into Rust binary.
- Docker image.
- GitHub Actions for test, build, release artifact, Docker build, and security checks.
- Migration guide from MoreStickersConverter.
- Instance migration guide.

Exit criteria:
- Release workflow produces binaries and container image.
- A clean environment can start MSM, import a sample pack, and serve it.

## Architecture Direction

MSM should use a Rust workspace with clear crate boundaries:

```text
crates/
  msm-domain/        # compatibility models, provider-neutral domain types, policies
  msm-storage/       # SQLx repositories, migrations, asset store interfaces
  msm-api/           # Axum routes and utoipa OpenAPI
  msm-cli/           # CLI binary
  msm-mcp/           # MCP server endpoint and tools
  msm-providers/     # Telegram and LINE provider implementations
  msm-app/           # final service binary and embedded frontend
web/
  app/               # Vue/Shadcn Vue/Tailwind v4 frontend
docs/
  agents/
  dev/
  status/
  user/
  superpowers/
```

The domain crate must be dependency-light and must not depend on Axum, SQLx, provider SDKs, or frontend code. The API, CLI, MCP, and Web UI all operate through the same domain and OpenAPI-compatible DTOs to prevent feature drift.

## P0 Detailed Design

### Repository Hygiene

Create `.gitignore` for:
- Rust build output: `target/`
- Node dependencies and builds: `node_modules/`, `dist/`, `.vite/`
- local env files: `.env`, `.env.*`, except examples
- database and asset local data: `data/`, `*.sqlite`, `*.sqlite3`
- editor and OS noise

Do not track generated dependencies. Existing `node_modules/` remains untracked.

### Documentation System

Human-facing docs:
- `README.md`: project purpose, current phase, quick verification command, links.
- `docs/dev/architecture.md`: architecture overview and crate boundaries.
- `docs/dev/compatibility.md`: MoreStickers format compatibility notes.
- `docs/user/README.md`: user-facing status and future usage docs.

Agent-facing progressive disclosure:
- `docs/agents/README.md`: start here; explains which file to read next.
- `docs/agents/project-map.md`: repo map and ownership boundaries.
- `docs/agents/compatibility.md`: export format and golden fixture rules.
- `docs/agents/status-protocol.md`: how to update status files before stopping.
- `docs/agents/testing.md`: which tests prove which layer.

Status docs:
- `docs/status/current.md`: phase, last action, last verification, next task, blockers.
- `docs/status/decisions.md`: accepted architectural decisions.
- `docs/status/checkpoints.md`: chronological development log.

### Git Discipline

Commits use normal engineering wording and must not include tool or bot authorship labels. Commit author must be:

```text
Leko <leko@leko.moe>
```

Recommended local command pattern:

```powershell
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: add MSM platform foundation design"
```

### Verification Baseline

P0 starts with documentation-only verification, then adds build/test commands after scaffolding:

```powershell
git status --short
```

After Rust workspace creation, the baseline becomes:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

After frontend creation, the baseline adds:

```powershell
npm run lint
npm run typecheck
npm run build
```

## P1 Detailed Design

### Compatibility Types

Rust structs in `msm-domain`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sticker {
    pub id: String,
    pub image: String,
    pub title: String,
    pub sticker_pack_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_animated: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StickerPack {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub logo: Sticker,
    pub stickers: Vec<Sticker>,
}
```

Dynamic types:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub refresh_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_headers: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicStickerPackMeta {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub logo: Sticker,
    pub dynamic: DynamicInfo,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicPackSetMeta {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub packs: Vec<DynamicStickerPackMeta>,
    pub refresh_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_headers: Option<std::collections::BTreeMap<String, String>>,
}
```

Use `BTreeMap` for header serialization stability in golden tests.

### Provider ID Helpers

P1 implements pure helper functions only:
- Telegram pack ID: `MoreStickers:Telegram:Pack:{sticker_set_name}`
- Telegram sticker ID: `MoreStickers:Telegram:Sticker:{sticker_set_name}:{file_unique_id}`
- LINE sticker pack ID: `MoreStickers:Line:Pack:{id}`
- LINE sticker ID: `MoreStickers:Line:Sticker:{pack_id}:{sticker_id}`
- LINE emoji pack ID: `MoreStickers:Line:Emoji-Pack:{id}`
- LINE emoji ID: `MoreStickers:Line-Emoji:{pack_id}:{emoji_id}`

Provider network fetch and parsing wait until P6.

### Asset URL Resolution

MSM has two public URL concepts:
- Public app URL: base URL of the MSM instance.
- Public asset URL: optional system-wide CDN/public asset base URL.

Resolution rule:
- If system public asset URL is configured, exported sticker `image` uses it.
- Otherwise exported sticker `image` uses the MSM public app URL.
- The path suffix remains stable, for example `/assets/packs/{pack_public_id}/{filename}`.
- Existing imported external image URLs are preserved until the asset is internalized by a later storage/provider workflow.

This keeps Cloudflare/CDN substitution centralized without changing pack identity.

### Golden Fixtures

Fixtures live under:

```text
crates/msm-domain/tests/fixtures/
  telegram_pack.stickerpack.json
  line_sticker_pack.stickerpack.json
  line_emoji_pack.stickerpack.json
  dynamic_pack_set.json
```

Golden tests:
- parse each fixture successfully;
- serialize back to stable pretty JSON where field order is controlled by struct order;
- assert required field names use camelCase;
- assert optional `author`, `filename`, `isAnimated`, `dynamic.version`, and `authHeaders` behavior.

### Error Handling

P1 exposes typed errors:
- invalid JSON;
- missing required field;
- invalid sticker pack extension when reading from a path;
- invalid provider ID input for helper functions;
- invalid base URL for asset URL resolution.

API-specific HTTP errors are not part of P1.

## Public/Private Permission Model Preview

Detailed policy belongs to P3, but P1/P2 must not block it.

Sticker pack visibility:
- Public pack: exported metadata and image assets can be read anonymously unless a tenant policy disables public sharing globally.
- Private pack: metadata and images require one of: authenticated Web UI session, PAT with asset read scope, pack-specific secret, or subscription token that grants access.

Subscription group visibility:
- Public subscription group: group metadata endpoint can be read anonymously; included private packs still require either a signed/secret image URL strategy or auth headers in dynamic pack metadata.
- Private subscription group: group metadata requires auth, PAT, pack-specific secret, or subscription secret.

Recommended P3 direction:
- Treat subscription access as a grant that can include pack metadata and asset read capability for selected packs.
- Do not let a public subscription group accidentally make private pack assets globally public.
- Use explicit generated subscription tokens or `authHeaders` for private groups.

## Testing Strategy

P0:
- Documentation link/path checks.
- Repository hygiene checks.
- CI command availability.

P1:
- Unit tests for serialization and provider ID helpers.
- Golden tests for fixture compatibility.
- Property-style roundtrip tests for generated sticker packs.
- URL resolution tests for default public URL and CDN asset URL.

P2-P4:
- SQL migration tests for SQLite and PostgreSQL.
- Repository integration tests.
- API integration tests.
- OpenAPI contract generation tests.

P5-P9:
- CLI smoke tests.
- Provider fixture tests.
- Web UI E2E tests.
- MCP schema/auth tests.

P10-P11:
- Security regression tests.
- Release smoke tests.
- Backup/restore and instance migration tests.

## Design Decisions

1. Preserve MoreStickers JSON as the external contract even if internal tables use richer IDs and metadata.
2. Keep provider implementation out of the domain crate.
3. Use phased delivery with one spec and one implementation plan per major phase.
4. Build API as the canonical capability surface; CLI, Web UI, and MCP must not invent separate behavior.
5. Use status and agent docs from P0 onward so every interrupted session has a recovery path.
6. Implement SQLite and PostgreSQL through the same repository interfaces and migrations.
7. Embed the frontend dist only after the frontend shell exists; do not block backend/domain work on UI packaging.

## Open Questions For Later Specs

1. License selection for MSM.
2. Exact frontend package manager and Shadcn Vue initialization preset.
3. Whether Telegram bot compatibility remains as an optional ingestion mode or is replaced by CLI/API provider jobs only.
4. Whether private asset URLs use signed URLs, bearer auth only, pack secrets only, or a combination.
5. Whether OIDC is implemented with a direct crate integration or an identity-provider-agnostic OAuth/OIDC layer.

These questions do not block P0/P1 because the foundation and compatibility core can be built without finalizing them.

## Approval Scope

Approval of this document authorizes writing an implementation plan for P0/P1 only. Later phases require their own deeper specs and implementation plans before code changes.
