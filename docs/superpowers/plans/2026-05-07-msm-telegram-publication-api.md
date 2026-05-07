# MSM Telegram Publication API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose durable Telegram publication history through protected HTTP API and OpenAPI.

**Architecture:** Keep publication persistence in `msm-storage` and expose read-only API routes in `msm-api`. Publication reads require `export.read` and the PAT user must own the source pack behind the publication.

**Tech Stack:** Rust, Axum, utoipa, in-memory SQLite API tests, PAT scope enforcement.

---

## File Structure

- Modify `crates/msm-api/src/dto.rs`: add list query and response DTOs for Telegram publications.
- Modify `crates/msm-api/src/routes/exports.rs`: add list/get publication handlers and response mapping.
- Modify `crates/msm-api/src/lib.rs`: mount `/api/v1/telegram-publications` and `/api/v1/telegram-publications/{publication_id}`.
- Modify `crates/msm-api/src/openapi.rs`: register paths and schemas.
- Modify `docs/status/current.md`, `docs/status/checkpoints.md`, `docs/status/implementation-matrix.md`, and `docs/agents/testing.md`: update current status and verification docs.

## Task 1: API Routes And OpenAPI

**Files:**
- Modify: `crates/msm-api/src/dto.rs`
- Modify: `crates/msm-api/src/routes/exports.rs`
- Modify: `crates/msm-api/src/lib.rs`
- Modify: `crates/msm-api/src/openapi.rs`

- [x] Write a failing API test that `GET /api/v1/telegram-publications?packId=pack_1` requires `export.read`, enforces pack ownership, and returns publication records.
- [x] Write a failing API test that `GET /api/v1/telegram-publications/{publication_id}` returns one owned publication and rejects another PAT user.
- [x] Write a failing OpenAPI assertion for the new paths.
- [x] Add `ListTelegramPublicationsQuery`.
- [x] Add `TelegramPublicationResponse`.
- [x] Add list and get handlers using `Permission::ExportRead`.
- [x] Add owner checks by loading the source pack for each publication/pack.
- [x] Mount routes in `build_router`.
- [x] Register paths and schemas in `openapi.rs`.
- [x] Run `cargo test -p msm-api --locked`.
- [x] Run `cargo clippy -p msm-api --all-targets --locked -- -D warnings`.
- [x] Commit with message `feat: expose Telegram publication API`.

## Task 2: Documentation And Verification

**Files:**
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/agents/testing.md`

- [ ] Document new protected API routes and remaining CLI/MCP/Web exposure work.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test -p msm-api --locked`.
- [ ] Run `cargo clippy -p msm-api --all-targets --locked -- -D warnings`.
- [ ] Run `git diff --check`.
- [ ] Commit with message `docs: document Telegram publication API`.

## Self-Review

- This plan is read-only API exposure only; it does not create or mutate remote Telegram sets.
- This plan does not add CLI/MCP/Web surfaces yet; those should consume this API in later slices.
- Tests use repository-seeded publications and do not call Telegram.
