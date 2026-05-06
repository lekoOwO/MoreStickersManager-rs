# Provider Normalization

P6 adds a provider interface for converting provider-specific JSON into
`MoreStickers`-compatible sticker packs.

## Implemented Providers

- `telegram`: normalizes a Telegram sticker set fixture into a pack with
  `MoreStickers:Telegram:Pack:{stickerSetName}` and
  `MoreStickers:Telegram:Sticker:{stickerSetName}:{fileUniqueId}` IDs.
- `line-stickers`: normalizes a LINE sticker fixture into
  `MoreStickers:Line:Pack:{packId}` and
  `MoreStickers:Line:Sticker:{packId}:{stickerId}` IDs.
- `line-emojis`: normalizes a LINE emoji fixture into
  `MoreStickers:Line:Emoji-Pack:{packId}` and
  `MoreStickers:Line-Emoji:{packId}:{emojiId}` IDs.

## Planned Providers

Signal, WhatsApp, Kakao, Band, OGQ, and Viber are registered as planned
providers only. They must not appear as implemented capabilities until they can
normalize payloads and preserve MoreStickers-compatible export output.

## Current Scope

The provider crate accepts already-fetched JSON. It does not yet fetch remote
provider APIs, download assets, store assets, or import normalized packs into
the database. Those tasks should compose the provider trait with storage and API
services in later phases.

## Provider Versus Export Target

Providers are input-side normalizers. Export targets are output-side publishers
or serializers. Keep these boundaries separate even when the same external
service appears on both sides.

- Telegram as a provider means importing or normalizing an existing Telegram
  sticker set into an MSM pack.
- Telegram as an export target means converting MSM assets and using a Telegram
  bot to create or update a Telegram sticker set.
- MoreStickers is currently an export format and compatibility contract, not a
  remote provider.

P24 plans a target-neutral export pipeline inspired by moe-sticker-bot. The
pipeline should put format conversion in `msm-media`, target orchestration in
`msm-exporters`, and Telegram Bot API calls in `msm-telegram`.
