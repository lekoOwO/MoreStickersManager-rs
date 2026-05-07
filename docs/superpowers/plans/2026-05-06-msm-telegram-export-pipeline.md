# MSM Telegram Export Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a modular export pipeline that can convert MSM sticker assets and publish packs to Telegram sticker sets while preserving existing MoreStickers export compatibility.

**Architecture:** Add target-neutral media conversion and exporter layers, then implement Telegram as the first remote publication target. Existing Provider code remains input-only; MoreStickers and Telegram are modeled as export targets behind durable jobs shared by Web, API, CLI, and MCP.

**Tech Stack:** Rust workspace crates, SQLx SQLite/PostgreSQL-compatible migrations, Axum, utoipa, teloxide, reqwest for non-Telegram HTTP clients, serde, tokio, ffmpeg/ffprobe command execution, Vue 3, Tailwind CSS v4, Shadcn Vue-style components, Vitest.

---

## File Structure

- Create `crates/msm-media`: media probing, target media profiles, conversion planning, converter command execution, and prepared media cache types.
- Create `crates/msm-exporters`: export target traits, target registry, target capabilities, MoreStickers export adapter, Telegram export planner.
- Create `crates/msm-telegram`: teloxide bot boundary, token-redacting config types, Bot API URL configuration, and boundary tests.
- Modify `crates/msm-storage`: migrations and repositories for export targets, jobs, events, prepared media assets, and Telegram publications.
- Modify `crates/msm-api`: OpenAPI DTOs and protected routes for targets, jobs, and Telegram export requests.
- Modify `crates/msm-app`: converter configuration, exporter registry composition, and background export worker.
- Modify `crates/msm-cli`: `msm exports ...` commands.
- Modify `crates/msm-mcp`: export target/job tools.
- Modify `apps/web`: export target settings, pack export wizard, and job progress UI.
- Modify `docs/status/*`, `docs/dev/*`, `docs/user/*`, and `docs/agents/*`: keep handoff state current after each phase.

## Task 1: Media Profile Foundation

**Files:**
- Create: `crates/msm-media/Cargo.toml`
- Create: `crates/msm-media/src/lib.rs`
- Create: `crates/msm-media/src/profile.rs`
- Create: `crates/msm-media/src/plan.rs`
- Create: `crates/msm-media/tests/profile_tests.rs`
- Modify: `Cargo.toml`

- [x] Add `msm-media` to the workspace.
- [x] Define `MediaKind`, `StickerTargetProfile`, `PreparedMediaSpec`, and `ConversionPlan`.
- [x] Add tests proving static images map to Telegram static profiles, videos map to Telegram video profiles, unsupported inputs return typed errors, and profile keys are stable.
- [x] Run `cargo test -p msm-media --locked`.
- [x] Commit with message `feat: add media profile foundation`.

## Task 2: Converter Command Planning

**Files:**
- Create: `crates/msm-media/src/command.rs`
- Create: `crates/msm-media/src/error.rs`
- Create: `crates/msm-media/tests/command_tests.rs`
- Modify: `crates/msm-media/src/lib.rs`

- [x] Add a shell-free command builder that returns executable path, argument vector, input path, output path, timeout, and expected MIME type.
- [x] Test Telegram static image, video sticker, and thumbnail command plans without running ffmpeg.
- [x] Ensure arguments are vectors, not interpolated strings.
- [x] Run `cargo test -p msm-media --locked`.
- [x] Commit with message `feat: plan media conversion commands`.

## Task 3: Prepared Media Cache Storage

**Files:**
- Create: `crates/msm-storage/migrations/0003_export_pipeline.sql`
- Create: `crates/msm-storage/src/export_jobs.rs`
- Create: `crates/msm-storage/tests/export_job_repository_tests.rs`
- Modify: `crates/msm-storage/src/lib.rs`

- [x] Add tables for `export_targets`, `export_jobs`, `export_job_events`, `prepared_media_assets`, and `telegram_publications`.
- [x] Add repository methods to create targets, create jobs, append events, update status, and upsert prepared assets by source hash plus profile key.
- [x] Test queued/running/succeeded/failed transitions and ordered event reads.
- [x] Run `cargo test -p msm-storage --locked`.
- [x] Commit with message `feat: persist export jobs`.

