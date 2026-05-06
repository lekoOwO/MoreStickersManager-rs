#![doc = "Export target abstractions for MoreStickersManager-rs."]

pub mod morestickers;
pub mod registry;
pub mod target;

pub use morestickers::{MoreStickersExportArtifact, MoreStickersExportTarget};
pub use registry::{ExportRegistry, ExportRegistryError};
pub use target::{
    ExportCapabilities, ExportError, ExportPlan, ExportRequest, ExportResult, ExportTarget,
    ExportTargetKind,
};
