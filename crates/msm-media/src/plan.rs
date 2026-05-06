use crate::profile::{MediaKind, PreparedMediaSpec, StickerTargetProfile};

/// Result type returned by media planning operations.
pub type MediaPlanResult<T> = Result<T, MediaPlanError>;

/// Errors produced while selecting target media profiles.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MediaPlanError {
    /// The source media kind cannot satisfy the requested target profile.
    #[error("unsupported source media for {target_profile}: {source_kind:?}")]
    UnsupportedSource {
        /// Target profile group that rejected the source.
        target_profile: String,
        /// Source media kind that could not be planned.
        source_kind: MediaKind,
    },
}

/// Deterministic plan for preparing one source asset for one target profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionPlan {
    source_kind: MediaKind,
    prepared_media: PreparedMediaSpec,
}

impl ConversionPlan {
    /// Builds a conversion plan for a Telegram regular sticker set asset.
    ///
    /// # Errors
    ///
    /// Returns [`MediaPlanError::UnsupportedSource`] when `source_kind` does
    /// not map to the initial Telegram regular-sticker profiles.
    pub fn for_telegram_regular_sticker(source_kind: MediaKind) -> MediaPlanResult<Self> {
        let prepared_media = match source_kind {
            MediaKind::StaticImage => PreparedMediaSpec::new(
                StickerTargetProfile::telegram_static_sticker(),
                "image/png",
                "png",
            ),
            MediaKind::AnimatedImage | MediaKind::Video => PreparedMediaSpec::new(
                StickerTargetProfile::telegram_video_sticker(),
                "video/webm",
                "webm",
            ),
            MediaKind::Unsupported(_) => {
                return Err(MediaPlanError::UnsupportedSource {
                    target_profile: "telegram.sticker.regular".to_owned(),
                    source_kind,
                });
            }
        };

        Ok(Self {
            source_kind,
            prepared_media,
        })
    }

    /// Source media kind selected for this plan.
    #[must_use]
    pub fn source_kind(&self) -> MediaKind {
        self.source_kind.clone()
    }

    /// Target profile selected by this plan.
    #[must_use]
    pub fn profile(&self) -> StickerTargetProfile {
        self.prepared_media.profile().clone()
    }

    /// Prepared media output expected from this plan.
    #[must_use]
    pub const fn prepared_media(&self) -> &PreparedMediaSpec {
        &self.prepared_media
    }
}
