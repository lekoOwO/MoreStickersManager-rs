# Project Map

## Implemented in P0/P1

- `crates/msm-domain`: MoreStickers-compatible domain models and pure helpers.
- `crates/msm-domain/src/authz.rs`: pure authorization policy primitives and evaluators.
- `crates/msm-storage`: SQL storage primitives, local asset storage, and portable export/import.
- `crates/msm-api`: Axum API routes and utoipa OpenAPI generation.
- `crates/msm-cli`: HTTP CLI client for MSM API operations, including pack list/import/export/rename/delete and PAT lifecycle commands.
- `crates/msm-providers`: provider registry plus Telegram and LINE fixture normalizers.
- `crates/msm-app`: runnable Axum service binary composing storage, API, assets, and Web UI static serving.
- `crates/msm-mcp`: MCP `/mcp` JSON-RPC endpoint with pack list/import/export/update/delete tools.
- `crates/msm-media`: media profile foundation with source media kinds, Telegram static/video target profiles, prepared media specs, and conversion plan selection.
- `apps/web`: Vue/Vite Web UI foundation with theme, i18n, Shadcn Vue-style primitives, mock fallback, protected API client, PAT panel, pack CRUD controls, and pack import UI.
- `components.json`: Shadcn Vue configuration for the Web UI workspace.
- `docs/status`: current state and development log.
- `docs/status/implementation-matrix.md`: implemented-versus-planned feature truth source.
- `docs/dev`: human developer references.
- `docs/agents`: progressive disclosure handoff docs.

## Not Implemented Yet

- MCP auth/session/SSE hardening.
- Folder, tag, subscription-group, and pack access-management APIs.
- Provider network integrations and asset download orchestration.
- Media converter command planning, probing, execution, and prepared media cache persistence.
- Export target registry and remote publication jobs.
- Telegram Bot API sticker set creation and Web-managed export workflow.

Do not add cross-layer dependencies to `msm-domain`.
For feature completion status, prefer `../status/implementation-matrix.md` over
inferring from old phase names.
