use axum::{http::HeaderMap, Json};

use msm_domain::Permission;
use msm_providers::{
    line_sticker_pack_fetch_plan, telegram_sticker_set_fetch_plan, ProviderAssetDownloadStrategy,
    ProviderRemoteFetchPlan,
};

use crate::{
    auth::require_pat,
    dto::{
        CreateProviderImportPlanRequest, ProviderHttpHeaderResponse,
        ProviderHttpRequestPlanResponse, ProviderImportPlanResponse,
    },
    rbac::require_tenant_resource_access,
    ApiError, ApiResult, ApiState,
};
use axum::extract::State;

#[utoipa::path(
    post,
    path = "/api/v1/provider-imports/plan",
    tag = "provider-imports",
    request_body = CreateProviderImportPlanRequest,
    responses(
        (status = 200, description = "Provider import fetch plan", body = ProviderImportPlanResponse),
        (status = 400, description = "Unsupported provider or invalid remote ID", body = crate::error::ApiErrorBody),
        (status = 403, description = "PAT cannot import providers into this tenant", body = crate::error::ApiErrorBody)
    )
)]
/// Creates a provider import fetch plan for runtime execution.
///
/// # Errors
///
/// Returns an error when authorization fails, the provider is unsupported, or
/// the requested provider/base URL/remote ID cannot produce a safe fetch plan.
pub async fn create_provider_import_plan(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateProviderImportPlanRequest>,
) -> ApiResult<Json<ProviderImportPlanResponse>> {
    let pat = require_pat(&headers, &state, Permission::ProviderImport).await?;
    pat.require_user(&request.owner_user_id)?;
    require_tenant_resource_access(
        &state,
        &pat,
        &request.tenant_id,
        &request.owner_user_id,
        Permission::ProviderImport,
        "PAT user cannot import provider packs into this tenant",
    )
    .await?;

    let plan = match request.provider_id.as_str() {
        "telegram" => telegram_sticker_set_fetch_plan(
            request
                .base_url
                .as_deref()
                .unwrap_or("https://api.telegram.org"),
            &request.remote_id,
        ),
        "line-stickers" => line_sticker_pack_fetch_plan(
            request
                .base_url
                .as_deref()
                .unwrap_or("https://store.line.me"),
            &request.remote_id,
        ),
        other => {
            return Err(ApiError::BadRequest(format!(
                "unsupported provider import source: {other}"
            )));
        }
    }
    .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    Ok(Json(ProviderImportPlanResponse::from(plan)))
}

impl From<ProviderRemoteFetchPlan> for ProviderImportPlanResponse {
    fn from(plan: ProviderRemoteFetchPlan) -> Self {
        Self {
            provider_id: plan.provider_id,
            remote_id: plan.remote_id,
            metadata_request: ProviderHttpRequestPlanResponse {
                method: plan.metadata_request.method,
                url: plan.metadata_request.url,
                redacted_headers: plan
                    .metadata_request
                    .redacted_headers
                    .into_iter()
                    .map(|(name, value)| ProviderHttpHeaderResponse { name, value })
                    .collect(),
            },
            asset_strategy: match plan.asset_strategy {
                ProviderAssetDownloadStrategy::TelegramBotFileApi => "telegramBotFileApi",
                ProviderAssetDownloadStrategy::DirectRemoteUrls => "directRemoteUrls",
            }
            .to_owned(),
        }
    }
}
