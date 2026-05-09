use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use msm_api::{
    auth::{require_pat, VerifiedPat},
    ApiError, ApiState,
};
use msm_domain::{Permission, StickerPack};
use msm_exporters::{
    ExportCapabilities, ExportTarget as ExportTargetTrait, ExportTargetKind,
    MoreStickersExportTarget,
};
use msm_storage::models::{
    ExportJobEventRecord, ExportJobRecord, ExportTargetRecord, FolderRecord, NewExportJob,
    NewExportTarget, NewTag, PackVisibility, RoleRecord, StickerPackRecord,
    SubscriptionAccessResourceType, SubscriptionAccessTokenRecord, SubscriptionGroupRecord,
    TagRecord, TelegramPublicationRecord, TenantMemberRecord, TenantRecord, UserRecord,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

use crate::{
    protocol::{initialize_result, CallToolResult, JsonRpcRequest, JsonRpcResponse},
    tools::{
        execution_error_result, list_tools_result, success_result, ADD_PACK_TO_FOLDER,
        ADD_PACK_TO_SUBSCRIPTION_GROUP, ADD_TAG_TO_PACK, CREATE_EXPORT_JOB, CREATE_EXPORT_TARGET,
        CREATE_FOLDER, CREATE_SUBSCRIPTION_GROUP, CREATE_SUBSCRIPTION_LINK, CREATE_TAG,
        DELETE_STICKER_PACK, EXPORT_STICKER_PACK, GET_EXPORT_JOB, GET_PAT_SCOPE_POLICY,
        GET_TELEGRAM_PUBLICATION, GET_TENANT_SETTINGS, IMPORT_STICKER_PACK, LIST_EXPORT_JOB_EVENTS,
        LIST_EXPORT_TARGETS, LIST_EXPORT_TARGET_KINDS, LIST_FOLDERS, LIST_FOLDER_PACKS,
        LIST_PACK_TAGS, LIST_STICKER_PACKS, LIST_SUBSCRIPTION_GROUPS,
        LIST_SUBSCRIPTION_GROUP_PACKS, LIST_SUBSCRIPTION_LINKS, LIST_TAGS,
        LIST_TELEGRAM_PUBLICATIONS, LIST_TENANT_MEMBERS, LIST_TENANT_ROLES,
        REMOVE_PACK_FROM_FOLDER, REMOVE_PACK_FROM_SUBSCRIPTION_GROUP, REMOVE_TAG_FROM_PACK,
        REVOKE_SUBSCRIPTION_LINK, ROTATE_SUBSCRIPTION_LINK, SET_TENANT_MEMBER_ROLE,
        SET_TENANT_USER_STATUS, UPDATE_STICKER_PACK, UPDATE_TENANT_SETTINGS, UPSERT_TENANT_ROLE,
    },
};

const REDACTED_SECRET: &str = "<redacted>";

pub async fn mcp_post(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> (StatusCode, Json<JsonRpcResponse>) {
    let response = handle_mcp_message_with_headers(state, headers, request).await;
    (StatusCode::OK, Json(response))
}

pub async fn handle_mcp_message(state: ApiState, request: JsonRpcRequest) -> JsonRpcResponse {
    handle_mcp_message_with_headers(state, HeaderMap::new(), request).await
}

async fn handle_mcp_message_with_headers(
    state: ApiState,
    headers: HeaderMap,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let id = request.id.unwrap_or(Value::Null);
    if request.jsonrpc != "2.0" {
        return JsonRpcResponse::error(id, -32600, "Invalid JSON-RPC version");
    }

    match request.method.as_str() {
        "initialize" => serialize_success(id, initialize_result()),
        "ping" => JsonRpcResponse::success(id, json!({})),
        "tools/list" => serialize_success(id, list_tools_result()),
        "tools/call" => call_tool(state, &headers, id, request.params).await,
        _ => JsonRpcResponse::error(id, -32601, "Method not found"),
    }
}

async fn call_tool(
    state: ApiState,
    headers: &HeaderMap,
    id: Value,
    params: Option<Value>,
) -> JsonRpcResponse {
    let params = match parse_params::<CallToolParams>(params) {
        Ok(params) => params,
        Err(response) => return response.with_id(id),
    };
    let arguments = params.arguments.unwrap_or_else(|| json!({}));

    let result = match params.name.as_str() {
        LIST_STICKER_PACKS => list_sticker_packs(&state, headers, arguments).await,
        EXPORT_STICKER_PACK => export_sticker_pack(&state, headers, arguments).await,
        IMPORT_STICKER_PACK => import_sticker_pack(&state, headers, arguments).await,
        UPDATE_STICKER_PACK => update_sticker_pack(&state, headers, arguments).await,
        DELETE_STICKER_PACK => delete_sticker_pack(&state, headers, arguments).await,
        LIST_FOLDERS => list_folders(&state, headers, arguments).await,
        CREATE_FOLDER => create_folder(&state, headers, arguments).await,
        LIST_FOLDER_PACKS => list_folder_packs(&state, headers, arguments).await,
        ADD_PACK_TO_FOLDER => add_pack_to_folder(&state, headers, arguments).await,
        REMOVE_PACK_FROM_FOLDER => remove_pack_from_folder(&state, headers, arguments).await,
        LIST_TAGS => list_tags(&state, headers, arguments).await,
        CREATE_TAG => create_tag(&state, headers, arguments).await,
        LIST_PACK_TAGS => list_pack_tags(&state, headers, arguments).await,
        ADD_TAG_TO_PACK => add_tag_to_pack(&state, headers, arguments).await,
        REMOVE_TAG_FROM_PACK => remove_tag_from_pack(&state, headers, arguments).await,
        LIST_SUBSCRIPTION_GROUPS => list_subscription_groups(&state, headers, arguments).await,
        CREATE_SUBSCRIPTION_GROUP => create_subscription_group(&state, headers, arguments).await,
        LIST_SUBSCRIPTION_GROUP_PACKS => {
            list_subscription_group_packs(&state, headers, arguments).await
        }
        ADD_PACK_TO_SUBSCRIPTION_GROUP => {
            add_pack_to_subscription_group(&state, headers, arguments).await
        }
        REMOVE_PACK_FROM_SUBSCRIPTION_GROUP => {
            remove_pack_from_subscription_group(&state, headers, arguments).await
        }
        CREATE_SUBSCRIPTION_LINK => create_subscription_link(&state, headers, arguments).await,
        LIST_SUBSCRIPTION_LINKS => list_subscription_links(&state, headers, arguments).await,
        ROTATE_SUBSCRIPTION_LINK => rotate_subscription_link(&state, headers, arguments).await,
        REVOKE_SUBSCRIPTION_LINK => revoke_subscription_link(&state, headers, arguments).await,
        GET_PAT_SCOPE_POLICY => get_pat_scope_policy(&state, headers, arguments).await,
        LIST_TENANT_MEMBERS => list_tenant_members(&state, headers, arguments).await,
        SET_TENANT_MEMBER_ROLE => set_tenant_member_role(&state, headers, arguments).await,
        GET_TENANT_SETTINGS => get_tenant_settings(&state, headers, arguments).await,
        UPDATE_TENANT_SETTINGS => update_tenant_settings(&state, headers, arguments).await,
        SET_TENANT_USER_STATUS => set_tenant_user_status(&state, headers, arguments).await,
        LIST_TENANT_ROLES => list_tenant_roles(&state, headers, arguments).await,
        UPSERT_TENANT_ROLE => upsert_tenant_role(&state, headers, arguments).await,
        LIST_EXPORT_TARGET_KINDS => list_export_target_kinds(&state, headers, arguments).await,
        LIST_EXPORT_TARGETS => list_export_targets(&state, headers, arguments).await,
        CREATE_EXPORT_TARGET => create_export_target(&state, headers, arguments).await,
        CREATE_EXPORT_JOB => create_export_job(&state, headers, arguments).await,
        GET_EXPORT_JOB => get_export_job(&state, headers, arguments).await,
        LIST_EXPORT_JOB_EVENTS => list_export_job_events(&state, headers, arguments).await,
        LIST_TELEGRAM_PUBLICATIONS => list_telegram_publications(&state, headers, arguments).await,
        GET_TELEGRAM_PUBLICATION => get_telegram_publication(&state, headers, arguments).await,
        _ => Err("Unknown tool".to_owned()),
    };

    match result {
        Ok(result) => serialize_success(id, result),
        Err(message) => serialize_success(id, execution_error_result(message)),
    }
}

async fn list_sticker_packs(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListStickerPacksArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackRead).await?;
    pat.require_user(&args.user_id)
        .map_err(auth_error_message)?;

    let packs = state
        .repository()
        .list_user_sticker_packs(&args.user_id)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Found {} sticker pack(s).", packs.len()),
        json!({ "packs": packs }),
    ))
}

