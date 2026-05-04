# P22 Web Pack CRUD Controls Plan

Date: 2026-05-04

## Goal

Expose the P21 pack rename, visibility update, and delete operations from the Web dashboard.

## Scope

- Extend the Web pack API client with:
  - `updateStickerPack`
  - `deleteStickerPack`
- Keep mock mode read-only for local preview when no API base URL is configured.
- Add dashboard controls per pack:
  - title input
  - public/private visibility select
  - save button
  - delete button
- Reload pack data after successful save/delete.
- Add tests with an injected fake pack client.

## UX Constraints

- Preserve the existing responsive dashboard and local Shadcn Vue-style primitives.
- Keep controls visible and direct for this bootstrap slice.
- Defer confirmation dialogs, optimistic updates, route guards, and bulk actions to later phases.

## Verification Targets

- `npm run web:test`
- `npm run web:typecheck`
- `npm run web:build`
- Full workspace verification before commit.

## Follow-Up

- Add confirmation dialog for destructive delete.
- Add dedicated pack detail/edit route.
- Add create/import UI.
- Add folder/tag/subscription-group UI after backend endpoints exist.
