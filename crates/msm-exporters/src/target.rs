use std::fmt;

/// Result type used by exporter planning operations.
pub type ExportResult<T> = Result<T, ExportError>;

/// Stable export target kind key.
#[derive(
    Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(transparent)]
pub struct ExportTargetKind(String);

impl ExportTargetKind {
    /// Creates an export target kind key.
    #[must_use]
    pub fn new(kind: impl Into<String>) -> Self {
        Self(kind.into())
    }

    /// Returns the target kind as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExportTargetKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Capability metadata shared with API, Web, CLI, and MCP surfaces.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportCapabilities {
    /// Stable target kind key.
    pub kind: ExportTargetKind,
    /// Human-readable display name.
    pub display_name: String,
    /// Whether this target publishes to a remote service.
    pub supports_remote_publication: bool,
    /// Whether this target requires media conversion before export.
    pub supports_media_conversion: bool,
    /// Whether this target requires stored credentials or secrets.
    pub requires_credentials: bool,
}

/// Target-neutral request to plan an export.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportRequest {
    /// Source MSM pack ID.
    pub pack_id: String,
    /// Configured export target ID.
    pub target_id: String,
    /// Target-specific options as JSON.
    pub options_json: String,
}

/// Target-neutral export plan summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportPlan {
    /// Target kind selected for this plan.
    pub target_kind: ExportTargetKind,
    /// Source MSM pack ID.
    pub pack_id: String,
    /// Ordered high-level planning steps.
    pub steps: Vec<String>,
}

/// Export target implementation boundary.
pub trait ExportTarget: fmt::Debug + Send + Sync {
    /// Stable target kind.
    fn kind(&self) -> ExportTargetKind;

    /// Capability metadata for this target.
    fn capabilities(&self) -> ExportCapabilities;

    /// Plans an export operation for this target.
    ///
    /// # Errors
    ///
    /// Returns an exporter-specific error when the request cannot be planned.
    fn plan(&self, request: ExportRequest) -> ExportResult<ExportPlan>;
}

/// Errors raised by target implementations.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ExportError {
    /// Target-specific planning failure.
    #[error("export planning failed for {target_kind}: {message}")]
    Planning {
        /// Target kind that failed.
        target_kind: ExportTargetKind,
        /// Human-readable failure message.
        message: String,
    },

    /// Target serialization failure.
    #[error("export serialization failed for {target_kind}: {message}")]
    Serialization {
        /// Target kind that failed.
        target_kind: ExportTargetKind,
        /// Human-readable failure message.
        message: String,
    },
}
