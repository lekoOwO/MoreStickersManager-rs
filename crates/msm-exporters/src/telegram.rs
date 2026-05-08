use msm_domain::StickerPack;
use msm_media::{ConversionPlan, MediaKind};
use std::collections::HashSet;
use teloxide::types::{InputFile, InputSticker, StickerFormat};

/// Telegram target configuration that is independent from stored secrets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramTargetConfig {
    /// Telegram bot username without relying on a live `getMe` call.
    pub bot_username: String,
    /// Telegram user ID that owns the sticker set.
    pub owner_user_id: i64,
}

/// Options used to plan one Telegram sticker set export.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramExportOptions {
    /// Target bot/user configuration.
    pub target: TelegramTargetConfig,
    /// Desired sticker set name slug.
    pub set_name_slug: String,
    /// Sticker set title.
    pub set_title: String,
    /// Telegram sticker set type.
    pub set_type: TelegramStickerSetType,
    /// Default emoji assigned to each sticker until per-sticker emoji metadata exists.
    pub default_emoji: String,
    /// Existing Telegram sticker set names known before planning create-only exports.
    pub existing_sticker_set_names: Vec<String>,
}

/// Telegram sticker set type used by the planner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramStickerSetType {
    /// Regular sticker set.
    Regular,
    /// Custom emoji set.
    CustomEmoji,
}

impl TelegramStickerSetType {
    #[must_use]
    const fn max_stickers(self) -> usize {
        match self {
            Self::Regular => 120,
            Self::CustomEmoji => 200,
        }
    }
}

/// Planned Telegram export operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramExportPlan {
    /// Normalized Telegram sticker set name.
    pub sticker_set_name: String,
    /// Telegram sticker set title.
    pub title: String,
    /// Telegram user ID that owns the set.
    pub owner_user_id: i64,
    /// Telegram sticker set type.
    pub set_type: TelegramStickerSetType,
    /// Stickers used in `createNewStickerSet`.
    pub initial_stickers: Vec<PlannedTelegramSticker>,
    /// Stickers appended with `addStickerToSet`.
    pub append_stickers: Vec<PlannedTelegramSticker>,
}

/// Remote reconciliation mode for an already published Telegram sticker set.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TelegramReconcileMode {
    /// Only create a missing set. Existing sets are rejected.
    CreateOnly,
    /// Add missing MSM stickers while leaving remote-only stickers untouched.
    AppendMissing,
    /// Make the remote set match MSM by updating changed stickers and deleting extras.
    Mirror,
}

/// Known remote Telegram sticker set state.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramRemoteSet {
    /// Telegram sticker set name.
    pub sticker_set_name: String,
    /// Telegram sticker set title.
    pub title: String,
    /// Remote stickers mapped back to MSM sticker IDs when known.
    pub stickers: Vec<TelegramRemoteSticker>,
}

/// Known remote Telegram sticker metadata for reconciliation.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramRemoteSticker {
    /// MSM sticker compatibility ID.
    pub sticker_id: String,
    /// Telegram file ID needed by update/delete Bot API methods.
    pub telegram_file_id: String,
    /// MSM media profile key last used for this sticker.
    pub target_profile_key: String,
    /// Telegram emoji list.
    pub emoji_list: Vec<String>,
    /// Telegram search keywords.
    pub keywords: Vec<String>,
}

/// Planned remote mutation or no-op for one Telegram reconciliation run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TelegramReconcileOperation {
    /// Create the set, then append the overflow stickers.
    CreateSet {
        /// Stickers sent to `createNewStickerSet`.
        initial_stickers: Vec<PlannedTelegramSticker>,
        /// Stickers sent to `addStickerToSet`.
        append_stickers: Vec<PlannedTelegramSticker>,
    },
    /// Update the remote sticker set title.
    SetTitle {
        /// Desired title.
        title: String,
    },
    /// Keep an existing remote sticker unchanged.
    KeepSticker {
        /// MSM sticker compatibility ID.
        sticker_id: String,
        /// Telegram file ID.
        telegram_file_id: String,
    },
    /// Add a missing sticker to the remote set.
    AddSticker {
        /// Desired sticker.
        sticker: PlannedTelegramSticker,
    },
    /// Replace an existing remote sticker with the desired prepared media.
    ReplaceSticker {
        /// Telegram file ID currently in the set.
        old_telegram_file_id: String,
        /// Desired sticker.
        sticker: PlannedTelegramSticker,
    },
    /// Delete a remote-only sticker from the set.
    DeleteSticker {
        /// MSM sticker compatibility ID if previously known.
        sticker_id: String,
        /// Telegram file ID.
        telegram_file_id: String,
    },
}

/// Planned Telegram remote reconciliation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramReconcilePlan {
    /// Telegram sticker set name.
    pub sticker_set_name: String,
    /// Reconciliation mode used to build the operation list.
    pub mode: TelegramReconcileMode,
    /// Ordered remote operations.
    pub operations: Vec<TelegramReconcileOperation>,
}

