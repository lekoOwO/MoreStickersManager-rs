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

#[derive(Clone, Copy, Debug, serde::Deserialize, utoipa::ToSchema)]
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
    pub options: serde_json::Value,
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
