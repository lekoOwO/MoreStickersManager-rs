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
