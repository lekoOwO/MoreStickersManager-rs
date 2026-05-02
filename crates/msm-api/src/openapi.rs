#![allow(clippy::needless_for_each)]

use axum::Json;
use utoipa::OpenApi;

use crate::{
    dto::{HealthResponse, ImportPackRequest},
    error::ApiErrorBody,
    routes::{assets, health, packs},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::healthz,
        assets::read_asset,
        packs::import_pack,
        packs::list_packs,
        packs::export_pack
    ),
    components(schemas(HealthResponse, ApiErrorBody, ImportPackRequest)),
    tags((name = "system", description = "System endpoints"))
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