async fn export_sticker_pack(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ExportStickerPackArgs>(arguments)?;
    let _pat = require_tool_pat(state, headers, Permission::PackRead).await?;
    let pack = state
        .repository()
        .find_sticker_pack(&args.pack_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Sticker pack `{}` was not found.", args.pack_id))?;

    Ok(success_result(
        format!("Exported sticker pack `{}`.", args.pack_id),
        json!({ "pack": pack }),
    ))
}

async fn import_sticker_pack(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ImportStickerPackArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::ImportRun).await?;
    pat.require_user(&args.owner_user_id)
        .map_err(auth_error_message)?;

    let visibility = match args.visibility.as_str() {
        "public" => PackVisibility::Public,
        "private" => PackVisibility::Private,
        _ => return Err("visibility must be `public` or `private`".to_owned()),
    };
    let pack: StickerPack = serde_json::from_value(args.pack).map_err(|error| error.to_string())?;

    state
        .repository()
        .upsert_sticker_pack(
            &args.pack_id,
            &args.tenant_id,
            &args.owner_user_id,
            visibility,
            None,
            &pack,
        )
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Imported sticker pack `{}`.", args.pack_id),
        json!({ "imported": true, "packId": args.pack_id }),
    ))
}

async fn update_sticker_pack(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<UpdateStickerPackArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let visibility = parse_visibility(&args.visibility)?;

    let updated = state
        .repository()
        .update_sticker_pack_metadata(&args.pack_id, &pat.user_id, &args.title, visibility)
        .await
        .map_err(|error| error.to_string())?;
    if !updated {
        return Err(format!("Sticker pack `{}` was not found.", args.pack_id));
    }

    Ok(success_result(
        format!("Updated sticker pack `{}`.", args.pack_id),
        json!({ "updated": true, "packId": args.pack_id }),
    ))
}

async fn delete_sticker_pack(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<DeleteStickerPackArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackDelete).await?;

    let deleted = state
        .repository()
        .delete_sticker_pack(&args.pack_id, &pat.user_id)
        .await
        .map_err(|error| error.to_string())?;
    if !deleted {
        return Err(format!("Sticker pack `{}` was not found.", args.pack_id));
    }

    Ok(success_result(
        format!("Deleted sticker pack `{}`.", args.pack_id),
        json!({ "deleted": true, "packId": args.pack_id }),
    ))
}

async fn list_folders(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListFoldersArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    pat.require_user(&args.owner_user_id)
        .map_err(auth_error_message)?;
    let folders = state
        .repository()
        .list_folders(&args.tenant_id, &args.owner_user_id)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|folder| {
            json!({
                "id": folder.id,
                "tenantId": folder.tenant_id,
                "ownerUserId": folder.owner_user_id,
                "name": folder.name,
                "createdAt": folder.created_at
            })
        })
        .collect::<Vec<_>>();

    Ok(success_result(
        format!("Found {} folder(s).", folders.len()),
        json!({ "folders": folders }),
    ))
}

async fn create_folder(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<CreateFolderArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    pat.require_user(&args.owner_user_id)
        .map_err(auth_error_message)?;
    let folder = state
        .repository()
        .create_folder(&args.id, &args.tenant_id, &args.owner_user_id, &args.name)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Created folder `{}`.", args.id),
        json!({
            "folder": {
                "id": folder.id,
                "tenantId": folder.tenant_id,
                "ownerUserId": folder.owner_user_id,
                "name": folder.name,
                "createdAt": folder.created_at
            }
        }),
    ))
}

