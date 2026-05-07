#![allow(clippy::needless_for_each)]

use axum::Json;
use utoipa::OpenApi;

use crate::{
    dto::{
        CreateExportJobRequest, CreateExportTargetRequest, CreatePersonalAccessTokenRequest,
        CreatedPersonalAccessTokenResponse, ExportJobEventResponse, ExportJobResponse,
        ExportTargetKindResponse, ExportTargetResponse, HealthResponse, ImportPackRequest,
        LocalUserResponse, LoginLocalUserRequest, PersonalAccessTokenResponse,
        RegisterLocalUserRequest, TelegramPublicationResponse, UpdateExportTargetRequest,
        UpdatePackRequest,
    },
    error::ApiErrorBody,
    routes::{assets, auth, exports, health, packs, pats},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::healthz,
        auth::register_local_user,
        auth::login_local_user,
        assets::read_asset,
        packs::import_pack,
        packs::list_packs,
        packs::update_pack,
        packs::delete_pack,
        packs::export_pack,
        exports::list_target_kinds,
        exports::list_targets,
        exports::create_target,
        exports::update_target,
        exports::delete_target,
        exports::create_job,
        exports::get_job,
        exports::list_job_events,
        exports::list_telegram_publications,
        exports::get_telegram_publication,
        pats::create_pat,
        pats::list_pats,
        pats::revoke_pat
    ),
    components(schemas(
        HealthResponse,
        ApiErrorBody,
        ImportPackRequest,
        UpdatePackRequest,
        RegisterLocalUserRequest,
        LoginLocalUserRequest,
        LocalUserResponse,
        ExportTargetKindResponse,
        CreateExportTargetRequest,
        UpdateExportTargetRequest,
        ExportTargetResponse,
        CreateExportJobRequest,
        ExportJobResponse,
        ExportJobEventResponse,
        TelegramPublicationResponse,
        CreatePersonalAccessTokenRequest,
        CreatedPersonalAccessTokenResponse,
        PersonalAccessTokenResponse
    )),
    tags((name = "system", description = "System endpoints"))
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
