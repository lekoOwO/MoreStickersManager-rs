# Checkpoints

## 2026-05-02

- Added and approved the MSM platform roadmap and P0/P1 foundation design.
- Started P0/P1 implementation planning.

## 2026-05-02 P0/P1 Implementation

- Added repository hygiene and documentation baseline.
- Added Rust workspace and `msm-domain`.
- Added MoreStickers-compatible models, provider ID helpers, asset URL resolver, and golden tests.
- Added CI baseline.
- Verified workspace with format, clippy, and tests.

## 2026-05-02 P2 Storage Implementation

- Added P2 storage and asset core design and implementation plan.
- Added `msm-storage` crate.
- Added database URL config, local asset store, P2 schema models, SQLite migration runner, SQLite repository operations, and portable user export/import.
- Verified focused storage tests while implementing each component.

## 2026-05-02 P3 Authorization Implementation

- Added P3 authorization domain design and implementation plan.
- Added `msm-domain::authz` with principal, permission, role, resource, access context, and policy decision types.
- Added pack and subscription policy evaluators.
- Verified authorization behavior with dedicated integration tests.

## 2026-05-02 P4 API Implementation

- Added P4 backend API and OpenAPI design and implementation plan.
- Added `msm-api` crate with Axum router state, API error model, DTOs, and utoipa document generation.
- Added `/healthz`, `/openapi.json`, pack import/list/export routes, and local asset read route.
- Verified route behavior with API crate tests.

## 2026-05-03 P5 CLI Implementation

- Added P5 CLI client design and implementation plan.
- Added `msm-cli` crate with `msm` binary.
- Added clap command model, human/JSON output helpers, reqwest API client, and command execution over an `MsmClient` trait.
- Verified CLI parser and execution behavior with fake-client tests.

## 2026-05-03 P6 Provider Implementation

- Added P6 provider interface design and implementation plan.
- Added `msm-providers` crate with provider metadata, capability registry, and provider trait.
- Added Telegram fixture normalization with MoreStickersConverter-compatible IDs and self-hosted image URL output.
- Added LINE sticker and LINE emoji fixture normalization with upstream-compatible IDs.
- Verified provider registry and normalizer behavior with focused unit tests.

## 2026-05-03 P7 Web UI Foundation

- Added P7 Web UI foundation design and implementation plan.
- Added root npm workspace and `apps/web` Vite Vue application.
- Added Tailwind CSS v4 design tokens, Shadcn Vue-compatible `Button`, `Card`, and `Badge` primitives, and `components.json`.
- Added persisted theme and locale preference controllers with tests.
- Added responsive dashboard shell with desktop side rail, mobile navigation, theme toggle, language toggle, and mock sticker-pack metrics.
- Verified frontend typecheck, tests, and production build during implementation.

## 2026-05-03 P8 Web API Client

- Added P8 Web API client design and implementation plan.
- Added typed frontend pack API client for `/api/v1/packs?userId=...`.
- Added mapping from current P4 `StickerPackRecord` JSON into dashboard `StickerPackSummary` data.
- Added mock fallback when `VITE_MSM_API_BASE_URL` is unset.
- Connected dashboard data loading through the client boundary.
- Verified frontend typecheck, tests, and production build during implementation.

## 2026-05-03 P9 Service Binary

- Added P9 service binary design and implementation plan.
- Added `msm-app` crate.
- Added environment-based runtime config for bind address, database URL, asset directory, and Web UI dist directory.
- Added startup composition for storage migrations, local asset store, API router, and Web UI static serving with SPA fallback.
- Verified `msm-app` format, clippy, and tests during implementation.

## 2026-05-03 P10 Embedded Web Assets

- Added P10 embedded Web asset design and implementation plan.
- Added `msm-app` build script that embeds `apps/web/dist` when present and a placeholder dist when absent.
- Replaced disk-only Web fallback with a disk-first and embedded-second fallback handler.
- Added safe Web path normalization and embedded index tests.
- Verified `msm-app` format, clippy, and tests during implementation.

## 2026-05-04 P11 MCP Endpoint

- Added P11 MCP endpoint design and implementation plan.
- Added `msm-mcp` crate with JSON-RPC and MCP tool response shapes.
- Added MCP tool definitions for pack list, pack export, and pack import.
- Added `/mcp` route and mounted it in `msm-app`.
- Added MCP route tests for initialize, tools/list, tools/call, and unknown methods.
- Verified focused MCP and app integration tests during implementation.

## 2026-05-04 P12 PAT Foundation

- Added P12 PAT foundation design and implementation plan.
- Added stable `msm-domain::Permission` scope keys and roundtrip tests.
- Added PAT creation, listing, verification, expiry rejection, and revocation in `msm-storage`.
- Added random token secret generation and SHA-256 secret hashing.
- Verified focused domain and storage tests plus storage clippy during implementation.

## 2026-05-04 P13 PAT Management API

- Added P13 PAT management API design and implementation plan.
- Added PAT create, list, and revoke DTOs and routes.
- Added hash-free PAT response mapping and create-only raw token output.
- Added OpenAPI coverage for PAT endpoints.
- Added API tests for create/list/revoke and unknown scope rejection.
- Verified API tests and clippy during implementation.

## 2026-05-04 P14 CLI PAT Commands

- Added P14 CLI PAT command design and implementation plan.
- Added `msm pats create`, `msm pats list`, and `msm pats revoke`.
- Added PAT request/response DTOs to the CLI client boundary.
- Added reqwest calls for `POST /api/v1/pats`, `GET /api/v1/pats?userId=...`, and `DELETE /api/v1/pats/{token_id}`.
- Added human and JSON output formatting for PAT operations.
- Verified CLI PAT parser and fake-client execution tests during implementation.

## 2026-05-04 P15 API/MCP PAT Enforcement

- Added P15 API/MCP PAT enforcement design and implementation plan.
- Added API Bearer PAT verification helper with `401 Unauthorized` and `403 Forbidden` responses.
- Protected pack list/export with `pack.read` and pack import with `import.run`.
- Added user ownership guards for user-scoped pack list/import operations.
- Added MCP `tools/call` PAT enforcement while keeping initialize, ping, and tools/list public.
- Added CLI `--pat` and `MSM_PAT` forwarding to reqwest Bearer auth.
- Verified focused API, MCP, and CLI enforcement tests plus clippy during implementation.

## 2026-05-04 P16 Web PAT Management

- Added P16 Web PAT management design and implementation plan.
- Added Web API client Bearer PAT forwarding for protected pack API calls.
- Added typed Web PAT create/list/revoke client methods.
- Added browser-local PAT storage seeded by `VITE_MSM_PAT`.
- Added a responsive PAT panel for storing, creating, listing, and revoking tokens.
- Replaced mojibake i18n strings with readable Traditional Chinese and English labels.
- Verified Web typecheck, tests, and production build during implementation.