/// Target-neutral planned sticker data that can be converted to teloxide input later.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedTelegramSticker {
    /// Source sticker compatibility ID.
    pub sticker_id: String,
    /// Target media profile key selected by `msm-media`.
    pub target_profile_key: String,
    /// Teloxide sticker format.
    pub format: StickerFormat,
    /// Telegram emoji list.
    pub emoji_list: Vec<String>,
    /// Search keywords.
    pub keywords: Vec<String>,
}

impl PlannedTelegramSticker {
    /// Converts this planned sticker to teloxide `InputSticker` using the prepared file.
    #[must_use]
    pub fn to_input_sticker(&self, sticker: InputFile) -> InputSticker {
        InputSticker {
            sticker,
            format: self.format.clone(),
            emoji_list: self.emoji_list.clone(),
            mask_position: None,
            keywords: self.keywords.clone(),
        }
    }
}

/// Telegram export planner.
#[derive(Clone, Debug, Default)]
pub struct TelegramExportPlanner;

impl TelegramExportPlanner {
    /// Plans Telegram sticker set creation without executing network requests.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the options or pack cannot satisfy Telegram constraints.
    pub fn plan_pack(
        pack: &StickerPack,
        options: TelegramExportOptions,
    ) -> Result<TelegramExportPlan, TelegramTargetError> {
        let sticker_count = pack.stickers.len();
        let max = options.set_type.max_stickers();
        if sticker_count > max {
            return Err(TelegramTargetError::TooManyStickers {
                set_type: options.set_type,
                count: sticker_count,
                max,
            });
        }
        if pack.stickers.is_empty() {
            return Err(TelegramTargetError::EmptyPack);
        }

        let emoji = normalized_default_emoji(&options.default_emoji)?;
        let sticker_set_name =
            normalize_set_name(&options.set_name_slug, &options.target.bot_username)?;
        if options
            .existing_sticker_set_names
            .iter()
            .any(|existing| existing == &sticker_set_name)
        {
            return Err(TelegramTargetError::TargetSetAlreadyExists { sticker_set_name });
        }

        let mut planned = Vec::with_capacity(pack.stickers.len());
        for sticker in &pack.stickers {
            let source_kind = if sticker.is_animated.unwrap_or(false) {
                MediaKind::AnimatedImage
            } else {
                MediaKind::StaticImage
            };
            let conversion_plan = ConversionPlan::for_telegram_regular_sticker(source_kind)
                .map_err(|error| TelegramTargetError::MediaPlan {
                    message: error.to_string(),
                })?;

            planned.push(PlannedTelegramSticker {
                sticker_id: sticker.id.clone(),
                target_profile_key: conversion_plan.profile().profile_key().to_owned(),
                format: sticker_format_for_profile(conversion_plan.profile().profile_key()),
                emoji_list: vec![emoji.clone()],
                keywords: keyword_for_sticker_title(&sticker.title),
            });
        }

        let append_stickers = planned.split_off(planned.len().min(50));
        Ok(TelegramExportPlan {
            sticker_set_name,
            title: options.set_title,
            owner_user_id: options.target.owner_user_id,
            set_type: options.set_type,
            initial_stickers: planned,
            append_stickers,
        })
    }

    /// Plans remote reconciliation for an already built Telegram export plan.
    ///
    /// # Errors
    ///
    /// Returns `TargetSetAlreadyExists` when `CreateOnly` sees an existing
    /// remote set.
    pub fn plan_reconciliation(
        export_plan: TelegramExportPlan,
        remote: Option<TelegramRemoteSet>,
        mode: TelegramReconcileMode,
    ) -> Result<TelegramReconcilePlan, TelegramTargetError> {
        let sticker_set_name = export_plan.sticker_set_name.clone();
        let Some(remote) = remote else {
            return Ok(TelegramReconcilePlan {
                sticker_set_name,
                mode,
                operations: vec![TelegramReconcileOperation::CreateSet {
                    initial_stickers: export_plan.initial_stickers,
                    append_stickers: export_plan.append_stickers,
                }],
            });
        };

        if mode == TelegramReconcileMode::CreateOnly {
            return Err(TelegramTargetError::TargetSetAlreadyExists { sticker_set_name });
        }

        let TelegramRemoteSet {
            title: remote_title,
            stickers: remote_stickers,
            ..
        } = remote;
        let desired_stickers = export_plan
            .initial_stickers
            .into_iter()
            .chain(export_plan.append_stickers)
            .collect::<Vec<_>>();
        let mut operations = Vec::new();
        let mut seen_remote_sticker_ids = HashSet::new();

        if mode == TelegramReconcileMode::Mirror && remote_title != export_plan.title {
            operations.push(TelegramReconcileOperation::SetTitle {
                title: export_plan.title,
            });
        }

        for sticker in desired_stickers {
            let remote_sticker = remote_stickers
                .iter()
                .find(|remote| remote.sticker_id == sticker.sticker_id);
            if let Some(remote_sticker) = remote_sticker {
                seen_remote_sticker_ids.insert(remote_sticker.sticker_id.clone());
                if mode == TelegramReconcileMode::Mirror
                    && !remote_sticker_matches_plan(remote_sticker, &sticker)
                {
                    operations.push(TelegramReconcileOperation::ReplaceSticker {
                        old_telegram_file_id: remote_sticker.telegram_file_id.clone(),
                        sticker,
                    });
                } else {
                    operations.push(TelegramReconcileOperation::KeepSticker {
                        sticker_id: remote_sticker.sticker_id.clone(),
                        telegram_file_id: remote_sticker.telegram_file_id.clone(),
                    });
                }
            } else {
                operations.push(TelegramReconcileOperation::AddSticker { sticker });
            }
        }

        if mode == TelegramReconcileMode::Mirror {
            for remote_sticker in remote_stickers {
                if !seen_remote_sticker_ids.contains(&remote_sticker.sticker_id) {
                    operations.push(TelegramReconcileOperation::DeleteSticker {
                        sticker_id: remote_sticker.sticker_id,
                        telegram_file_id: remote_sticker.telegram_file_id,
                    });
                }
            }
        }

        Ok(TelegramReconcilePlan {
            sticker_set_name,
            mode,
            operations,
        })
    }
}

