# MSM Telegram Publication CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI commands for reading Telegram publication history through the protected API.

**Architecture:** Extend the existing `msm exports` CLI group with publication read commands. Keep HTTP logic in `client.rs`, command dispatch in `command.rs`, and formatting in `output.rs`.

**Tech Stack:** Rust, clap, reqwest, async trait fake client tests, JSON/human output formatting.

---

## File Structure

- Modify `crates/msm-cli/src/client.rs`: add `TelegramPublication` response model and API client methods.
- Modify `crates/msm-cli/src/command.rs`: add `msm exports publications list/get` commands and tests.
- Modify `crates/msm-cli/src/output.rs`: add human/JSON formatters for publication records.
- Modify docs/status files after implementation.

## Task 1: CLI Publication Commands

**Files:**
- Modify: `crates/msm-cli/src/client.rs`
- Modify: `crates/msm-cli/src/command.rs`
- Modify: `crates/msm-cli/src/output.rs`

- [x] Write failing parse tests for `exports publications list --pack-id pack_1` and `exports publications get --publication-id telegram_pub_1`.
- [x] Write failing execution tests for list and get commands using `FakeClient`.
- [x] Add `TelegramPublication` response model.
- [x] Add `list_telegram_publications(pack_id)` and `get_telegram_publication(publication_id)` to `MsmClient`.
- [x] Implement reqwest calls to `/api/v1/telegram-publications?packId=...` and `/api/v1/telegram-publications/{publication_id}`.
- [x] Add `ExportPublicationCommand`.
- [x] Wire command execution to client methods.
- [x] Add human and JSON output formatters.
- [x] Run `cargo test -p msm-cli --locked`.
- [x] Run `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`.
- [x] Commit with message `feat: add Telegram publication CLI`.

## Task 2: Documentation And Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/agents/testing.md`

- [ ] Document CLI publication list/get commands and remaining MCP/Web work.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test -p msm-cli --locked`.
- [ ] Run `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`.
- [ ] Run `git diff --check`.
- [ ] Commit with message `docs: document Telegram publication CLI`.

## Self-Review

- This plan only reads publication history; it does not mutate remote Telegram state.
- Commands require whatever PAT is passed globally to have `export.read`.
- MCP and Web exposure remain later slices.
