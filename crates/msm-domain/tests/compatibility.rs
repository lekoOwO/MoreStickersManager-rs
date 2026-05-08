use msm_domain::{
    build_dynamic_subscription_payload, line_emoji_id, line_emoji_pack_id, line_sticker_id,
    line_sticker_pack_id, resolve_asset_url, subscription_bearer_headers, telegram_pack_id,
    telegram_sticker_id, AssetUrlConfig, AssetUrlInput, DynamicPackSetMeta, StickerPack,
    SubscriptionPackInput, SubscriptionPayloadInput,
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

#[test]
fn telegram_fixture_roundtrips() {
    let input = include_str!("fixtures/telegram_pack.stickerpack.json");
    let pack = StickerPack::from_json_str(input).unwrap();

    assert_eq!(pack.id, "MoreStickers:Telegram:Pack:sample_pack");
    assert_eq!(pack.logo.sticker_pack_id, pack.id);
    assert_eq!(pack.stickers.len(), 1);

    let output = pack.to_pretty_json().unwrap();
    assert!(output.contains("\"stickerPackId\""));
    assert!(output.contains("\"isAnimated\""));
}

#[test]
fn line_sticker_fixture_roundtrips() {
    let input = include_str!("fixtures/line_sticker_pack.stickerpack.json");
    let pack = StickerPack::from_json_str(input).unwrap();

    assert_eq!(pack.id, "MoreStickers:Line:Pack:12345");
    assert_eq!(pack.author.unwrap().name, "LINE Author");
    assert_eq!(
        pack.stickers[0].sticker_pack_id,
        "MoreStickers:Line:Pack:12345"
    );
}

#[test]
fn line_emoji_fixture_roundtrips() {
    let input = include_str!("fixtures/line_emoji_pack.stickerpack.json");
    let pack = StickerPack::from_json_str(input).unwrap();

    assert_eq!(pack.id, "MoreStickers:Line:Emoji-Pack:abcde");
    assert_eq!(pack.stickers[0].is_animated, Some(true));
}

#[test]
fn dynamic_pack_set_fixture_roundtrips() {
    let input = include_str!("fixtures/dynamic_pack_set.json");
    let pack_set = DynamicPackSetMeta::from_json_str(input).unwrap();

    assert_eq!(pack_set.id, "sub_sample");
    assert_eq!(pack_set.packs.len(), 1);
    assert_eq!(
        pack_set.packs[0].dynamic.refresh_url,
        "https://msm.example/api/public/packs/sample_pack/stickerpack"
    );

    let output = pack_set.to_pretty_json().unwrap();
    assert!(output.contains("\"refreshUrl\""));
    assert!(output.contains("\"authHeaders\""));
}

#[test]
fn public_subscription_payload_omits_auth_headers() {
    let payload = build_dynamic_subscription_payload(SubscriptionPayloadInput {
        id: "sub_public".to_owned(),
        version: Some("1".to_owned()),
        title: Some("Public Feed".to_owned()),
        author: None,
        refresh_url: "https://msm.example/api/public/subscriptions/sub_public".to_owned(),
        auth_headers: None,
        packs: vec![SubscriptionPackInput {
            pack: minimal_pack("public"),
            refresh_url: "https://msm.example/api/public/packs/pack_public/stickerpack".to_owned(),
        }],
    });

    assert_eq!(payload.id, "sub_public");
    assert!(payload.auth_headers.is_none());
    assert_eq!(payload.packs.len(), 1);
    assert!(payload.packs[0].dynamic.auth_headers.is_none());
    assert_eq!(
        payload.packs[0].dynamic.refresh_url,
        "https://msm.example/api/public/packs/pack_public/stickerpack"
    );
}

#[test]
fn protected_subscription_payload_reuses_bearer_auth_headers() {
    let auth_headers = subscription_bearer_headers("sub_secret_123");
    let payload = build_dynamic_subscription_payload(SubscriptionPayloadInput {
        id: "sub_private".to_owned(),
        version: Some("1".to_owned()),
        title: Some("Private Feed".to_owned()),
        author: None,
        refresh_url: "https://msm.example/api/public/subscriptions/sub_private".to_owned(),
        auth_headers: Some(auth_headers.clone()),
        packs: vec![SubscriptionPackInput {
            pack: minimal_pack("private"),
            refresh_url: "https://msm.example/api/public/packs/pack_private/stickerpack".to_owned(),
        }],
    });

    assert_eq!(
        payload
            .auth_headers
            .as_ref()
            .unwrap()
            .get("Authorization")
            .unwrap(),
        "Bearer sub_secret_123"
    );
    assert_eq!(payload.packs[0].dynamic.auth_headers, Some(auth_headers));
    assert_eq!(payload.packs[0].id, "MoreStickers:Telegram:Pack:private");
    assert_eq!(payload.packs[0].logo.sticker_pack_id, payload.packs[0].id);
}

#[test]
fn optional_fields_are_skipped_when_absent() {
    let pack = StickerPack {
        id: "MoreStickers:Telegram:Pack:minimal".to_owned(),
        title: "Minimal".to_owned(),
        author: None,
        logo: msm_domain::Sticker {
            id: "MoreStickers:Telegram:Sticker:minimal:file".to_owned(),
            image: "https://msm.example/assets/packs/minimal/file.webp".to_owned(),
            title: "file".to_owned(),
            sticker_pack_id: "MoreStickers:Telegram:Pack:minimal".to_owned(),
            filename: None,
            is_animated: None,
        },
        stickers: Vec::new(),
    };

    let output = pack.to_pretty_json().unwrap();
    assert!(!output.contains("\"author\""));
    assert!(!output.contains("\"filename\""));
    assert!(!output.contains("\"isAnimated\""));
}

fn minimal_pack(id_suffix: &str) -> StickerPack {
    let pack_id = format!("MoreStickers:Telegram:Pack:{id_suffix}");
    let sticker = msm_domain::Sticker {
        id: format!("MoreStickers:Telegram:Sticker:{id_suffix}:file"),
        image: format!("https://msm.example/assets/packs/{id_suffix}/file.webp"),
        title: "file".to_owned(),
        sticker_pack_id: pack_id.clone(),
        filename: Some("file.webp".to_owned()),
        is_animated: Some(false),
    };

    StickerPack {
        id: pack_id,
        title: format!("Pack {id_suffix}"),
        author: None,
        logo: sticker.clone(),
        stickers: vec![sticker],
    }
}
