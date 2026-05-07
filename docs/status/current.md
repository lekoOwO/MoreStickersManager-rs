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

Current task:
- Continue Task 9 Worker Execution from `docs/superpowers/plans/2026-05-06-msm-telegram-export-pipeline.md`, specifically real converter execution/cache writes and background loop composition.

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

Next step:
- Task 9 continuation: add converter execution/cache writes and decide how the service starts/stops the worker loop.

Known issues:
- PowerShell profile emits an fnm symlink permission warning in this environment.
- Docker CLI is not installed in this environment, so Docker image build is deferred to GitHub Actions or a Docker-enabled machine.
- `skills-lock.json` is an existing untracked local skill lock file and is not part of MSM source changes.