async fn list_folder_packs(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<FolderPacksArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let _folder = require_owned_folder(state, &args.folder_id, &pat.user_id).await?;
    let pack_ids = state
        .repository()
        .list_folder_pack_ids(&args.folder_id)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Found {} folder pack(s).", pack_ids.len()),
        json!({ "packIds": pack_ids }),
    ))
}

async fn add_pack_to_folder(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<AddPackToFolderArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let folder = require_owned_folder(state, &args.folder_id, &pat.user_id).await?;
    let pack = load_owned_pack_record(state, &args.pack_id, &pat.user_id).await?;
    require_same_tenant(&folder.tenant_id, &pack.tenant_id)?;
    let link = state
        .repository()
        .add_pack_to_folder(&args.folder_id, &args.pack_id, args.sort_order)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!(
            "Added pack `{}` to folder `{}`.",
            args.pack_id, args.folder_id
        ),
        json!({
            "folderPack": {
                "folderId": link.folder_id,
                "packId": link.pack_id,
                "sortOrder": link.sort_order
            }
        }),
    ))
}

async fn remove_pack_from_folder(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<RemovePackFromFolderArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let folder = require_owned_folder(state, &args.folder_id, &pat.user_id).await?;
    let pack = load_owned_pack_record(state, &args.pack_id, &pat.user_id).await?;
    require_same_tenant(&folder.tenant_id, &pack.tenant_id)?;
    let removed = state
        .repository()
        .remove_pack_from_folder(&args.folder_id, &args.pack_id)
        .await
        .map_err(|error| error.to_string())?;
    require_removed(removed)?;

    Ok(success_result(
        format!(
            "Removed pack `{}` from folder `{}`.",
            args.pack_id, args.folder_id
        ),
        json!({ "removed": true, "folderId": args.folder_id, "packId": args.pack_id }),
    ))
}

async fn list_tags(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListTagsArgs>(arguments)?;
    let _pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let tags = state
        .repository()
        .list_tags(&args.tenant_id)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|tag| {
            json!({
                "id": tag.id,
                "tenantId": tag.tenant_id,
                "name": tag.name,
                "createdAt": tag.created_at
            })
        })
        .collect::<Vec<_>>();

    Ok(success_result(
        format!("Found {} tag(s).", tags.len()),
        json!({ "tags": tags }),
    ))
}

async fn create_tag(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<CreateTagArgs>(arguments)?;
    let _pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let tag = state
        .repository()
        .create_tag(NewTag {
            id: &args.id,
            tenant_id: &args.tenant_id,
            name: &args.name,
        })
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Created tag `{}`.", args.id),
        json!({
            "tag": {
                "id": tag.id,
                "tenantId": tag.tenant_id,
                "name": tag.name,
                "createdAt": tag.created_at
            }
        }),
    ))
}

async fn list_pack_tags(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<PackTagsArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let _pack = load_owned_pack_record(state, &args.pack_id, &pat.user_id).await?;
    let tag_ids = state
        .repository()
        .list_pack_tag_ids(&args.pack_id)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Found {} pack tag(s).", tag_ids.len()),
        json!({ "tagIds": tag_ids }),
    ))
}

async fn add_tag_to_pack(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<AddTagToPackArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let pack = load_owned_pack_record(state, &args.pack_id, &pat.user_id).await?;
    let tag = require_tag(state, &args.tag_id).await?;
    require_same_tenant(&pack.tenant_id, &tag.tenant_id)?;
    let link = state
        .repository()
        .add_tag_to_pack(&args.pack_id, &args.tag_id)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Added tag `{}` to pack `{}`.", args.tag_id, args.pack_id),
        json!({ "packTag": { "packId": link.pack_id, "tagId": link.tag_id } }),
    ))
}

async fn remove_tag_from_pack(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<RemoveTagFromPackArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PackUpdate).await?;
    let _pack = load_owned_pack_record(state, &args.pack_id, &pat.user_id).await?;
    let removed = state
        .repository()
        .remove_tag_from_pack(&args.pack_id, &args.tag_id)
        .await
        .map_err(|error| error.to_string())?;
    require_removed(removed)?;

    Ok(success_result(
        format!(
            "Removed tag `{}` from pack `{}`.",
            args.tag_id, args.pack_id
        ),
        json!({ "removed": true, "packId": args.pack_id, "tagId": args.tag_id }),
    ))
}

async fn list_subscription_groups(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListSubscriptionGroupsArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::SubscriptionRead).await?;
    pat.require_user(&args.owner_user_id)
        .map_err(auth_error_message)?;
    let groups = state
        .repository()
        .list_subscription_groups(&args.tenant_id, &args.owner_user_id)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|group| subscription_group_value(&group))
        .collect::<Vec<_>>();

    Ok(success_result(
        format!("Found {} subscription group(s).", groups.len()),
        json!({ "subscriptionGroups": groups }),
    ))
}

async fn create_subscription_group(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<CreateSubscriptionGroupArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::SubscriptionCreate).await?;
    pat.require_user(&args.owner_user_id)
        .map_err(auth_error_message)?;
    let visibility = parse_visibility(&args.visibility)?;
    let group = state
        .repository()
        .create_subscription_group(
            &args.id,
            &args.tenant_id,
            &args.owner_user_id,
            &args.title,
            visibility,
        )
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Created subscription group `{}`.", args.id),
        json!({ "subscriptionGroup": subscription_group_value(&group) }),
    ))
}

async fn list_subscription_group_packs(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<SubscriptionGroupPacksArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::SubscriptionRead).await?;
    let _group =
        require_owned_subscription_group(state, &args.subscription_group_id, &pat.user_id).await?;
    let pack_ids = state
        .repository()
        .list_subscription_pack_ids(&args.subscription_group_id)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Found {} subscription group pack(s).", pack_ids.len()),
        json!({ "packIds": pack_ids }),
    ))
}

