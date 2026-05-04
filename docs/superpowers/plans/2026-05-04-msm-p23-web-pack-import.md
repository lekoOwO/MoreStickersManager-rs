# P23 Web Pack Import Plan

Date: 2026-05-04

## Goal

Expose MoreStickers `.stickerpack` import from the Web dashboard.

## Scope

- Extend the Web pack API client with `importStickerPack`.
- Add a dashboard import form for:
  - internal MSM pack ID
  - public/private visibility
  - pasted `.stickerpack` JSON
- Use configured tenant/user identity from props or environment defaults.
- Reload the dashboard pack list after successful import.
- Add API client and dashboard tests.

## UX Constraints

- Pasted JSON is acceptable for this bootstrap slice.
- File picker, drag-and-drop, import preview, validation summaries, and provider fetch flows are later phases.
- Mock mode remains read-only for mutating operations.

## Verification Targets

- `npm run web:test`
- `npm run web:typecheck`
- `npm run web:build`
- Full Rust/Web verification before commit.

## Follow-Up

- Add `.stickerpack` file picker and import preview.
- Add server-side import validation feedback in the UI.
- Add provider fetch UI after provider network integrations exist.