## 2026-05-04 P17 GitHub Actions Release And Docker

- Added P17 release and Docker workflow design and implementation plan.
- Expanded CI to Rust, Web, and cross-platform service build jobs.
- Added GHCR multi-arch Docker publishing workflow.
- Added main-branch prerelease and tag release workflows with binary artifact matrices.
- Added Dockerfile and `.dockerignore` for the all-in-one `msm-app` service image.
- Verified local Rust/Web/service build equivalents; Docker CLI was unavailable locally.

## 2026-05-04 P18 Local Auth Bootstrap

- Added P18 local auth bootstrap design and implementation plan.
- Added Argon2-backed local password credential storage.
- Added `local_user_credentials` migration.
- Added local user registration and password verification repository methods.
- Added local register/login API endpoints.
- Login now issues a PAT using the existing PAT response shape.
- Verified focused storage/API tests plus full Rust/Web verification.

## 2026-05-04 P19 Web Local Login

- Added P19 Web local login design and implementation plan.
- Added Web local auth API client for register/login endpoints.
- Added Web local register/login panel.
- Successful Web login now stores the returned PAT through the existing browser-local token flow.
- Verified Web typecheck, tests, build, and full Rust workspace checks.

## 2026-05-04 P20 Admin Bootstrap Policy

- Added P20 admin bootstrap policy design and implementation plan.
- Extended local registration with optional `tenantId`, `tenantName`, and `tenantRole`.
- Local registration can now create an initial tenant and add the new user as an admin member.
- Verified admin bootstrap API behavior plus full Rust/Web verification.

## 2026-05-04 P21 Pack CRUD Foundation

- Added P21 pack CRUD foundation implementation plan.
- Added owned sticker pack metadata update in storage, synchronizing the indexed title and embedded MoreStickers-compatible JSON title.
- Added owned sticker pack deletion in storage.
- Added `PATCH /api/v1/packs/{pack_id}` with `pack.update` PAT enforcement.
- Added `DELETE /api/v1/packs/{pack_id}` with `pack.delete` PAT enforcement.
- Added CLI `msm packs rename` and `msm packs delete`.
- Added MCP `msm.update_sticker_pack` and `msm.delete_sticker_pack` tools.
- Verified focused storage, API, CLI, and MCP tests during implementation.

## 2026-05-04 P22 Web Pack CRUD Controls

- Added P22 Web pack CRUD controls implementation plan.
- Extended the Web pack API client with update and delete methods.
- Added per-pack dashboard controls for title, visibility, save, and delete.
- Added injected-client dashboard tests for rename/delete behavior.
- Verified Web tests, typecheck, and production build during implementation.

## 2026-05-04 P23 Web Pack Import

- Added P23 Web pack import implementation plan.
- Extended the Web pack API client with `importStickerPack`.
- Added a dashboard import form for internal pack ID, visibility, and pasted `.stickerpack` JSON.
- Added API client and injected-client dashboard tests for Web import behavior.
- Verified Web tests, typecheck, and production build during implementation.

## 2026-05-06 P24 Telegram Export Pipeline Planning

- Analyzed moe-sticker-bot-style capabilities for arbitrary sticker media conversion, Telegram bot sticker set creation, and Web-managed sticker workflows.
- Added a design that separates provider imports from export targets.
- Planned `msm-media`, `msm-exporters`, and `msm-telegram` boundaries for future MoreStickers, Telegram, and additional output targets.
- Added a phased implementation plan covering media conversion, export jobs, Telegram Bot API, Web, API, CLI, MCP, security, and testing.

## 2026-05-06 P24 Status Documentation Cleanup

- Added `docs/status/implementation-matrix.md` as the concise implemented-versus-planned feature map.
- Updated handoff docs to point new workers at the implementation matrix immediately after current status.
- Updated README and user docs so they no longer imply the project is still at P21 or Web foundation only.

## 2026-05-07 P25 Media Profile Foundation

- Added `crates/msm-media` to the Rust workspace.
- Added `MediaKind`, `StickerTargetProfile`, `PreparedMediaSpec`, and `ConversionPlan`.
- Added Telegram regular sticker profile planning for static image and video/animated sources.
- Added typed unsupported-source errors and profile-key stability tests.
- Verified with `cargo test -p msm-media --locked`.

## 2026-05-07 P25 Converter Command Planning

- Added shell-free `ConversionCommand` and `ConverterToolchain` types.
- Added ffmpeg argument planning for Telegram static image, video sticker, and thumbnail outputs.
- Kept command planning execution-free so tests do not require ffmpeg.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-media --locked`, `cargo clippy -p msm-media --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Export Job Persistence

- Added `0003_export_pipeline.sql` migration with `export_targets`, `export_jobs`, `export_job_events`, `prepared_media_assets`, and `telegram_publications`.
- Added storage models and repository methods for creating export targets/jobs, updating job status, appending ordered job events, and upserting prepared media cache records.
- Added integration tests for target/job/event roundtrip, success/failure status payloads, and prepared media upsert behavior.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-storage --locked`, `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Exporter Registry

- Added `crates/msm-exporters` to the Rust workspace.
- Added `ExportTargetKind`, `ExportCapabilities`, `ExportRequest`, `ExportPlan`, `ExportResult`, and `ExportTarget`.
- Added an in-memory `ExportRegistry` with duplicate target kind rejection and stable capability ordering.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-exporters --locked`, `cargo clippy -p msm-exporters --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 MoreStickers Export Target

- Added `MoreStickersExportTarget` as the first concrete `msm-exporters` target.
- Wrapped existing `StickerPack::to_pretty_json()` output without changing `.stickerpack` compatibility.
- Added artifact metadata for suggested file name, MIME type, and serialized bytes.
- Verified byte-for-byte equality with the domain serialization helper.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-exporters --locked`, `cargo clippy -p msm-exporters --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Telegram Bot Framework Boundary

- Added `crates/msm-telegram` to the Rust workspace.
- Switched Telegram bot integration to `teloxide` instead of a custom Bot API HTTP client.
- Added redacted `TelegramBotToken` and `TelegramBotConfig`.
- Added configurable Bot API URL support and `teloxide::Bot` construction.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-telegram --locked`, `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Telegram Export Planner

- Added Telegram export planning in `msm-exporters`.
- Added Telegram set name normalization with `_by_<bot_username>` suffix handling, invalid bot username rejection, and 64-character Telegram name limit handling.
- Added regular/custom emoji sticker count limits, create/append batching, default emoji validation, and create-only existing-set conflict errors.
- Mapped static and animated MSM stickers through `msm-media` target profiles into teloxide `StickerFormat` and `InputSticker` data.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-exporters --locked`, `cargo clippy -p msm-exporters --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Export API And OpenAPI

