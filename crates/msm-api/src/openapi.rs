#![allow(clippy::needless_for_each)]

use axum::Json;
use utoipa::OpenApi;

use crate::{
    dto::{
        CreatePersonalAccessTokenRequest, CreatedPersonalAccessTokenResponse, HealthResponse,
        ImportPackRequest, LocalUserResponse, LoginLocalUserRequest, PersonalAccessTokenResponse,
        RegisterLocalUserRequest,
    },
    error::ApiErrorBody,
    routes::{assets, auth, health, packs, pats},
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
        packs::export_pack,
        pats::create_pat,
        pats::list_pats,
        pats::revoke_pat
    ),
    components(schemas(
        HealthResponse,
        ApiErrorBody,
        ImportPackRequest,
        RegisterLocalUserRequest,
        LoginLocalUserRequest,
        LocalUserResponse,
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
