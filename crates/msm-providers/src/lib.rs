#![doc = "Provider normalization primitives for MoreStickersManager-rs."]

pub mod error;
pub mod line;
pub mod registry;
pub mod telegram;

use msm_domain::StickerPack;

pub use error::{ProviderError, ProviderResult};
pub use registry::{all_provider_metadata, ProviderCapability, ProviderMetadata, ProviderStatus};

/// Normalizes provider-specific payloads into `MoreStickers` sticker packs.
pub trait StickerProvider {
    /// Returns static metadata for this provider implementation.
    fn metadata(&self) -> ProviderMetadata;

    /// Converts provider-specific JSON into a `MoreStickers`-compatible sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is not valid JSON for the provider schema, required
    /// fields are missing, provider IDs are invalid, or the public base URL is empty.
    fn normalize_pack_json(
        &self,
        input: &str,
        public_base_url: &str,
    ) -> ProviderResult<StickerPack>;
}
