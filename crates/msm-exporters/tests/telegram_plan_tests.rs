use msm_domain::{Sticker, StickerPack};
use msm_exporters::{
    TelegramExportOptions, TelegramExportPlanner, TelegramStickerSetType, TelegramTargetConfig,
    TelegramTargetError,
};
use teloxide::types::{InputFile, StickerFormat};

#[test]
fn sticker_set_name_appends_bot_suffix_and_sanitizes_slug() {
    let plan = TelegramExportPlanner::plan_pack(
        &sample_pack(1),
        TelegramExportOptions {
            target: TelegramTargetConfig {
                bot_username: "MSM_Bot".into(),
                owner_user_id: 42,
            },
            set_name_slug: "My Pack!!".into(),
            set_title: "My Pack".into(),
            set_type: TelegramStickerSetType::Regular,
            default_emoji: "😀".into(),
            existing_sticker_set_names: Vec::new(),
        },
    )
    .unwrap();

    assert_eq!(plan.sticker_set_name, "my_pack_by_msm_bot");
}

#[test]
fn sticker_set_name_preserves_existing_bot_suffix() {
    let plan = TelegramExportPlanner::plan_pack(
        &sample_pack(1),
        TelegramExportOptions {
            target: TelegramTargetConfig {
                bot_username: "msm_bot".into(),
                owner_user_id: 42,
            },
            set_name_slug: "already_by_msm_bot".into(),
            set_title: "Already".into(),
            set_type: TelegramStickerSetType::Regular,
            default_emoji: "😀".into(),
            existing_sticker_set_names: Vec::new(),
        },
    )
    .unwrap();

    assert_eq!(plan.sticker_set_name, "already_by_msm_bot");
}

#[test]
fn sticker_set_name_rejects_empty_bot_username() {
    let error = TelegramExportPlanner::plan_pack(
        &sample_pack(1),
        TelegramExportOptions {
            target: TelegramTargetConfig {
                bot_username: "!!!".into(),
                owner_user_id: 42,
            },
            ..default_options()
        },
    )
    .unwrap_err();

    assert_eq!(error, TelegramTargetError::InvalidBotUsername);
}

#[test]
fn sticker_set_name_truncates_stem_to_telegram_limit() {
    let plan = TelegramExportPlanner::plan_pack(
        &sample_pack(1),
        TelegramExportOptions {
            set_name_slug: "a".repeat(80),
            ..default_options()
        },
    )
    .unwrap();

    assert_eq!(plan.sticker_set_name.len(), 64);
    assert!(plan.sticker_set_name.ends_with("_by_msm_bot"));
}

#[test]
fn initial_batch_contains_at_most_fifty_stickers() {
    let plan = TelegramExportPlanner::plan_pack(&sample_pack(51), default_options()).unwrap();

    assert_eq!(plan.initial_stickers.len(), 50);
    assert_eq!(plan.append_stickers.len(), 1);
}

#[test]
fn regular_sets_reject_more_than_telegram_regular_limit() {
    let error = TelegramExportPlanner::plan_pack(&sample_pack(121), default_options()).unwrap_err();

    assert_eq!(
        error,
        TelegramTargetError::TooManyStickers {
            set_type: TelegramStickerSetType::Regular,
            count: 121,
            max: 120,
        }
    );
}

#[test]
fn custom_emoji_sets_allow_two_hundred_items() {
    let plan = TelegramExportPlanner::plan_pack(
        &sample_pack(200),
        TelegramExportOptions {
            set_type: TelegramStickerSetType::CustomEmoji,
            ..default_options()
        },
    )
    .unwrap();

    assert_eq!(plan.initial_stickers.len(), 50);
    assert_eq!(plan.append_stickers.len(), 150);
}

#[test]
fn empty_default_emoji_is_rejected() {
    let error = TelegramExportPlanner::plan_pack(
        &sample_pack(1),
        TelegramExportOptions {
            default_emoji: " ".into(),
            ..default_options()
        },
    )
    .unwrap_err();

    assert_eq!(error, TelegramTargetError::InvalidEmojiList);
}

#[test]
fn create_only_exports_reject_existing_set_names() {
    let error = TelegramExportPlanner::plan_pack(
        &sample_pack(1),
        TelegramExportOptions {
            existing_sticker_set_names: vec!["sample_by_msm_bot".into()],
            ..default_options()
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        TelegramTargetError::TargetSetAlreadyExists {
            sticker_set_name: "sample_by_msm_bot".into(),
        }
    );
}

#[test]
fn mixed_static_and_animated_sources_plan_static_and_video_formats() {
    let pack = StickerPack {
        stickers: vec![sample_sticker(0, false), sample_sticker(1, true)],
        ..sample_pack(0)
    };
    let plan = TelegramExportPlanner::plan_pack(&pack, default_options()).unwrap();

    assert_eq!(plan.initial_stickers[0].format, StickerFormat::Static);
    assert_eq!(plan.initial_stickers[1].format, StickerFormat::Video);
    assert_eq!(
        plan.initial_stickers[1].target_profile_key,
        "telegram.sticker.video.v1"
    );
}

#[test]
fn planned_sticker_converts_to_teloxide_input_sticker() {
    let plan = TelegramExportPlanner::plan_pack(&sample_pack(1), default_options()).unwrap();

    let input =
        plan.initial_stickers[0].to_input_sticker(InputFile::file_id("telegram-file-id".into()));

    assert_eq!(input.format, StickerFormat::Static);
    assert_eq!(input.emoji_list, vec!["😀"]);
    assert_eq!(input.keywords, vec!["Sample 0"]);
}

fn default_options() -> TelegramExportOptions {
    TelegramExportOptions {
        target: TelegramTargetConfig {
            bot_username: "msm_bot".into(),
            owner_user_id: 42,
        },
        set_name_slug: "sample".into(),
        set_title: "Sample".into(),
        set_type: TelegramStickerSetType::Regular,
        default_emoji: "😀".into(),
        existing_sticker_set_names: Vec::new(),
    }
}

fn sample_pack(count: usize) -> StickerPack {
    let logo = sample_sticker(0, false);
    StickerPack {
        id: "MoreStickers:Telegram:Pack:sample".to_owned(),
        title: "Sample".to_owned(),
        author: None,
        logo,
        stickers: (0..count)
            .map(|index| sample_sticker(index, false))
            .collect(),
    }
}

fn sample_sticker(index: usize, animated: bool) -> Sticker {
    Sticker {
        id: format!("MoreStickers:Telegram:Sticker:sample:file_{index}"),
        image: format!("https://msm.example/assets/packs/sample/file_{index}.webp"),
        title: format!("Sample {index}"),
        sticker_pack_id: "MoreStickers:Telegram:Pack:sample".to_owned(),
        filename: Some(format!("file_{index}.webp")),
        is_animated: Some(animated),
    }
}
