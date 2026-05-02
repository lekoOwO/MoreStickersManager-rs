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

impl From<PackVisibilityDto> for msm_storage::models::PackVisibility {
    fn from(value: PackVisibilityDto) -> Self {
        match value {
            PackVisibilityDto::Public => Self::Public,
            PackVisibilityDto::Private => Self::Private,
        }
    }
}
