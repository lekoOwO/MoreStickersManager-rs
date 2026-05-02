# MSM P4 API and OpenAPI Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first HTTP API slice with OpenAPI generation and storage-backed pack operations.

**Architecture:** Create `msm-api` as a thin Axum crate. It composes `msm-domain` and `msm-storage` but does not own domain policy or storage logic.

**Tech Stack:** Axum, utoipa, tower, Tokio, serde, msm-domain, msm-storage.

---

## Task 1: Scaffold API Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/msm-api/Cargo.toml`
- Create: `crates/msm-api/src/lib.rs`
- Create: `crates/msm-api/src/state.rs`
- Create: `crates/msm-api/src/error.rs`
- Create: `crates/msm-api/src/dto.rs`

- [ ] Add `crates/msm-api` to workspace members.
- [ ] Add dependencies for `axum`, `http-body-util`, `mime_guess`, `msm-domain`, `msm-storage`, `serde`, `serde_json`, `tokio`, `tower`, and `utoipa`.
- [ ] Define `ApiState`, `ApiError`, `ApiErrorBody`, and initial DTOs.
- [ ] Run `cargo test -p msm-api` and commit with `chore: scaffold API crate`.

## Task 2: Health and OpenAPI Routes

**Files:**
- Create: `crates/msm-api/src/openapi.rs`
- Create: `crates/msm-api/src/routes/health.rs`
- Modify: `crates/msm-api/src/lib.rs`

- [ ] Add `build_router(state)` returning an Axum router.
- [ ] Add `GET /healthz`.
- [ ] Add `GET /openapi.json`.
- [ ] Generate OpenAPI with utoipa.
- [ ] Add tests for health and OpenAPI path presence.
- [ ] Commit with `feat: add API health and OpenAPI routes`.

## Task 3: Pack Import/List/Export Routes

**Files:**
- Create: `crates/msm-api/src/routes/packs.rs`
- Modify: `crates/msm-api/src/dto.rs`
- Modify: `crates/msm-api/src/lib.rs`

- [ ] Add `POST /api/v1/packs/import`.
- [ ] Add `GET /api/v1/packs?userId=...`.
- [ ] Add `GET /api/v1/packs/{pack_id}/stickerpack`.
- [ ] Add integration tests with SQLite storage.
- [ ] Commit with `feat: add pack API routes`.

## Task 4: Asset Read Route

**Files:**
- Create: `crates/msm-api/src/routes/assets.rs`
- Modify: `crates/msm-api/src/lib.rs`

- [ ] Add `GET /assets/packs/{pack_public_id}/{filename}`.
- [ ] Resolve asset key through `msm-storage::AssetKey`.
- [ ] Return 404 JSON error for invalid/missing assets.
- [ ] Add test writing bytes through `LocalAssetStore` and reading them through HTTP.
- [ ] Commit with `feat: add asset API route`.

## Task 5: Docs and Verification

**Files:**
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] Document `msm-api`.
- [ ] Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] Commit with `docs: update P4 API status`.

## Completion Criteria

- API crate compiles and tests pass.
- OpenAPI JSON contains initial routes.
- Pack import/export/list works through HTTP.
- Asset route reads local asset bytes.
- Workspace verification passes.