- Added `export.read`, `export.run`, and `export.target.manage` permission keys.
- Added protected export target kind, export target CRUD, queued export job creation, job status, and job event routes.
- Added OpenAPI schemas for target capabilities, target config responses with token redaction, export job requests, job responses, and job events.
- Added API tests for PAT scope enforcement, token redaction, source pack ownership checks, queued job creation, event reads, and OpenAPI route presence.
- Verified with `cargo test -p msm-api --locked`, `cargo clippy -p msm-api -p msm-storage -p msm-domain --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Export Worker Foundation

- Added `ExportWorkerConfig` for ffmpeg path, ffprobe path, and max concurrent jobs.
- Added `ExportWorker` methods to pick the oldest queued job, move jobs through running/succeeded/failed states, and write ordered job events.
- Added MoreStickers job execution that serializes `.stickerpack` output metadata.
- Added Telegram dry-run job execution that uses the Telegram planner and records planned set name, create/append counts, and target media profiles without network calls.
- Added app tests for mocked MoreStickers and Telegram export jobs.
- Verified with `cargo test -p msm-app --locked` and `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.

## 2026-05-07 Export Worker Cache And Loop

- Added a prepared media executor boundary for worker-controlled media preparation.
- Added prepared media cache writes when a media executor returns prepared output metadata.
- Added worker enabled and poll interval configuration through `MSM_EXPORT_WORKER_ENABLED` and `MSM_EXPORT_WORKER_POLL_INTERVAL_MS`.
- Added optional service startup composition for the export worker polling loop.
- Verified with `cargo test -p msm-app --locked`, `cargo clippy -p msm-app --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Process Prepared Media Executor

- Added a process-backed prepared media executor for ffmpeg-compatible conversion execution.
- Reused `msm-media` shell-free conversion command planning from the app worker.
- Added converter timeout handling, non-zero exit status errors, output directory creation, and prepared output metadata reads.
- Added tests with an injected command runner so verification does not require ffmpeg to be installed.
- Verified with `cargo test -p msm-app --locked` and `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.

## 2026-05-07 Export Target Bootstrap Config

- Added `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON` to declare startup export targets.
- Added idempotent export target create/update during service initialization.
- Added tests for config parsing, invalid JSON rejection, and repository create/update behavior.
- Verified with `cargo test -p msm-app --locked` and `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.

## 2026-05-07 Export CLI Commands

- Added `msm exports kinds`.
- Added `msm exports targets list` and `msm exports targets create`.
- Added `msm exports jobs create`, `msm exports jobs get`, and `msm exports jobs events`.
- Added export target/job DTOs and API calls to the CLI client boundary.
- Added human and JSON output formatting for export targets, jobs, and events.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-cli --locked`, and `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`.

## 2026-05-07 Export MCP Tools

- Added MCP tools for export target kind list, export target list/create, export job create/get, and export job event reads.
- Added export target/job tool schemas to `tools/list`.
- Enforced `export.read`, `export.run`, and `export.target.manage` PAT scopes on export tools.
- Reused pack owner and target tenant checks before queueing export jobs.
- Redacted token-like and secret-like config fields in MCP export target responses.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-cli -p msm-mcp --locked`, and `cargo clippy -p msm-cli -p msm-mcp --all-targets --locked -- -D warnings`.

## 2026-05-07 Web Export Workflow

- Added typed Web export API client methods for target kinds, target CRUD, export job creation, job reads, and job event reads.
- Added `ExportTargetPanel` for target kind discovery, target creation, Telegram bot token validation, and redacted config display.
- Added `PackExportWizard` with pack/target selection, options JSON, conversion summary, privacy notice, job queueing, refresh, and result link rendering.
- Added `ExportJobTimeline` for current job status and ordered event display.
- Wired the export workflow into the existing dashboard using the stored browser-local PAT.
- Added Traditional Chinese and English labels for new export UI strings.
- Added injected-client tests for wizard success, conflict errors, token redaction, Telegram token validation, and progress rendering.
- Verified with `npm run web:typecheck`, `npm run web:test`, and `npm run web:build`.

## 2026-05-07 Export Pipeline Documentation And Full Verification

- Updated architecture docs with current CLI/MCP/Web export surfaces and worker boundaries.
- Updated provider docs to clarify provider input adapters versus export targets and current Telegram export status.
- Updated user docs with Telegram target setup, PAT scopes, ffmpeg/ffprobe configuration, export worker enablement, and current dry-run limitation.
- Updated agent project map and testing docs for export workflow handoff.
- Verified with `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `npm run web:typecheck`, `npm run web:test`, and `npm run web:build`.

## 2026-05-07 Telegram Publish Boundary

- Added a mockable `TelegramStickerSetApi` trait in `msm-telegram`.
- Added `TelegramPublishRequest`, `TelegramPublishSticker`, `TelegramPublishedSet`, and `TelegramPublishError`.
- Added `publish_sticker_set` orchestration that creates a sticker set with the initial batch and appends remaining stickers in order.
- Added no-network tests using a recording fake API.
- Verified with `cargo test -p msm-telegram --locked` and `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`.

## 2026-05-07 Teloxide Sticker Set Adapter

- Added `TeloxideTelegramStickerSetApi` as the `teloxide::Bot` implementation of the sticker set API boundary.
- Wired create and append publication calls through teloxide requester methods with explicit `.send().await`.
- Added typed validation for Telegram owner user IDs before constructing `teloxide::types::UserId`.
- Normalized teloxide request errors into `TelegramPublishError::Api`.
- Added a no-network construction test that proves the adapter implements the mockable publication trait.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-telegram --locked`, and `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`.

## 2026-05-07 Telegram Worker Publication

- Added `TelegramPublicationExecutor` and `TelegramPublicationRequest` to the worker layer.
- Added the default `TeloxideTelegramPublicationExecutor`, which builds a configured teloxide bot and calls the `msm-telegram` publish orchestrator.
- Added `ExportWorker::with_media_and_telegram_executors` so tests can inject both media preparation and Telegram publication dependencies.
- Kept Telegram jobs in dry-run mode by default; only job options with `"dryRun": false` execute remote publication.
- Converted prepared media cache outputs into `InputFile::file` paths under `prepared_media_dir` and then into teloxide `InputSticker` values.
- Persisted successful remote jobs as `telegramPublished` result JSON with sticker set URL, count, sticker type, dry-run flag, and prepared media summaries.
- Added failure handling so publisher errors mark the job failed and store an error summary.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-app --locked`, and `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.

## 2026-05-07 Web Telegram Publication Results

- Added shared Web result-link extraction for `telegramUrl`, `stickerSetUrl`, `url`, and `kind: "telegramPublished"` fallback results.
- Updated the pack export wizard to use the shared result-link helper.
- Updated the export job timeline to show completed Telegram sticker set links.
- Added Web tests for wizard and timeline Telegram publication URL rendering.
- Verified with `npm run web:typecheck`, `npm run web:test`, and `npm run web:build`.

## 2026-05-07 Telegram Publication Documentation And Full Verification

- Updated architecture docs for the completed Telegram publication boundaries.
- Updated provider docs to clarify Telegram provider input normalization versus Telegram export target publication.
- Updated progressive disclosure docs to reflect the dry-run default, `dryRun:false` publication path, target token requirements, prepared media dependency, and no-network test strategy.
- Confirmed remaining gaps are reconciliation/update/delete policy, publication-table repository APIs, and broader product features tracked in the implementation matrix.
- Verified with `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `npm run web:typecheck`, `npm run web:test`, and `npm run web:build`.

