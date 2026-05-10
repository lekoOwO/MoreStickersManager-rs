use crate::MediaKind;

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

    /// ffprobe output could not be parsed into normalized media facts.
    #[error("ffprobe output parse error: {message}")]
    ProbeParse {
        /// Human-readable parser failure summary.
        message: String,
    },

    /// Prepared media probe facts violate target constraints.
    #[error("prepared media validation failed for {profile_key}: {message}")]
    TargetValidation {
        /// Target profile key that rejected the media.
        profile_key: String,
        /// Human-readable validation summary.
        message: String,
    },
}