async fn add_pack_to_subscription_group(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<AddPackToSubscriptionGroupArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::SubscriptionCreate).await?;
    let group =
        require_owned_subscription_group(state, &args.subscription_group_id, &pat.user_id).await?;
    let pack = load_owned_pack_record(state, &args.pack_id, &pat.user_id).await?;
    require_same_tenant(&group.tenant_id, &pack.tenant_id)?;
    let link = state
        .repository()
        .add_pack_to_subscription_group(&args.subscription_group_id, &args.pack_id, args.sort_order)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!(
            "Added pack `{}` to subscription group `{}`.",
            args.pack_id, args.subscription_group_id
        ),
        json!({
            "subscriptionGroupPack": {
                "subscriptionGroupId": link.subscription_group_id,
                "packId": link.pack_id,
                "sortOrder": link.sort_order
            }
        }),
    ))
}

async fn remove_pack_from_subscription_group(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<RemovePackFromSubscriptionGroupArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::SubscriptionCreate).await?;
    let group =
        require_owned_subscription_group(state, &args.subscription_group_id, &pat.user_id).await?;
    let pack = load_owned_pack_record(state, &args.pack_id, &pat.user_id).await?;
    require_same_tenant(&group.tenant_id, &pack.tenant_id)?;
    let removed = state
        .repository()
        .remove_pack_from_subscription_group(&args.subscription_group_id, &args.pack_id)
        .await
        .map_err(|error| error.to_string())?;
    require_removed(removed)?;

    Ok(success_result(
        format!(
            "Removed pack `{}` from subscription group `{}`.",
            args.pack_id, args.subscription_group_id
        ),
        json!({
            "removed": true,
            "subscriptionGroupId": args.subscription_group_id,
            "packId": args.pack_id
        }),
    ))
}

async fn create_subscription_link(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<CreateSubscriptionLinkArgs>(arguments)?;
    let (tenant_id, owner_user_id, resource_type) =
        authorize_subscription_link_create(state, headers, &args).await?;
    let created = state
        .repository()
        .create_subscription_access_token(
            &args.id,
            &tenant_id,
            &owner_user_id,
            resource_type,
            &args.resource_id,
        )
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Created subscription link `{}`.", args.id),
        json!({
            "subscriptionLink": subscription_access_token_value(&created.record),
            "token": created.token
        }),
    ))
}

async fn list_subscription_links(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListSubscriptionLinksArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::SubscriptionManageAccess).await?;
    pat.require_user(&args.user_id)
        .map_err(auth_error_message)?;
    let links = state
        .repository()
        .list_subscription_access_tokens(&args.user_id)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(subscription_access_token_value)
        .collect::<Vec<_>>();

    Ok(success_result(
        format!("Found {} subscription link(s).", links.len()),
        json!({ "subscriptionLinks": links }),
    ))
}

async fn rotate_subscription_link(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<SubscriptionLinkTokenArgs>(arguments)?;
    let record = require_subscription_access_token(state, &args.token_id).await?;
    authorize_subscription_link_existing(state, headers, &record).await?;
    let rotated = state
        .repository()
        .rotate_subscription_access_token(&args.token_id)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Rotated subscription link `{}`.", args.token_id),
        json!({
            "subscriptionLink": subscription_access_token_value(&rotated.record),
            "token": rotated.token
        }),
    ))
}

async fn revoke_subscription_link(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<SubscriptionLinkTokenArgs>(arguments)?;
    let record = require_subscription_access_token(state, &args.token_id).await?;
    authorize_subscription_link_existing(state, headers, &record).await?;
    state
        .repository()
        .revoke_subscription_access_token(&args.token_id)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Revoked subscription link `{}`.", args.token_id),
        json!({ "revoked": true, "tokenId": args.token_id }),
    ))
}

async fn get_pat_scope_policy(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<PatScopePolicyArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::PatManage).await?;
    pat.require_user(&args.user_id)
        .map_err(auth_error_message)?;

    let mut allowed_scopes = msm_api::allowed_pat_scopes_for_user(state, &args.user_id)
        .await
        .map_err(auth_error_message)?
        .into_iter()
        .map(Permission::as_key)
        .collect::<Vec<_>>();
    allowed_scopes.sort_unstable();

    Ok(success_result(
        format!("Found {} allowed PAT scope(s).", allowed_scopes.len()),
        json!({ "userId": args.user_id, "allowedScopes": allowed_scopes }),
    ))
}

async fn list_tenant_members(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListTenantMembersArgs>(arguments)?;
    let _pat = require_tenant_admin(
        state,
        headers,
        &args.tenant_id,
        Permission::TenantManageMembers,
    )
    .await?;
    let members = state
        .repository()
        .list_tenant_members(&args.tenant_id)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(tenant_member_value)
        .collect::<Vec<_>>();

    Ok(success_result(
        format!("Found {} tenant member(s).", members.len()),
        json!({ "members": members }),
    ))
}

async fn set_tenant_member_role(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<SetTenantMemberRoleArgs>(arguments)?;
    let _pat = require_tenant_admin(
        state,
        headers,
        &args.tenant_id,
        Permission::TenantManageMembers,
    )
    .await?;
    if !matches!(args.role.as_str(), "admin" | "user") {
        return Err("role must be `admin` or `user`".to_owned());
    }
    let member = state
        .repository()
        .upsert_tenant_member(&args.tenant_id, &args.user_id, &args.role)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!(
            "Set tenant member `{}` role to `{}`.",
            args.user_id, args.role
        ),
        json!({ "member": tenant_member_value(&member) }),
    ))
}

async fn get_tenant_settings(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<TenantIdArgs>(arguments)?;
    let _pat = require_tenant_admin(
        state,
        headers,
        &args.tenant_id,
        Permission::TenantManageSettings,
    )
    .await?;
    let settings = state
        .repository()
        .find_tenant(&args.tenant_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "tenant not found".to_owned())?;

    Ok(success_result(
        format!("Read tenant settings `{}`.", args.tenant_id),
        json!({ "settings": tenant_settings_value(&settings) }),
    ))
}

