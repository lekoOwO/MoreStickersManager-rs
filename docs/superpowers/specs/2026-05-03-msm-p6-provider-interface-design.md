# MSM P6 Provider Interface Design

Date: 2026-05-03
Phase: P6

## Purpose

P6 introduces provider abstractions and fixture-driven normalization for external sticker sources. Telegram and LINE become provider implementations without coupling provider logic into storage, API, CLI, or MCP layers.

## Scope

In scope for the first P6 slice:
- `msm-providers` crate.
- Provider capability metadata and registry.
- Provider trait for normalizing provider-native pack payloads into `msm-domain::StickerPack`.
- Telegram normalization from fixture JSON shaped after Telegram sticker set/file metadata.
- LINE sticker normalization from fixture JSON shaped after Equicord-parsed LINE sticker packs.
- LINE emoji normalization from fixture JSON shaped after Equicord-parsed LINE emoji packs.
- Placeholder registry entries for future Signal, WhatsApp, Kakao, Band, OGQ, and Viber.
- Tests proving provider outputs match P1 ID conventions.

Out of scope:
- Telegram Bot API network calls.
- LINE HTML parsing/network calls.
- Downloading image assets.
- Background jobs and retries.
- API/provider job endpoints.

## Architecture

```text
crates/
  msm-providers/
    src/
      error.rs
      lib.rs
      registry.rs
      telegram.rs
      line.rs
```

`msm-providers` depends on `msm-domain` only for normalized output and ID helpers. It may use serde for fixture/native payload decoding.

## Provider Trait

```rust
pub trait StickerProvider {
    fn metadata(&self) -> ProviderMetadata;
    fn normalize_pack_json(&self, input: &str, public_base_url: &str) -> ProviderResult<StickerPack>;
}
```

The `public_base_url` is used by providers that need to build MSM-hosted asset URLs, especially Telegram. LINE fixture normalization may keep upstream image URLs because P6 does not download assets yet.

## Provider Metadata

Metadata fields:
- `id`
- `display_name`
- `capabilities`
- `status`

Capabilities:
- `NormalizeFixture`
- `FetchRemote`
- `DownloadAssets`
- `AnimatedStickers`
- `EmojiPacks`

Status:
- `Implemented`
- `Planned`

## Telegram Normalization

Input fixture:

```json
{
  "name": "sample_pack",
  "title": "Sample",
  "stickers": [
    {
      "fileUniqueId": "file_1",
      "emoji": "smile",
      "extension": "webp",
      "isAnimated": false
    }
  ]
}
```

Output uses:
- pack ID: `MoreStickers:Telegram:Pack:{name}`
- sticker ID: `MoreStickers:Telegram:Sticker:{name}:{fileUniqueId}`
- image: `{public_base_url}/sticker/telegram/{name}/{fileUniqueId}.{extension}`
- filename: `{fileUniqueId}.{extension}`

This preserves MoreStickersConverter's export behavior.

## LINE Normalization

P6 uses fixture JSON after parsing, not raw HTML:

```json
{
  "id": "12345",
  "title": "Sample LINE Stickers",
  "author": { "name": "LINE Author", "url": "https://store.line.me/..." },
  "mainImage": {},
  "stickers": []
}
```

Output matches Equicord ID conventions:
- sticker pack: `MoreStickers:Line:Pack:{id}`
- sticker: `MoreStickers:Line:Sticker:{pack_id}:{sticker_id}`
- emoji pack: `MoreStickers:Line:Emoji-Pack:{id}`
- emoji: `MoreStickers:Line-Emoji:{pack_id}:{emoji_id}`

## Testing

Tests:
- registry includes implemented Telegram, LINE stickers, LINE emojis;
- registry includes planned future providers;
- Telegram fixture normalizes to P1-compatible IDs and asset URL;
- LINE sticker fixture normalizes to Equicord-compatible IDs;
- LINE emoji fixture normalizes to Equicord-compatible IDs;
- invalid provider payloads return typed errors.

## Documentation Updates

Update:
- `docs/dev/architecture.md`
- `docs/agents/project-map.md`
- `docs/agents/testing.md`
- `docs/status/current.md`
- `docs/status/checkpoints.md`

## Design Decisions

1. P6 normalizes fixture/native payloads only; remote fetches come later.
2. Providers depend on domain helpers, not storage or API.
3. Future providers are represented in registry metadata before implementation.
4. Telegram builds MSM-hosted URLs; LINE keeps upstream URLs until asset downloading exists.
