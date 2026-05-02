# MSM P8 Web API Client Design

## Scope

P8 connects the P7 dashboard boundary to the P4 pack listing API shape. It keeps
mock data available for tests and for local static preview when no API base URL
is configured.

## Goals

- Add a typed frontend API client for `/api/v1/packs?userId=...`.
- Map current P4 `StickerPackRecord` JSON into P7 `StickerPackSummary`.
- Keep provider and visibility inference deterministic.
- Add frontend tests for URL construction, response mapping, and mock fallback.
- Keep the dashboard component consuming the same `StickerPackSummary` model.

## Non-Goals

- No authentication headers, PAT storage, OIDC, or tenant switching in P8.
- No write operations, imports, exports, or CRUD from the Web UI in P8.
- No Rust API route changes unless the current JSON shape blocks the client.

## Design

`apps/web/src/lib/sticker-packs.ts` becomes the typed model and fixture module.
`apps/web/src/lib/api-client.ts` owns HTTP concerns:

- `createPackClient(options)` returns `listStickerPacks()`.
- If `baseUrl` is absent, the client returns mock packs.
- If `baseUrl` is present, it requests `/api/v1/packs?userId=<userId>`.
- It maps `StickerPackRecord` fields into UI summaries.

Provider inference uses `sourceProvider` first when present. If absent, it reads
the compatibility ID prefix:

- `MoreStickers:Telegram:*` -> `Telegram`
- `MoreStickers:Line:Emoji-Pack:*` -> `LINE Emojis`
- `MoreStickers:Line:*` -> `LINE Stickers`

Visibility maps `Public`/`public` to `public`, `Private`/`private` to
`private`; member visibility remains a UI/mock-only state until backend RBAC
models expose it.

## Testing

Vitest tests cover mock fallback, encoded `userId` query construction, and
mapping of API records into dashboard summaries.
