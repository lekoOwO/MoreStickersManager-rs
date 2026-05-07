# MSM Telegram Publication Worker Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist successful Telegram worker publication results into the durable `telegram_publications` table.

**Architecture:** Keep publication execution in `msm-app` and storage persistence in `msm-storage`. After a non-dry-run Telegram job succeeds, the worker records the published set by `(target_id, sticker_set_name)` using the storage repository upsert API.

**Tech Stack:** Rust, `msm-app` worker tests with injected fake publishers, SQLx-backed `msm-storage` repository APIs.

---

## File Structure

- Modify `crates/msm-app/src/export_worker.rs`: call `upsert_telegram_publication` after a successful publication.
- Modify `crates/msm-app/tests/export_worker_tests.rs`: assert successful publication jobs create or update a durable publication record.
- Modify `docs/status/current.md`, `docs/status/checkpoints.md`, `docs/status/implementation-matrix.md`, and `docs/agents/testing.md`: update handoff and testing status.

## Task 1: Worker Publication Persistence

**Files:**
- Modify: `crates/msm-app/src/export_worker.rs`
- Modify: `crates/msm-app/tests/export_worker_tests.rs`

- [x] Write a failing test that a successful `dryRun:false` Telegram job creates a `telegram_publications` record.
- [x] Add worker persistence after `TelegramPublicationExecutor::publish` returns successfully.
- [x] Use a stable publication ID derived from target ID and sticker set name while relying on `(target_id, sticker_set_name)` upsert for updates.
- [x] Preserve dry-run behavior; dry-run jobs must not create publication records.
- [x] Run `cargo test -p msm-app --test export_worker_tests --locked`.
- [x] Run `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- [x] Commit with message `feat: persist Telegram publication jobs`.

## Task 2: Documentation And Verification

**Files:**
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/agents/testing.md`

- [ ] Document that successful worker publication jobs now write durable publication records.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test -p msm-app --locked`.
- [ ] Run `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- [ ] Run `git diff --check`.
- [ ] Commit with message `docs: document Telegram worker publication persistence`.

## Self-Review

- This plan does not expose publication history through API/CLI/MCP/Web yet.
- This plan does not call Telegram in tests; fake publication executors remain the worker test boundary.
- This plan keeps retries and update/delete reconciliation deferred.