## 2026-05-07 Telegram Publication Repository

- Added typed `TelegramPublicationRecord` and `NewTelegramPublication` storage models for the existing `telegram_publications` table.
- Added `upsert_telegram_publication`, preserving the original ID while updating records by `(target_id, sticker_set_name)`.
- Added `find_telegram_publication`, `find_telegram_publication_by_target_set`, and `list_telegram_publications_for_pack`.
- Added repository tests for create/find/list behavior and upsert update behavior.
- Documented that worker persistence into `telegram_publications` is still the next integration step.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-storage --locked`, `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Telegram Worker Publication Persistence

- Updated successful non-dry-run Telegram worker jobs to upsert `telegram_publications` records.
- Used stable publication IDs derived from target ID and sticker set name while relying on the storage unique key for updates.
- Verified dry-run jobs do not create Telegram publication records.
- Kept Telegram network calls behind injected fake publishers in tests.
- Documented that API/CLI/MCP/Web publication-history exposure and remote reconciliation remain later slices.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-app --locked`, `cargo clippy -p msm-app --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-07 Telegram Publication API

- Added `TelegramPublicationResponse` and `ListTelegramPublicationsQuery`.
- Added `GET /api/v1/telegram-publications?packId=...` for owned pack publication history.
- Added `GET /api/v1/telegram-publications/{publication_id}` for reading one owned publication.
- Enforced `export.read` PAT scope and source pack ownership on both routes.
- Registered publication routes and schemas in OpenAPI.
- Added API tests for missing scope, owner mismatch, list response shape, get response shape, and OpenAPI path registration.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-api --locked`, `cargo clippy -p msm-api --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Telegram Publication CLI

- Added `msm exports publications list --pack-id <pack_id>`.
- Added `msm exports publications get --publication-id <publication_id>`.
- Added CLI DTOs, reqwest calls, fake-client execution tests, and human/JSON formatting for Telegram publication records.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-cli --locked`, `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Web Export UX Publication History

- Reviewed the current Web UI implementation and kept the existing RWD shell because the immediate UX gap was in the export workflow, not the app frame.
- Added Web export API client methods for Telegram publication history list/get with Bearer PAT forwarding.
- Added a persisted Telegram publication history panel to the pack export wizard, scoped to the selected source pack.
- Added Traditional Chinese and English labels for loading, empty, error, refresh, and publication metadata states.
- Added Web tests for API URL construction, Bearer auth forwarding, and wizard history rendering.
- Verified with `npm run web:typecheck`, `npm run web:test`, `npm run web:build`, and `git diff --check`.

## 2026-05-08 Telegram Publication MCP Tools

- Added `msm.list_telegram_publications` and `msm.get_telegram_publication` to MCP `tools/list`.
- Added MCP tool execution for persisted Telegram publication history.
- Enforced `export.read` and source pack ownership before returning publication records.
- Corrected the Web publication DTO to match the API/MCP storage shape (`packId`, `jobId`, `stickerType`, `updatedAt`).
- Added MCP tests for list/get behavior and missing `export.read` rejection.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-mcp --locked`, `cargo clippy -p msm-mcp --all-targets --locked -- -D warnings`, `npm run web:typecheck`, `npm run web:test`, `npm run web:build`, and `git diff --check`.

## 2026-05-08 Export Job Retry Policy

- Added `0004_export_job_retries.sql` with `attempt_count`, `max_attempts`, and `next_attempt_at`.
- Added storage retry helpers for due queued job selection, retry requeue, and terminal failure attempt accounting.
- Added worker retry behavior: retryable failures requeue until `max_attempts` is exhausted, append a `retry_scheduled` event, and respect `next_attempt_at` before polling picks the job again.
- Added `MSM_EXPORT_RETRY_BACKOFF_MS`, defaulting to 60 seconds.
- Exposed retry metadata through API, CLI, MCP, and Web export job DTOs.
- Added storage and worker tests for retry metadata, backoff skip behavior, and terminal failure after exhausting the attempt budget.
- Verified with focused Rust/Web checks recorded in `docs/status/current.md`.

## 2026-05-08 Telegram Reconciliation Policy

- Added pure `msm-exporters` reconciliation types for known remote Telegram set/sticker state.
- Added `TelegramReconcileMode` with `CreateOnly`, `AppendMissing`, and `Mirror` policies.
- Added ordered reconciliation operations for set creation, title update, sticker keep/add/replace, and remote-only sticker deletion.
- Added planner tests proving create-only rejection, append-missing non-destructive behavior, and mirror update/add/delete behavior without Telegram network access.
- Documented that policy modeling exists, while remote state fetch and destructive execution remain pending worker/Telegram-boundary work.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-exporters --test telegram_plan_tests --locked`, `cargo clippy -p msm-exporters --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Telegram Mutation Boundary

- Added `TelegramStickerSetMutation` for title update, sticker add, sticker replace, and sticker delete operations.
- Extended the mockable `TelegramStickerSetApi` trait with set-title, replace-sticker, and delete-sticker methods.
- Implemented the new methods in `TeloxideTelegramStickerSetApi` using teloxide requester methods.
- Added `apply_sticker_set_mutations` to execute mutation sequences in order.
- Added no-network tests with a recording fake to prove mutation ordering.
- Documented that worker reconciliation mapping remains pending and destructive mirror execution is not enabled by default.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-telegram --locked`, `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Telegram Worker Reconciliation Dry-Run

- Added serde support for Telegram reconciliation modes and supplied remote set/sticker state.
- Extended Telegram export job options with `reconcileMode` and `remoteSet`.
- Added dry-run result reconciliation summaries with mode, operation count, mutation count, and operation labels.
- Kept Telegram publisher execution disabled for dry-run reconciliation summaries.
- Added worker tests proving append-missing dry-runs report planned mutation counts without calling Telegram.
- Documented that non-dry-run mutation execution and remote state retrieval remain future work.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-app --test export_worker_tests --locked`, `cargo clippy -p msm-app -p msm-exporters --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Telegram Guarded Reconciliation Execution

- Added `TelegramMutationExecutor` and `TelegramMutationRequest` to the app worker boundary.
- Added a teloxide-backed mutation executor that reuses the `msm-telegram` mutation orchestrator.
- Added `executeReconciliation` job option; non-dry-run reconciliation refuses to run unless explicitly enabled.
- Wired supplied-state append-missing reconciliation operations into Telegram add/replace/title/delete mutation requests.
- Added a worker test proving append-missing reconciliation executes mutations through an injected fake and does not call the create-set publisher.
- Persisted reconciled Telegram publication metadata after successful mutation execution.
- Documented that remote state retrieval and destructive mirror safety controls remain pending.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-app --test export_worker_tests --locked`, `cargo clippy -p msm-app --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Telegram Mirror Safety

- Added `allowDestructiveReconciliation` job option for mirror-mode replace/delete operations.
- Added worker guard that rejects mirror reconciliation plans containing replace/delete unless destructive reconciliation is explicitly allowed.
- Added a no-network worker test proving mirror delete does not call the mutation executor without the extra allowance.
- Updated user, architecture, testing, matrix, checkpoint, and current-status docs.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-app --test export_worker_tests --locked`, `cargo clippy -p msm-app --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Telegram Remote Fetch Boundary

- Added `TelegramFetchedStickerSet` and `TelegramFetchedSticker` DTOs for remote Bot API state.
- Extended the mockable `TelegramStickerSetApi` trait with `get_sticker_set`.
- Implemented `TeloxideTelegramStickerSetApi::get_sticker_set` through teloxide `getStickerSet`.
- Added `fetch_sticker_set` orchestration function with a no-network recording fake test.
- Documented that fetched Telegram metadata still needs persisted per-sticker MSM source ID mapping before worker reconciliation can use it automatically.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-telegram --locked`, `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Telegram Sticker Mapping Storage

- Added migration `0005_telegram_sticker_mappings.sql` with durable mappings from MSM source sticker IDs to Telegram file IDs per publication, target, and sticker set.
- Added `TelegramStickerMappingRecord` and `NewTelegramStickerMapping` storage models.
- Added repository methods to upsert mappings, find a mapping by target/set/source sticker, and list mappings for a publication.
- Added repository tests proving upsert updates retain stable IDs and ordered publication listing works.
- Documented that worker publication/reconciliation still needs to populate mappings from fetched remote state.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-storage --test export_job_repository_tests --locked`, `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Telegram Post-Publication Mapping Population

- Added `TelegramRemoteStateExecutor` and `TelegramRemoteStateRequest` to the app worker boundary.
- Added a teloxide-backed remote-state executor using `msm-telegram::fetch_sticker_set`.
- Added worker injection for publication, mutation, and remote-state executors together so tests stay no-network.
- Updated successful Telegram publication jobs to fetch the remote sticker set after publication and persist per-sticker mappings by planned sticker order.
- Added worker tests proving mappings persist Telegram file IDs and file unique IDs after a fake publish plus fake remote fetch.
- Documented that mapping refresh after reconciliation mutation execution remains the next slice.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-app --test export_worker_tests --locked`, `cargo clippy -p msm-app --all-targets --locked -- -D warnings`, and `git diff --check`.

