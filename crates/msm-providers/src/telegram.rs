use msm_domain::{telegram_pack_id, telegram_sticker_id, Author, Sticker, StickerPack};
use serde::Deserialize;

use crate::{
    registry::{ProviderStatus, TELEGRAM_CAPABILITIES},
    ProviderError, ProviderMetadata, ProviderResult, StickerProvider,
};

/// Telegram fixture normalizer.
#[derive(Clone, Copy, Debug, Default)]
pub struct TelegramProvider;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelegramStickerSet {
    name: String,
    title: String,
    #[serde(default)]
    author: Option<String>,
    stickers: Vec<TelegramSticker>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelegramSticker {
    file_unique_id: String,
    #[serde(default)]
    emoji: Option<String>,
    #[serde(default = "default_extension")]
    extension: String,
    #[serde(default)]
    is_animated: bool,
}

impl StickerProvider for TelegramProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "telegram",
            display_name: "Telegram",
            capabilities: TELEGRAM_CAPABILITIES,
            status: ProviderStatus::Implemented,
        }
    }

    fn normalize_pack_json(
        &self,
        input: &str,
        public_base_url: &str,
    ) -> ProviderResult<StickerPack> {
        let fixture: TelegramStickerSet = serde_json::from_str(input)?;
        normalize_telegram_sticker_set(&fixture, public_base_url)
    }
}

fn normalize_telegram_sticker_set(
    sticker_set: &TelegramStickerSet,
    public_base_url: &str,
) -> ProviderResult<StickerPack> {
    if sticker_set.stickers.is_empty() {
        return Err(ProviderError::InvalidPayload(
            "Telegram sticker set must contain at least one sticker".to_owned(),
        ));
    }

    let pack_id = telegram_pack_id(&sticker_set.name)?;
    let base_url = normalized_base_url(public_base_url)?;
    let stickers = sticker_set
        .stickers
        .iter()
        .map(|sticker| normalize_telegram_sticker(sticker, &sticker_set.name, &pack_id, &base_url))
        .collect::<ProviderResult<Vec<_>>>()?;

    Ok(StickerPack {
        id: pack_id,
        title: sticker_set.title.clone(),
        author: sticker_set.author.as_ref().map(|name| Author {
            name: name.clone(),
            url: None,
        }),
        logo: stickers[0].clone(),
        stickers,
    })
}

fn normalize_telegram_sticker(
    sticker: &TelegramSticker,
    sticker_set_name: &str,
    pack_id: &str,
    base_url: &str,
) -> ProviderResult<Sticker> {
    let extension = sticker.extension.trim_start_matches('.');
    if extension.is_empty() {
        return Err(ProviderError::InvalidPayload(
            "Telegram sticker extension must not be empty".to_owned(),
        ));
    }

    let filename = format!("{}.{}", sticker.file_unique_id, extension);
    Ok(Sticker {
        id: telegram_sticker_id(sticker_set_name, &sticker.file_unique_id)?,
        image: format!("{base_url}/sticker/telegram/{sticker_set_name}/{filename}"),
        title: sticker
            .emoji
            .clone()
            .unwrap_or_else(|| sticker.file_unique_id.clone()),
        sticker_pack_id: pack_id.to_owned(),
        filename: Some(filename),
        is_animated: Some(sticker.is_animated),
    })
}

fn default_extension() -> String {
    "webp".to_owned()
}

fn normalized_base_url(public_base_url: &str) -> ProviderResult<String> {
    let trimmed = public_base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(ProviderError::InvalidPayload(
            "public base URL must not be empty".to_owned(),
        ));
    }

    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use crate::{telegram::TelegramProvider, StickerProvider};

    #[test]
    fn normalizes_telegram_fixture_to_more_stickers_pack() {
        let input = r#"{
            "name": "cat_pack",
            "title": "Cat Pack",
            "author": "Leko",
            "stickers": [
                {
                    "fileUniqueId": "cat_1",
                    "emoji": "cat",
                    "extension": "webp",
                    "isAnimated": false
                },
                {
                    "fileUniqueId": "cat_2",
                    "emoji": "dance",
                    "extension": ".tgs",
                    "isAnimated": true
                }
            ]
        }"#;

        let pack = TelegramProvider
            .normalize_pack_json(input, "https://msm.example.test/")
            .expect("fixture should normalize");

        assert_eq!(pack.id, "MoreStickers:Telegram:Pack:cat_pack");
        assert_eq!(pack.author.expect("author").name, "Leko");
        assert_eq!(pack.logo.id, "MoreStickers:Telegram:Sticker:cat_pack:cat_1");
        assert_eq!(pack.stickers[0].filename.as_deref(), Some("cat_1.webp"));
        assert_eq!(
            pack.stickers[0].image,
            "https://msm.example.test/sticker/telegram/cat_pack/cat_1.webp"
        );
        assert_eq!(pack.stickers[1].filename.as_deref(), Some("cat_2.tgs"));
        assert_eq!(pack.stickers[1].is_animated, Some(true));
    }

    #[test]
    fn rejects_empty_telegram_sticker_sets() {
        let input = r#"{
            "name": "empty_pack",
            "title": "Empty Pack",
            "stickers": []
        }"#;

        let error = TelegramProvider
            .normalize_pack_json(input, "https://msm.example.test")
            .expect_err("empty packs should be invalid");

        assert!(error.to_string().contains("at least one sticker"));
    }
}
