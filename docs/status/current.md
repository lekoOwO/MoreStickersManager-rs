# Current Status

Phase: Product-data management surfaces after P33 Telegram reconciliation parity.

Last completed:
- P23 Web pack import: dashboard `.stickerpack` JSON import backed by the protected pack import API.
- P24 Telegram export pipeline analysis: documented moe-sticker-bot-inspired media conversion, export target, Telegram Bot API, job, Web/API/CLI/MCP parity, and phased implementation plan.
- P24 status/documentation cleanup: added `docs/status/implementation-matrix.md` as the implemented-versus-planned source of truth.
- P25 Task 1 media profile foundation: added `msm-media` with source media kinds, Telegram static/video target profiles, prepared media specs, conversion plan selection, and profile tests.
- P25 Task 2 converter command planning: added shell-free ffmpeg command plans for Telegram static image, video sticker, and thumbnail outputs.
- Task 3 export job persistence: added SQLite tables and repository methods for export targets, export jobs, ordered job events, prepared media assets, and Telegram publication records.
- Task 4 exporter registry: added `msm-exporters` with target kind keys, capability metadata, export request/plan types, target trait, and duplicate-safe registry.
- Task 5 MoreStickers export target: wrapped existing `.stickerpack` serialization as a concrete `morestickers` export target with byte-for-byte compatibility tests.
- Task 6 Telegram bot framework boundary: added `msm-telegram` using `teloxide`, with redacted token/config handling, configurable Bot API URL, and `teloxide::Bot` construction tests.
- Task 7 Telegram export planner: added Telegram set name normalization, size checks, initial/append batching, create-only conflict handling, media profile mapping, and teloxide `InputSticker` conversion planning.
- Task 8 export API and OpenAPI: added export permissions, target kind/target CRUD routes, queued job creation, job status/event reads, token-redacted target responses, and OpenAPI schemas.
- Task 9 export worker foundation: added worker config, queued job pickup, running/succeeded/failed transitions, job events, MoreStickers artifact execution, and Telegram dry-run planning without network calls.
- Task 9 worker cache/loop slice: added prepared media executor boundary, prepared media cache writes, worker enabled/poll interval config, and optional service worker loop spawning.
- Task 9 process converter slice: added process-backed prepared media executor using shell-free `msm-media` conversion commands, timeout handling, exit status validation, and output metadata reads.
- Task 9 target bootstrap config: added `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON` parsing and idempotent export target create/update during service initialization.
- Task 10 CLI export commands: added `msm exports kinds`, export target list/create, export job create/get/events commands, API client calls, and human/JSON output.
- Task 10 MCP export tools: added export target kind/list/create tools, export job create/get/events tools, PAT scope enforcement, owner checks, schema coverage, and token-redacted target responses.
- Task 11 Web export workflow: added typed export API client functions, target settings panel, pack export wizard, job timeline, Traditional Chinese/English labels, Telegram token validation, redacted target display, and injected-client tests.
- Task 12 documentation and full verification: updated architecture/provider/user/agent docs for export boundaries, Telegram setup, ffmpeg requirements, Web/CLI/MCP workflows, and ran full Rust/Web baseline verification.
- Telegram publication Task 1: added a mockable `msm-telegram` publish boundary with create-then-append orchestration, publication result metadata, and no-network tests.
- Telegram publication Task 2: added the `teloxide::Bot` sticker set API adapter, typed owner ID validation, teloxide request error normalization, public re-export, and a no-network adapter construction test.
- Telegram publication Task 3: added worker-level Telegram publication executor injection, kept `dryRun` defaulting to true, wired `dryRun:false` jobs to prepared media file paths and teloxide publication, persisted `telegramPublished` results, and covered publisher failure handling.
- Telegram publication Task 4: added shared Web export result-link extraction and rendered completed Telegram sticker set URLs in both the export wizard and job timeline.
- Telegram publication Task 5: documented the completed dry-run/default publication path, target token requirements, prepared media dependency, no-network test strategy, and current remaining reconciliation work.
- Telegram publication repository slice: added typed `telegram_publications` storage models plus upsert/find/list repository methods for future reconciliation and API exposure.
- Telegram worker publication persistence slice: successful `dryRun:false` worker jobs now upsert durable Telegram publication records by target and sticker set name; dry-run jobs still do not create publication records.
- Telegram publication API slice: added protected `GET /api/v1/telegram-publications?packId=...` and `GET /api/v1/telegram-publications/{publication_id}` routes with OpenAPI schemas, `export.read` enforcement, and pack ownership checks.
- Telegram publication CLI slice: added `msm exports publications list --pack-id ...` and `msm exports publications get --publication-id ...` with human/JSON output.
- Web export UX/publication history slice: reviewed the current frontend, kept the existing shell instead of a full rewrite, and added persisted Telegram publication history to the export wizard for the selected pack.
- Telegram publication MCP slice: added `msm.list_telegram_publications` and `msm.get_telegram_publication` tools with `export.read` enforcement and pack ownership checks.
- Export job retry policy slice: added retry metadata, due-job backoff selection, worker requeue behavior for retryable failures, terminal failure after attempt budget exhaustion, and `MSM_EXPORT_RETRY_BACKOFF_MS`.
- Telegram reconciliation policy slice: added pure exporter-level reconciliation planning for create-only, append-missing, and mirror modes, including title updates, sticker replacement, remote-only deletion, and no-network tests.
- Telegram mutation boundary slice: extended `msm-telegram` with mockable title update, sticker add, sticker replace, and sticker delete mutations backed by teloxide adapter methods and no-network ordering tests.
- Telegram worker reconciliation dry-run slice: export jobs can accept `reconcileMode` plus optional `remoteSet` options and include reconciliation operation/mutation summaries in Telegram dry-run results without calling Telegram.
- Telegram guarded reconciliation execution slice: non-dry-run `appendMissing` reconciliation can execute planned Telegram mutations through an injected executor when `executeReconciliation:true` and `remoteSet` are supplied; regular publication remains unchanged.
- Telegram mirror safety slice: mirror reconciliation replace/delete operations now require `allowDestructiveReconciliation:true` in addition to `executeReconciliation:true`.
- Telegram remote fetch boundary slice: `msm-telegram` can fetch remote sticker set metadata through a mockable `getStickerSet` boundary backed by teloxide, with no-network tests.
- Telegram sticker mapping storage slice: added durable storage for MSM source sticker ID to Telegram file ID mappings per publication/target/sticker set.
- Telegram post-publication mapping slice: successful non-dry-run Telegram publication now fetches the remote sticker set through an injected executor and persists per-sticker Telegram file mappings by planned sticker order.
- Development environment manager slice: added a Node.js dev manager for starting/stopping API and Web dev services, status checks, environment profile switching, tracked env examples, npm/pnpm script shortcuts, Windows-safe hidden process spawning, local runtime directory creation, and handoff/user documentation.
- Web workspace redesign slice: replaced the crowded single-page/card-heavy dashboard with a wider Ant Design-inspired shell, workspace tabs, local login/PAT dialogs, pack import dialog, stat strip, and table-style pack management while preserving API-backed operations.
- Development bootstrap usability slice: the development profile now waits for API health, creates or reuses a local dev account, creates a PAT, writes `VITE_MSM_PAT` to `.env.local`, imports a sample pack, and starts Web afterward so live API-backed UI actions work immediately.
- Web desktop/mobile UX correction slice: Vite now loads env files from the repository root, runtime status distinguishes Live API/API-needs-PAT/Mock preview, desktop uses a full-width workbench layout with an icon rail plus context panel, mobile uses separate compact pack cards, buttons expose pointer/pressed states, and light/dark blue tokens are more vivid.
- Web native navigation correction slice: desktop navigation is now a single collapsible rail, duplicate in-content tabs are suppressed when the shell controls the active section, top-bar controls are limited to global actions, dark theme returns to a near-black Ant Design-like blue palette, and Playwright E2E uses installed Microsoft Edge instead of downloaded Chromium.
- Web UI QA hardening slice: fixed expanded desktop brand clipping, removed the incorrect collapsed `API` runtime label, replaced free-form PAT scope text inputs with selectable scope cards, translated remaining fixed zh-TW dashboard/access-token labels, and added Edge E2E coverage for those regressions.
- Web rail containment follow-up: collapsed desktop rail header now stacks the MS logo and expand control vertically, keeping both controls inside the rail with E2E bounding-box coverage.
- Telegram reconciliation mapping refresh slice: successful non-dry-run reconciliation mutation jobs now fetch remote Telegram sticker set state after mutation execution and refresh per-sticker MSM-to-Telegram file mappings.
- Telegram automatic remote-state reconciliation slice: non-dry-run reconciliation jobs can omit `remoteSet`; the worker fetches Telegram remote metadata, maps Telegram file IDs through stored sticker mappings, and builds the planner's `TelegramRemoteSet` automatically.
- Web Telegram reconciliation controls slice: export wizard now exposes Telegram dry-run, reconciliation mode, execute-reconciliation, and destructive mirror guard controls, merging them into export job options without requiring hand-written JSON.
- CLI/MCP Telegram reconciliation affordances slice: CLI export job creation now has Telegram-specific flags for live mode, reconciliation mode, execute-reconciliation, set-name slug, default emoji, and destructive mirror opt-in; MCP `msm.create_export_job` now accepts equivalent named fields and no longer requires raw `options` for this workflow.
- API/OpenAPI Telegram options documentation slice: `CreateExportJobRequest.options` now points to a typed `TelegramExportJobOptions` OpenAPI schema that documents dry-run, reconciliation mode, execution guard, destructive mirror guard, remote state, and set naming fields while preserving JSON request flexibility.
- Telegram destructive mirror operator runbook slice: added a user-facing runbook for safe dry-run review, append-missing operation, guarded mirror execution, review checklist, and recovery notes.
- Product-data API planning slice: added the implementation plan for folder, tag, subscription-group, and pack access metadata APIs.
- Product-data storage repository slice: added folder records and CRUD/listing methods, tag records and create/list/delete methods, subscription-group list/rename/delete methods, and a storage integration test covering the first product-data metadata lifecycle.
- Product-data API route slice: added protected folder create/list, tag create/list, and subscription-group create/list API routes plus DTOs and OpenAPI registration.
- Product-data CLI surface slice: added `msm metadata folders`, `msm metadata tags`, and `msm metadata subscription-groups` create/list commands backed by the protected API client, with human/JSON output and fake-client tests.
- Product-data MCP surface slice: added `msm.list_folders`, `msm.create_folder`, `msm.list_tags`, `msm.create_tag`, `msm.list_subscription_groups`, and `msm.create_subscription_group` tools with PAT scope enforcement.
- Product-data Web surface slice: added an Organize workspace section with API-backed folder, tag, and subscription-group create/list management, navigation integration, i18n labels, and selectable subscription PAT scopes.
- Product-data membership storage slice: added repository primitives for folder-pack links, pack-tag links, and subscription-group pack links, including ordered listing and removal.

