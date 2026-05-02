use msm_domain::{
    line_emoji_id, line_emoji_pack_id, line_sticker_id, line_sticker_pack_id, Author, Sticker,
    StickerPack,
};
use serde::Deserialize;

use crate::{
    registry::{ProviderStatus, LINE_EMOJI_CAPABILITIES, LINE_STICKER_CAPABILITIES},
    ProviderError, ProviderMetadata, ProviderResult, StickerProvider,
};

/// LINE sticker fixture normalizer.
#[derive(Clone, Copy, Debug, Default)]
pub struct LineStickerProvider;

/// LINE emoji fixture normalizer.
#[derive(Clone, Copy, Debug, Default)]
pub struct LineEmojiProvider;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinePack {
    id: String,
    title: String,
    #[serde(default)]
    author: Option<LineAuthor>,
    #[serde(default)]
    main_image: Option<String>,
    stickers: Vec<LineSticker>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LineAuthor {
    name: String,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LineSticker {
    id: String,
    #[serde(default)]
    title: Option<String>,
    static_url: String,
    #[serde(default)]
    animation_url: Option<String>,
}

impl StickerProvider for LineStickerProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "line-stickers",
            display_name: "LINE Stickers",
            capabilities: LINE_STICKER_CAPABILITIES,
            status: ProviderStatus::Implemented,
        }
    }

    fn normalize_pack_json(
        &self,
        input: &str,
        _public_base_url: &str,
    ) -> ProviderResult<StickerPack> {
        let pack: LinePack = serde_json::from_str(input)?;
        normalize_line_pack(&pack, LinePackKind::Sticker)
    }
}

impl StickerProvider for LineEmojiProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "line-emojis",
            display_name: "LINE Emojis",
            capabilities: LINE_EMOJI_CAPABILITIES,
            status: ProviderStatus::Implemented,
        }
    }

    fn normalize_pack_json(
        &self,
        input: &str,
        _public_base_url: &str,
    ) -> ProviderResult<StickerPack> {
        let pack: LinePack = serde_json::from_str(input)?;
        normalize_line_pack(&pack, LinePackKind::Emoji)
    }
}

#[derive(Clone, Copy)]
enum LinePackKind {
    Sticker,
    Emoji,
}

fn normalize_line_pack(pack: &LinePack, kind: LinePackKind) -> ProviderResult<StickerPack> {
    if pack.stickers.is_empty() {
        return Err(ProviderError::InvalidPayload(
            "LINE pack must contain at least one sticker".to_owned(),
        ));
    }

    let pack_id = match kind {
        LinePackKind::Sticker => line_sticker_pack_id(&pack.id)?,
        LinePackKind::Emoji => line_emoji_pack_id(&pack.id)?,
    };
    let stickers = pack
        .stickers
        .iter()
        .map(|sticker| normalize_line_sticker(sticker, &pack.id, &pack_id, kind))
        .collect::<ProviderResult<Vec<_>>>()?;
    let mut logo = stickers[0].clone();
    if let Some(main_image) = pack.main_image.as_deref().filter(|value| !value.is_empty()) {
        main_image.clone_into(&mut logo.image);
    }

    Ok(StickerPack {
        id: pack_id,
        title: pack.title.clone(),
        author: pack.author.as_ref().map(|author| Author {
            name: author.name.clone(),
            url: author.url.clone(),
        }),
        logo,
        stickers,
    })
}

fn normalize_line_sticker(
    sticker: &LineSticker,
    source_pack_id: &str,
    pack_id: &str,
    kind: LinePackKind,
) -> ProviderResult<Sticker> {
    let animation_url = sticker
        .animation_url
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    let image = animation_url.unwrap_or(&sticker.static_url);
    if image.trim().is_empty() {
        return Err(ProviderError::InvalidPayload(
            "LINE sticker image URL must not be empty".to_owned(),
        ));
    }

    let id = match kind {
        LinePackKind::Sticker => line_sticker_id(source_pack_id, &sticker.id)?,
        LinePackKind::Emoji => line_emoji_id(source_pack_id, &sticker.id)?,
    };

    Ok(Sticker {
        id,
        image: image.to_owned(),
        title: sticker.title.clone().unwrap_or_else(|| sticker.id.clone()),
        sticker_pack_id: pack_id.to_owned(),
        filename: None,
        is_animated: Some(animation_url.is_some()),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        line::{LineEmojiProvider, LineStickerProvider},
        StickerProvider,
    };

    #[test]
    fn normalizes_line_sticker_fixture_to_more_stickers_pack() {
        let input = r#"{
            "id": "line_cats",
            "title": "LINE Cats",
            "author": { "name": "LINE", "url": "https://line.example.test" },
            "mainImage": "https://cdn.example.test/main.png",
            "stickers": [
                {
                    "id": "001",
                    "title": "Wave",
                    "staticUrl": "https://cdn.example.test/001.png"
                },
                {
                    "id": "002",
                    "staticUrl": "https://cdn.example.test/002.png",
                    "animationUrl": "https://cdn.example.test/002.apng"
                }
            ]
        }"#;

        let pack = LineStickerProvider
            .normalize_pack_json(input, "ignored")
            .expect("fixture should normalize");

        assert_eq!(pack.id, "MoreStickers:Line:Pack:line_cats");
        assert_eq!(pack.logo.image, "https://cdn.example.test/main.png");
        assert_eq!(
            pack.stickers[0].id,
            "MoreStickers:Line:Sticker:line_cats:001"
        );
        assert_eq!(pack.stickers[0].title, "Wave");
        assert_eq!(pack.stickers[0].is_animated, Some(false));
        assert_eq!(pack.stickers[1].image, "https://cdn.example.test/002.apng");
        assert_eq!(pack.stickers[1].is_animated, Some(true));
    }

    #[test]
    fn normalizes_line_emoji_fixture_to_more_stickers_pack() {
        let input = r#"{
            "id": "line_emoji_cats",
            "title": "LINE Emoji Cats",
            "stickers": [
                {
                    "id": "e001",
                    "staticUrl": "https://cdn.example.test/e001.png"
                }
            ]
        }"#;

        let pack = LineEmojiProvider
            .normalize_pack_json(input, "ignored")
            .expect("fixture should normalize");

        assert_eq!(pack.id, "MoreStickers:Line:Emoji-Pack:line_emoji_cats");
        assert_eq!(
            pack.stickers[0].id,
            "MoreStickers:Line-Emoji:line_emoji_cats:e001"
        );
        assert_eq!(pack.logo.id, "MoreStickers:Line-Emoji:line_emoji_cats:e001");
    }

    #[test]
    fn rejects_empty_line_packs() {
        let input = r#"{
            "id": "empty",
            "title": "Empty",
            "stickers": []
        }"#;

        let error = LineStickerProvider
            .normalize_pack_json(input, "ignored")
            .expect_err("empty packs should be invalid");

        assert!(error.to_string().contains("at least one sticker"));
    }
}
