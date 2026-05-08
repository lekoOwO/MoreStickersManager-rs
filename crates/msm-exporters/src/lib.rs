#![doc = "Export target abstractions for MoreStickersManager-rs."]

pub mod morestickers;
pub mod registry;
pub mod target;
pub mod telegram;

pub use morestickers::{MoreStickersExportArtifact, MoreStickersExportTarget};
pub use registry::{ExportRegistry, ExportRegistryError};
pub use target::{
    ExportCapabilities, ExportError, ExportPlan, ExportRequest, ExportResult, ExportTarget,
    ExportTargetKind,
};
pub use telegram::{
    PlannedTelegramSticker, TelegramExportOptions, TelegramExportPlan, TelegramExportPlanner,
    TelegramReconcileMode, TelegramReconcileOperation, TelegramReconcilePlan, TelegramRemoteSet,
    TelegramRemoteSticker, TelegramStickerSetType, TelegramTargetConfig, TelegramTargetError,
};
