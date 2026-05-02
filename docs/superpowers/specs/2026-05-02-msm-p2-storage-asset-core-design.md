# MSM P2 Storage and Asset Core Design

Date: 2026-05-02
Phase: P2

## Purpose

P2 adds durable storage and local asset management while preserving the P1 `.stickerpack` compatibility boundary. It must support SQLite and PostgreSQL, prepare for multi-tenant authorization, and provide deterministic asset URL resolution without implementing the API, Web UI, CLI, MCP endpoint, or provider network jobs.

## Scope

In scope:
- `msm-storage` crate.
- SQLx migrations for the P2 schema.
- SQLite and PostgreSQL pool configuration types.
- Repository interfaces and SQLx implementation for tenants, users, sticker packs, stickers, subscription groups, tags, folders, system settings, PAT metadata, and audit logs.
- Local filesystem asset store.
- Asset key validation and path traversal protection.
- Export/import service for one user's portable data.
- Integration tests using SQLite.
- PostgreSQL support in code and CI design, with live PostgreSQL tests gated by environment variables.

Out of scope:
- HTTP API routes.
- Auth sessions.
- OIDC.
- PAT token issuance and hashing implementation beyond metadata table shape.
- Full RBAC policy evaluator.
- Provider downloads.
- Cloud object storage.

## Architecture

`msm-storage` depends on `msm-domain` and owns persistence concerns. Domain types remain provider-neutral and storage-free.

```text
crates/
  msm-domain/
  msm-storage/
    migrations/
    src/
      asset.rs
      config.rs
      db.rs
      error.rs
      lib.rs
      models.rs
      repositories.rs
      sqlite.rs
      portability.rs
```

The crate exposes:
- storage models using stable string IDs;
- repository traits for later API/CLI/MCP use;
- SQLx-backed repository implementation;
- local asset store implementation;
- portable export/import DTOs.

## Database Strategy

Use SQLx with runtime Tokio and Rustls:

```toml
sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio-rustls", "sqlite", "postgres", "macros", "migrate", "uuid", "chrono", "json"] }
```

P2 tests use SQLite by default because it is deterministic and local. PostgreSQL support is implemented through `AnyPool`-style configuration only if it does not weaken compile-time clarity; otherwise use explicit SQLite/Postgres connection enum wrappers.

Preferred design:
- `DatabaseUrl` parses `sqlite:` and `postgres:` URLs.
- `DbPool` is an enum over `SqlitePool` and `PgPool`.
- Repository methods dispatch internally.
- Migration SQL is duplicated per database only when syntax diverges.

This avoids hiding SQL dialect differences until tests are available.

## Schema

P2 schema intentionally includes future columns needed by P3/P4 so migrations do not churn immediately.

Tables:
- `tenants`: tenant identity and public asset URL override.
- `users`: local user identity fields and enable/disable state.
- `tenant_members`: user membership and coarse role.
- `roles`: built-in/custom role names.
- `permissions`: known permission keys.
- `role_permissions`: role-permission mapping.
- `sticker_packs`: internal pack metadata, visibility, owner, source provider, compatibility ID.
- `stickers`: sticker metadata and asset references.
- `folders`: user-created folders.
- `folder_packs`: folder to pack mapping.
- `tags`: tenant tags.
- `pack_tags`: tag to pack mapping.
- `subscription_groups`: named subscription groups and visibility.
- `subscription_group_packs`: ordered packs inside groups.
- `system_settings`: global settings, including public app URL and public asset URL.
- `personal_access_tokens`: PAT metadata only; raw token handling waits for P10.
- `audit_log`: append-only security and mutation events.

SQLite migration is the executable baseline. PostgreSQL migration can be added in the same task if SQLx migration layout supports per-database directories cleanly; otherwise P2 keeps SQL portable enough to run under both.

## Asset Store

P2 implements local filesystem storage:

```text
data/
  assets/
    packs/
      {pack_public_id}/
        {filename}
```

Rules:
- Asset keys are logical paths, not raw filesystem paths.
- `pack_public_id` and `filename` are validated before path construction.
- Reject empty components, `..`, separators inside components, Windows drive prefixes, and NUL bytes.
- Writes are atomic enough for local development: write to a temporary file in the same directory, then rename.
- Reads return bytes and a best-effort media type derived from extension.

The asset store does not decide authorization. It only prevents path traversal and stores bytes.

## Portable Export/Import

P2 introduces a portable user export format for later CLI/API use:

```json
{
  "version": 1,
  "exportedAt": "2026-05-02T00:00:00Z",
  "user": {
    "id": "user_1",
    "displayName": "Leko"
  },
  "packs": [],
  "subscriptionGroups": []
}
```

P2 export includes metadata and P1-compatible sticker packs. Binary assets are referenced by logical asset keys in this phase. Archive packaging can be added later.

## Testing

Unit tests:
- database URL parsing;
- asset key validation;
- asset URL resolution using P1 domain helper;
- portable export JSON shape.

SQLite integration tests:
- run migrations on an in-memory or temp-file SQLite database;
- insert tenant, user, pack, sticker, subscription group;
- export user data and import into a second SQLite database;
- verify pack compatibility ID, sticker count, and subscription membership survive.

Asset integration tests:
- write/read/delete asset bytes;
- reject traversal attempts;
- reject invalid names.

PostgreSQL:
- code compiles with PostgreSQL feature support;
- live test runs only when `MSM_TEST_POSTGRES_URL` is set.

## CI Changes

P2 CI continues to run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

No Dockerized PostgreSQL CI is required in P2. Add it when P4 API integration tests need stronger database parity.

## Status and Handoff

Every P2 task updates:
- `docs/status/current.md` with current work and last verification.
- `docs/status/checkpoints.md` at completion.
- `docs/agents/project-map.md` after `msm-storage` exists.
- `docs/agents/testing.md` with P2 verification commands.

## Design Decisions

1. Keep storage outside `msm-domain`.
2. Implement SQLite tests first because they are local and deterministic.
3. Include PostgreSQL support in code, but do not require a local PostgreSQL service for baseline CI.
4. Store raw PAT metadata shape now, but defer token generation, hashing, scopes, and auth semantics to P10.
5. Keep the local asset store authorization-free; authorization belongs to P3/P4.
