use msm_domain::{
    line_emoji_id, line_emoji_pack_id, line_sticker_id, line_sticker_pack_id, telegram_pack_id,
    telegram_sticker_id,
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
