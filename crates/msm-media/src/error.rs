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
}
