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