## 2026-05-08 Development Environment Manager

- Added `scripts/dev-manager.mjs`, a dependency-free Node.js manager for local API/Web service start, stop, restart, and status operations.
- Added environment profile commands for listing profiles, initializing `.env.<name>` files from tracked examples, and switching the active profile.
- Added `.env.development.example` and `.env.testing.example` for repeatable local API/Web configuration without committing private overrides.
- Added root npm shortcuts: `dev`, `dev:start`, `dev:stop`, `dev:restart`, `dev:status`, and `dev:env`.
- Added `pnpm-workspace.yaml`, Windows-safe hidden wrapper process spawning, local runtime directory creation, split stdout/stderr logs, and direct local Vite startup for Web dev mode.
- Documented usage in README, user docs, project map, testing guide, current status, and the implementation matrix.
- Verified with `node scripts/dev-manager.mjs --help`, `node scripts/dev-manager.mjs env list`, `node scripts/dev-manager.mjs env init development`, `node scripts/dev-manager.mjs env use testing`, `node scripts/dev-manager.mjs status`, `npm run dev:status`, `node scripts/dev-manager.mjs env use development`, `node scripts/dev-manager.mjs stop`, `pnpm run dev:start`, `pnpm run dev:status`, API/Web HTTP checks, `pnpm run dev:stop`, `npm run dev:start`, `npm run dev:status`, API/Web HTTP checks, `npm run dev:stop`, hidden wrapper regression checks, and `git diff --check`.

## 2026-05-08 Web Workspace Redesign

- Reworked the Web shell into a wider desktop workspace with Ant Design-inspired blue/gray tokens and a less card-heavy visual system.
- Replaced placeholder sidebar links with real workspace section state for overview, packs, exports, and targets.
- Moved local login and PAT management into dialogs so authentication controls no longer dominate the main dashboard.
- Reworked pack management into a metrics strip, tabbed workspace, table-style pack rows, and a dialog for `.stickerpack` import.
- Kept the existing API client boundaries and injected-client tests for pack CRUD, import, export targets, export jobs, and Telegram publication history.
- Verified with `npm run web:typecheck`, `npm run web:test`, `npm run web:build`, `pnpm run dev:start`, API/Web HTTP smoke checks, and `pnpm run dev:stop`.

## 2026-05-08 Development Bootstrap Usability

- Added `MSM_DEV_BOOTSTRAP_ENABLED` flow to the development manager.
- The manager now waits for API health, registers or reuses the local dev account, creates a fresh PAT, writes `VITE_MSM_PAT` into a managed `.env.local` block, and imports a small sample pack before starting Web.
- Updated development/testing env examples so development bootstraps by default and testing remains isolated by default.
- Documented the bootstrap behavior in README, user docs, agent docs, project map, testing guide, implementation matrix, and current status.
- Verified with `node --check scripts/dev-manager.mjs`, `pnpm run dev:stop`, `node scripts/dev-manager.mjs env use development`, `pnpm run dev:start`, `pnpm run dev:status`, `Invoke-WebRequest -UseBasicParsing http://127.0.0.1:3000/healthz`, PAT-authenticated `GET /api/v1/packs?userId=user_1`, `Invoke-WebRequest -UseBasicParsing http://127.0.0.1:5173`, `pnpm run dev:stop`, repeated `pnpm run dev:start` with an existing valid PAT, repeated `pnpm run dev:stop`, and `git diff --check`.

## 2026-05-08 Web Desktop/Mobile UX Correction

