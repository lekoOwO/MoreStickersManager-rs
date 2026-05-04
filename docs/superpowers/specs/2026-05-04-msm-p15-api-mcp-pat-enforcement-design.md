# MSM P15 API/MCP PAT Enforcement Design

## Scope

P15 wires P12/P13 Personal Access Tokens into protected API and MCP operations.
It is a first enforcement slice: Bearer PATs are verified against storage, then
stable scope keys gate pack operations.

## Goals

- Add API Bearer PAT extraction and verification.
- Return `401 Unauthorized` when a protected operation has no valid PAT.
- Return `403 Forbidden` when a valid PAT lacks the required scope or tries to
  act as a different user.
- Protect API operations:
  - `GET /api/v1/packs?userId=...` requires `pack.read` and matching PAT user.
  - `POST /api/v1/packs/import` requires `import.run` and matching owner user.
  - `GET /api/v1/packs/{pack_id}/stickerpack` requires `pack.read`.
- Protect MCP `tools/call` operations:
  - `msm.list_sticker_packs` requires `pack.read` and matching PAT user.
  - `msm.export_sticker_pack` requires `pack.read`.
  - `msm.import_sticker_pack` requires `import.run` and matching owner user.
- Keep `healthz`, `openapi.json`, MCP `initialize`, MCP `ping`, and MCP
  `tools/list` public.
- Preserve existing CLI commands by adding a global `--pat` option and
  `MSM_PAT` fallback in the reqwest client.

## Non-Goals

- No OIDC, SSO, cookie sessions, or local password login.
- No PAT tenant binding in storage.
- No Web UI token persistence.
- No asset privacy enforcement because the current asset route cannot yet map
  an asset URL back to a pack visibility record.
- No enforcement on PAT lifecycle endpoints; these remain a bootstrap/admin
  placeholder until the login/admin model exists.

## Authorization Model

Protected handlers call a small API auth helper:

```text
Authorization: Bearer msm_pat_<token_id>_<secret>
```

The helper verifies the token through `StorageRepository::verify_personal_access_token`.
When verification succeeds, handlers check the returned record scopes.

For user-scoped operations, the request user must match `PersonalAccessTokenRecord.user_id`.
This prevents a PAT for one local user from listing or importing as another
local user in the current pre-tenant-binding model.