async fn update_tenant_settings(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<UpdateTenantSettingsArgs>(arguments)?;
    let _pat = require_tenant_admin(
        state,
        headers,
        &args.tenant_id,
        Permission::TenantManageSettings,
    )
    .await?;
    let settings = state
        .repository()
        .update_tenant_settings(
            &args.tenant_id,
            &args.name,
            args.public_asset_url.as_deref(),
        )
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Updated tenant settings `{}`.", args.tenant_id),
        json!({ "settings": tenant_settings_value(&settings) }),
    ))
}

async fn set_tenant_user_status(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<SetTenantUserStatusArgs>(arguments)?;
    let _pat = require_tenant_admin(
        state,
        headers,
        &args.tenant_id,
        Permission::TenantManageUsers,
    )
    .await?;
    state
        .repository()
        .find_tenant_member(&args.tenant_id, &args.user_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "tenant user not found".to_owned())?;
    let user = state
        .repository()
        .set_user_disabled(&args.user_id, args.is_disabled)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Updated tenant user `{}` status.", args.user_id),
        json!({ "user": tenant_user_value(&user) }),
    ))
}

async fn list_tenant_roles(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<TenantIdArgs>(arguments)?;
    let _pat = require_tenant_admin(
        state,
        headers,
        &args.tenant_id,
        Permission::TenantManageRoles,
    )
    .await?;
    let roles = state
        .repository()
        .list_role_templates(&args.tenant_id)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(tenant_role_value)
        .collect::<Vec<_>>();

    Ok(success_result(
        format!("Found {} tenant role template(s).", roles.len()),
        json!({ "roles": roles }),
    ))
}

async fn upsert_tenant_role(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<UpsertTenantRoleArgs>(arguments)?;
    let _pat = require_tenant_admin(
        state,
        headers,
        &args.tenant_id,
        Permission::TenantManageRoles,
    )
    .await?;
    let permissions = args
        .permissions
        .iter()
        .map(|key| Permission::from_key(key).ok_or_else(|| format!("unknown permission `{key}`")))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let role = state
        .repository()
        .upsert_role_template(&args.role_id, &args.tenant_id, &args.name, &permissions)
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Upserted tenant role `{}`.", args.role_id),
        json!({ "role": tenant_role_value(&role) }),
    ))
}

async fn list_export_target_kinds(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let _args = parse_arguments::<NoArgs>(arguments)?;
    let _pat = require_tool_pat(state, headers, Permission::ExportRead).await?;
    let target_kinds = export_target_capabilities();

    Ok(success_result(
        format!("Found {} export target kind(s).", target_kinds.len()),
        json!({ "targetKinds": target_kinds }),
    ))
}

async fn list_export_targets(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListExportTargetsArgs>(arguments)?;
    let _pat = require_tool_pat(state, headers, Permission::ExportRead).await?;
    let targets = state
        .repository()
        .list_export_targets(&args.tenant_id)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|target| export_target_value(&target))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(success_result(
        format!("Found {} export target(s).", targets.len()),
        json!({ "targets": targets }),
    ))
}

async fn create_export_target(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<CreateExportTargetArgs>(arguments)?;
    let _pat = require_tool_pat(state, headers, Permission::ExportTargetManage).await?;
    validate_export_target_kind(&args.kind)?;
    let config_json = serde_json::to_string(&args.config).map_err(|error| error.to_string())?;
    let target = state
        .repository()
        .create_export_target(NewExportTarget {
            id: &args.id,
            tenant_id: &args.tenant_id,
            kind: &args.kind,
            name: &args.name,
            config_json: &config_json,
            is_enabled: args.is_enabled,
        })
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Created export target `{}`.", args.id),
        json!({ "target": export_target_value(&target)? }),
    ))
}

async fn create_export_job(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<CreateExportJobArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::ExportRun).await?;
    let pack = state
        .repository()
        .find_sticker_pack_record(&args.source_pack_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Sticker pack `{}` was not found.", args.source_pack_id))?;
    if pack.owner_user_id != pat.user_id {
        return Err("PAT user does not own the source pack".to_owned());
    }
    if pack.tenant_id != args.tenant_id {
        return Err("source pack tenant does not match request tenant".to_owned());
    }

    let target = state
        .repository()
        .find_export_target(&args.target_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Export target `{}` was not found.", args.target_id))?;
    if target.tenant_id != args.tenant_id {
        return Err("export target tenant does not match request tenant".to_owned());
    }
    if !target.is_enabled {
        return Err("export target is disabled".to_owned());
    }

    let options = build_export_job_options(&args)?;
    let request_json = serde_json::to_string(&CreateExportJobRequest {
        id: args.id.clone(),
        tenant_id: args.tenant_id.clone(),
        source_pack_id: args.source_pack_id.clone(),
        target_id: args.target_id.clone(),
        options,
    })
    .map_err(|error| error.to_string())?;
    let job = state
        .repository()
        .create_export_job(NewExportJob {
            id: &args.id,
            tenant_id: &args.tenant_id,
            owner_user_id: &pat.user_id,
            source_pack_id: &args.source_pack_id,
            target_id: &args.target_id,
            request_json: &request_json,
            max_attempts: 3,
        })
        .await
        .map_err(|error| error.to_string())?;

    Ok(success_result(
        format!("Queued export job `{}`.", args.id),
        json!({ "job": export_job_value(&job)? }),
    ))
}

async fn get_export_job(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ExportJobArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::ExportRead).await?;
    let job = load_owned_export_job(state, &args.job_id, &pat.user_id).await?;

    Ok(success_result(
        format!("Read export job `{}`.", args.job_id),
        json!({ "job": export_job_value(&job)? }),
    ))
}

async fn list_export_job_events(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ExportJobArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::ExportRead).await?;
    let _job = load_owned_export_job(state, &args.job_id, &pat.user_id).await?;
    let events = state
        .repository()
        .list_export_job_events(&args.job_id)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|event| export_job_event_value(&event))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(success_result(
        format!("Found {} export job event(s).", events.len()),
        json!({ "events": events }),
    ))
}

