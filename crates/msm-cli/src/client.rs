use async_trait::async_trait;
use msm_domain::StickerPack;
use url::Url;

use crate::{command::PackVisibility, output::HealthResponse, CliError, CliResult};

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPackPayload {
    pub tenant_id: String,
    pub owner_user_id: String,
    pub pack_id: String,
    pub visibility: PackVisibility,
    pub pack: StickerPack,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePackPayload {
    pub pack_id: String,
    pub title: String,
    pub visibility: PackVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePersonalAccessTokenPayload {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedPersonalAccessToken {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub token: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalAccessToken {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

#[async_trait]
pub trait MsmClient {
    async fn health(&self) -> CliResult<HealthResponse>;
    async fn list_packs(&self, user_id: &str) -> CliResult<Vec<StickerPack>>;
    async fn import_pack(&self, payload: ImportPackPayload) -> CliResult<()>;
    async fn export_pack(&self, pack_id: &str) -> CliResult<StickerPack>;
    async fn update_pack(&self, payload: UpdatePackPayload) -> CliResult<()>;
    async fn delete_pack(&self, pack_id: &str) -> CliResult<()>;
    async fn create_pat(
        &self,
        payload: CreatePersonalAccessTokenPayload,
    ) -> CliResult<CreatedPersonalAccessToken>;
    async fn list_pats(&self, user_id: &str) -> CliResult<Vec<PersonalAccessToken>>;
    async fn revoke_pat(&self, token_id: &str) -> CliResult<()>;
}

#[derive(Clone)]
pub struct ReqwestMsmClient {
    base_url: Url,
    http: reqwest::Client,
    bearer_token: Option<String>,
}

impl ReqwestMsmClient {
    /// Creates a new HTTP-backed MSM client.
    ///
    /// # Errors
    ///
    /// Returns an error when `base_url` is not a valid absolute URL.
    pub fn new(base_url: &str) -> CliResult<Self> {
        Self::new_with_pat(base_url, None)
    }

    /// Creates a new HTTP-backed MSM client with an optional Bearer PAT.
    ///
    /// # Errors
    ///
    /// Returns an error when `base_url` is not a valid absolute URL.
    pub fn new_with_pat(base_url: &str, bearer_token: Option<String>) -> CliResult<Self> {
        Ok(Self {
            base_url: Url::parse(base_url).map_err(|source| CliError::InvalidBaseUrl {
                url: base_url.to_owned(),
                source,
            })?,
            http: reqwest::Client::new(),
            bearer_token,
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

    #[must_use]
    pub fn bearer_token(&self) -> Option<&str> {
        self.bearer_token.as_deref()
    }

    fn authorize(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.bearer_token() {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }
}

#[async_trait]
impl MsmClient for ReqwestMsmClient {
    async fn health(&self) -> CliResult<HealthResponse> {
        Ok(self
            .authorize(self.http.get(self.endpoint("/healthz")?))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_packs(&self, user_id: &str) -> CliResult<Vec<StickerPack>> {
        Ok(self
            .authorize(self.http.get(self.endpoint("/api/v1/packs")?))
            .query(&[("userId", user_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn import_pack(&self, payload: ImportPackPayload) -> CliResult<()> {
        self.authorize(self.http.post(self.endpoint("/api/v1/packs/import")?))
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn export_pack(&self, pack_id: &str) -> CliResult<StickerPack> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint(&format!("/api/v1/packs/{pack_id}/stickerpack"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn update_pack(&self, payload: UpdatePackPayload) -> CliResult<()> {
        self.authorize(
            self.http
                .patch(self.endpoint(&format!("/api/v1/packs/{}", payload.pack_id))?),
        )
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    async fn delete_pack(&self, pack_id: &str) -> CliResult<()> {
        self.authorize(
            self.http
                .delete(self.endpoint(&format!("/api/v1/packs/{pack_id}"))?),
        )
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    async fn create_pat(
        &self,
        payload: CreatePersonalAccessTokenPayload,
    ) -> CliResult<CreatedPersonalAccessToken> {
        Ok(self
            .authorize(self.http.post(self.endpoint("/api/v1/pats")?))
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_pats(&self, user_id: &str) -> CliResult<Vec<PersonalAccessToken>> {
        Ok(self
            .authorize(self.http.get(self.endpoint("/api/v1/pats")?))
            .query(&[("userId", user_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn revoke_pat(&self, token_id: &str) -> CliResult<()> {
        self.authorize(
            self.http
                .delete(self.endpoint(&format!("/api/v1/pats/{token_id}"))?),
        )
        .send()
        .await?
        .error_for_status()?;
        Ok(())
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

    #[test]
    fn reqwest_client_stores_configured_pat() {
        let client = ReqwestMsmClient::new_with_pat(
            "https://msm.example",
            Some("msm_pat_cli1_secret".to_owned()),
        )
        .unwrap();

        assert_eq!(client.bearer_token(), Some("msm_pat_cli1_secret"));
    }
}
