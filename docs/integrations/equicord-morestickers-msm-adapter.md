# Equicord moreStickers MSM Adapter

This document defines the upstream Discord plugin work needed for Equicord
`src/equicordplugins/moreStickers` to consume MSM-managed sticker packs and
subscription groups.

Reference plugin source:

- https://github.com/Equicord/Equicord/tree/main/src/equicordplugins/moreStickers

Prepared patch artifact:

- `docs/integrations/equicord-morestickers-msm-adapter-patch.md`
- `docs/integrations/patches/equicord-morestickers-msm-adapter.patch`

## Current Compatibility

MSM already emits the same static `.stickerpack` shape consumed by
moreStickers:

- `StickerPack`: `id`, `title`, `author`, `logo`, `stickers`
- `Sticker`: `id`, `image`, `title`, `stickerPackId`, `filename`,
  `isAnimated`

MSM also emits the dynamic pack-set shape already represented in the plugin's
`DynamicPackSetMeta` and `DynamicStickerPackMeta` types:

- `GET /api/public/packs/{pack_id}/subscription`
- `GET /api/public/packs/{pack_id}/stickerpack`
- `GET /api/public/subscriptions/{subscription_group_id}`

Protected MSM subscriptions include `authHeaders.Authorization = "Bearer ..."`
when read through a subscription access token.

## MSM-Side Requirements

MSM must allow the Discord renderer origins used by Equicord to fetch
subscription payloads and assets directly:

```env
MSM_CORS_ALLOWED_ORIGINS=https://discord.com,https://canary.discord.com,https://ptb.discord.com
```

Do not route MSM subscription traffic through a public CORS proxy. Subscription
tokens and PATs are bearer credentials and must only be sent to the configured
MSM origin.

For private packs, the plugin must use the subscription `authHeaders` for both
pack refresh requests and asset fetches. Plain `<img src>` is not sufficient for
private MSM assets because browser image tags cannot attach authorization
headers.

## Plugin Changes

Add an MSM subscription tab to the moreStickers settings modal.

Inputs:

- MSM subscription URL.
- Optional subscription token or PAT.
- Optional display label.

Behavior:

- Fetch the subscription URL directly with configured headers.
- Validate the returned object as `DynamicPackSetMeta`.
- Persist the subscription metadata in `MoreStickers:DynamicPackSetMetas`.
- Fetch each `DynamicStickerPackMeta.dynamic.refreshUrl`.
- Save each fetched sticker pack locally and retain its dynamic metadata.
- Show sync status, last sync time, and the last error in the settings UI.

Refresh behavior:

- Refresh dynamic pack sets on plugin startup.
- Provide a manual "Sync now" button per subscription.
- Remove local packs that disappear from the remote pack set.
- Update existing packs when their dynamic `version` changes, and allow manual
  refresh even when the version is unchanged.

Asset behavior:

- For public MSM assets, direct image URLs can continue to be used.
- For protected MSM assets, load images through a small header-aware blob cache:
  fetch the asset with the pack/subscription auth headers, create an object URL,
  use that URL for picker thumbnails and upload conversion, and revoke stale
  object URLs when packs are removed or refreshed.
- `sendSticker` should fetch protected MSM assets directly with their auth
  headers rather than through the current global CORS proxy.

Security behavior:

- Never send MSM `Authorization` headers to the LINE CORS proxy or any third
  party.
- Redact subscription tokens and PATs from logs and toast messages.
- Only attach stored MSM auth headers to URLs whose origin matches the original
  subscription origin or an explicitly trusted MSM asset origin.

## Suggested Module Split

- `msm.ts`: MSM URL detection, direct fetch, dynamic payload validation,
  subscription add/sync/remove helpers.
- `assetCache.ts`: header-aware asset blob cache for protected MSM assets.
- `stickers.ts`: dynamic pack persistence fixes and pack refresh orchestration.
- `components/misc.tsx`: MSM subscription settings tab and sync status controls.
- `upload.ts`: protected asset fetch path for `sendSticker`.

## Acceptance Criteria

- A public MSM pack subscription URL can be added from the plugin UI and appears
  in the sticker picker without importing a file.
- A public MSM subscription-group URL can be added and keeps multiple packs in
  sync.
- A protected MSM pack or group subscription works with a subscription token and
  does not leak the token to third-party CORS proxies.
- Manual sync adds new packs, updates changed packs, and removes packs no longer
  present in the subscription group.
- Sticker thumbnail rendering and sending work for both public and protected
  MSM-hosted assets.
- Existing LINE URL/HTML import and local `.stickerpack` file import continue to
  work.
