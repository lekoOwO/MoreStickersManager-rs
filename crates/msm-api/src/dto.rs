#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImportPackRequest {
    pub tenant_id: String,
    pub owner_user_id: String,
    pub pack_id: String,
    pub visibility: PackVisibilityDto,
    pub pack: serde_json::Value,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePackRequest {
    pub title: String,
    pub visibility: PackVisibilityDto,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum PackVisibilityDto {
    Public,
    Private,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ListPacksQuery {
    pub user_id: String,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ListFoldersQuery {
    pub tenant_id: String,
    pub owner_user_id: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderRequest {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub name: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderResponse {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPackMembershipRequest {
    pub sort_order: i64,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderPackResponse {
    pub folder_id: String,
    pub pack_id: String,
    pub sort_order: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ListTagsQuery {
    pub tenant_id: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TagResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PackTagResponse {
    pub pack_id: String,
    pub tag_id: String,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ListSubscriptionGroupsQuery {
    pub tenant_id: String,
    pub owner_user_id: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionGroupRequest {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub title: String,
    pub visibility: PackVisibilityDto,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionGroupResponse {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub title: String,
    pub visibility: PackVisibilityDto,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionGroupPackResponse {
    pub subscription_group_id: String,
    pub pack_id: String,
    pub sort_order: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterLocalUserRequest {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub tenant_id: Option<String>,
    pub tenant_name: Option<String>,
    pub tenant_role: Option<String>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LocalUserResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginLocalUserRequest {
    pub email: String,
    pub password: String,
    pub token_id: String,
    pub token_name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePersonalAccessTokenRequest {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatedPersonalAccessTokenResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub token: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PersonalAccessTokenResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ListPersonalAccessTokensQuery {
    pub user_id: String,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ListExportTargetsQuery {
    pub tenant_id: String,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ListTelegramPublicationsQuery {
    pub pack_id: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportTargetKindResponse {
    pub kind: String,
    pub display_name: String,
    pub supports_remote_publication: bool,
    pub supports_media_conversion: bool,
    pub requires_credentials: bool,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateExportTargetRequest {
    pub id: String,
    pub tenant_id: String,
    pub kind: String,
    pub name: String,
    pub config: serde_json::Value,
    pub is_enabled: bool,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExportTargetRequest {
    pub name: String,
    pub config: serde_json::Value,
    pub is_enabled: bool,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportTargetResponse {
    pub id: String,
    pub tenant_id: String,
    pub kind: String,
    pub name: String,
    pub config: serde_json::Value,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateExportJobRequest {
    pub id: String,
    pub tenant_id: String,
    pub source_pack_id: String,
    pub target_id: String,
    #[schema(value_type = TelegramExportJobOptions)]
    pub options: serde_json::Value,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TelegramExportJobOptions {
    /// Slug input used to build the Telegram sticker set name.
    pub set_name_slug: Option<String>,
    /// Telegram sticker set title. Defaults to the MSM pack title.
    pub set_title: Option<String>,
    /// Emoji used for stickers without provider-specific emoji metadata.
    pub default_emoji: Option<String>,
    /// Defaults to true. Set false to allow worker-side Telegram mutation.
    pub dry_run: Option<bool>,
    /// Remote reconciliation strategy for an existing Telegram sticker set.
    #[schema(inline)]
    pub reconcile_mode: Option<TelegramReconcileModeOption>,
    /// Required before reconciliation mutations are executed.
    pub execute_reconciliation: Option<bool>,
    /// Required in addition to executeReconciliation for mirror replace/delete.
    pub allow_destructive_reconciliation: Option<bool>,
    /// Optional caller-supplied Telegram remote state. If omitted, the worker can derive it from fetched Telegram metadata and stored mappings.
    pub remote_set: Option<serde_json::Value>,
    /// Existing Telegram set names used by create-only dry-run planning.
    pub existing_sticker_set_names: Option<Vec<String>>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum TelegramReconcileModeOption {
    CreateOnly,
    AppendMissing,
    Mirror,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportJobResponse {
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

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportJobEventResponse {
    pub job_id: String,
    pub sequence: i64,
    pub level: String,
    pub stage: String,
    pub message: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TelegramPublicationResponse {
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

impl From<PackVisibilityDto> for msm_storage::models::PackVisibility {
    fn from(value: PackVisibilityDto) -> Self {
        match value {
            PackVisibilityDto::Public => Self::Public,
            PackVisibilityDto::Private => Self::Private,
        }
    }
}

impl From<msm_storage::models::PackVisibility> for PackVisibilityDto {
    fn from(value: msm_storage::models::PackVisibility) -> Self {
        match value {
            msm_storage::models::PackVisibility::Public => Self::Public,
            msm_storage::models::PackVisibility::Private => Self::Private,
        }
    }
}

impl From<msm_storage::models::FolderRecord> for FolderResponse {
    fn from(record: msm_storage::models::FolderRecord) -> Self {
        Self {
            id: record.id,
            tenant_id: record.tenant_id,
            owner_user_id: record.owner_user_id,
            name: record.name,
            created_at: record.created_at.to_rfc3339(),
        }
    }
}

impl From<msm_storage::models::FolderPackRecord> for FolderPackResponse {
    fn from(record: msm_storage::models::FolderPackRecord) -> Self {
        Self {
            folder_id: record.folder_id,
            pack_id: record.pack_id,
            sort_order: record.sort_order,
        }
    }
}

impl From<msm_storage::models::TagRecord> for TagResponse {
    fn from(record: msm_storage::models::TagRecord) -> Self {
        Self {
            id: record.id,
            tenant_id: record.tenant_id,
            name: record.name,
            created_at: record.created_at.to_rfc3339(),
        }
    }
}

impl From<msm_storage::models::PackTagRecord> for PackTagResponse {
    fn from(record: msm_storage::models::PackTagRecord) -> Self {
        Self {
            pack_id: record.pack_id,
            tag_id: record.tag_id,
        }
    }
}

impl From<msm_storage::models::SubscriptionGroupRecord> for SubscriptionGroupResponse {
    fn from(record: msm_storage::models::SubscriptionGroupRecord) -> Self {
        Self {
            id: record.id,
            tenant_id: record.tenant_id,
            owner_user_id: record.owner_user_id,
            title: record.title,
            visibility: record.visibility.into(),
            created_at: record.created_at.to_rfc3339(),
        }
    }
}

impl From<msm_storage::models::SubscriptionGroupPackRecord> for SubscriptionGroupPackResponse {
    fn from(record: msm_storage::models::SubscriptionGroupPackRecord) -> Self {
        Self {
            subscription_group_id: record.subscription_group_id,
            pack_id: record.pack_id,
            sort_order: record.sort_order,
        }
    }
}

impl From<msm_storage::models::UserRecord> for LocalUserResponse {
    fn from(record: msm_storage::models::UserRecord) -> Self {
        Self {
            id: record.id,
            email: record.email,
            display_name: record.display_name,
        }
    }
}

impl From<msm_storage::models::PersonalAccessTokenRecord> for PersonalAccessTokenResponse {
    fn from(record: msm_storage::models::PersonalAccessTokenRecord) -> Self {
        Self {
            id: record.id,
            user_id: record.user_id,
            name: record.name,
            scopes: record
                .scopes
                .into_iter()
                .map(msm_domain::Permission::as_key)
                .map(ToOwned::to_owned)
                .collect(),
            expires_at: record.expires_at,
            revoked_at: record.revoked_at,
            created_at: record.created_at,
        }
    }
}

impl From<msm_storage::models::CreatedPersonalAccessToken> for CreatedPersonalAccessTokenResponse {
    fn from(created: msm_storage::models::CreatedPersonalAccessToken) -> Self {
        let record = PersonalAccessTokenResponse::from(created.record);
        Self {
            id: record.id,
            user_id: record.user_id,
            name: record.name,
            token: created.token,
            scopes: record.scopes,
            expires_at: record.expires_at,
            revoked_at: record.revoked_at,
            created_at: record.created_at,
        }
    }
}
