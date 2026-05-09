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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderPayload {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPackMembershipPayload {
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderPack {
    pub folder_id: String,
    pub pack_id: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagPayload {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackTag {
    pub pack_id: String,
    pub tag_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionGroupPayload {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub title: String,
    pub visibility: PackVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionGroup {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub title: String,
    pub visibility: PackVisibility,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionGroupPack {
    pub subscription_group_id: String,
    pub pack_id: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionAccessResourceType {
    Pack,
    SubscriptionGroup,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionAccessTokenPayload {
    pub id: String,
    pub resource_type: SubscriptionAccessResourceType,
    pub resource_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionAccessToken {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub resource_type: SubscriptionAccessResourceType,
    pub resource_id: String,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedSubscriptionAccessToken {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub resource_type: SubscriptionAccessResourceType,
    pub resource_id: String,
    pub token: String,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantMember {
    pub tenant_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTenantMemberPayload {
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantSettings {
    pub tenant_id: String,
    pub name: String,
    pub public_asset_url: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTenantSettingsPayload {
    pub name: String,
    pub public_asset_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub is_disabled: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTenantUserStatusPayload {
    pub is_disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantRole {
    pub id: String,
    pub tenant_id: Option<String>,
    pub name: String,
    pub permissions: Vec<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTenantRolePayload {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTargetKind {
    pub kind: String,
    pub display_name: String,
    pub supports_remote_publication: bool,
    pub supports_media_conversion: bool,
    pub requires_credentials: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExportTargetPayload {
    pub id: String,
    pub tenant_id: String,
    pub kind: String,
    pub name: String,
    pub config: serde_json::Value,
    pub is_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTarget {
    pub id: String,
    pub tenant_id: String,
    pub kind: String,
    pub name: String,
    pub config: serde_json::Value,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExportJobPayload {
    pub id: String,
    pub tenant_id: String,
    pub source_pack_id: String,
    pub target_id: String,
    pub options: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportJob {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub source_pack_id: String,
    pub target_id: String,
    pub status: String,
    pub request: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error_summary: Option<String>,
    pub attempt_count: i64,
    pub max_attempts: i64,
    pub next_attempt_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportJobEvent {
    pub job_id: String,
    pub sequence: i64,
    pub level: String,
    pub stage: String,
    pub message: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramPublication {
    pub id: String,
    pub pack_id: String,
    pub target_id: String,
    pub job_id: String,
    pub sticker_set_name: String,
    pub sticker_set_url: String,
    pub sticker_count: i64,
    pub sticker_type: String,
    pub created_at: String,
    pub updated_at: String,
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
    async fn list_tenant_members(&self, tenant_id: &str) -> CliResult<Vec<TenantMember>>;
    async fn set_tenant_member_role(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: &str,
    ) -> CliResult<TenantMember>;
    async fn get_tenant_settings(&self, tenant_id: &str) -> CliResult<TenantSettings>;
    async fn update_tenant_settings(
        &self,
        tenant_id: &str,
        payload: UpdateTenantSettingsPayload,
    ) -> CliResult<TenantSettings>;
    async fn update_tenant_user_status(
        &self,
        tenant_id: &str,
        user_id: &str,
        payload: UpdateTenantUserStatusPayload,
    ) -> CliResult<TenantUser>;
    async fn list_tenant_roles(&self, tenant_id: &str) -> CliResult<Vec<TenantRole>>;
    async fn upsert_tenant_role(
        &self,
        tenant_id: &str,
        role_id: &str,
        payload: UpsertTenantRolePayload,
    ) -> CliResult<TenantRole>;
    async fn create_folder(&self, payload: CreateFolderPayload) -> CliResult<Folder>;
    async fn list_folders(&self, tenant_id: &str, owner_user_id: &str) -> CliResult<Vec<Folder>>;
    async fn create_tag(&self, payload: CreateTagPayload) -> CliResult<Tag>;
    async fn list_tags(&self, tenant_id: &str) -> CliResult<Vec<Tag>>;
    async fn create_subscription_group(
        &self,
        payload: CreateSubscriptionGroupPayload,
    ) -> CliResult<SubscriptionGroup>;
    async fn list_subscription_groups(
        &self,
        tenant_id: &str,
        owner_user_id: &str,
    ) -> CliResult<Vec<SubscriptionGroup>>;
    async fn add_pack_to_folder(
        &self,
        folder_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> CliResult<FolderPack>;
    async fn list_folder_pack_ids(&self, folder_id: &str) -> CliResult<Vec<String>>;
    async fn remove_pack_from_folder(&self, folder_id: &str, pack_id: &str) -> CliResult<()>;
    async fn add_tag_to_pack(&self, pack_id: &str, tag_id: &str) -> CliResult<PackTag>;
    async fn list_pack_tag_ids(&self, pack_id: &str) -> CliResult<Vec<String>>;
    async fn remove_tag_from_pack(&self, pack_id: &str, tag_id: &str) -> CliResult<()>;
    async fn add_pack_to_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> CliResult<SubscriptionGroupPack>;
    async fn list_subscription_group_pack_ids(
        &self,
        subscription_group_id: &str,
    ) -> CliResult<Vec<String>>;
    async fn remove_pack_from_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
    ) -> CliResult<()>;
    async fn create_subscription_access_token(
        &self,
        payload: CreateSubscriptionAccessTokenPayload,
    ) -> CliResult<CreatedSubscriptionAccessToken>;
    async fn list_subscription_access_tokens(
        &self,
        user_id: &str,
    ) -> CliResult<Vec<SubscriptionAccessToken>>;
    async fn rotate_subscription_access_token(
        &self,
        token_id: &str,
    ) -> CliResult<CreatedSubscriptionAccessToken>;
    async fn revoke_subscription_access_token(&self, token_id: &str) -> CliResult<()>;
    async fn list_export_target_kinds(&self) -> CliResult<Vec<ExportTargetKind>>;
    async fn list_export_targets(&self, tenant_id: &str) -> CliResult<Vec<ExportTarget>>;
    async fn create_export_target(
        &self,
        payload: CreateExportTargetPayload,
    ) -> CliResult<ExportTarget>;
    async fn create_export_job(&self, payload: CreateExportJobPayload) -> CliResult<ExportJob>;
    async fn get_export_job(&self, job_id: &str) -> CliResult<ExportJob>;
    async fn list_export_job_events(&self, job_id: &str) -> CliResult<Vec<ExportJobEvent>>;
    async fn list_telegram_publications(
        &self,
        pack_id: &str,
    ) -> CliResult<Vec<TelegramPublication>>;
    async fn get_telegram_publication(
        &self,
        publication_id: &str,
    ) -> CliResult<TelegramPublication>;
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

    async fn list_tenant_members(&self, tenant_id: &str) -> CliResult<Vec<TenantMember>> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint(&format!("/api/v1/tenants/{tenant_id}/members"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn set_tenant_member_role(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: &str,
    ) -> CliResult<TenantMember> {
        Ok(self
            .authorize(
                self.http
                    .put(self.endpoint(&format!("/api/v1/tenants/{tenant_id}/members/{user_id}"))?),
            )
            .json(&UpsertTenantMemberPayload {
                role: role.to_owned(),
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn get_tenant_settings(&self, tenant_id: &str) -> CliResult<TenantSettings> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint(&format!("/api/v1/tenants/{tenant_id}/settings"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn update_tenant_settings(
        &self,
        tenant_id: &str,
        payload: UpdateTenantSettingsPayload,
    ) -> CliResult<TenantSettings> {
        Ok(self
            .authorize(
                self.http
                    .put(self.endpoint(&format!("/api/v1/tenants/{tenant_id}/settings"))?),
            )
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn update_tenant_user_status(
        &self,
        tenant_id: &str,
        user_id: &str,
        payload: UpdateTenantUserStatusPayload,
    ) -> CliResult<TenantUser> {
        Ok(self
            .authorize(self.http.put(self.endpoint(&format!(
                "/api/v1/tenants/{tenant_id}/users/{user_id}/status"
            ))?))
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_tenant_roles(&self, tenant_id: &str) -> CliResult<Vec<TenantRole>> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint(&format!("/api/v1/tenants/{tenant_id}/roles"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn upsert_tenant_role(
        &self,
        tenant_id: &str,
        role_id: &str,
        payload: UpsertTenantRolePayload,
    ) -> CliResult<TenantRole> {
        Ok(self
            .authorize(
                self.http
                    .put(self.endpoint(&format!("/api/v1/tenants/{tenant_id}/roles/{role_id}"))?),
            )
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn create_folder(&self, payload: CreateFolderPayload) -> CliResult<Folder> {
        Ok(self
            .authorize(self.http.post(self.endpoint("/api/v1/folders")?))
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_folders(&self, tenant_id: &str, owner_user_id: &str) -> CliResult<Vec<Folder>> {
        Ok(self
            .authorize(self.http.get(self.endpoint("/api/v1/folders")?))
            .query(&[("tenantId", tenant_id), ("ownerUserId", owner_user_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn create_tag(&self, payload: CreateTagPayload) -> CliResult<Tag> {
        Ok(self
            .authorize(self.http.post(self.endpoint("/api/v1/tags")?))
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_tags(&self, tenant_id: &str) -> CliResult<Vec<Tag>> {
        Ok(self
            .authorize(self.http.get(self.endpoint("/api/v1/tags")?))
            .query(&[("tenantId", tenant_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn create_subscription_group(
        &self,
        payload: CreateSubscriptionGroupPayload,
    ) -> CliResult<SubscriptionGroup> {
        Ok(self
            .authorize(
                self.http
                    .post(self.endpoint("/api/v1/subscription-groups")?),
            )
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_subscription_groups(
        &self,
        tenant_id: &str,
        owner_user_id: &str,
    ) -> CliResult<Vec<SubscriptionGroup>> {
        Ok(self
            .authorize(self.http.get(self.endpoint("/api/v1/subscription-groups")?))
            .query(&[("tenantId", tenant_id), ("ownerUserId", owner_user_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn add_pack_to_folder(
        &self,
        folder_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> CliResult<FolderPack> {
        Ok(self
            .authorize(
                self.http
                    .put(self.endpoint(&format!("/api/v1/folders/{folder_id}/packs/{pack_id}"))?),
            )
            .json(&UpsertPackMembershipPayload { sort_order })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_folder_pack_ids(&self, folder_id: &str) -> CliResult<Vec<String>> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint(&format!("/api/v1/folders/{folder_id}/packs"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn remove_pack_from_folder(&self, folder_id: &str, pack_id: &str) -> CliResult<()> {
        self.authorize(
            self.http
                .delete(self.endpoint(&format!("/api/v1/folders/{folder_id}/packs/{pack_id}"))?),
        )
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    async fn add_tag_to_pack(&self, pack_id: &str, tag_id: &str) -> CliResult<PackTag> {
        Ok(self
            .authorize(
                self.http
                    .put(self.endpoint(&format!("/api/v1/packs/{pack_id}/tags/{tag_id}"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_pack_tag_ids(&self, pack_id: &str) -> CliResult<Vec<String>> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint(&format!("/api/v1/packs/{pack_id}/tags"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn remove_tag_from_pack(&self, pack_id: &str, tag_id: &str) -> CliResult<()> {
        self.authorize(
            self.http
                .delete(self.endpoint(&format!("/api/v1/packs/{pack_id}/tags/{tag_id}"))?),
        )
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    async fn add_pack_to_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> CliResult<SubscriptionGroupPack> {
        Ok(self
            .authorize(self.http.put(self.endpoint(&format!(
                "/api/v1/subscription-groups/{subscription_group_id}/packs/{pack_id}"
            ))?))
            .json(&UpsertPackMembershipPayload { sort_order })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_subscription_group_pack_ids(
        &self,
        subscription_group_id: &str,
    ) -> CliResult<Vec<String>> {
        Ok(self
            .authorize(self.http.get(self.endpoint(&format!(
                "/api/v1/subscription-groups/{subscription_group_id}/packs"
            ))?))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn remove_pack_from_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
    ) -> CliResult<()> {
        self.authorize(self.http.delete(self.endpoint(&format!(
            "/api/v1/subscription-groups/{subscription_group_id}/packs/{pack_id}"
        ))?))
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    async fn create_subscription_access_token(
        &self,
        payload: CreateSubscriptionAccessTokenPayload,
    ) -> CliResult<CreatedSubscriptionAccessToken> {
        Ok(self
            .authorize(
                self.http
                    .post(self.endpoint("/api/v1/subscription-access-tokens")?),
            )
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_subscription_access_tokens(
        &self,
        user_id: &str,
    ) -> CliResult<Vec<SubscriptionAccessToken>> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint("/api/v1/subscription-access-tokens")?),
            )
            .query(&[("userId", user_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn rotate_subscription_access_token(
        &self,
        token_id: &str,
    ) -> CliResult<CreatedSubscriptionAccessToken> {
        Ok(self
            .authorize(self.http.patch(self.endpoint(&format!(
                "/api/v1/subscription-access-tokens/{token_id}/rotate"
            ))?))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn revoke_subscription_access_token(&self, token_id: &str) -> CliResult<()> {
        self.authorize(
            self.http
                .delete(self.endpoint(&format!("/api/v1/subscription-access-tokens/{token_id}"))?),
        )
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    async fn list_export_target_kinds(&self) -> CliResult<Vec<ExportTargetKind>> {
        Ok(self
            .authorize(self.http.get(self.endpoint("/api/v1/export-target-kinds")?))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_export_targets(&self, tenant_id: &str) -> CliResult<Vec<ExportTarget>> {
        Ok(self
            .authorize(self.http.get(self.endpoint("/api/v1/export-targets")?))
            .query(&[("tenantId", tenant_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn create_export_target(
        &self,
        payload: CreateExportTargetPayload,
    ) -> CliResult<ExportTarget> {
        Ok(self
            .authorize(self.http.post(self.endpoint("/api/v1/export-targets")?))
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn create_export_job(&self, payload: CreateExportJobPayload) -> CliResult<ExportJob> {
        Ok(self
            .authorize(self.http.post(self.endpoint("/api/v1/export-jobs")?))
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn get_export_job(&self, job_id: &str) -> CliResult<ExportJob> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint(&format!("/api/v1/export-jobs/{job_id}"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_export_job_events(&self, job_id: &str) -> CliResult<Vec<ExportJobEvent>> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint(&format!("/api/v1/export-jobs/{job_id}/events"))?),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn list_telegram_publications(
        &self,
        pack_id: &str,
    ) -> CliResult<Vec<TelegramPublication>> {
        Ok(self
            .authorize(
                self.http
                    .get(self.endpoint("/api/v1/telegram-publications")?),
            )
            .query(&[("packId", pack_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn get_telegram_publication(
        &self,
        publication_id: &str,
    ) -> CliResult<TelegramPublication> {
        Ok(self
            .authorize(
                self.http.get(
                    self.endpoint(&format!("/api/v1/telegram-publications/{publication_id}"))?,
                ),
            )
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
