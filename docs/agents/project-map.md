# Project Map

## Implemented in P0/P1

- `crates/msm-domain`: MoreStickers-compatible domain models and pure helpers.
- `crates/msm-storage`: SQL storage primitives, local asset storage, and portable export/import.
- `docs/status`: current state and development log.
- `docs/dev`: human developer references.
- `docs/agents`: progressive disclosure handoff docs.

## Not Implemented Yet

- API server.
- Database storage.
- Web UI.
- CLI.
- MCP endpoint.
- Provider network integrations.

Do not add cross-layer dependencies to `msm-domain`.
