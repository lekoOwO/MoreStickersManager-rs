use async_trait::async_trait;
use msm_domain::StickerPack;
use url::Url;

use crate::{
    CliError, CliResult,
    command::PackVisibility,
    output::HealthResponse,
};

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPackPayload {
    pub tenant_id: String,
    pub owner_user_id: String,
    pub pack_id: String,
    pub visibility: PackVisibility,
    pub pack: StickerPack,
}

#[async_trait]
pub trait MsmClient {
    async fn health(&self) -> CliResult<HealthResponse>;
    async fn list_packs(&self, user_id: &str) -> CliResult<Vec<StickerPack>>;
    async fn import_pack(&self, payload: ImportPackPayload) -> CliResult<()>;
    async fn export_pack(&self, pack_id: &str) -> CliResult<StickerPack>;
}

#[derive(Clone)]
pub struct ReqwestMsmClient {
    base_url: Url,
    http: reqwest::Client,
}

impl ReqwestMsmClient {
    /// Creates a new HTTP-backed MSM client.
    ///
    /// # Errors
    ///
    /// Returns an error when `base_url` is not a valid absolute URL.
    pub fn new(base_url: &str) -> CliResult<Self> {
        Ok(Self {
            base_url: Url::parse(base_url).map_err(|source| CliError::InvalidBaseUrl {
                url: base_url.to_owned(),
                source,
            })?,
            http: reqwest::Client::new(),
        })
    }

    /// Joins an API path onto the configured base URL.
    ///
    /// # Errors
    ///
    /// Returns an error when the path cannot be joined to the base URL.
    pub fn endpoint(&self, path: &str) -> CliResult<Url> {
        self.base_url
            .join(path)
            .map_err(|source| CliError::InvalidBaseUrl {
                url: path.to_owned(),
                source,
            })
    }
}

#[async_trait]
impl MsmClient for ReqwestMsmClient {
    async fn health(&self) -> CliResult<HealthResponse> {
        Ok(self
            .http
            .get(self.endpoint("/healthz")?)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_packs(&self, user_id: &str) -> CliResult<Vec<StickerPack>> {
        Ok(self
            .http
            .get(self.endpoint("/api/v1/packs")?)
            .query(&[("userId", user_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn import_pack(&self, payload: ImportPackPayload) -> CliResult<()> {
        self.http
            .post(self.endpoint("/api/v1/packs/import")?)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn export_pack(&self, pack_id: &str) -> CliResult<StickerPack> {
        Ok(self
            .http
            .get(self.endpoint(&format!("/api/v1/packs/{pack_id}/stickerpack"))?)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ReqwestMsmClient;

    #[test]
    fn endpoint_joins_paths_to_base_url() {
        let client = ReqwestMsmClient::new("https://msm.example/base/").unwrap();

        assert_eq!(
            client.endpoint("/api/v1/packs").unwrap().as_str(),
            "https://msm.example/api/v1/packs"
        );
    }

    #[test]
    fn rejects_invalid_base_url() {
        assert!(ReqwestMsmClient::new("not a url").is_err());
    }
}
