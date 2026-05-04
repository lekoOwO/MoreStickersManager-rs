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