- Set Vite `envDir` to the repository root so `web:dev` and dev-manager Vite both see `.env.development` and `.env.local`.
- Replaced binary Connected/Mock status with Live API, API needs PAT, and Mock preview states based on API base URL plus browser PAT.
- Reworked the desktop shell into a full-width product workbench with an icon rail, context panel, and content area instead of a centered widened app.
- Added a separate compact mobile pack-card layout while preserving the desktop table-style pack management layout.
- Fixed the `MoreStickersManager` brand overflow with constrained context-panel typography.
- Added pointer cursor and pressed/hover motion states for interactive buttons and controls.
- Increased blue accent chroma in both light and dark themes.
- Verified with `npm run web:typecheck`, `npm run web:test`, `npm run web:build`, `pnpm run dev:stop`, `pnpm run dev:start`, API health check, PAT-authenticated pack list check, Web HTTP check, Vite module env check, and `pnpm run dev:stop`.

## 2026-05-08 Web Native Navigation Correction

- Removed the wide non-collapsible context sidebar in favor of one collapsible desktop navigation rail.
- Suppressed `PackDashboard`'s internal tab strip when `AppShell` owns the active section, eliminating duplicate synchronized navigation.
- Kept the top bar for global actions only: runtime status, login, PAT, language, and theme.
- Moved narrow desktop pack management to the card layout to avoid horizontal page overflow.
- Recalibrated dark theme to a near-black background and Ant Design-like blue instead of fluorescent blue.
- Added Playwright E2E coverage for live API status, absence of mock preview when API/PAT env is present, single navigation source, section switching, sidebar collapse/expand, and narrow desktop overflow.
- Configured Playwright to use the installed Microsoft Edge channel instead of downloaded Chromium, and removed downloaded Playwright browser artifacts from the local profile.
- Verified with `npm run web:typecheck`, `npm run web:test`, `npm run web:build`, `npm run web:e2e`, and confirmed `%LOCALAPPDATA%\ms-playwright` does not exist after E2E.

## 2026-05-08 Web UI QA Hardening

- Moved the expanded desktop brand label out of the cramped rail header so `MoreStickersManager` is not clipped by the collapse control.
- Replaced the collapsed runtime status text with a compact state dot, preventing Mock/API state from showing the wrong `API` label or breaking the rail layout.
- Replaced PAT/local-login free-form scope fields with selectable scope cards and explanatory labels.
- Translated remaining fixed zh-TW dashboard and access-token labels, including provider/status/import-dialog and scope-selection UI.
- Extended Playwright E2E to assert brand non-clipping, collapsed runtime label behavior, selectable PAT scopes, zh-TW fixed chrome labels, and import-dialog translation on desktop, narrow desktop, and mobile where applicable.
- Verified with `npm run web:typecheck`, `npm run web:test`, `npm run web:build`, `npm run web:e2e`, and confirmed `%LOCALAPPDATA%\ms-playwright` does not exist after E2E.

## 2026-05-09 Web Rail Containment And Telegram Reconciliation Mappings

- Fixed the collapsed desktop rail header by stacking the MS logo and expand control vertically inside the narrow rail instead of forcing them into one overflowing row.
- Added Playwright bounding-box coverage proving collapsed rail controls stay inside the rail with at least minimal horizontal breathing room.
- Updated successful non-dry-run Telegram reconciliation mutation jobs to fetch remote sticker set state after mutation execution.
- Reused the post-publication mapping persistence path so reconciliation jobs refresh MSM source sticker ID to Telegram file ID mappings by planned sticker order.
- Added a no-network worker test proving reconciliation mapping refresh calls the injected remote-state executor and persists updated Telegram file IDs.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-app --test export_worker_tests --locked`, `cargo clippy -p msm-app --all-targets --locked -- -D warnings`, `npm run web:typecheck`, `npm run web:test`, `npm run web:build`, `npm run web:e2e`, and confirmed `%LOCALAPPDATA%\ms-playwright` does not exist after E2E.

## 2026-05-09 Telegram Automatic Remote-State Reconciliation

- Added worker-side automatic `TelegramRemoteSet` construction for non-dry-run reconciliation jobs when callers omit `remoteSet`.
- The worker now fetches remote Telegram sticker set metadata, loads stored mappings for the matching publication, and maps Telegram file IDs back to MSM source sticker IDs before planning reconciliation.
- Unknown remote stickers are represented as remote-only placeholders so mirror mode can still plan guarded deletions later.
- Zero-mutation reconciliation no longer calls the mutation executor; it still persists the publication record and reports a reconciled result.
- Added a no-network worker test proving append-missing reconciliation can use stored mappings plus fetched metadata without caller-supplied `remoteSet`.
- Verified with `cargo fmt --all -- --check`, `cargo test -p msm-app --test export_worker_tests --locked`, and `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.

## 2026-05-09 Web Telegram Reconciliation Controls

- Added explicit Telegram export controls for dry-run, reconciliation mode, execute-reconciliation, and destructive mirror opt-in.
- Kept the advanced export options JSON field available for extra worker options while avoiding default payload changes unless the new controls are touched.
- Wired selected controls into export job options as `dryRun`, `reconcileMode`, `executeReconciliation`, and `allowDestructiveReconciliation`.
- Added Traditional Chinese and English labels for the reconciliation controls and guard explanations.
- Added a Web export UI test proving append-missing execution options are queued without hand-writing JSON.
- Verified with `npm run web:typecheck`, `npm run web:test`, and `npm run web:build`.

## 2026-05-09 Documentation Progress And Roadmap Cleanup

- Added `docs/status/roadmap.md` as the concise handoff view for current focus,
  immediate plan, later planned work, and verification expectations.
- Updated current status and implementation matrix dates around the P33
  Telegram reconciliation usability focus.
- Updated agent handoff/status protocol docs so roadmap and matrix changes are
  part of normal pause/resume hygiene.
- Corrected user and architecture docs to reflect implemented Telegram Web
  controls, automatic remote-state reconciliation, and mapping refresh after
  reconciliation mutation jobs.
- Moved CLI/MCP reconciliation convenience affordances into the explicit
  remaining-work list.
- Verified with `git diff --check`.

## 2026-05-09 CLI/MCP Telegram Reconciliation Affordances

- Added CLI export job flags for Telegram live mode, dry-run override,
  reconciliation mode, execute-reconciliation, set-name slug, default emoji, and
  destructive mirror opt-in.
- Added MCP `msm.create_export_job` named Telegram fields for the same worker
  options while keeping raw `options` available for advanced callers.
- Made MCP export job `options` optional so named Telegram fields can queue a
  reconciliation job without an opaque JSON object.
- Updated README, user docs, roadmap, implementation matrix, project map, and
  current status for the new CLI/MCP parity.
