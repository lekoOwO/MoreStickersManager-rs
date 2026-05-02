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