async fn list_telegram_publications(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListTelegramPublicationsArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::ExportRead).await?;
    let _pack = load_owned_pack_record(state, &args.pack_id, &pat.user_id).await?;
    let publications = state
        .repository()
        .list_telegram_publications_for_pack(&args.pack_id)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|publication| telegram_publication_value(&publication))
        .collect::<Vec<_>>();

    Ok(success_result(
        format!("Found {} Telegram publication(s).", publications.len()),
        json!({ "publications": publications }),
    ))
}

async fn get_telegram_publication(
    state: &ApiState,
    headers: &HeaderMap,
    arguments: Value,
) -> Result<CallToolResult, String> {
    let args = parse_arguments::<GetTelegramPublicationArgs>(arguments)?;
    let pat = require_tool_pat(state, headers, Permission::ExportRead).await?;
    let publication = state
        .repository()
        .find_telegram_publication(&args.publication_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| {
            format!(
                "Telegram publication `{}` was not found.",
                args.publication_id
            )
        })?;
    let _pack = load_owned_pack_record(state, &publication.pack_id, &pat.user_id).await?;

    Ok(success_result(
        format!("Read Telegram publication `{}`.", args.publication_id),
        json!({ "publication": telegram_publication_value(&publication) }),
    ))
}

async fn require_tool_pat(
    state: &ApiState,
    headers: &HeaderMap,
    required: Permission,
) -> Result<VerifiedPat, String> {
    require_pat(headers, state, required)
        .await
        .map_err(auth_error_message)
}

async fn require_tenant_admin(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &str,
    required: Permission,
) -> Result<VerifiedPat, String> {
    let pat = require_tool_pat(state, headers, required).await?;
    let member = state
        .repository()
        .find_tenant_member(tenant_id, &pat.user_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "PAT user is not a tenant admin".to_owned())?;
    if member.role == "admin" {
        Ok(pat)
    } else {
        Err("PAT user is not a tenant admin".to_owned())
    }
}

fn auth_error_message(error: ApiError) -> String {
    match error {
        ApiError::Unauthorized(message) => format!("Personal Access Token unauthorized: {message}"),
        ApiError::Forbidden(message)
        | ApiError::BadRequest(message)
        | ApiError::NotFound(message)
        | ApiError::Internal(message) => message,
    }
}

fn serialize_success(id: Value, result: impl serde::Serialize) -> JsonRpcResponse {
    match serde_json::to_value(result) {
        Ok(value) => JsonRpcResponse::success(id, value),
        Err(error) => JsonRpcResponse::error(id, -32603, error.to_string()),
    }
}

fn parse_params<T: for<'de> Deserialize<'de>>(params: Option<Value>) -> Result<T, ErrorTemplate> {
    serde_json::from_value(params.unwrap_or_else(|| json!({})))
        .map_err(|error| ErrorTemplate(-32602, error.to_string()))
}

fn parse_arguments<T: for<'de> Deserialize<'de>>(arguments: Value) -> Result<T, String> {
    serde_json::from_value(arguments).map_err(|error| error.to_string())
}

fn parse_visibility(value: &str) -> Result<PackVisibility, String> {
    match value {
        "public" => Ok(PackVisibility::Public),
        "private" => Ok(PackVisibility::Private),
        _ => Err("visibility must be `public` or `private`".to_owned()),
    }
}

fn export_target_capabilities() -> Vec<Value> {
    let mut capabilities = vec![
        MoreStickersExportTarget.capabilities(),
        ExportCapabilities {
            kind: ExportTargetKind::new("telegram"),
            display_name: "Telegram".to_owned(),
            supports_remote_publication: true,
            supports_media_conversion: true,
            requires_credentials: true,
        },
    ];
    capabilities.sort_by(|left, right| left.kind.cmp(&right.kind));
    capabilities
        .into_iter()
        .map(|capability| {
            json!({
                "kind": capability.kind.as_str(),
                "displayName": capability.display_name,
                "supportsRemotePublication": capability.supports_remote_publication,
                "supportsMediaConversion": capability.supports_media_conversion,
                "requiresCredentials": capability.requires_credentials
            })
        })
        .collect()
}

fn validate_export_target_kind(kind: &str) -> Result<(), String> {
    if export_target_capabilities()
        .iter()
        .any(|capability| capability["kind"] == kind)
    {
        Ok(())
    } else {
        Err(format!("unknown export target kind: {kind}"))
    }
}

fn export_target_value(record: &ExportTargetRecord) -> Result<Value, String> {
    Ok(json!({
        "id": record.id,
        "tenantId": record.tenant_id,
        "kind": record.kind,
        "name": record.name,
        "config": redacted_config(&record.config_json)?,
        "isEnabled": record.is_enabled,
        "createdAt": record.created_at,
        "updatedAt": record.updated_at
    }))
}

fn export_job_value(record: &ExportJobRecord) -> Result<Value, String> {
    Ok(json!({
        "id": record.id,
        "tenantId": record.tenant_id,
        "ownerUserId": record.owner_user_id,
        "sourcePackId": record.source_pack_id,
        "targetId": record.target_id,
        "status": record.status.as_str(),
        "request": parse_json_value(&record.request_json)?,
        "result": record.result_json.as_deref().map(parse_json_value).transpose()?,
        "errorSummary": record.error_summary,
        "attemptCount": record.attempt_count,
        "maxAttempts": record.max_attempts,
        "nextAttemptAt": record.next_attempt_at,
        "createdAt": record.created_at,
        "updatedAt": record.updated_at
    }))
}

fn export_job_event_value(record: &ExportJobEventRecord) -> Result<Value, String> {
    Ok(json!({
        "jobId": record.job_id,
        "sequence": record.sequence,
        "level": record.level,
        "stage": record.stage,
        "message": record.message,
        "metadata": parse_json_value(&record.metadata_json)?,
        "createdAt": record.created_at
    }))
}