/// Telegram planner errors.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TelegramTargetError {
    /// Source pack has no stickers.
    #[error("Telegram exports require at least one sticker")]
    EmptyPack,

    /// Sticker count exceeds Telegram set limits.
    #[error("too many stickers for {set_type:?}: {count} > {max}")]
    TooManyStickers {
        /// Telegram set type.
        set_type: TelegramStickerSetType,
        /// Sticker count in the source pack.
        count: usize,
        /// Maximum allowed count.
        max: usize,
    },

    /// Default emoji list is invalid.
    #[error("Telegram exports require a non-empty default emoji")]
    InvalidEmojiList,

    /// Bot username cannot produce a valid Telegram sticker set suffix.
    #[error("Telegram exports require a non-empty bot username")]
    InvalidBotUsername,

    /// Media planning failed.
    #[error("Telegram media planning failed: {message}")]
    MediaPlan {
        /// Planning failure detail.
        message: String,
    },

    /// Create-only export found an existing set name.
    #[error("Telegram sticker set already exists: {sticker_set_name}")]
    TargetSetAlreadyExists {
        /// Existing sticker set name.
        sticker_set_name: String,
    },
}

fn normalize_set_name(slug: &str, bot_username: &str) -> Result<String, TelegramTargetError> {
    let bot = sanitize_component(bot_username.trim_start_matches('@'));
    if bot.is_empty() {
        return Err(TelegramTargetError::InvalidBotUsername);
    }
    let suffix = format!("_by_{bot}");
    let mut base = sanitize_component(slug);
    if base.is_empty() {
        "pack".clone_into(&mut base);
    }

    let mut stem = base
        .strip_suffix(&suffix)
        .map_or_else(|| base.clone(), ToOwned::to_owned);
    if stem.is_empty() {
        "pack".clone_into(&mut stem);
    }

    let max_stem_len = 64usize.saturating_sub(suffix.len());
    if stem.len() > max_stem_len {
        stem.truncate(max_stem_len);
        stem = stem.trim_matches('_').to_owned();
        if stem.is_empty() {
            "pack".clone_into(&mut stem);
        }
    }

    Ok(format!("{stem}{suffix}"))
}

fn sanitize_component(value: &str) -> String {
    let mut output = String::new();
    let mut previous_underscore = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            previous_underscore = false;
        } else if !previous_underscore {
            output.push('_');
            previous_underscore = true;
        }
    }
    output.trim_matches('_').to_owned()
}

fn normalized_default_emoji(value: &str) -> Result<String, TelegramTargetError> {
    let value = value.trim();
    if value.is_empty() {
        Err(TelegramTargetError::InvalidEmojiList)
    } else {
        Ok(value.to_owned())
    }
}

fn sticker_format_for_profile(profile_key: &str) -> StickerFormat {
    if profile_key == "telegram.sticker.video.v1" {
        StickerFormat::Video
    } else {
        StickerFormat::Static
    }
}

fn keyword_for_sticker_title(title: &str) -> Vec<String> {
    let title = title.trim();
    if title.is_empty() {
        Vec::new()
    } else {
        vec![title.chars().take(64).collect()]
    }
}

fn remote_sticker_matches_plan(
    remote: &TelegramRemoteSticker,
    planned: &PlannedTelegramSticker,
) -> bool {
    remote.target_profile_key == planned.target_profile_key
        && remote.emoji_list == planned.emoji_list
        && remote.keywords == planned.keywords
}
