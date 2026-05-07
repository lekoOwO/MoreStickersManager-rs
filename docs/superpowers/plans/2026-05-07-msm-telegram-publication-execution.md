# MSM Telegram Publication Execution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Telegram dry-run worker output with a mockable `teloxide` publication path that can create a Telegram sticker set and append remaining stickers without real network calls in CI.

**Architecture:** Keep Telegram HTTP calls inside `msm-telegram`. `msm-exporters` remains the planner and `msm-app` remains the durable worker. The worker prepares media, maps prepared files into `teloxide::types::InputSticker`, calls an injected publisher, and records publication results/events.

**Tech Stack:** Rust workspace crates, `teloxide`, `async-trait`, `tokio`, SQLx-backed storage repositories, mocked publisher tests, no live Telegram network access in tests.

---

## File Structure

- Create `crates/msm-telegram/src/publish.rs`: mockable publisher trait, publish request/result types, `TeloxideTelegramPublisher`, and Telegram error normalization.
- Modify `crates/msm-telegram/src/lib.rs`: re-export publish boundary types.
- Create `crates/msm-telegram/tests/publish_tests.rs`: tests for mocked create/append order and teloxide request type construction where possible without sending requests.
- Modify `crates/msm-app/src/export_worker.rs`: inject a Telegram publisher beside the prepared media executor, load prepared media paths, call publisher for non-dry-run jobs, and persist result metadata.
- Modify `crates/msm-app/tests/export_worker_tests.rs`: tests for successful Telegram publication, publisher failure -> failed job, and dry-run still avoiding publisher calls.
- Modify `docs/status/current.md`, `docs/status/checkpoints.md`, `docs/status/implementation-matrix.md`, `docs/agents/testing.md`, `README.md`, and `docs/user/README.md`: document publication status and test commands after each completed slice.

## Task 1: Telegram Publisher Boundary

**Files:**
- Create: `crates/msm-telegram/src/publish.rs`
- Modify: `crates/msm-telegram/src/lib.rs`
- Create: `crates/msm-telegram/tests/publish_tests.rs`

- [ ] Add `async-trait` to `crates/msm-telegram/Cargo.toml`.
- [ ] Write failing tests for a mock `TelegramPublisher` receiving one create request and one append request from a `TelegramPublishRequest`.
- [ ] Add `TelegramPublishRequest`, `TelegramPublishSticker`, `TelegramPublishedSet`, `TelegramPublishError`, and `TelegramPublisher`.
- [ ] Implement `publish_sticker_set` orchestration that calls `create_new_sticker_set` for initial stickers and `add_sticker_to_set` for append stickers through a smaller `TelegramStickerSetApi` trait.
- [ ] Test that append calls preserve order and use the same owner/set name as create.
- [ ] Run `cargo test -p msm-telegram --locked`.
- [ ] Run `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`.
- [ ] Commit with message `feat: add Telegram publish boundary`.

## Task 2: Teloxide Requester Adapter

**Files:**
- Modify: `crates/msm-telegram/src/publish.rs`
- Modify: `crates/msm-telegram/tests/publish_tests.rs`

- [ ] Add a `TeloxideTelegramStickerSetApi` wrapper around `teloxide::Bot`.
- [ ] Implement create by calling `Requester::create_new_sticker_set(UserId(owner_user_id), name, title, stickers).sticker_type(sticker_type).await`.
- [ ] Implement append by calling `Requester::add_sticker_to_set(UserId(owner_user_id), name, sticker).await`.
- [ ] Keep adapter tests network-free by testing request payload construction through the orchestration trait with a recording fake; do not call Telegram.
- [ ] Run `cargo test -p msm-telegram --locked`.
- [ ] Run `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`.
- [ ] Commit with message `feat: add teloxide sticker set adapter`.

## Task 3: Worker Publication Injection

**Files:**
- Modify: `crates/msm-app/src/export_worker.rs`
- Modify: `crates/msm-app/tests/export_worker_tests.rs`

- [ ] Add a `TelegramPublicationExecutor` trait to the worker layer that accepts the planned Telegram export plus prepared media outputs and returns `TelegramPublishedSet`.
- [ ] Add `ExportWorker::with_media_and_telegram_executors` so tests can inject both media and publisher dependencies.
- [ ] Parse `dryRun` from job options; default to `true` until remote publication is explicitly requested with `"dryRun": false`.
- [ ] For `"dryRun": true`, preserve the current `TelegramDryRun` result and assert no publisher calls occur.
- [ ] For `"dryRun": false`, convert prepared media output asset keys into local paths under `prepared_media_dir`, build `InputFile::file`, convert planned stickers to `InputSticker`, call the publisher, and persist a `TelegramPublished` result.
- [ ] Append job events for `telegram.prepare`, `telegram.publish.create`, `telegram.publish.append`, and `succeeded` stages.
- [ ] Run `cargo test -p msm-app --locked`.
- [ ] Run `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- [ ] Commit with message `feat: publish Telegram export jobs`.

## Task 4: Publication Result API/UI Surface

**Files:**
- Modify: `apps/web/src/components/PackExportWizard.vue`
- Modify: `apps/web/src/components/ExportJobTimeline.vue`
- Modify: `apps/web/src/components/__tests__/export-ui.test.ts`
- Modify: `docs/user/README.md`

- [ ] Update Web result-link extraction to recognize `telegramUrl`, `stickerSetUrl`, and `result.kind === "telegramPublished"`.
- [ ] Add a test where a completed job result renders the Telegram sticker set URL.
- [ ] Document that current remote publication requires `"dryRun": false` and a configured Telegram target token.
- [ ] Run `npm run web:typecheck`, `npm run web:test`, and `npm run web:build`.
- [ ] Commit with message `feat: surface Telegram publication results`.

## Task 5: Documentation And Verification

**Files:**
- Modify: `docs/dev/architecture.md`
- Modify: `docs/dev/providers.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/agents/testing.md`
- Modify: `README.md`
- Modify: `docs/user/README.md`

- [ ] Document `dryRun` behavior, Telegram bot token handling, prepared media dependency, and no-network test strategy.
- [ ] Update progressive disclosure docs so the next worker can resume from publication result/status accurately.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [ ] Run `cargo test --workspace --locked`.
- [ ] Run `npm run web:typecheck`.
- [ ] Run `npm run web:test`.
- [ ] Run `npm run web:build`.
- [ ] Commit with message `docs: document Telegram publication`.

## Self-Review

- The plan keeps provider import and export publication separate.
- The plan keeps real Telegram network access outside CI by injecting traits and using recording fakes.
- The plan defaults remote jobs to dry-run unless the user explicitly passes `"dryRun": false`.
- The plan uses `teloxide` for Telegram publication instead of a custom HTTP client.
- The plan preserves existing MoreStickers `.stickerpack` export behavior.
