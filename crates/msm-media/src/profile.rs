/// Source media categories understood by the export media planner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MediaKind {
    /// A single-frame image such as PNG, JPEG, or WebP.
    StaticImage,
    /// A frame-based animated image such as GIF, APNG, or animated WebP.
    AnimatedImage,
    /// A video source that can be converted to a target video sticker profile.
    Video,
    /// A source MIME type or format that the planner does not support yet.
    Unsupported(String),
}

/// Target-specific media constraints for a prepared sticker asset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StickerTargetProfile {
    profile_key: &'static str,
    label: &'static str,
    max_file_size_bytes: u64,
    canvas_size_px: u16,
    max_duration_ms: Option<u32>,
}

impl StickerTargetProfile {
    /// Returns the first static Telegram regular-sticker profile supported by MSM.
    #[must_use]
    pub const fn telegram_static_sticker() -> Self {
        Self {
            profile_key: "telegram.sticker.static.v1",
            label: "Telegram static sticker",
            max_file_size_bytes: 512 * 1024,
            canvas_size_px: 512,
            max_duration_ms: None,
        }
    }

    /// Returns the first video Telegram regular-sticker profile supported by MSM.
    #[must_use]
    pub const fn telegram_video_sticker() -> Self {
        Self {
            profile_key: "telegram.sticker.video.v1",
            label: "Telegram video sticker",
            max_file_size_bytes: 256 * 1024,
            canvas_size_px: 512,
            max_duration_ms: Some(3_000),
        }
    }

    /// Returns the first Telegram thumbnail profile supported by MSM.
    #[must_use]
    pub const fn telegram_thumbnail() -> Self {
        Self {
            profile_key: "telegram.thumbnail.static.v1",
            label: "Telegram thumbnail",
            max_file_size_bytes: 32 * 1024,
            canvas_size_px: 100,
            max_duration_ms: None,
        }
    }

    /// Stable profile key used for cache entries and API capability output.
    #[must_use]
    pub const fn profile_key(&self) -> &'static str {
        self.profile_key
    }

    /// Human-readable label for diagnostics and UI summaries.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }

    /// Maximum output file size accepted by the target profile.
    #[must_use]
    pub const fn max_file_size_bytes(&self) -> u64 {
        self.max_file_size_bytes
    }

    /// Square canvas size expected by the target profile.
    #[must_use]
    pub const fn canvas_size_px(&self) -> u16 {
        self.canvas_size_px
    }

    /// Maximum animation or video duration accepted by the target profile.
    #[must_use]
    pub const fn max_duration_ms(&self) -> Option<u32> {
        self.max_duration_ms
    }
}

/// Output media expected after conversion for a specific target profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedMediaSpec {
    profile: StickerTargetProfile,
    mime_type: &'static str,
    extension: &'static str,
}

impl PreparedMediaSpec {
    /// Creates a prepared media specification.
    #[must_use]
    pub const fn new(
        profile: StickerTargetProfile,
        mime_type: &'static str,
        extension: &'static str,
    ) -> Self {
        Self {
            profile,
            mime_type,
            extension,
        }
    }

    /// Target profile this output satisfies.
    #[must_use]
    pub const fn profile(&self) -> &StickerTargetProfile {
        &self.profile
    }

    /// MIME type of the prepared output.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        self.mime_type
    }

    /// File extension for the prepared output, without a leading dot.
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        self.extension
    }
}
