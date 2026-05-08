# Current Status

Phase: P25 export worker foundation.

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

Current task:
- Continue from Telegram sticker mapping storage: mappings can be persisted and queried; next step is wiring worker publication/reconciliation to populate mappings from fetched remote state.

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

Next step:
- Populate Telegram sticker mappings from fetched remote state after publication/reconciliation.

Known issues:
- PowerShell profile emits an fnm symlink permission warning in this environment.
- Docker CLI is not installed in this environment, so Docker image build is deferred to GitHub Actions or a Docker-enabled machine.
- `skills-lock.json` is an existing untracked local skill lock file and is not part of MSM source changes.
