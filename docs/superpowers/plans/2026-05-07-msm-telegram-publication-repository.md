# MSM Telegram Publication Repository Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add durable repository methods for the existing `telegram_publications` table so Telegram sticker set publication state can be queried and updated.

**Architecture:** Keep this slice inside `msm-storage`. The migration already defines `telegram_publications`; this plan adds typed models and SQLite repository methods that future worker/API reconciliation code can call without changing schema.

**Tech Stack:** Rust, SQLx SQLite repository patterns, `tokio` integration tests, TDD.

---

## File Structure

- Modify `crates/msm-storage/src/models.rs`: add `TelegramPublicationRecord` and `NewTelegramPublication`.
- Modify `crates/msm-storage/src/export_jobs.rs`: add upsert/find/list repository methods and row mapping.
- Modify `crates/msm-storage/tests/export_job_repository_tests.rs`: add tests for create, find by ID, find by target/set, list by pack, and upsert update behavior.
- Modify `docs/status/current.md`, `docs/status/checkpoints.md`, `docs/status/implementation-matrix.md`, and `docs/agents/testing.md`: update status and testing references.

## Task 1: Storage Models And Repository Methods

**Files:**
- Modify: `crates/msm-storage/src/models.rs`
- Modify: `crates/msm-storage/src/export_jobs.rs`
- Modify: `crates/msm-storage/tests/export_job_repository_tests.rs`

- [x] Write a failing test that upserts a Telegram publication, finds it by ID, finds it by target/set, and lists it by pack.
- [x] Write a failing test that a second upsert with the same target/set updates URL, job ID, sticker count, and sticker type.
- [x] Add `TelegramPublicationRecord` with all columns from `telegram_publications`.
- [x] Add `NewTelegramPublication` matching insert/update input fields.
- [x] Add `upsert_telegram_publication`.
- [x] Add `find_telegram_publication`.
- [x] Add `find_telegram_publication_by_target_set`.
- [x] Add `list_telegram_publications_for_pack`.
- [x] Run `cargo test -p msm-storage --test export_job_repository_tests --locked`.
- [x] Run `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`.
- [x] Commit with message `feat: add Telegram publication repository`.

## Task 2: Documentation And Verification

**Files:**
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/agents/testing.md`

- [x] Document that `telegram_publications` now has repository APIs but worker persistence into that table remains a next integration step.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test -p msm-storage --locked`.
- [x] Run `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`.
- [x] Run `git diff --check`.
- [x] Commit with message `docs: document Telegram publication repository`.

## Self-Review

- This plan does not change schema because migration `0003_export_pipeline.sql` already contains the table.
- This plan does not wire worker writes yet; it only creates the durable repository boundary for a later isolated slice.
- Tests remain SQLite in-memory and do not call Telegram.
