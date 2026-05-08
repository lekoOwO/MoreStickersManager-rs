use msm_domain::{Sticker, StickerPack};
use msm_exporters::{
    TelegramExportOptions, TelegramExportPlanner, TelegramReconcileMode,
    TelegramReconcileOperation, TelegramRemoteSet, TelegramRemoteSticker, TelegramStickerSetType,
    TelegramTargetConfig, TelegramTargetError,
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

#[test]
fn reconcile_without_remote_set_creates_the_desired_set() {
    let export_plan =
        TelegramExportPlanner::plan_pack(&sample_pack(51), default_options()).unwrap();
    let reconcile_plan = TelegramExportPlanner::plan_reconciliation(
        export_plan.clone(),
        None,
        TelegramReconcileMode::CreateOnly,
    )
    .unwrap();

    assert_eq!(reconcile_plan.sticker_set_name, "sample_by_msm_bot");
    assert_eq!(
        reconcile_plan.operations,
        vec![TelegramReconcileOperation::CreateSet {
            initial_stickers: export_plan.initial_stickers,
            append_stickers: export_plan.append_stickers,
        }]
    );
}

#[test]
fn create_only_reconciliation_rejects_existing_remote_set() {
    let export_plan = TelegramExportPlanner::plan_pack(&sample_pack(1), default_options()).unwrap();
    let error = TelegramExportPlanner::plan_reconciliation(
        export_plan,
        Some(remote_set(vec![])),
        TelegramReconcileMode::CreateOnly,
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
fn append_missing_reconciliation_keeps_remote_extras() {
    let export_plan = TelegramExportPlanner::plan_pack(&sample_pack(2), default_options()).unwrap();
    let existing = export_plan.initial_stickers[0].clone();
    let missing = export_plan.initial_stickers[1].clone();
    let reconcile_plan = TelegramExportPlanner::plan_reconciliation(
        export_plan,
        Some(remote_set(vec![
            remote_sticker(&existing, "tg_existing"),
            TelegramRemoteSticker {
                sticker_id: "remote-only".into(),
                telegram_file_id: "tg_remote_only".into(),
                target_profile_key: "telegram.sticker.static.v1".into(),
                emoji_list: vec!["😀".into()],
                keywords: vec!["Remote only".into()],
            },
        ])),
        TelegramReconcileMode::AppendMissing,
    )
    .unwrap();

    assert_eq!(
        reconcile_plan.operations,
        vec![
            TelegramReconcileOperation::KeepSticker {
                sticker_id: existing.sticker_id,
                telegram_file_id: "tg_existing".into(),
            },
            TelegramReconcileOperation::AddSticker { sticker: missing },
        ]
    );
}

#[test]
fn mirror_reconciliation_updates_adds_and_deletes_remote_drift() {
    let export_plan = TelegramExportPlanner::plan_pack(&sample_pack(2), default_options()).unwrap();
    let changed = export_plan.initial_stickers[0].clone();
    let missing = export_plan.initial_stickers[1].clone();
    let reconcile_plan = TelegramExportPlanner::plan_reconciliation(
        export_plan,
        Some(TelegramRemoteSet {
            sticker_set_name: "sample_by_msm_bot".into(),
            title: "Old title".into(),
            stickers: vec![
                TelegramRemoteSticker {
                    sticker_id: changed.sticker_id.clone(),
                    telegram_file_id: "tg_changed".into(),
                    target_profile_key: "telegram.sticker.video.v1".into(),
                    emoji_list: vec!["😺".into()],
                    keywords: vec!["old".into()],
                },
                TelegramRemoteSticker {
                    sticker_id: "remote-only".into(),
                    telegram_file_id: "tg_remote_only".into(),
                    target_profile_key: "telegram.sticker.static.v1".into(),
                    emoji_list: vec!["😀".into()],
                    keywords: vec!["Remote only".into()],
                },
            ],
        }),
        TelegramReconcileMode::Mirror,
    )
    .unwrap();

    assert_eq!(
        reconcile_plan.operations,
        vec![
            TelegramReconcileOperation::SetTitle {
                title: "Sample".into(),
            },
            TelegramReconcileOperation::ReplaceSticker {
                old_telegram_file_id: "tg_changed".into(),
                sticker: changed,
            },
            TelegramReconcileOperation::AddSticker { sticker: missing },
            TelegramReconcileOperation::DeleteSticker {
                sticker_id: "remote-only".into(),
                telegram_file_id: "tg_remote_only".into(),
            },
        ]
    );
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

fn remote_set(stickers: Vec<TelegramRemoteSticker>) -> TelegramRemoteSet {
    TelegramRemoteSet {
        sticker_set_name: "sample_by_msm_bot".into(),
        title: "Sample".into(),
        stickers,
    }
}

fn remote_sticker(
    sticker: &msm_exporters::PlannedTelegramSticker,
    telegram_file_id: &str,
) -> TelegramRemoteSticker {
    TelegramRemoteSticker {
        sticker_id: sticker.sticker_id.clone(),
        telegram_file_id: telegram_file_id.into(),
        target_profile_key: sticker.target_profile_key.clone(),
        emoji_list: sticker.emoji_list.clone(),
        keywords: sticker.keywords.clone(),
    }
}
