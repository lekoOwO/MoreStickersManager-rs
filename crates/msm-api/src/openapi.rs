#![allow(clippy::needless_for_each)]

use axum::Json;
use utoipa::OpenApi;

use crate::{
    dto::{
        CreateExportJobRequest, CreateExportTargetRequest, CreateFolderRequest,
        CreatePersonalAccessTokenRequest, CreateSubscriptionAccessTokenRequest,
        CreateSubscriptionGroupRequest, CreateTagRequest, CreatedPersonalAccessTokenResponse,
        CreatedSubscriptionAccessTokenResponse, ExportJobEventResponse, ExportJobResponse,
        ExportTargetKindResponse, ExportTargetResponse, FolderPackResponse, FolderResponse,
        HealthResponse, ImportPackRequest, LocalUserResponse, LoginLocalUserRequest,
        PackTagResponse, PersonalAccessTokenResponse, RegisterLocalUserRequest,
        SubscriptionAccessResourceTypeDto, SubscriptionAccessTokenResponse,
        SubscriptionGroupPackResponse, SubscriptionGroupResponse, TagResponse,
        TelegramExportJobOptions, TelegramPublicationResponse, TelegramReconcileModeOption,
        UpdateExportTargetRequest, UpdatePackRequest, UpsertPackMembershipRequest,
    },
    error::ApiErrorBody,
    routes::{
        assets, auth, exports, health, metadata, packs, pats, subscription_access_tokens,
        subscriptions,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::healthz,
        auth::register_local_user,
        auth::login_local_user,
        assets::read_asset,
        subscriptions::public_pack_stickerpack,
        subscriptions::public_pack_subscription,
        subscriptions::public_subscription_group,
        packs::import_pack,
        packs::list_packs,
        packs::update_pack,
        packs::delete_pack,
        packs::export_pack,
        metadata::create_folder,
        metadata::list_folders,
        metadata::list_folder_pack_ids,
        metadata::add_pack_to_folder,
        metadata::remove_pack_from_folder,
        metadata::create_tag,
        metadata::list_tags,
        metadata::list_pack_tag_ids,
        metadata::add_tag_to_pack,
        metadata::remove_tag_from_pack,
        metadata::create_subscription_group,
        metadata::list_subscription_groups,
        metadata::list_subscription_group_pack_ids,
        metadata::add_pack_to_subscription_group,
        metadata::remove_pack_from_subscription_group,
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
        pats::revoke_pat,
        subscription_access_tokens::create_subscription_access_token,
        subscription_access_tokens::list_subscription_access_tokens,
        subscription_access_tokens::rotate_subscription_access_token,
        subscription_access_tokens::revoke_subscription_access_token
    ),
    components(schemas(
        HealthResponse,
        ApiErrorBody,
        ImportPackRequest,
        UpdatePackRequest,
        CreateFolderRequest,
        FolderResponse,
        UpsertPackMembershipRequest,
        FolderPackResponse,
        CreateTagRequest,
        TagResponse,
        PackTagResponse,
        CreateSubscriptionGroupRequest,
        SubscriptionGroupResponse,
        SubscriptionGroupPackResponse,
        RegisterLocalUserRequest,
        LoginLocalUserRequest,
        LocalUserResponse,
        ExportTargetKindResponse,
        CreateExportTargetRequest,
        UpdateExportTargetRequest,
        ExportTargetResponse,
        CreateExportJobRequest,
        TelegramExportJobOptions,
        TelegramReconcileModeOption,
        ExportJobResponse,
        ExportJobEventResponse,
        TelegramPublicationResponse,
        CreatePersonalAccessTokenRequest,
        CreatedPersonalAccessTokenResponse,
        PersonalAccessTokenResponse,
        SubscriptionAccessResourceTypeDto,
        CreateSubscriptionAccessTokenRequest,
        SubscriptionAccessTokenResponse,
        CreatedSubscriptionAccessTokenResponse
    )),
    tags((name = "system", description = "System endpoints"))
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
