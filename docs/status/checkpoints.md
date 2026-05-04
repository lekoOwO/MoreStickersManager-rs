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
