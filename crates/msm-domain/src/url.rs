use url::Url;

use crate::{DomainError, DomainResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetUrlConfig {
    public_app_url: Url,
    public_asset_url: Option<Url>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetUrlInput<'a> {
    pub pack_public_id: &'a str,
    pub filename: &'a str,
}

impl AssetUrlConfig {
    /// Creates URL resolution config with an MSM public app URL.
    ///
    /// # Errors
    ///
    /// Returns an error when `public_app_url` is not an absolute URL.
    pub fn new(public_app_url: &str) -> DomainResult<Self> {
        Ok(Self {
            public_app_url: parse_base_url(public_app_url)?,
            public_asset_url: None,
        })
    }

    /// Sets the optional system-wide public asset URL.
    ///
    /// # Errors
    ///
    /// Returns an error when `public_asset_url` is not an absolute URL.
    pub fn with_public_asset_url(mut self, public_asset_url: &str) -> DomainResult<Self> {
        self.public_asset_url = Some(parse_base_url(public_asset_url)?);
        Ok(self)
    }

    fn asset_base_url(&self) -> &Url {
        self.public_asset_url
            .as_ref()
            .unwrap_or(&self.public_app_url)
    }
}

/// Resolves the exported public URL for an internal sticker asset.
///
/// # Errors
///
/// Returns an error when the configured base URL cannot accept path segments.
pub fn resolve_asset_url(
    config: &AssetUrlConfig,
    input: &AssetUrlInput<'_>,
) -> DomainResult<String> {
    let mut url = config.asset_base_url().clone();
    {
        let url_text = url.to_string();
        let mut segments = url
            .path_segments_mut()
            .map_err(|()| DomainError::InvalidAssetUrlBase { url: url_text })?;
        segments.pop_if_empty();
        segments.extend(["assets", "packs", input.pack_public_id, input.filename]);
    }
    Ok(url.to_string())
}

fn parse_base_url(value: &str) -> DomainResult<Url> {
    Url::parse(value).map_err(|source| DomainError::InvalidBaseUrl {
        url: value.to_owned(),
        source,
    })
}