- Targeted RED/GREEN verification passed for `cargo test -p msm-cli
  parses_export_job_create_telegram_reconciliation_flags --locked`,
  `cargo test -p msm-cli
  executes_export_job_create_with_telegram_reconciliation_flags --locked`, and
  `cargo test -p msm-mcp
  tools_call_creates_telegram_reconciliation_job_without_raw_options --locked`.
- Full verification passed with `cargo fmt --all -- --check`,
  `cargo test -p msm-cli -p msm-mcp --locked`,
  `cargo clippy -p msm-cli -p msm-mcp --all-targets --locked -- -D warnings`,
  and `git diff --check`.

## 2026-05-09 API OpenAPI Telegram Options Documentation

- Added `TelegramExportJobOptions` and `TelegramReconcileModeOption` OpenAPI
  schemas for target-specific Telegram export job options.
- Pointed `CreateExportJobRequest.options` at the typed Telegram options schema
  while keeping the runtime API request field as flexible JSON.
- Documented dry-run, set naming, reconciliation mode, execution guard,
  destructive mirror guard, remote state, and existing-set fields in the schema.
- Updated README, user docs, roadmap, implementation matrix, project map, and
  current status so API callers no longer need worker-source inspection to
  discover Telegram reconciliation options.
- Targeted RED/GREEN verification passed for `cargo test -p msm-api
  openapi_documents_telegram_export_job_options --locked`.
- Full verification passed with `cargo fmt --all -- --check`,
  `cargo test -p msm-api --locked`,
  `cargo clippy -p msm-api --all-targets --locked -- -D warnings`, and
  `git diff --check`.

## 2026-05-09 Telegram Mirror Runbook And Product Data API Plan

- Added `docs/user/telegram-reconciliation-runbook.md` with safe operator flow,
  append-missing examples, guarded mirror examples, review checklist, and
  recovery notes.
- Linked the runbook from user and architecture docs.
- Added `docs/superpowers/plans/2026-05-09-msm-product-data-api.md` as the next
  implementation plan for folder, tag, subscription-group, and pack access
  metadata APIs.
- Updated roadmap, implementation matrix, project map, and current status so the
  active next slice is product-data API implementation.
- Verified with `git diff --check`.

## 2026-05-09 Product Data Storage Primitives

- Added `FolderRecord`, `TagRecord`, and `NewTag` storage models.
- Added folder create/list/rename/delete repository methods.
- Added tag create/list/delete repository methods.
- Changed `create_subscription_group` to return `SubscriptionGroupRecord` and
  added subscription group list/rename/delete repository methods.
- Added `product_data_repository_tests` coverage for folder, tag, and
  subscription group metadata lifecycle.
- Updated the product-data API implementation plan and status docs; next step is
  API routes and OpenAPI schemas.
- Verified with `cargo fmt --all -- --check`,
  `cargo test -p msm-storage --locked`,
  `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and
  `git diff --check`.

## 2026-05-09 Product Data API Routes

- Added folder create/list DTOs, protected API routes, and OpenAPI schemas.
- Added tag create/list DTOs, protected API routes, and OpenAPI schemas.
- Added subscription group create/list DTOs, protected API routes, and OpenAPI
  schemas.
- Mounted `/api/v1/folders`, `/api/v1/tags`, and
  `/api/v1/subscription-groups`.
- Added API tests for product metadata routes and OpenAPI path registration.
- Updated user docs, product-data implementation plan, roadmap, implementation
  matrix, and current status.
- Verified with `cargo fmt --all -- --check`,
  `cargo test -p msm-storage -p msm-api --locked`,
  `cargo clippy -p msm-storage -p msm-api --all-targets --locked -- -D warnings`,
  and `git diff --check`.

## 2026-05-09 Product Data CLI Commands

- Added CLI client DTOs and reqwest calls for folder create/list, tag
  create/list, and subscription-group create/list API routes.
- Added `msm metadata folders`, `msm metadata tags`, and
  `msm metadata subscription-groups` command groups.
- Added human/JSON formatting for folder, tag, and subscription-group
  responses.
- Added parser and fake-client execution tests for the new metadata commands.
- Updated README, user docs, roadmap, implementation matrix, current status,
  project map, and agent testing notes.
- Verified with `cargo fmt --all -- --check`,
  `cargo test -p msm-cli --locked`,
  `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`, and
  `git diff --check`.

## 2026-05-09 Product Data MCP Tools

- Added MCP tool definitions for folder create/list, tag create/list, and
  subscription-group create/list workflows.
- Added MCP handler support backed by the same storage repository methods and
  PAT scope model as the API routes.
- Added `tools/list`, product metadata tool execution, and scope enforcement
  tests.
- Updated README, user docs, roadmap, implementation matrix, current status,
  project map, and agent testing notes.
- Verified with `cargo fmt --all -- --check`,
  `cargo test -p msm-mcp --locked`,
  `cargo clippy -p msm-mcp --all-targets --locked -- -D warnings`, and
  `git diff --check`.

## 2026-05-09 Product Data Web Management

- Added product metadata API client functions for folder, tag, and
  subscription-group create/list workflows.
- Added a Web Organize workspace section with desktop-friendly three-column
  management for folders, tags, and subscription groups.
- Added navigation, Traditional Chinese/English labels, and selectable
  subscription PAT scopes for Web local-login/PAT dialogs.
- Added API client and Vue component tests for the metadata surface.
- Updated README, user docs, roadmap, implementation matrix, and current
  status.
- Verified with `npm run web:typecheck`, `npm run web:test`, and
  `npm run web:build`.

## 2026-05-09 Product Data Membership Storage

- Added `FolderPackRecord`, `PackTagRecord`, and
  `SubscriptionGroupPackRecord` models.
- Added repository methods for adding, ordered listing, and removing folder-pack
  links.
- Added repository methods for adding, listing, and removing pack-tag links.
- Changed subscription-group pack insertion to return a link record and support
  sort-order upsert; added removal support.
- Added storage integration coverage for pack membership lifecycle across
  folders, tags, and subscription groups.
- Verified with `cargo fmt --all -- --check`,
  `cargo test -p msm-storage --test product_data_repository_tests --locked`,
  `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and
  `git diff --check`.

## 2026-05-09 Product Data Membership API

- Added protected folder-pack membership API routes for ordered list, upsert,
  and remove operations.
- Added protected pack-tag membership API routes for list, assign, and remove
  operations.
- Added protected subscription-group pack membership API routes for ordered
  list, upsert, and remove operations.
- Added DTOs and OpenAPI path/schema registration for membership request and
  response payloads.
- Added ownership and tenant checks so membership routes require the PAT user to
  own the pack and folder or subscription group being linked.
- Added API integration coverage for membership add/list/remove behavior and
  OpenAPI path registration.

## 2026-05-09 Product Data Membership CLI

