# P21 Pack CRUD Foundation Plan

Date: 2026-05-04

## Goal

Add the first complete sticker-pack management slice beyond import/list/export:
owned pack rename, visibility update, and delete across API, CLI, and MCP.

## Scope

- Storage repository methods for owned pack metadata update and owned pack deletion.
- API routes:
  - `PATCH /api/v1/packs/{pack_id}`
  - `DELETE /api/v1/packs/{pack_id}`
- CLI commands:
  - `msm packs rename --pack-id <id> --title <title> --visibility <public|private>`
  - `msm packs delete --pack-id <id>`
- MCP tools:
  - `msm.update_sticker_pack`
  - `msm.delete_sticker_pack`
- Tests for storage, API, CLI, and MCP behavior.

## Permission Model

- Rename/visibility update requires `pack.update`.
- Delete requires `pack.delete`.
- Operations are owner-scoped through the PAT user ID.
- Non-owner operations intentionally return not found in the storage/API slice to avoid exposing pack ownership details.

## Compatibility Notes

- Rename updates both the indexed `sticker_packs.title` column and the embedded MoreStickers-compatible `sticker_pack_json.title`.
- Sticker IDs, sticker contents, provider compatibility IDs, and export JSON shape remain unchanged.
- Delete relies on existing foreign-key cascade behavior for sticker rows, folder links, tag links, and subscription-group links.

## Verification Targets

- `cargo test -p msm-storage --locked`
- `cargo test -p msm-api --locked`
- `cargo test -p msm-cli --locked`
- `cargo test -p msm-mcp --locked`
- Full workspace and Web verification before commit.

## Follow-Up

- Add Web UI pack rename/delete controls.
- Add explicit create-from-upload flow beyond import.
- Add folder/tag/subscription-group management.
- Add tenant admin override behavior after detailed RBAC APIs exist.
