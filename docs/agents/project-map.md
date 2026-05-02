# Project Map

## Implemented in P0/P1

- `crates/msm-domain`: MoreStickers-compatible domain models and pure helpers.
- `crates/msm-domain/src/authz.rs`: pure authorization policy primitives and evaluators.
- `crates/msm-storage`: SQL storage primitives, local asset storage, and portable export/import.
- `crates/msm-api`: Axum API routes and utoipa OpenAPI generation.
- `crates/msm-cli`: HTTP CLI client for MSM API operations.
- `crates/msm-providers`: provider registry plus Telegram and LINE fixture normalizers.
- `apps/web`: Vue/Vite Web UI foundation with theme, i18n, Shadcn Vue-style primitives, and mock dashboard.
- `components.json`: Shadcn Vue configuration for the Web UI workspace.
- `docs/status`: current state and development log.
- `docs/dev`: human developer references.
- `docs/agents`: progressive disclosure handoff docs.

## Not Implemented Yet

- API server.
- Web UI API integration.
- MCP endpoint.
- Provider network integrations and asset download orchestration.

Do not add cross-layer dependencies to `msm-domain`.
