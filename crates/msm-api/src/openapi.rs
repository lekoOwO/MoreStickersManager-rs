#![allow(clippy::needless_for_each)]

use axum::Json;
use utoipa::OpenApi;

use crate::{
    dto::{
        CreatePersonalAccessTokenRequest, CreatedPersonalAccessTokenResponse, HealthResponse,
        ImportPackRequest, PersonalAccessTokenResponse,
    },
    error::ApiErrorBody,
    routes::{assets, health, packs, pats},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::healthz,
        assets::read_asset,
        packs::import_pack,
        packs::list_packs,
        packs::export_pack,
        pats::create_pat,
        pats::list_pats,
        pats::revoke_pat
    ),
    components(schemas(
        HealthResponse,
        ApiErrorBody,
        ImportPackRequest,
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