## Task 4: Exporter Trait And Registry

**Files:**
- Create: `crates/msm-exporters/Cargo.toml`
- Create: `crates/msm-exporters/src/lib.rs`
- Create: `crates/msm-exporters/src/target.rs`
- Create: `crates/msm-exporters/src/registry.rs`
- Create: `crates/msm-exporters/tests/registry_tests.rs`
- Modify: `Cargo.toml`

- [x] Add `ExportTarget`, `ExportTargetKind`, `ExportCapabilities`, `ExportRequest`, `ExportPlan`, and `ExportResult` types.
- [x] Add an in-memory registry that rejects duplicate target kinds and returns stable capability metadata.
- [x] Test registry lookup, duplicate rejection, and serializable capability output.
- [x] Run `cargo test -p msm-exporters --locked`.
- [x] Commit with message `feat: add exporter registry`.

## Task 5: MoreStickers Export Adapter

**Files:**
- Create: `crates/msm-exporters/src/morestickers.rs`
- Create: `crates/msm-exporters/tests/morestickers_export_tests.rs`
- Modify: `crates/msm-exporters/src/lib.rs`

- [x] Wrap existing `.stickerpack` serialization as an `ExportTarget` named `morestickers`.
- [x] Test output byte-for-byte equality with the current domain export helper.
- [x] Run `cargo test -p msm-exporters --locked`.
- [x] Commit with message `feat: add MoreStickers export target`.

## Task 6: Telegram Bot Framework Boundary

**Files:**
- Create: `crates/msm-telegram/Cargo.toml`
- Create: `crates/msm-telegram/src/lib.rs`
- Create: `crates/msm-telegram/src/bot.rs`
- Create: `crates/msm-telegram/tests/bot_tests.rs`
- Modify: `Cargo.toml`

- [x] Add `teloxide` as the Telegram Bot framework dependency instead of maintaining a custom Bot API HTTP client.
- [x] Add `TelegramBotToken` and `TelegramBotConfig` with redacted token formatting and configurable Bot API URL.
- [x] Add a `build_bot` boundary that returns `teloxide::Bot` and applies the configured API URL.
- [x] Test token redaction, invalid token rejection, Bot API URL validation, and `teloxide::Bot` construction.
- [x] Run `cargo test -p msm-telegram --locked`.
- [x] Commit with message `feat: add Telegram bot framework boundary`.

## Task 7: Telegram Export Planner

**Files:**
- Create: `crates/msm-exporters/src/telegram.rs`
- Create: `crates/msm-exporters/tests/telegram_plan_tests.rs`
- Modify: `crates/msm-exporters/src/lib.rs`

- [x] Add Telegram target config, sticker set name normalization, set size checks, and `InputSticker` planning.
- [x] Test `_by_<bot_username>` suffix handling, 64-character set name normalization, invalid bot username rejection, 1-50 initial sticker batching, 120 regular sticker limit, 200 custom emoji limit, emoji list validation, create-only conflict handling, and mixed static/video plan output.
- [x] Return typed conflicts for create-only exports when the target set already exists.
- [x] Run `cargo test -p msm-exporters --locked`.
- [x] Commit with message `feat: plan Telegram sticker exports`.

## Task 8: Export API And OpenAPI

**Files:**
- Create: `crates/msm-api/src/export_routes.rs`
- Create: `crates/msm-api/tests/export_routes_tests.rs`
- Modify: `crates/msm-api/src/lib.rs`
- Modify: `crates/msm-api/src/openapi.rs`
- Modify: `crates/msm-domain/src/authz.rs`

- [x] Add `export.read`, `export.run`, and `export.target.manage` permission keys.
- [x] Add routes for target kinds, target CRUD, job creation, job status, and job event reads.
- [x] Add OpenAPI schemas for target capabilities, Telegram config with redacted token responses, export job requests, and job results.
- [x] Test PAT/RBAC enforcement, token redaction, pack ownership checks, and OpenAPI route presence.
- [x] Run `cargo test -p msm-api --locked`.
- [x] Commit with message `feat: add export API`.