fn telegram_publication_value(record: &TelegramPublicationRecord) -> Value {
    json!({
        "id": record.id,
        "packId": record.pack_id,
        "targetId": record.target_id,
        "jobId": record.job_id,
        "stickerSetName": record.sticker_set_name,
        "stickerSetUrl": record.sticker_set_url,
        "stickerCount": record.sticker_count,
        "stickerType": record.sticker_type,
        "createdAt": record.created_at,
        "updatedAt": record.updated_at
    })
}

fn subscription_group_value(record: &SubscriptionGroupRecord) -> Value {
    json!({
        "id": record.id,
        "tenantId": record.tenant_id,
        "ownerUserId": record.owner_user_id,
        "title": record.title,
        "visibility": record.visibility.as_str(),
        "createdAt": record.created_at
    })
}

fn subscription_access_token_value(record: &SubscriptionAccessTokenRecord) -> Value {
    json!({
        "id": record.id,
        "tenantId": record.tenant_id,
        "ownerUserId": record.owner_user_id,
        "resourceType": subscription_access_resource_type_value(&record.resource_type),
        "resourceId": record.resource_id,
        "revokedAt": record.revoked_at,
        "createdAt": record.created_at,
        "updatedAt": record.updated_at
    })
}

fn tenant_member_value(record: &TenantMemberRecord) -> Value {
    json!({
        "tenantId": record.tenant_id,
        "userId": record.user_id,
        "role": record.role,
        "createdAt": record.created_at
    })
}

fn tenant_settings_value(record: &TenantRecord) -> Value {
    json!({
        "tenantId": record.id,
        "name": record.name,
        "publicAssetUrl": record.public_asset_url,
        "createdAt": record.created_at
    })
}

fn tenant_user_value(record: &UserRecord) -> Value {
    json!({
        "id": record.id,
        "email": record.email,
        "displayName": record.display_name,
        "isDisabled": record.is_disabled,
        "createdAt": record.created_at
    })
}

fn tenant_role_value(record: &RoleRecord) -> Value {
    let permissions = record
        .permissions
        .iter()
        .map(|permission| permission.as_key())
        .collect::<Vec<_>>();
    json!({
        "id": record.id,
        "tenantId": record.tenant_id,
        "name": record.name,
        "permissions": permissions,
        "createdAt": record.created_at
    })
}

fn subscription_access_resource_type_value(
    resource_type: &SubscriptionAccessResourceType,
) -> &'static str {
    match resource_type {
        SubscriptionAccessResourceType::Pack => "pack",
        SubscriptionAccessResourceType::SubscriptionGroup => "subscriptionGroup",
    }
}

