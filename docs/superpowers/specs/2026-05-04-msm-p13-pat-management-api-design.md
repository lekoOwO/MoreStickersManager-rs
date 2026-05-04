# MSM P13 PAT Management API Design

## Scope

P13 exposes the P12 PAT storage lifecycle through HTTP API and OpenAPI. It does
not yet require Bearer PAT authentication on existing API or MCP routes.

## Goals

- Add PAT DTOs to `msm-api`.
- Add PAT routes:
  - `POST /api/v1/pats`
  - `GET /api/v1/pats?userId=...`
  - `DELETE /api/v1/pats/{token_id}`
- Return the raw PAT only from create.
- Hide token hashes from API responses.
- Add route tests.
- Update OpenAPI.

## Non-Goals

- No route authentication enforcement in P13.
- No Web UI PAT management in P13.
- No CLI PAT commands in P13.

## API Contracts

Create request:

```json
{
  "id": "cli1",
  "userId": "user_1",
  "name": "CLI",
  "scopes": ["pack.read", "asset.read"],
  "expiresAt": null
}
```

Create response includes `token`. List responses never include `token` or
`tokenHash`.
