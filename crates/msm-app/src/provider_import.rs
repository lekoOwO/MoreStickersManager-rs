use std::{future::Future, pin::Pin};

use msm_domain::StickerPack;
use msm_providers::{ProviderAssetDownloadStrategy, ProviderRemoteFetchPlan};
use msm_storage::{AssetKey, LocalAssetStore};

/// Result type for provider import runtime helpers.
pub type ProviderImportResult<T> = Result<T, ProviderImportError>;

/// Provider import runtime errors.
#[derive(Debug, thiserror::Error)]
pub enum ProviderImportError {
    /// Provider runtime fetch failed.
    #[error("provider fetch failed: {0}")]
    Fetch(String),

    /// Provider runtime asset download failed.
    #[error("provider asset download failed: {0}")]
    AssetDownload(String),

    /// Asset storage failed.
    #[error("asset storage error: {0}")]
    Storage(#[from] msm_storage::StorageError),

    /// Remote asset URL cannot produce a safe local filename.
    #[error("remote asset URL has no usable filename: {url}")]
    MissingFilename {
        /// Remote URL that could not be mapped.
        url: String,
    },

    /// Direct internalization currently only supports providers with direct remote URLs.
    #[error("unsupported provider asset strategy for direct internalization: {strategy:?}")]
    UnsupportedAssetStrategy {
        /// Unsupported strategy.
        strategy: ProviderAssetDownloadStrategy,
    },
}

/// Executes metadata requests described by `ProviderRemoteFetchPlan`.
pub trait ProviderMetadataFetcher {
    /// Fetches provider metadata bytes for a planned request.
    ///
    /// # Errors
    ///
    /// Returns an error when the runtime cannot fetch the provider metadata.
    fn fetch_metadata<'a>(
        &'a self,
        plan: &'a ProviderRemoteFetchPlan,
    ) -> Pin<Box<dyn Future<Output = ProviderImportResult<Vec<u8>>> + Send + 'a>>;
}

/// Downloads provider asset URLs during internalization.
pub trait ProviderAssetDownloader {
    /// Downloads one provider asset URL.
    ///
    /// # Errors
    ///
    /// Returns an error when the runtime cannot download the remote asset.
    fn download_asset<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = ProviderImportResult<Vec<u8>>> + Send + 'a>>;
}

/// Fetches provider metadata bytes through an injected runtime fetcher.
///
/// # Errors
///
/// Returns an error when the injected fetcher cannot execute the plan.
pub async fn fetch_provider_metadata(
    plan: &ProviderRemoteFetchPlan,
    fetcher: &impl ProviderMetadataFetcher,
) -> ProviderImportResult<Vec<u8>> {
    fetcher.fetch_metadata(plan).await
}

/// Downloads direct remote sticker assets into the local asset store and rewrites
/// sticker image URLs to MSM-hosted asset URLs.
///
/// # Errors
///
/// Returns an error when the provider strategy is not direct URLs, a filename is
/// unsafe/missing, the downloader fails, or the local asset store cannot write bytes.
pub async fn internalize_direct_remote_pack_assets(
    pack: &StickerPack,
    pack_public_id: &str,
    public_asset_base_url: &str,
    strategy: ProviderAssetDownloadStrategy,
    downloader: &(impl ProviderAssetDownloader + ?Sized),
    asset_store: &LocalAssetStore,
) -> ProviderImportResult<StickerPack> {
    if strategy != ProviderAssetDownloadStrategy::DirectRemoteUrls {
        return Err(ProviderImportError::UnsupportedAssetStrategy { strategy });
    }

    let base_url = public_asset_base_url.trim().trim_end_matches('/');
    let mut rewritten = pack.clone();
    for sticker in &mut rewritten.stickers {
        let source_url = sticker.image.clone();
        let filename = remote_filename(&source_url)?;
        let key = AssetKey::new(pack_public_id, filename.clone())?;
        let bytes = downloader.download_asset(&source_url).await?;
        asset_store.write(&key, &bytes).await?;
        sticker.filename = Some(filename.clone());
        sticker.image = format!("{base_url}/assets/packs/{pack_public_id}/{filename}");
    }
    if let Some(first) = rewritten.stickers.first() {
        rewritten.logo = first.clone();
    }

    Ok(rewritten)
}

