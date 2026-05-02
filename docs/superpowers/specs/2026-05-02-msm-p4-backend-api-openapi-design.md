# MSM P4 Backend API and OpenAPI Design

Date: 2026-05-02
Phase: P4

## Purpose

P4 introduces the HTTP API surface and OpenAPI contract. The first API milestone should expose health, OpenAPI JSON, sticker pack import/export, asset read, and storage-backed pack listing with the P1-P3 domain boundaries intact.

## Scope

In scope for the initial P4 slice:
- `msm-api` crate.
- Axum router factory.
- utoipa OpenAPI document generation.
- Health endpoint.
- OpenAPI JSON endpoint.
- Storage-backed pack listing endpoint.
- Storage-backed single pack export endpoint.
- Minimal JSON import endpoint for `.stickerpack` payloads.
- Asset read endpoint backed by `LocalAssetStore`.
- API error response model.
- Integration tests using SQLite and local asset temp dirs.

Deferred within P4 or later:
- Session login.
- OIDC.
- Full RBAC middleware.
- Multipart upload.
- Provider jobs.
- Admin settings endpoints.
- Web frontend embedding.

## Architecture

```text
crates/
  msm-api/
    src/
      dto.rs
      error.rs
      lib.rs
      openapi.rs
      routes/
        health.rs
        packs.rs
        assets.rs
      state.rs
```

`msm-api` depends on:
- `msm-domain`
- `msm-storage`
- `axum`
- `utoipa`
- `tower`
- `serde`
- `serde_json`
- `tokio`

The API crate owns HTTP concerns only. It should not implement business rules that belong in `msm-domain` or storage behavior that belongs in `msm-storage`.

## Initial Endpoints

### `GET /healthz`

Returns:

```json
{
  "status": "ok"
}
```

### `GET /openapi.json`

Returns generated OpenAPI JSON.

### `GET /api/v1/packs`

Temporary development endpoint for P4:
- accepts query `userId`;
- returns sticker packs owned by that user.

Later auth middleware will replace `userId` query usage with authenticated principal context.

### `GET /api/v1/packs/{pack_id}/stickerpack`

Returns a P1-compatible `StickerPack` JSON payload from storage.

### `POST /api/v1/packs/import`

Temporary development endpoint:

```json
{
  "tenantId": "tenant_1",
  "ownerUserId": "user_1",
  "packId": "pack_1",
  "visibility": "private",
  "pack": {}
}
```

Stores the pack through `msm-storage`.

### `GET /assets/packs/{pack_public_id}/{filename}`

Reads from `LocalAssetStore`.

Authorization is not enforced in initial P4 because auth middleware comes later. The route exists to prove asset serving behavior and response types.

## Error Model

All API errors return:

```json
{
  "error": {
    "code": "not_found",
    "message": "Pack not found"
  }
}
```

Initial codes:
- `bad_request`
- `not_found`
- `conflict`
- `internal`

## Testing

Integration tests should:
- call `/healthz`;
- call `/openapi.json` and verify endpoint paths are present;
- import a pack through HTTP and export it back;
- list packs for a user;
- write an asset through `LocalAssetStore` in setup and fetch it through HTTP;
- verify missing pack returns 404 JSON error.

## CI

Existing workspace commands remain sufficient:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Design Decisions

1. P4 starts with a thin API slice instead of the full product API.
2. Temporary query/body fields stand in for future authenticated principal context.
3. OpenAPI is generated from the API crate immediately so clients can align early.
4. Authorization is not duplicated in handlers; P4 will call P3 policy functions only after principal extraction exists.
