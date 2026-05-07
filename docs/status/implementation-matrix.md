# Implementation Matrix

Last updated: 2026-05-07.

This file is the quick truth source for what MSM can do today versus what is
only designed or planned.

## Current Capability Status

| Area | Status | Notes |
| --- | --- | --- |
| Repository hygiene | Implemented | `.gitignore`, workspace layout, documentation structure, CI baseline, release workflows, Dockerfile, and `.dockerignore` exist. |
| MoreStickers compatibility | Implemented | `msm-domain` preserves `.stickerpack` JSON shape and upstream-compatible IDs. |
| Domain authorization | Implemented | Pure RBAC/PAT policy primitives and pack/subscription access evaluators exist in `msm-domain::authz`. |
| Storage foundation | Implemented | SQLite storage primitives, migrations, local asset storage, pack/sticker records, PAT storage, local credentials, export target/job tables, prepared media cache records, Telegram publication records, and portable user export/import exist. |
| PostgreSQL support | Not implemented | SQLx is used, but current migrations/repositories are SQLite-focused. |
| API/OpenAPI | Implemented | Axum API has health, OpenAPI, asset read, pack import/list/export/update/delete, PAT lifecycle, local auth bootstrap, export target, and export job routes. |
| CLI | Implemented | `msm-cli` supports health, pack list/import/export/rename/delete, PAT create/list/revoke, export target kinds, export target list/create, export job create/get/events, and Bearer PAT forwarding. |
| MCP endpoint | Implemented | `/mcp` supports JSON-RPC initialize, ping, tools/list, pack list/export/import/update/delete tools, and export target/job tools. |
| MCP auth | Partially implemented | Pack and export tool calls enforce Bearer PAT scopes. SSE/session hardening is not implemented. |
| Provider normalization | Implemented | `msm-providers` normalizes already-fetched Telegram, LINE sticker, and LINE emoji fixtures into MoreStickers-compatible packs. |
| Provider network fetch | Not implemented | Remote provider API fetch, download, and asset internalization are planned future work. |
| Service binary | Implemented | `msm-app` composes storage migrations, API routes, local asset store, MCP route, Web static serving, and export worker foundation types. |
| Embedded Web dist | Implemented | `msm-app` embeds `apps/web/dist` when present and a placeholder when absent; runtime disk override remains available. |
| Web UI foundation | Implemented | Vue/Vite, Tailwind CSS v4, Shadcn Vue-style local primitives, RWD dashboard shell, theme toggle, and i18n exist. |
| Web API integration | Implemented | Web pack and export API clients can use API-backed pack/export operations with stored PAT; pack list still has mock fallback for local preview. |
| Web auth/PAT UI | Implemented | Local register/login panel and PAT create/list/revoke panel exist. |
| Web pack management | Implemented | Dashboard supports pack list, rename, visibility edit, delete, pasted `.stickerpack` import, export target setup, and export job queue/status views. |
| Local auth bootstrap | Implemented | Register/login APIs store Argon2 password credentials and login returns a PAT. Registration can bootstrap a tenant admin. |
| Multi-tenant model | Partially implemented | Tenant/user records and bootstrap admin path exist. Full tenant administration UI and complete RBAC management are not implemented. |
| PAT support | Implemented | Raw token is returned once, only the secret hash is stored, and protected pack API/MCP operations enforce scopes. |
| GitHub Actions | Implemented | CI, Docker publish, prerelease, and tag release workflows exist. Local Docker verification is blocked by missing Docker CLI. |

## Planned Work Not Implemented

| Area | Planned entrypoint | Notes |
| --- | --- | --- |
| Media conversion pipeline | P25 | Partially implemented: `msm-media` now has source media kinds, Telegram static/video/thumbnail profiles, prepared output specs, conversion plan selection, and shell-free ffmpeg command planning. Probing, converter execution, and cache persistence are not implemented. |
| Export target registry | P27 | Partially implemented: `msm-exporters` has target kind keys, capability metadata, request/plan types, target trait, duplicate-safe registry, a concrete `morestickers` target, and Telegram sticker set planning. Remote execution is not wired. |
| Telegram bot framework boundary | P28 | Implemented foundation: `msm-telegram` uses `teloxide`, redacts bot tokens, validates configurable Bot API URLs, builds `teloxide::Bot`, and exposes mockable sticker set create/append execution through teloxide requester methods. |
| Telegram sticker set export | P29-P32 | Partially implemented: planner normalizes Telegram set names, enforces size constraints, splits create/append batches, maps static/animated MSM stickers to Telegram media profiles, builds teloxide `InputSticker` data, exposes Web/API/CLI/MCP job surfaces, the app worker can publish when job options explicitly set `"dryRun": false`, and the Web export wizard/timeline surface completed sticker set URLs. Remote reconciliation is not implemented. |
| Export jobs | P29-P30 | Partially implemented: storage tables/repositories, protected API/OpenAPI routes, CLI commands, MCP tools, Web target/job views, prepared media cache writes, Telegram publication repository APIs, process-backed ffmpeg executor, target bootstrap config, optional app worker loop, Telegram dry-run jobs, and Telegram publication jobs exist. Retries and worker persistence into `telegram_publications` are not implemented. |
| Folder/tag management | Future phase | User-managed pack folders and tags are not implemented. |
| Subscription groups | Future phase | Pack/group subscription links and moreStickers auto-update integration are not implemented. |
| Fine-grained pack sharing UI | Future phase | Current visibility update exists; member access management and secret-based pack asset access are not wired. |
| Asset privacy enforcement | Future phase | Pack APIs enforce PAT scopes, but asset route privacy and private pack asset credentials are not complete. |
| OIDC/SSO | Future phase | Local auth bootstrap exists; OIDC/SSO provider configuration and login flow are not implemented. |
| System-wide CDN public asset URL | Future phase | Domain URL resolver supports CDN preference; admin configuration UI/API is not implemented. |
| Tenant admin console | Future phase | Bootstrap admin exists; user/role/tenant management surfaces are not implemented. |
| Full user migration UI/API | Future phase | Storage portability helpers exist; complete Web/API/CLI migration workflow is not implemented. |
| Future providers | Future phases | Signal, WhatsApp, Kakao, Band, OGQ, and Viber are registered as planned only. |
| Remote target sync/update/delete | P33+ | Initial Telegram plan is create-only; reconciliation policies are deferred. |

## Current Next Phase

Continue by wiring successful Telegram worker publications into the new `telegram_publications` repository APIs.
