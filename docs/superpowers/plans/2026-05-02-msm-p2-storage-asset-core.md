# MSM P2 Storage and Asset Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add durable SQLite/PostgreSQL-ready storage primitives, local asset storage, and portable export/import support.

**Architecture:** Create `msm-storage` as a separate crate depending on `msm-domain`. Keep authorization out of the asset store and expose SQLite-tested repositories that can be used by later API/CLI/MCP phases.

**Tech Stack:** Rust, SQLx, Tokio, serde, chrono, uuid, tempfile.

---

## Task 1: Scaffold `msm-storage`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/msm-storage/Cargo.toml`
- Create: `crates/msm-storage/src/lib.rs`
- Modify: `docs/agents/project-map.md`

- [ ] Add `crates/msm-storage` to the workspace members.
- [ ] Create `msm-storage` dependencies: `msm-domain`, `async-trait`, `chrono`, `serde`, `serde_json`, `sqlx`, `tempfile`, `thiserror`, `tokio`, `uuid`.
- [ ] Create module exports for `asset`, `config`, `db`, `error`, `models`, `portability`, and `repositories`.
- [ ] Update the agent project map to mark `msm-storage` as implemented in P2.
- [ ] Run `cargo test -p msm-storage` and verify it fails only because modules are not implemented yet.
- [ ] Commit with `chore: scaffold storage crate`.

## Task 2: Error and Config Types

**Files:**
- Create: `crates/msm-storage/src/error.rs`
- Create: `crates/msm-storage/src/config.rs`
- Modify: `crates/msm-storage/src/lib.rs`

- [ ] Add `StorageError` and `StorageResult`.
- [ ] Add `DatabaseKind` and `DatabaseConfig` parsing for `sqlite:` and `postgres:` URLs.
- [ ] Add tests for SQLite URL parsing, PostgreSQL URL parsing, and unsupported schemes.
- [ ] Run `cargo test -p msm-storage config`.
- [ ] Commit with `feat: add storage configuration`.

## Task 3: Local Asset Store

**Files:**
- Create: `crates/msm-storage/src/asset.rs`

- [ ] Add `AssetKey` with `pack_public_id` and `filename`.
- [ ] Validate empty strings, `..`, `/`, `\`, `:`, and NUL bytes.
- [ ] Add `LocalAssetStore` with `write`, `read`, `delete`, and `path_for_test`.
- [ ] Write assets through a same-directory temporary file and rename.
- [ ] Add tests for write/read/delete and traversal rejection.
- [ ] Run `cargo test -p msm-storage asset`.
- [ ] Commit with `feat: add local asset store`.

## Task 4: Storage Models and SQLite Migration

**Files:**
- Create: `crates/msm-storage/src/models.rs`
- Create: `crates/msm-storage/migrations/0001_initial.sql`

- [ ] Add model structs for tenant, user, pack, sticker, subscription group, and portable export rows.
- [ ] Add SQLite-compatible initial migration for P2 tables.
- [ ] Keep timestamps as RFC3339 text for SQLite portability.
- [ ] Run `cargo test -p msm-storage models`.
- [ ] Commit with `feat: add storage schema models`.

## Task 5: Database Pool and Migrations

**Files:**
- Create: `crates/msm-storage/src/db.rs`

- [ ] Add `DbPool` enum with SQLite implemented and PostgreSQL recognized but returning a clear unsupported-at-runtime error until Pg migrations are added.
- [ ] Add `connect_sqlite`, `connect`, and `run_migrations`.
- [ ] Add SQLite migration test using a temp database.
- [ ] Run `cargo test -p msm-storage db`.
- [ ] Commit with `feat: add database migration runner`.

## Task 6: Repository Implementation

**Files:**
- Create: `crates/msm-storage/src/repositories.rs`

- [ ] Add `StorageRepository` with SQLite-backed methods for tenant creation, user creation, membership creation, pack upsert, sticker insertion, subscription group creation, subscription pack insertion, pack lookup, and subscription lookup.
- [ ] Use explicit SQL with bind parameters.
- [ ] Add integration test that inserts tenant, user, pack, sticker, subscription group, and membership.
- [ ] Run `cargo test -p msm-storage repository`.
- [ ] Commit with `feat: add SQLite storage repository`.

## Task 7: Portable Export and Import

**Files:**
- Create: `crates/msm-storage/src/portability.rs`
- Modify: `crates/msm-storage/src/repositories.rs`

- [ ] Add `PortableUserExport`, `PortableUser`, and `PortableSubscriptionGroup`.
- [ ] Export one user's packs as P1 `StickerPack` values.
- [ ] Import into another database by creating missing tenant/user rows and upserting packs/stickers.
- [ ] Add test exporting from one SQLite database and importing into another.
- [ ] Run `cargo test -p msm-storage portability`.
- [ ] Commit with `feat: add portable user export import`.

## Task 8: Docs, Status, and Full Verification

**Files:**
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] Document `cargo test -p msm-storage`.
- [ ] Update status current and checkpoint files.
- [ ] Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] Commit with `docs: update P2 storage status`.

## P2 Completion Criteria

- `msm-storage` compiles under workspace checks.
- SQLite migration test passes.
- Asset traversal tests pass.
- Portable export/import test passes.
- Tracked working tree is clean after commits.
