# Testing Guide

## P1 Domain Tests

Run:

```powershell
cargo test -p msm-domain
```

These tests prove:

- `.stickerpack` fixtures parse.
- serialized JSON uses camelCase field names.
- optional fields are skipped when absent.
- provider ID helpers match upstream conventions.
- asset URL resolution chooses CDN URL before app URL.

## Workspace Tests

Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## GitHub Actions Coverage

Workflows:

- `.github/workflows/ci.yml` runs Rust fmt/clippy/tests, Web typecheck/tests/build, and cross-platform `msm-app` build checks.
- `.github/workflows/docker.yml` builds and publishes GHCR images for `main` and `v*` tags.
- `.github/workflows/prerelease.yml` publishes a moving `prerelease` release from `main`.
- `.github/workflows/release.yml` publishes binary artifacts for `v*` tags.

Local equivalents:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run web:typecheck
npm run web:test
npm run web:build
cargo build --locked -p msm-app
```

Docker image verification requires Docker:

```powershell
docker build -t morestickersmanager-rs .
```

## P2 Storage Tests

Run:

```powershell
cargo test -p msm-storage
```

These tests prove:

- database URL parsing rejects unsupported schemes;
- local asset keys reject traversal attempts;
- local asset bytes can be written, read, and deleted;
- SQLite migrations create the P2 schema;
- repository operations can create tenant, user, pack, sticker, and subscription records;
- portable user data can be exported from one SQLite database and imported into another.

## P3 Authorization Tests

Run:

```powershell
cargo test -p msm-domain --test authorization
```

These tests prove:

- tenant admins can manage in-tenant resources but cannot cross tenants;
- owners can manage owned private packs;
- tenant members can only read member-access private packs;
- anonymous access is limited to public resources;
- PAT access requires matching scopes;
- pack secrets and subscription secrets only grant their narrow resource access;
- public subscription groups do not globally expose private pack assets.

## P4 API Tests

Run:

```powershell
cargo test -p msm-api
```

These tests prove:

- `/healthz` returns `{"status":"ok"}`;
- `/openapi.json` exposes generated route metadata;
- pack import, list, and `.stickerpack` export work through HTTP;
- local asset bytes can be read through `/assets/packs/{pack_public_id}/{filename}`.

## P5 CLI Tests

Run:

```powershell
cargo test -p msm-cli
```

These tests prove:

- CLI arguments parse for health, list, import, and export commands;
- reqwest client endpoint URL construction is stable;
- command execution works against a fake client;
- import reads `.stickerpack` JSON from disk;
- export can print JSON to stdout.

## P6 Provider Tests

Run:

```powershell
cargo test -p msm-providers
```

These tests prove:

- Telegram fixture payloads normalize to upstream-compatible pack and sticker IDs;
- Telegram self-hosted image URLs follow the MoreStickersConverter URL pattern;
- LINE sticker fixture payloads normalize to upstream-compatible LINE sticker IDs;
- LINE emoji fixture payloads normalize to upstream-compatible LINE emoji IDs;
- empty provider packs are rejected before producing invalid `.stickerpack` output;
- implemented and planned providers are visible in the provider registry.

## P7 Web UI Tests

Run:

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

These tests prove:

- Vue and TypeScript compile for the `apps/web` workspace;
- theme preference defaults, persistence, and DOM class application work;
- locale preference defaults and message lookup work for Traditional Chinese and English;
- the dashboard renders mock pack totals, providers, and visibility labels;
- Vite can produce `apps/web/dist` for later Rust binary embedding.

## P8 Web API Client Tests

Run:

```powershell
npm run web:test
```

These tests prove:

- the Web pack client falls back to mock data when no API base URL is configured;
- `/api/v1/packs?userId=...` URL construction encodes user IDs correctly;
- P4 `StickerPackRecord` JSON maps into dashboard summaries;
- provider inference handles Telegram, LINE stickers, and LINE emojis;
- the dashboard still renders through the client boundary without network access in tests.

## P9 Service Binary Tests

Run:

```powershell
cargo test -p msm-app
cargo clippy -p msm-app --all-targets -- -D warnings
```

These tests prove:

- service configuration uses safe defaults;
- environment overrides are parsed correctly;
- invalid bind addresses fail before startup;
- the composition crate builds with API, storage, and static Web UI serving.

## P10 Embedded Web Asset Tests

Run:

```powershell
cargo test -p msm-app
cargo clippy -p msm-app --all-targets -- -D warnings
```

These tests prove:

- unsafe Web fallback paths reject traversal attempts;
- embedded `index.html` exists even when the real frontend dist was not built;
- the service crate still compiles with embedded asset support.

## P11 MCP Endpoint Tests

Run:

```powershell
cargo test -p msm-mcp
cargo clippy -p msm-mcp --all-targets -- -D warnings
cargo test -p msm-app
```

These tests prove:

- `/mcp` handles `initialize` and declares tool capability;
- `tools/list` exposes the pack list/export/import tools;
- `tools/call` can list, export, and import sticker packs through storage;
- unknown JSON-RPC methods return `-32601`;
- the service binary still mounts MCP alongside API and Web UI fallback.

## P12 PAT Foundation Tests

Run:

```powershell
cargo test -p msm-domain permission_keys
cargo test -p msm-storage personal_access_tokens
cargo clippy -p msm-storage --all-targets -- -D warnings
```

These tests prove:

- every domain permission has a stable scope key and can roundtrip from that key;
- unknown scope keys are rejected;
- PAT creation returns the raw token once and stores only a hash;
- valid tokens verify to active records;
- invalid, revoked, and expired tokens fail verification.

## P13 PAT Management API Tests

Run:

```powershell
cargo test -p msm-api
cargo clippy -p msm-api --all-targets -- -D warnings
```

These tests prove:

- OpenAPI includes `/api/v1/pats`;
- PAT create returns a raw token once;
- PAT list responses omit raw tokens and token hashes;
- PAT revoke invalidates storage verification;
- unknown PAT scopes return `400 Bad Request`.

## P14 CLI PAT Command Tests

Run:

```powershell
cargo test -p msm-cli pats
cargo clippy -p msm-cli --all-targets -- -D warnings
```

These tests prove:

- CLI arguments parse for PAT create, list, and revoke commands;
- repeated `--scope` arguments are preserved in create requests;
- command execution calls the PAT client methods through the `MsmClient` trait;
- human output prints the raw token only for create;
- human list output omits token secrets and token hashes.

## P15 API/MCP PAT Enforcement Tests

Run:

```powershell
cargo test -p msm-api pat_enforcement
cargo test -p msm-mcp pat_enforcement
cargo test -p msm-cli pat
cargo clippy -p msm-api -p msm-mcp -p msm-cli --all-targets -- -D warnings
```

These tests prove:

- protected pack API routes reject missing PATs with `401 Unauthorized`;
- protected pack API routes reject missing scopes or user mismatch with `403 Forbidden`;
- valid `pack.read` PATs can list/export packs;
- valid `import.run` PATs can import packs for their own user;
- MCP `tools/call` returns tool errors for missing PATs, missing scopes, and user mismatch;
- CLI can pass PATs through global `--pat` and configured reqwest Bearer auth.

## P16 Web PAT Management Tests

Run:

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

These tests prove:

- pack API client calls can include Bearer PAT headers;
- Web PAT client create/list/revoke methods call the P13 endpoints;
- PAT create responses expose the raw token returned by the API;
- the dashboard still renders pack metrics and provider labels;
- Traditional Chinese and English i18n messages remain available.

## P17 Workflow Tests

Run:

```powershell
git diff --check -- .github/workflows Dockerfile .dockerignore
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run web:typecheck
npm run web:test
npm run web:build
cargo build --locked -p msm-app
```

These tests prove:

- workflow and Docker files have no whitespace errors;
- CI commands match the local Rust and Web verification baseline;
- `msm-app` can build after Web dist generation for embedded Web assets.

## P18 Local Auth Bootstrap Tests

Run:

```powershell
cargo test -p msm-storage local_credentials
cargo test -p msm-api local_auth
cargo clippy -p msm-storage -p msm-api --all-targets --locked -- -D warnings
```

These tests prove:

- local password credentials are stored as Argon2 PHC hashes;
- correct passwords verify to active user records;
- wrong passwords fail verification;
- local register returns `201 Created`;
- local login returns a raw PAT once;
- wrong login passwords return `401 Unauthorized`;
- OpenAPI includes the local auth login path.

## P19 Web Local Login Tests

Run:

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

These tests prove:

- Web local auth client calls local register and login endpoints;
- Web local login client returns the raw PAT from login responses;
- the Web UI still typechecks, tests, and builds after adding the local login panel.

## P20 Admin Bootstrap Tests

Run:

```powershell
cargo test -p msm-api local_auth
cargo test -p msm-storage local_credentials
cargo clippy -p msm-api -p msm-storage --all-targets --locked -- -D warnings
```

These tests prove:

- local registration accepts optional tenant bootstrap fields;
- the first tenant registration can create an admin membership;
- local login still returns a PAT after bootstrap registration.

## P21 Pack CRUD Foundation Tests

Run:

```powershell
cargo test -p msm-storage pack
cargo test -p msm-api pack
cargo test -p msm-cli packs
cargo test -p msm-mcp pack
```

These tests prove:

- owned pack metadata updates synchronize indexed titles and embedded `.stickerpack` titles;
- owned pack deletion removes the pack through storage;
- API routes enforce `pack.update` and `pack.delete`;
- CLI and MCP expose rename/update and delete operations.

## P22 Web Pack CRUD Tests

Run:

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

These tests prove:

- the Web pack API client can update and delete packs;
- the dashboard exposes per-pack title, visibility, save, and delete controls;
- injected-client tests cover rename and delete behavior without network access.

## P23 Web Pack Import Tests

Run:

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

These tests prove:

- the Web pack API client can call protected pack import;
- the dashboard can import pasted `.stickerpack` JSON with internal pack ID and visibility;
- API-client and injected-client tests cover successful import behavior.

## P24 Telegram Export Pipeline Planning Checks

Run:

```powershell
git diff --check
```

These checks prove:

- the P24 design and implementation plan have no whitespace errors;
- current status, handoff docs, and user docs point to the same planned next phase.

## P25 Media And Export Persistence Tests

Run:

```powershell
cargo test -p msm-media --locked
cargo clippy -p msm-media --all-targets --locked -- -D warnings
cargo test -p msm-storage --locked
cargo clippy -p msm-storage --all-targets --locked -- -D warnings
```

These tests prove:

- Telegram static/video/thumbnail media profiles and command plans are stable;
- media command plans return executable path plus argument vectors without shell interpolation;
- export target, job, and event storage roundtrips work;
- export job success/failure payload updates are persisted;
- prepared media cache records upsert by source asset hash and profile key;
- Telegram publication records can be upserted, found by ID, found by target/set, and listed by pack.

## Exporter Registry Tests

Run:

```powershell
cargo test -p msm-exporters --locked
cargo clippy -p msm-exporters --all-targets --locked -- -D warnings
```

These tests prove:

- duplicate export target kinds are rejected;
- target lookup and capability listing are stable;
- capability metadata serializes to camelCase JSON for API and Web surfaces.

## MoreStickers Export Target Tests

Run:

```powershell
cargo test -p msm-exporters --locked
cargo clippy -p msm-exporters --all-targets --locked -- -D warnings
```

These tests prove:

- `MoreStickersExportTarget` emits bytes matching `StickerPack::to_pretty_json()`;
- target capabilities describe local serialization without credentials or media conversion;
- target planning returns the expected `.stickerpack` serialization step.

## Telegram Bot Framework Boundary Tests

Run:

```powershell
cargo test -p msm-telegram --locked
cargo clippy -p msm-telegram --all-targets --locked -- -D warnings
```

These tests prove:

- Telegram bot tokens are redacted in debug/display output;
- empty bot tokens are rejected;
- Bot API URLs are validated and applied to the constructed `teloxide::Bot`;
- MSM uses teloxide as the Telegram bot framework boundary instead of a custom HTTP client.

## Telegram Export Planner Tests

Run:

```powershell
cargo test -p msm-exporters --locked
cargo clippy -p msm-exporters --all-targets --locked -- -D warnings
```

These tests prove:

- Telegram sticker set names append and preserve `_by_<bot_username>` suffixes;
- invalid bot usernames, empty emoji defaults, excessive regular set sizes, and existing create-only set names return typed errors;
- sticker set names are constrained to Telegram's 64-character limit;
- create batches contain at most 50 stickers and append batches carry the remainder;
- regular sets cap at 120 stickers and custom emoji sets allow 200;
- static and animated MSM stickers map to static/video teloxide sticker formats;
- planned stickers can be converted to teloxide `InputSticker` values using prepared media files.

## Export API Tests

Run:

```powershell
cargo test -p msm-api --locked
cargo clippy -p msm-api -p msm-storage -p msm-domain --all-targets --locked -- -D warnings
```

These tests prove:

- OpenAPI exposes export target and export job paths;
- `export.read`, `export.run`, and `export.target.manage` PAT scopes are enforced;
- Telegram target config responses redact token-like fields;
- queued export job creation rejects PAT users that do not own the source pack;
- Telegram publication history routes require `export.read` and source pack ownership;
- export job status and ordered event reads work through protected routes.

## Export Worker Foundation Tests

Run:

```powershell
cargo test -p msm-app --locked
cargo clippy -p msm-app --all-targets --locked -- -D warnings
```

These tests prove:

- worker config reads ffmpeg, ffprobe, and concurrency settings from environment-like input;
- invalid worker concurrency is rejected;
- the worker can pick and run a queued MoreStickers export job without remote calls;
- the worker can plan a Telegram dry-run export job without network calls;
- worker execution records running/succeeded states and ordered job events;
- injected prepared media executors can write prepared media cache records for planned Telegram media;
- process-backed prepared media execution uses shell-free conversion command plans and can be tested with an injected command runner instead of installed ffmpeg;
- injected Telegram publication executors can publish `dryRun:false` jobs without network access in tests;
- dry-run Telegram jobs do not call the publication executor;
- successful non-dry-run Telegram jobs persist durable publication records;
- publisher failures mark export jobs as failed and persist an error summary;
- startup export target bootstrap config parses, rejects invalid JSON, and idempotently creates/updates configured targets.
- retryable worker failures requeue jobs with backoff until the configured attempt budget is exhausted;
- queued jobs with future `next_attempt_at` values are skipped by worker polling.

## Export CLI Tests

Run:

```powershell
cargo test -p msm-cli --locked
cargo clippy -p msm-cli --all-targets --locked -- -D warnings
```

These tests prove:

- CLI arguments parse for export target creation and export job creation;
- `msm exports kinds` calls the target kind API client boundary;
- target creation preserves JSON config and enabled/disabled state;
- export job creation supports JSON output;
- export job event reads render ordered event information.
- Telegram publication list/get commands call the publication history API and format sticker set links.

## Export MCP Tests

Run:

```powershell
cargo test -p msm-mcp --locked
cargo clippy -p msm-mcp --all-targets --locked -- -D warnings
```

These tests prove:

- `tools/list` exposes pack tools plus export target/job tools;
- export target kind listing requires `export.read`;
- export target creation requires `export.target.manage` and redacts token-like response config fields;
- export job creation requires `export.run`, validates source pack ownership, and stores queued job requests;
- export job event reads require `export.read` and return ordered event metadata.
- Telegram publication list/get tools require `export.read`, validate source pack ownership, and return persisted sticker set links.

## Web Export Workflow Tests

Run:

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

These tests prove:

- the Web export API client builds target/job URLs and forwards Bearer PATs;
- the Web export API client builds Telegram publication history URLs and forwards Bearer PATs;
- export result-link extraction recognizes completed Telegram publication URLs;
- export target CRUD client methods call the expected API endpoints;
- the export target panel validates Telegram bot token shape before target creation;
- redacted export target config values remain redacted in the UI;
- the pack export wizard queues jobs, renders job events, and surfaces conflict errors;
- the pack export wizard and job timeline render completed Telegram sticker set URLs;
- the pack export wizard loads persisted Telegram publication history for the selected pack;
- the Web app still typechecks and builds after wiring export workflow components into the dashboard.

## Telegram Publish Boundary Tests

Run:

```powershell
cargo test -p msm-telegram --locked
cargo clippy -p msm-telegram --all-targets --locked -- -D warnings
```

These tests prove:

- Telegram bot token/config redaction still works;
- the publish boundary creates a sticker set before appending stickers;
- append calls preserve planned sticker order;
- the teloxide adapter can be built from validated bot config and implements the same mockable sticker set API trait;
- publication tests use recording fakes and do not call Telegram.
