use std::collections::BTreeMap;

use crate::{ExportCapabilities, ExportTarget, ExportTargetKind};

/// In-memory export target registry.
#[derive(Default)]
pub struct ExportRegistry {
    targets: BTreeMap<ExportTargetKind, Box<dyn ExportTarget>>,
}

impl ExportRegistry {
    /// Registers one export target implementation.
    ///
    /// # Errors
    ///
    /// Returns [`ExportRegistryError::DuplicateTargetKind`] when a target with the same kind is
    /// already registered.
    pub fn register(&mut self, target: Box<dyn ExportTarget>) -> Result<(), ExportRegistryError> {
        let kind = target.kind();
        if self.targets.contains_key(&kind) {
            return Err(ExportRegistryError::DuplicateTargetKind { kind });
        }

        self.targets.insert(kind, target);
        Ok(())
    }

    /// Finds a target by kind.
    #[must_use]
    pub fn target(&self, kind: &ExportTargetKind) -> Option<&dyn ExportTarget> {
        self.targets.get(kind).map(Box::as_ref)
    }

    /// Returns target capabilities sorted by target kind.
    #[must_use]
    pub fn capabilities(&self) -> Vec<ExportCapabilities> {
        self.targets
            .values()
            .map(|target| target.capabilities())
            .collect()
    }
}

/// Errors raised while registering export targets.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ExportRegistryError {
    /// A target kind was registered more than once.
    #[error("duplicate export target kind: {kind}")]
    DuplicateTargetKind {
        /// Duplicate target kind.
        kind: ExportTargetKind,
    },
}
