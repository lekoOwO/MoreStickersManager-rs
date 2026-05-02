# MoreStickers Compatibility

MSM exports `.stickerpack` JSON compatible with Equicord moreStickers and MoreStickersConverter.

## Sticker

Required fields:

- `id`
- `image`
- `title`
- `stickerPackId`

Optional fields:

- `filename`
- `isAnimated`

## Sticker Pack

Required fields:

- `id`
- `title`
- `logo`
- `stickers`

Optional fields:

- `author`

## Provider ID Conventions

- Telegram pack: `MoreStickers:Telegram:Pack:{sticker_set_name}`
- Telegram sticker: `MoreStickers:Telegram:Sticker:{sticker_set_name}:{file_unique_id}`
- LINE sticker pack: `MoreStickers:Line:Pack:{id}`
- LINE sticker: `MoreStickers:Line:Sticker:{pack_id}:{sticker_id}`
- LINE emoji pack: `MoreStickers:Line:Emoji-Pack:{id}`
- LINE emoji: `MoreStickers:Line-Emoji:{pack_id}:{emoji_id}`

## Asset URL Rule

If the system public asset URL is configured, exported sticker images use that base URL. Otherwise they use the MSM public app URL.
