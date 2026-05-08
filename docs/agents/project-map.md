# Project Map

## Implemented in P0/P1

- `crates/msm-domain`: MoreStickers-compatible domain models and pure helpers.
- `crates/msm-domain/src/authz.rs`: pure authorization policy primitives and evaluators.
- `crates/msm-storage`: SQL storage primitives, local asset storage, portable export/import, PAT/local credential storage, and export target/job/retry/prepared media persistence.
- `crates/msm-api`: Axum API routes and utoipa OpenAPI generation, including export target/job routes.
- `crates/msm-cli`: HTTP CLI client for MSM API operations, including pack list/import/export/rename/delete, PAT lifecycle commands, export target/job commands, and Telegram publication history reads.
- `crates/msm-providers`: provider registry plus Telegram and LINE fixture normalizers.
- `crates/msm-exporters`: export target trait, target kind keys, capability metadata, request/plan types, duplicate-safe registry, concrete `morestickers` export target, Telegram sticker set planner, and Telegram reconciliation policy planner.
- `crates/msm-app`: runnable Axum service binary composing storage, API, assets, Web UI static serving, prepared media conversion, export worker execution, bounded export job retry handling, and Telegram reconciliation dry-run summaries.
- `crates/msm-mcp`: MCP `/mcp` JSON-RPC endpoint with pack list/import/export/update/delete tools, export target/job tools, and Telegram publication history tools.
- `crates/msm-media`: media profile foundation with source media kinds, Telegram static/video/thumbnail target profiles, prepared media specs, conversion plan selection, and shell-free ffmpeg command planning.
- `apps/web`: Vue/Vite Web UI foundation with theme, i18n, Shadcn Vue-style primitives, mock fallback, protected API clients, PAT panel, pack CRUD controls, pack import UI, export target panel, export wizard, Telegram publication history panel, and export job timeline.
- `crates/msm-telegram`: teloxide-based Telegram bot boundary with redacted token/config handling, Bot API URL configuration, mockable sticker set create/append execution, and mockable sticker set mutation execution.
- `components.json`: Shadcn Vue configuration for the Web UI workspace.
- `docs/status`: current state and development log.
- `docs/status/implementation-matrix.md`: implemented-versus-planned feature truth source.
- `docs/dev`: human developer references.
- `docs/agents`: progressive disclosure handoff docs.

## Not Implemented Yet

- MCP auth/session/SSE hardening.
- Folder, tag, subscription-group, and pack access-management APIs.
- Provider network integrations and asset download orchestration.
- Media probing through ffprobe.
- Guarded non-dry-run Telegram reconciliation mutation execution and remote state retrieval.

Do not add cross-layer dependencies to `msm-domain`.
For feature completion status, prefer `../status/implementation-matrix.md` over
inferring from old phase names.