- Added CLI DTOs and HTTP client calls for folder-pack, pack-tag, and
  subscription-group pack membership add/list/remove operations.
- Added `msm metadata folders packs`, `msm metadata pack-tags`, and
  `msm metadata subscription-groups packs` command groups.
- Added human/JSON output formatting for membership links, ID lists, and remove
  acknowledgements.
- Added parser and fake-client execution tests for the new membership commands.

## 2026-05-09 Product Data Membership MCP

- Added MCP tool definitions for folder-pack, pack-tag, and subscription-group
  pack membership add/list/remove operations.
- Added MCP handler support backed by the storage repository with PAT scope
  enforcement, resource ownership checks, and same-tenant validation.
- Added MCP tests for tool registry/schema coverage, successful membership
  lifecycle calls, and missing `pack.update` rejection.
- Updated PRD, README, user docs, roadmap, implementation matrix, and current
  status to mark MCP membership parity and leave Web membership controls as the
  next slice.

## 2026-05-09 Product Data Membership Web

- Added Web ProductMetadataClient methods for folder-pack, pack-tag, and
  subscription-group pack add/list/remove API routes.
- Added an Organize workspace membership console that lets users select a pack
  and link or unlink it from folders, tags, and subscription groups.
- Passed the live pack list into the Organize workspace so membership controls
  operate on user-visible packs rather than manual ID entry.
- Added API client and Vue component coverage for membership link operations.
- Updated PRD, README, user docs, roadmap, implementation matrix, and current
  status to mark product organization parity complete across API, CLI, MCP, and
  Web.

## 2026-05-09 Subscription Payload Contract

- Added pure domain input types and a builder for MoreStickers dynamic
  subscription pack-set metadata.
- Added a subscription Bearer auth-header helper so protected group payloads
  and per-pack refresh entries share the same credential contract.
- Added compatibility tests proving public payloads omit auth headers and
  protected payloads include the expected `Authorization` header at both group
  and pack-refresh levels.
- Updated PRD, roadmap, implementation matrix, and current status so the next
  slice starts from API endpoints instead of re-designing the payload shape.

## 2026-05-09 Subscription Endpoint API

- Added public API routes for per-pack dynamic subscription payloads, per-pack
  stickerpack refresh payloads, and subscription-group dynamic payloads.
- Added owner PAT fallback for private pack and private subscription-group
  payload reads while keeping anonymous access limited to public resources.
- Public subscription-group payloads now filter private packs for anonymous
  callers so public groups do not leak private pack entries.
- Registered the new routes in OpenAPI and added API integration coverage for
  anonymous public access, private anonymous rejection, owner PAT access, and
  dynamic refresh URLs.
- Updated PRD, roadmap, implementation matrix, and current status so the next
  slice can focus on persistent subscription secrets and link rotation.

## 2026-05-09 Subscription Access Token Storage

- Added `subscription_access_tokens` storage for pack and subscription-group
  link credentials.
- Added resource-type models for `pack` and `subscription_group` subscription
  access tokens.
- Added repository methods to create, verify, rotate, list, and revoke
  subscription access tokens while only storing token hashes.
- Added storage tests proving old tokens stop verifying after rotation and
  revoked tokens stop verifying.
- Updated PRD, roadmap, implementation matrix, and current status so the next
  slice wires token verification into public subscription endpoints.

## 2026-05-09 Subscription Access Token API Enforcement

- Public pack refresh, per-pack subscription, and subscription-group endpoints
  now accept matching `msm_sub_*` subscription access tokens for private
  resources.
- Protected dynamic subscription payloads now include refresh `Authorization`
  headers when the caller used a subscription access token, while owner PAT
  access does not get embedded back into payloads.
- Added API tests for private pack access, private subscription-group access,
  protected payload auth headers, and subscription token resource mismatch
  rejection.
- Updated PRD, roadmap, implementation matrix, and current status so the next
  slice can expose subscription link creation, rotation, and revocation
  controls across API/CLI/MCP/Web.

## 2026-05-09 Subscription Access Token Management API

- Added `/api/v1/subscription-access-tokens` create/list routes and
  `/api/v1/subscription-access-tokens/{token_id}` rotate/revoke routes.
- Create and rotate responses return the raw `msm_sub_*` secret once; list
  responses return metadata only and never expose raw secrets or token hashes.
- Pack default subscription links require `pack.manage_access` and pack
  ownership; subscription-group links require `subscription.manage_access` and
  group ownership.
- Added OpenAPI schemas and path registration for the new management routes.
- Updated PRD, roadmap, implementation matrix, and current status so the next
  slice can add CLI/MCP/Web controls on top of the API contract.

## 2026-05-09 Subscription Access Token CLI

- Added `msm subscription-links create`, `list`, `rotate`, and `revoke`
  commands.
- Added CLI client request/response models and HTTP calls for the subscription
  access token management API.
- Human output prints raw `msm_sub_*` secrets only for create/rotate, while
  list output is metadata-only and revoke output is a confirmation.
- Updated PRD, roadmap, implementation matrix, and current status so the next
  slice can add MCP/Web controls.

## 2026-05-09 Subscription Access Token MCP

- Added `msm.create_subscription_link`, `msm.list_subscription_links`,
  `msm.rotate_subscription_link`, and `msm.revoke_subscription_link` tools.
- Matched API authorization semantics: pack links require `pack.manage_access`,
  subscription-group links require `subscription.manage_access`, and both
  require resource ownership.
- MCP list responses expose metadata only; create and rotate responses expose
  the raw `msm_sub_*` secret once.
- Updated PRD, roadmap, implementation matrix, and current status so the next
  slice can add Web controls.

## 2026-05-09 Subscription Access Token Web

- Added Web API client methods for subscription link list/create/rotate/revoke.
- Added Organize UI controls to manage pack and subscription-group links,
  including metadata-only link listing and one-time raw secret display after
  create/rotate.
- Added Web tests for subscription link API calls and UI actions.
- Updated PRD, roadmap, implementation matrix, and current status so the next
  slice can focus on Web-session access and private asset authorization.

## 2026-05-09 Documentation Consolidation

- Added `docs/PRD.md` as the living requirements, current status, roadmap,
  surface parity, verification, and completion source.
- Reduced active Agent handoff docs to `docs/agents/README.md` and folded the
  old read order/status protocol into that file.
- Removed legacy per-phase `docs/superpowers` plans/specs and duplicated Agent
  handoff files now covered by the PRD, status docs, and developer docs.
- Updated README, roadmap, and implementation matrix to point to the PRD.

## 2026-05-09 PRD Self-Review Hardening

- Clarified PRD status meanings so feature-level "implemented" does not imply
  whole-project completion.
- Added a current implementation queue, current surface parity gaps, open
  product questions, and per-slice definition of done.
- Kept the next active implementation focus on product membership MCP and Web
  parity.