## Task 9: Worker Execution

**Files:**
- Create: `crates/msm-app/src/export_worker.rs`
- Create: `crates/msm-app/tests/export_worker_tests.rs`
- Modify: `crates/msm-app/src/main.rs`
- Modify: `crates/msm-app/src/config.rs`

- [x] Add worker config for ffmpeg path, ffprobe path, and max concurrent jobs.
- [x] Implement queued job polling, running status transition, event recording, exporter execution foundation, success result recording, and failure recording.
- [x] Implement prepared media executor boundary and prepared media cache writes.
- [x] Implement real ffmpeg converter process execution.
- [x] Implement target bootstrap config.
- [x] Implement background loop composition behind `MSM_EXPORT_WORKER_ENABLED`.
- [x] Test a mocked MoreStickers export job and a mocked Telegram dry-run export job without real Telegram network access.
- [x] Run `cargo test -p msm-app --locked`.
- [x] Commit worker foundation with message `feat: add export worker foundation`.
- [x] Commit worker cache/loop slice with message `feat: add export worker cache loop`.

## Task 10: CLI And MCP Parity

**Files:**
- Modify: `crates/msm-cli/src/commands.rs`
- Modify: `crates/msm-cli/src/client.rs`
- Modify: `crates/msm-cli/tests/cli_tests.rs`
- Modify: `crates/msm-mcp/src/tools.rs`
- Modify: `crates/msm-mcp/tests/mcp_tests.rs`

- [x] Add CLI commands to list target kinds, list/create targets, start exports, and read job status/events.
- [x] Add MCP tools for the same target/job operations.
- [x] Test human and JSON CLI output plus MCP tool schemas and PAT forwarding.
- [x] Run `cargo test -p msm-cli -p msm-mcp --locked`.
- [x] Commit CLI slice with message `feat: add export CLI commands` and MCP slice with message `feat: add export MCP tools`.

## Task 11: Web Target Settings And Export Wizard

**Files:**
- Create: `apps/web/src/lib/exportApi.ts`
- Create: `apps/web/src/components/ExportTargetPanel.vue`
- Create: `apps/web/src/components/PackExportWizard.vue`
- Create: `apps/web/src/components/ExportJobTimeline.vue`
- Create: `apps/web/src/components/__tests__/export-ui.test.ts`
- Modify: `apps/web/src/App.vue`
- Modify: `apps/web/src/i18n.ts`

- [x] Add typed Web client functions for target capabilities, target CRUD, job creation, and job polling.
- [x] Add a target settings panel with Telegram token validation and redacted display.
- [x] Add a pack-level export wizard with target selection, privacy notice, conversion summary, create button, progress timeline, and final Telegram link.
- [x] Add Traditional Chinese and English labels for all new UI strings.
- [x] Test wizard success, conflict error, token redaction, and job progress rendering with injected clients.
- [x] Run `npm run web:typecheck`, `npm run web:test`, and `npm run web:build`.
- [x] Commit with message `feat: add Web export workflow`.

## Task 12: Documentation And Full Verification

**Files:**
- Modify: `docs/dev/architecture.md`
- Modify: `docs/dev/providers.md`
- Modify: `docs/user/README.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [x] Document provider versus export target boundaries.
- [x] Document Telegram export setup, ffmpeg dependency, privacy warning, CLI commands, MCP tools, and Web flow.
- [x] Update progressive disclosure docs with current implementation status and next phase.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [x] Run `cargo test --workspace --locked`.
- [x] Run `npm run web:typecheck`.
- [x] Run `npm run web:test`.
- [x] Run `npm run web:build`.
- [x] Commit with message `docs: document export pipeline`.

## Self-Review

- The plan preserves `.stickerpack` compatibility by wrapping current MoreStickers export instead of replacing it.
- The plan keeps provider import and export publication separate.
- The plan uses teloxide for Telegram bot operations and keeps tokens redacted.
- The plan requires mocked Telegram tests and does not depend on real network calls in CI.
- The plan provides Web, API, CLI, and MCP parity before calling the feature complete.
