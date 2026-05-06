use msm_domain::StickerPack;

use crate::{
    ExportCapabilities, ExportError, ExportPlan, ExportRequest, ExportResult, ExportTarget,
    ExportTargetKind,
};

const MORESTICKERS_KIND: &str = "morestickers";

/// `MoreStickers` `.stickerpack` export target.
#[derive(Clone, Debug, Default)]
pub struct MoreStickersExportTarget;

impl MoreStickersExportTarget {
    /// Serializes a sticker pack into a MoreStickers-compatible artifact.
    ///
    /// # Errors
    ///
    /// Returns an error when domain serialization fails.
    pub fn export_pack(&self, pack: &StickerPack) -> ExportResult<MoreStickersExportArtifact> {
        let contents = pack
            .to_pretty_json()
            .map_err(|error| ExportError::Serialization {
                target_kind: morestickers_kind(),
                message: error.to_string(),
            })?
            .into_bytes();

        Ok(MoreStickersExportArtifact {
            file_name: format!("{}.stickerpack", pack_file_stem(&pack.id)),
            mime_type: "application/json",
            contents,
        })
    }
}

impl ExportTarget for MoreStickersExportTarget {
    fn kind(&self) -> ExportTargetKind {
        morestickers_kind()
    }

    fn capabilities(&self) -> ExportCapabilities {
        ExportCapabilities {
            kind: morestickers_kind(),
            display_name: "MoreStickers".to_owned(),
            supports_remote_publication: false,
            supports_media_conversion: false,
            requires_credentials: false,
        }
    }

    fn plan(&self, request: ExportRequest) -> ExportResult<ExportPlan> {
        Ok(ExportPlan {
            target_kind: morestickers_kind(),
            pack_id: request.pack_id,
            steps: vec!["serialize .stickerpack".to_owned()],
        })
    }
}

/// Serialized `MoreStickers` export artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoreStickersExportArtifact {
    /// Suggested output file name.
    pub file_name: String,
    /// Artifact MIME type.
    pub mime_type: &'static str,
    /// Serialized `.stickerpack` bytes.
    pub contents: Vec<u8>,
}

fn morestickers_kind() -> ExportTargetKind {
    ExportTargetKind::new(MORESTICKERS_KIND)
}

fn pack_file_stem(pack_id: &str) -> &str {
    pack_id.rsplit(':').next().unwrap_or(pack_id)
}