async fn load_owned_export_job(
    state: &ApiState,
    job_id: &str,
    user_id: &str,
) -> Result<ExportJobRecord, String> {
    let job = state
        .repository()
        .find_export_job(job_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Export job `{job_id}` was not found."))?;
    if job.owner_user_id == user_id {
        Ok(job)
    } else {
        Err("PAT user does not own the export job".to_owned())
    }
}

async fn load_owned_pack_record(
    state: &ApiState,
    pack_id: &str,
    user_id: &str,
) -> Result<StickerPackRecord, String> {
    let pack = state
        .repository()
        .find_sticker_pack_record(pack_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Sticker pack `{pack_id}` was not found."))?;
    if pack.owner_user_id == user_id {
        Ok(pack)
    } else {
        Err("PAT user does not own the source pack".to_owned())
    }
}

async fn require_owned_folder(
    state: &ApiState,
    folder_id: &str,
    user_id: &str,
) -> Result<FolderRecord, String> {
    let folder = state
        .repository()
        .find_folder_record(folder_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Folder `{folder_id}` was not found."))?;
    if folder.owner_user_id == user_id {
        Ok(folder)
    } else {
        Err("PAT user does not own the folder".to_owned())
    }
}

async fn require_tag(state: &ApiState, tag_id: &str) -> Result<TagRecord, String> {
    state
        .repository()
        .find_tag_record(tag_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Tag `{tag_id}` was not found."))
}

async fn require_owned_subscription_group(
    state: &ApiState,
    subscription_group_id: &str,
    user_id: &str,
) -> Result<SubscriptionGroupRecord, String> {
    let group = state
        .repository()
        .find_subscription_group_record(subscription_group_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Subscription group `{subscription_group_id}` was not found."))?;
    if group.owner_user_id == user_id {
        Ok(group)
    } else {
        Err("PAT user does not own the subscription group".to_owned())
    }
}

async fn authorize_subscription_link_create(
    state: &ApiState,
    headers: &HeaderMap,
    args: &CreateSubscriptionLinkArgs,
) -> Result<(String, String, SubscriptionAccessResourceType), String> {
    match args.resource_type.as_str() {
        "pack" => {
            let pat = require_tool_pat(state, headers, Permission::PackManageAccess).await?;
            let pack = load_owned_pack_record(state, &args.resource_id, &pat.user_id).await?;
            Ok((
                pack.tenant_id,
                pack.owner_user_id,
                SubscriptionAccessResourceType::Pack,
            ))
        }
        "subscriptionGroup" => {
            let pat =
                require_tool_pat(state, headers, Permission::SubscriptionManageAccess).await?;
            let group =
                require_owned_subscription_group(state, &args.resource_id, &pat.user_id).await?;
            Ok((
                group.tenant_id,
                group.owner_user_id,
                SubscriptionAccessResourceType::SubscriptionGroup,
            ))
        }
        _ => Err("resourceType must be `pack` or `subscriptionGroup`".to_owned()),
    }
}

async fn authorize_subscription_link_existing(
    state: &ApiState,
    headers: &HeaderMap,
    record: &SubscriptionAccessTokenRecord,
) -> Result<(), String> {
    let permission = match record.resource_type {
        SubscriptionAccessResourceType::Pack => Permission::PackManageAccess,
        SubscriptionAccessResourceType::SubscriptionGroup => Permission::SubscriptionManageAccess,
    };
    let pat = require_tool_pat(state, headers, permission).await?;
    pat.require_user(&record.owner_user_id)
        .map_err(auth_error_message)
}

async fn require_subscription_access_token(
    state: &ApiState,
    token_id: &str,
) -> Result<SubscriptionAccessTokenRecord, String> {
    state
        .repository()
        .find_subscription_access_token(token_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Subscription link `{token_id}` was not found."))
}

fn require_same_tenant(left_tenant_id: &str, right_tenant_id: &str) -> Result<(), String> {
    if left_tenant_id == right_tenant_id {
        Ok(())
    } else {
        Err("membership resources must belong to the same tenant".to_owned())
    }
}

fn require_removed(removed: bool) -> Result<(), String> {
    if removed {
        Ok(())
    } else {
        Err("membership link was not found".to_owned())
    }
}

fn redacted_config(config_json: &str) -> Result<Value, String> {
    let mut config = parse_json_value(config_json)?;
    redact_secrets(&mut config);
    Ok(config)
}

fn parse_json_value(value: &str) -> Result<Value, String> {
    serde_json::from_str(value).map_err(|error| error.to_string())
}

fn redact_secrets(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let key = key.to_ascii_lowercase();
                if key.contains("token") || key.contains("secret") {
                    *value = Value::String(REDACTED_SECRET.to_owned());
                } else {
                    redact_secrets(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact_secrets),
        _ => {}
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CallToolParams {
    name: String,
    #[serde(default)]
    arguments: Option<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListStickerPacksArgs {
    user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportStickerPackArgs {
    pack_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportStickerPackArgs {
    tenant_id: String,
    owner_user_id: String,
    pack_id: String,
    visibility: String,
    pack: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateStickerPackArgs {
    pack_id: String,
    title: String,
    visibility: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteStickerPackArgs {
    pack_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListFoldersArgs {
    tenant_id: String,
    owner_user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateFolderArgs {
    id: String,
    tenant_id: String,
    owner_user_id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FolderPacksArgs {
    folder_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddPackToFolderArgs {
    folder_id: String,
    pack_id: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemovePackFromFolderArgs {
    folder_id: String,
    pack_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListTagsArgs {
    tenant_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTagArgs {
    id: String,
    tenant_id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackTagsArgs {
    pack_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddTagToPackArgs {
    pack_id: String,
    tag_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveTagFromPackArgs {
    pack_id: String,
    tag_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListSubscriptionGroupsArgs {
    tenant_id: String,
    owner_user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSubscriptionGroupArgs {
    id: String,
    tenant_id: String,
    owner_user_id: String,
    title: String,
    visibility: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscriptionGroupPacksArgs {
    subscription_group_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddPackToSubscriptionGroupArgs {
    subscription_group_id: String,
    pack_id: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemovePackFromSubscriptionGroupArgs {
    subscription_group_id: String,
    pack_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSubscriptionLinkArgs {
    id: String,
    resource_type: String,
    resource_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListSubscriptionLinksArgs {
    user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscriptionLinkTokenArgs {
    token_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatScopePolicyArgs {
    user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListTenantMembersArgs {
    tenant_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetTenantMemberRoleArgs {
    tenant_id: String,
    user_id: String,
    role: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TenantIdArgs {
    tenant_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTenantSettingsArgs {
    tenant_id: String,
    name: String,
    public_asset_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetTenantUserStatusArgs {
    tenant_id: String,
    user_id: String,
    is_disabled: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertTenantRoleArgs {
    tenant_id: String,
    role_id: String,
    name: String,
    permissions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NoArgs {}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListExportTargetsArgs {
    tenant_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateExportTargetArgs {
    id: String,
    tenant_id: String,
    kind: String,
    name: String,
    config: Value,
    is_enabled: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateExportJobArgs {
    id: String,
    tenant_id: String,
    source_pack_id: String,
    target_id: String,
    #[serde(default)]
    options: Value,
    #[serde(default)]
    telegram_set_name_slug: Option<String>,
    #[serde(default)]
    telegram_default_emoji: Option<String>,
    #[serde(default)]
    telegram_dry_run: Option<bool>,
    #[serde(default)]
    telegram_reconcile_mode: Option<String>,
    #[serde(default)]
    telegram_execute_reconciliation: Option<bool>,
    #[serde(default)]
    telegram_allow_destructive_reconciliation: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateExportJobRequest {
    id: String,
    tenant_id: String,
    source_pack_id: String,
    target_id: String,
    options: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportJobArgs {
    job_id: String,
}

fn build_export_job_options(args: &CreateExportJobArgs) -> Result<Value, String> {
    let mut options = if args.options.is_null() {
        json!({})
    } else {
        args.options.clone()
    };
    let Some(object) = options.as_object_mut() else {
        return Err("export job options must be an object".to_owned());
    };

    if let Some(value) = &args.telegram_set_name_slug {
        object.insert("setNameSlug".to_owned(), Value::String(value.clone()));
    }
    if let Some(value) = &args.telegram_default_emoji {
        object.insert("defaultEmoji".to_owned(), Value::String(value.clone()));
    }
    if let Some(value) = args.telegram_dry_run {
        object.insert("dryRun".to_owned(), Value::Bool(value));
    }
    if let Some(value) = &args.telegram_reconcile_mode {
        if !matches!(value.as_str(), "createOnly" | "appendMissing" | "mirror") {
            return Err(
                "telegramReconcileMode must be createOnly, appendMissing, or mirror".to_owned(),
            );
        }
        object.insert("reconcileMode".to_owned(), Value::String(value.clone()));
    }
    if let Some(value) = args.telegram_execute_reconciliation {
        object.insert("executeReconciliation".to_owned(), Value::Bool(value));
    }
    if let Some(value) = args.telegram_allow_destructive_reconciliation {
        object.insert(
            "allowDestructiveReconciliation".to_owned(),
            Value::Bool(value),
        );
    }

    Ok(options)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListTelegramPublicationsArgs {
    pack_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetTelegramPublicationArgs {
    publication_id: String,
}

struct ErrorTemplate(i64, String);

impl ErrorTemplate {
    fn with_id(self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse::error(id, self.0, self.1)
    }
}
