#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImportPackRequest {
    pub tenant_id: String,
    pub owner_user_id: String,
    pub pack_id: String,
    pub visibility: PackVisibilityDto,
    pub pack: serde_json::Value,
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
