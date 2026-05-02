use msm_domain::{
    line_emoji_id, line_emoji_pack_id, line_sticker_id, line_sticker_pack_id, resolve_asset_url,
    telegram_pack_id, telegram_sticker_id, AssetUrlConfig, AssetUrlInput,
};

#[test]
fn telegram_ids_match_more_stickers_converter_conventions() {
    assert_eq!(
        telegram_pack_id("pack_name").unwrap(),
        "MoreStickers:Telegram:Pack:pack_name"
    );
    assert_eq!(
        telegram_sticker_id("pack_name", "file_unique_id").unwrap(),
        "MoreStickers:Telegram:Sticker:pack_name:file_unique_id"
    );
}

#[test]
fn line_sticker_ids_match_equicord_conventions() {
    assert_eq!(
        line_sticker_pack_id("12345").unwrap(),
        "MoreStickers:Line:Pack:12345"
    );
    assert_eq!(
        line_sticker_id("12345", "67890").unwrap(),
        "MoreStickers:Line:Sticker:12345:67890"
    );
}

#[test]
fn line_emoji_ids_match_equicord_conventions() {
    assert_eq!(
        line_emoji_pack_id("abcde").unwrap(),
        "MoreStickers:Line:Emoji-Pack:abcde"
    );
    assert_eq!(
        line_emoji_id("abcde", "fghij").unwrap(),
        "MoreStickers:Line-Emoji:abcde:fghij"
    );
}

#[test]
fn provider_ids_reject_empty_components() {
    assert!(telegram_pack_id("").is_err());
    assert!(telegram_sticker_id("pack", "").is_err());
    assert!(line_sticker_pack_id("").is_err());
    assert!(line_sticker_id("", "sticker").is_err());
    assert!(line_emoji_pack_id("").is_err());
    assert!(line_emoji_id("pack", "").is_err());
}

#[test]
fn asset_url_uses_app_public_url_when_cdn_is_absent() {
    let config = AssetUrlConfig::new("https://msm.example").unwrap();
    let input = AssetUrlInput {
        pack_public_id: "pack_name",
        filename: "file_unique_id.webp",
    };

    assert_eq!(
        resolve_asset_url(&config, &input).unwrap(),
        "https://msm.example/assets/packs/pack_name/file_unique_id.webp"
    );
}

#[test]
fn asset_url_uses_public_asset_url_when_configured() {
    let config = AssetUrlConfig::new("https://msm.example")
        .unwrap()
        .with_public_asset_url("https://cdn.example/msm")
        .unwrap();
    let input = AssetUrlInput {
        pack_public_id: "pack_name",
        filename: "file_unique_id.webp",
    };

    assert_eq!(
        resolve_asset_url(&config, &input).unwrap(),
        "https://cdn.example/msm/assets/packs/pack_name/file_unique_id.webp"
    );
}
