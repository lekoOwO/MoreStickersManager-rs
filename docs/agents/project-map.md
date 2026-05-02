# Project Map

## Implemented in P0/P1

- `crates/msm-domain`: MoreStickers-compatible domain models and pure helpers.
- `crates/msm-domain/src/authz.rs`: pure authorization policy primitives and evaluators.
- `crates/msm-storage`: SQL storage primitives, local asset storage, and portable export/import.
- `crates/msm-api`: Axum API routes and utoipa OpenAPI generation.
- `crates/msm-cli`: HTTP CLI client for MSM API operations.
- `crates/msm-providers`: provider registry plus Telegram and LINE fixture normalizers.
- `docs/status`: current state and development log.
- `docs/dev`: human developer references.
- `docs/agents`: progressive disclosure handoff docs.

## Not Implemented Yet

- API server.
- Web UI.
- MCP endpoint.
- Provider network integrations and asset download orchestration.

Do not add cross-layer dependencies to `msm-domain`.