Current task:
- Continue product-data management by exposing pack-folder/tag membership and
  subscription-group pack membership/link semantics through API/OpenAPI, then
  CLI/MCP/Web.

Short roadmap:
- See `docs/status/roadmap.md` for the concise current focus, immediate plan,
  later planned work, and verification expectations.

Last verification:
- P23 full verification passed before P24 docs.
- P24 docs-only verification: `git diff --check`.
- P25 Task 1: `cargo test -p msm-media --locked`.
- P25 Task 2: `cargo fmt --all -- --check`; `cargo test -p msm-media --locked`; `cargo clippy -p msm-media --all-targets --locked -- -D warnings`; `git diff --check`.
- Task 3: `cargo fmt --all -- --check`; `cargo test -p msm-storage --locked`; `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`; `git diff --check`.
- Task 4: `cargo fmt --all -- --check`; `cargo test -p msm-exporters --locked`; `cargo clippy -p msm-exporters --all-targets --locked -- -D warnings`; `git diff --check`.
- Task 5: `cargo fmt --all -- --check`; `cargo test -p msm-exporters --locked`; `cargo clippy -p msm-exporters --all-targets --locked -- -D warnings`; `git diff --check`.
- Task 6: `cargo fmt --all -- --check`; `cargo test -p msm-telegram --locked`; `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`; `git diff --check`.
- Task 7: `cargo fmt --all -- --check`; `cargo test -p msm-exporters --locked`; `cargo clippy -p msm-exporters --all-targets --locked -- -D warnings`; `git diff --check`.
- Task 8: `cargo test -p msm-api --locked`; `cargo clippy -p msm-api -p msm-storage -p msm-domain --all-targets --locked -- -D warnings`; `git diff --check`.
- Task 9 foundation: `cargo test -p msm-app --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- Task 9 cache/loop: `cargo test -p msm-app --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- Task 9 process converter: `cargo test -p msm-app --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- Task 9 target bootstrap: `cargo test -p msm-app --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- Task 10 CLI export commands: `cargo fmt --all -- --check`; `cargo test -p msm-cli --locked`; `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`.
- Task 10 CLI/MCP parity: `cargo fmt --all -- --check`; `cargo test -p msm-cli -p msm-mcp --locked`; `cargo clippy -p msm-cli -p msm-mcp --all-targets --locked -- -D warnings`.
- Task 11 Web export workflow: `npm run web:typecheck`; `npm run web:test`; `npm run web:build`.
- Task 12 full verification: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked`; `npm run web:typecheck`; `npm run web:test`; `npm run web:build`.
- Telegram publication Task 1: `cargo test -p msm-telegram --locked`; `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`.
- Telegram publication Task 2: `cargo fmt --all -- --check`; `cargo test -p msm-telegram --locked`; `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`.
- Telegram publication Task 3: `cargo fmt --all -- --check`; `cargo test -p msm-app --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- Telegram publication Task 4: `npm run web:typecheck`; `npm run web:test`; `npm run web:build`.
- Telegram publication Task 5 full verification: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked`; `npm run web:typecheck`; `npm run web:test`; `npm run web:build`.
- Telegram publication repository slice: `cargo fmt --all -- --check`; `cargo test -p msm-storage --test export_job_repository_tests --locked`; `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`.
- Telegram worker publication persistence slice: `cargo fmt --all -- --check`; `cargo test -p msm-app --test export_worker_tests --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- Telegram publication API slice: `cargo fmt --all -- --check`; `cargo test -p msm-api --locked`; `cargo clippy -p msm-api --all-targets --locked -- -D warnings`; `git diff --check`.
- Telegram publication CLI slice: `cargo fmt --all -- --check`; `cargo test -p msm-cli --locked`; `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`; `git diff --check`.
- Web export UX/publication history slice: `npm run web:typecheck`; `npm run web:test`; `npm run web:build`; `git diff --check`.
- Telegram publication MCP slice: `cargo fmt --all -- --check`; `cargo test -p msm-mcp --locked`; `cargo clippy -p msm-mcp --all-targets --locked -- -D warnings`; `npm run web:typecheck`; `npm run web:test`; `npm run web:build`; `git diff --check`.
- Export job retry policy slice: `cargo fmt --all -- --check`; `cargo test -p msm-storage --test export_job_repository_tests --locked`; `cargo test -p msm-app --test export_worker_tests --locked`; `cargo test -p msm-api --locked`; `cargo test -p msm-cli --locked`; `cargo test -p msm-mcp --locked`; `cargo clippy -p msm-storage -p msm-app -p msm-api -p msm-cli -p msm-mcp --all-targets --locked -- -D warnings`; `npm run web:typecheck`; `npm run web:test`; `npm run web:build`; `git diff --check`.
- Telegram reconciliation policy slice: `cargo fmt --all -- --check`; `cargo test -p msm-exporters --test telegram_plan_tests --locked`; `cargo clippy -p msm-exporters --all-targets --locked -- -D warnings`; `git diff --check`.
- Telegram mutation boundary slice: `cargo fmt --all -- --check`; `cargo test -p msm-telegram --locked`; `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`; `git diff --check`.
- Telegram worker reconciliation dry-run slice: `cargo fmt --all -- --check`; `cargo test -p msm-app --test export_worker_tests --locked`; `cargo clippy -p msm-app -p msm-exporters --all-targets --locked -- -D warnings`; `git diff --check`.
- Telegram guarded reconciliation execution slice: `cargo fmt --all -- --check`; `cargo test -p msm-app --test export_worker_tests --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`; `git diff --check`.
- Telegram mirror safety slice: `cargo fmt --all -- --check`; `cargo test -p msm-app --test export_worker_tests --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`; `git diff --check`.
- Telegram remote fetch boundary slice: `cargo fmt --all -- --check`; `cargo test -p msm-telegram --locked`; `cargo clippy -p msm-telegram --all-targets --locked -- -D warnings`; `git diff --check`.
- Telegram sticker mapping storage slice: `cargo fmt --all -- --check`; `cargo test -p msm-storage --test export_job_repository_tests --locked`; `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`; `git diff --check`.
- Telegram post-publication mapping slice: `cargo fmt --all -- --check`; `cargo test -p msm-app --test export_worker_tests --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`; `git diff --check`.
- Development environment manager slice: `node scripts/dev-manager.mjs --help`; `node scripts/dev-manager.mjs env list`; `node scripts/dev-manager.mjs env init development`; `node scripts/dev-manager.mjs env use testing`; `node scripts/dev-manager.mjs status`; `npm run dev:status`; `node scripts/dev-manager.mjs env use development`; `node scripts/dev-manager.mjs stop`; `pnpm run dev:start`; `pnpm run dev:status`; `Invoke-WebRequest -UseBasicParsing http://127.0.0.1:3000/healthz`; `Invoke-WebRequest -UseBasicParsing http://127.0.0.1:5173`; `pnpm run dev:stop`; `npm run dev:start`; `npm run dev:status`; API/Web HTTP checks; `npm run dev:stop`; hidden wrapper regression checks; `git diff --check`.
- Web workspace redesign slice: `npm run web:typecheck`; `npm run web:test`; `npm run web:build`; `pnpm run dev:start`; `pnpm run dev:status`; `Invoke-WebRequest -UseBasicParsing http://127.0.0.1:3000/healthz`; `Invoke-WebRequest -UseBasicParsing http://127.0.0.1:5173`; `pnpm run dev:stop`; `git diff --check`.
- Development bootstrap usability slice: `node --check scripts/dev-manager.mjs`; `pnpm run dev:stop`; `node scripts/dev-manager.mjs env use development`; `pnpm run dev:start`; `pnpm run dev:status`; `Invoke-WebRequest -UseBasicParsing http://127.0.0.1:3000/healthz`; PAT-authenticated `GET /api/v1/packs?userId=user_1`; `Invoke-WebRequest -UseBasicParsing http://127.0.0.1:5173`; `pnpm run dev:stop`; repeated `pnpm run dev:start` with existing valid PAT; repeated `pnpm run dev:stop`; `git diff --check`.
- Web desktop/mobile UX correction slice: `npm run web:typecheck`; `npm run web:test`; `npm run web:build`; `pnpm run dev:stop`; `pnpm run dev:start`; API health check; PAT-authenticated pack list check; Web HTTP check; Vite module env check; `pnpm run dev:stop`.
- Web native navigation correction slice: `npm run web:typecheck`; `npm run web:test`; `npm run web:build`; `npm run web:e2e` using installed Microsoft Edge; verified `%LOCALAPPDATA%\ms-playwright` does not exist after E2E.
- Web UI QA hardening slice: `npm run web:typecheck`; `npm run web:test`; `npm run web:build`; `npm run web:e2e` using installed Microsoft Edge; verified `%LOCALAPPDATA%\ms-playwright` does not exist after E2E.
- Web rail containment and Telegram reconciliation mapping refresh slice: `cargo fmt --all -- --check`; `cargo test -p msm-app --test export_worker_tests --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`; `npm run web:typecheck`; `npm run web:test`; `npm run web:build`; `npm run web:e2e` using installed Microsoft Edge; verified `%LOCALAPPDATA%\ms-playwright` does not exist after E2E; `git diff --check`.
- Telegram automatic remote-state reconciliation slice: `cargo fmt --all -- --check`; `cargo test -p msm-app --test export_worker_tests --locked`; `cargo clippy -p msm-app --all-targets --locked -- -D warnings`.
- Web Telegram reconciliation controls slice: `npm run web:typecheck`; `npm run web:test`; `npm run web:build`.
- Documentation progress and roadmap cleanup: `git diff --check`.
- CLI/MCP Telegram reconciliation affordances slice: targeted RED/GREEN tests for `cargo test -p msm-cli parses_export_job_create_telegram_reconciliation_flags --locked`, `cargo test -p msm-cli executes_export_job_create_with_telegram_reconciliation_flags --locked`, and `cargo test -p msm-mcp tools_call_creates_telegram_reconciliation_job_without_raw_options --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-cli -p msm-mcp --locked`, `cargo clippy -p msm-cli -p msm-mcp --all-targets --locked -- -D warnings`, and `git diff --check`.
- API/OpenAPI Telegram options documentation slice: targeted RED/GREEN test for `cargo test -p msm-api openapi_documents_telegram_export_job_options --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-api --locked`, `cargo clippy -p msm-api --all-targets --locked -- -D warnings`, and `git diff --check`.
- Telegram destructive mirror operator runbook and product-data API planning slice: docs-only verification with `git diff --check`.
- Product-data storage repository slice: RED/GREEN test with `cargo test -p msm-storage folders_tags_and_subscription_groups_can_be_managed --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-storage --locked`, `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and `git diff --check`.
- Product-data API route slice: RED/GREEN test with `cargo test -p msm-api metadata_routes_manage_folders_tags_and_subscriptions --locked`; OpenAPI path check with `cargo test -p msm-api openapi_endpoint_contains_health_path --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-storage -p msm-api --locked`, `cargo clippy -p msm-storage -p msm-api --all-targets --locked -- -D warnings`, and `git diff --check`.
- Product-data CLI surface slice: RED/GREEN tests with `cargo test -p msm-cli parses_metadata_commands --locked` and `cargo test -p msm-cli executes_metadata_commands --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-cli --locked`, `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`, and `git diff --check`.
- Product-data MCP surface slice: RED/GREEN tests with `cargo test -p msm-mcp tools_call_manages_folders --locked`, `cargo test -p msm-mcp tools_call_manages_tags --locked`, `cargo test -p msm-mcp tools_call_manages_subscription_groups --locked`, `cargo test -p msm-mcp tools_list_returns_pack_and_export_tools --locked`, `cargo test -p msm-mcp tool_registry_contains_pack_tools --locked`, and `cargo test -p msm-mcp pat_enforcement_metadata_tools_require_expected_scopes --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-mcp --locked`, `cargo clippy -p msm-mcp --all-targets --locked -- -D warnings`, and `git diff --check`.
- Product-data Web surface slice: RED/GREEN tests with `npm run web:test -- api-client` and `npm run web:test -- product-metadata-ui`; full verification with `npm run web:typecheck`, `npm run web:test`, and `npm run web:build`.
- Product-data membership storage slice: RED/GREEN test with `cargo test -p msm-storage pack_memberships_can_be_managed --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-storage --test product_data_repository_tests --locked`, `cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and `git diff --check`.
- Product-data membership API slice: RED/GREEN test with `cargo test -p msm-api metadata_routes_manage_pack_memberships --locked`; OpenAPI path check with `cargo test -p msm-api openapi_endpoint_contains_health_path --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-api --locked`, `cargo clippy -p msm-api -p msm-storage --all-targets --locked -- -D warnings`, and `git diff --check`.
- Product-data membership CLI slice: RED/GREEN tests with `cargo test -p msm-cli parses_metadata_membership_commands --locked`, `cargo test -p msm-cli executes_metadata_folder_membership_commands --locked`, `cargo test -p msm-cli executes_metadata_pack_tag_membership_commands --locked`, and `cargo test -p msm-cli executes_metadata_subscription_group_membership_commands --locked`; full verification with `cargo fmt --all -- --check`, `cargo test -p msm-cli --locked`, `cargo clippy -p msm-cli --all-targets --locked -- -D warnings`, and `git diff --check`.

Next step:
- Continue product-data metadata by adding MCP and Web controls for pack-folder/tag membership and subscription-group pack membership/link operations.

Known issues:
- PowerShell profile emits an fnm symlink permission warning in this environment.
- Docker CLI is not installed in this environment, so Docker image build is deferred to GitHub Actions or a Docker-enabled machine.
- `skills-lock.json` is an existing untracked local skill lock file and is not part of MSM source changes.