fn remote_filename(url: &str) -> ProviderImportResult<String> {
    let without_query = url.split(['?', '#']).next().unwrap_or(url);
    let filename = without_query.rsplit('/').next().unwrap_or_default().trim();
    if filename.is_empty() {
        return Err(ProviderImportError::MissingFilename {
            url: url.to_owned(),
        });
    }
    Ok(filename.to_owned())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        future::Future,
        pin::Pin,
        sync::{Arc, Mutex},
    };

    use msm_domain::{Author, Sticker, StickerPack};
    use msm_providers::{
        line_sticker_pack_fetch_plan, ProviderAssetDownloadStrategy, ProviderRemoteFetchPlan,
    };
    use msm_storage::LocalAssetStore;

    use super::{
        fetch_provider_metadata, internalize_direct_remote_pack_assets, ProviderAssetDownloader,
        ProviderImportResult, ProviderMetadataFetcher,
    };

    #[derive(Default)]
    struct FakeFetcher {
        seen_urls: Arc<Mutex<Vec<String>>>,
    }

    impl ProviderMetadataFetcher for FakeFetcher {
        fn fetch_metadata<'a>(
            &'a self,
            plan: &'a ProviderRemoteFetchPlan,
        ) -> Pin<Box<dyn Future<Output = ProviderImportResult<Vec<u8>>> + Send + 'a>> {
            Box::pin(async move {
                self.seen_urls
                    .lock()
                    .expect("lock")
                    .push(plan.metadata_request.url.clone());
                Ok(br#"{"id":"line_cats"}"#.to_vec())
            })
        }
    }

    struct FakeDownloader {
        bytes: BTreeMap<String, Vec<u8>>,
    }

    impl ProviderAssetDownloader for FakeDownloader {
        fn download_asset<'a>(
            &'a self,
            url: &'a str,
        ) -> Pin<Box<dyn Future<Output = ProviderImportResult<Vec<u8>>> + Send + 'a>> {
            Box::pin(async move { Ok(self.bytes.get(url).cloned().unwrap_or_default()) })
        }
    }

    #[tokio::test]
    async fn fetches_provider_metadata_through_injected_runtime() {
        let plan =
            line_sticker_pack_fetch_plan("https://store.line.me", "line_cats").expect("fetch plan");
        let fetcher = FakeFetcher::default();

        let bytes = fetch_provider_metadata(&plan, &fetcher)
            .await
            .expect("metadata bytes");

        assert_eq!(bytes, br#"{"id":"line_cats"}"#);
        assert_eq!(
            fetcher.seen_urls.lock().expect("lock").as_slice(),
            ["https://store.line.me/stickershop/product/line_cats/en"]
        );
    }

    #[tokio::test]
    async fn internalizes_direct_remote_pack_assets_into_local_store() {
        let temp = tempfile::tempdir().expect("tempdir");
        let asset_store = LocalAssetStore::new(temp.path());
        let mut bytes = BTreeMap::new();
        bytes.insert(
            "https://cdn.example.test/001.png".to_owned(),
            b"static".to_vec(),
        );
        bytes.insert(
            "https://cdn.example.test/002.apng?download=1".to_owned(),
            b"animated".to_vec(),
        );
        let downloader = FakeDownloader { bytes };
        let pack = sample_pack();

        let rewritten = internalize_direct_remote_pack_assets(
            &pack,
            "pack_1",
            "https://msm.example.test/",
            ProviderAssetDownloadStrategy::DirectRemoteUrls,
            &downloader,
            &asset_store,
        )
        .await
        .expect("internalized pack");

        assert_eq!(
            rewritten.stickers[0].image,
            "https://msm.example.test/assets/packs/pack_1/001.png"
        );
        assert_eq!(rewritten.stickers[0].filename.as_deref(), Some("001.png"));
        assert_eq!(rewritten.stickers[1].filename.as_deref(), Some("002.apng"));
        assert_eq!(rewritten.logo.image, rewritten.stickers[0].image);
        assert_eq!(
            tokio::fs::read(temp.path().join("assets/packs/pack_1/001.png"))
                .await
                .expect("stored static"),
            b"static"
        );
        assert_eq!(
            tokio::fs::read(temp.path().join("assets/packs/pack_1/002.apng"))
                .await
                .expect("stored animated"),
            b"animated"
        );
    }

    fn sample_pack() -> StickerPack {
        StickerPack {
            id: "MoreStickers:Line:Pack:line_cats".to_owned(),
            title: "LINE Cats".to_owned(),
            author: Some(Author {
                name: "LINE".to_owned(),
                url: None,
            }),
            logo: Sticker {
                id: "logo".to_owned(),
                image: "https://cdn.example.test/001.png".to_owned(),
                title: "Logo".to_owned(),
                sticker_pack_id: "MoreStickers:Line:Pack:line_cats".to_owned(),
                filename: None,
                is_animated: Some(false),
            },
            stickers: vec![
                Sticker {
                    id: "001".to_owned(),
                    image: "https://cdn.example.test/001.png".to_owned(),
                    title: "Wave".to_owned(),
                    sticker_pack_id: "MoreStickers:Line:Pack:line_cats".to_owned(),
                    filename: None,
                    is_animated: Some(false),
                },
                Sticker {
                    id: "002".to_owned(),
                    image: "https://cdn.example.test/002.apng?download=1".to_owned(),
                    title: "Dance".to_owned(),
                    sticker_pack_id: "MoreStickers:Line:Pack:line_cats".to_owned(),
                    filename: None,
                    is_animated: Some(true),
                },
            ],
        }
    }
}
