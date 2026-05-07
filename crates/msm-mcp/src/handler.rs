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
    ExportJobEventRecord, ExportJobRecord, ExportTargetRecord, NewExportJob, NewExportTarget,
    PackVisibility,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    protocol::{initialize_result, CallToolResult, JsonRpcRequest, JsonRpcResponse},
    tools::{
        execution_error_result, list_tools_result, success_result, CREATE_EXPORT_JOB,
        CREATE_EXPORT_TARGET, DELETE_STICKER_PACK, EXPORT_STICKER_PACK, GET_EXPORT_JOB,
        IMPORT_STICKER_PACK, LIST_EXPORT_JOB_EVENTS, LIST_EXPORT_TARGETS, LIST_EXPORT_TARGET_KINDS,
        LIST_STICKER_PACKS, UPDATE_STICKER_PACK,
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
        LIST_EXPORT_TARGET_KINDS => list_export_target_kinds(&state, headers, arguments).await,
        LIST_EXPORT_TARGETS => list_export_targets(&state, headers, arguments).await,
        CREATE_EXPORT_TARGET => create_export_target(&state, headers, arguments).await,
        CREATE_EXPORT_JOB => create_export_job(&state, headers, arguments).await,
        GET_EXPORT_JOB => get_export_job(&state, headers, arguments).await,
        LIST_EXPORT_JOB_EVENTS => list_export_job_events(&state, headers, arguments).await,
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

    let request_json = serde_json::to_string(&CreateExportJobRequest {
        id: args.id.clone(),
        tenant_id: args.tenant_id.clone(),
        source_pack_id: args.source_pack_id.clone(),
        target_id: args.target_id.clone(),
        options: args.options,
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

async fn require_tool_pat(
    state: &ApiState,
    headers: &HeaderMap,
    required: Permission,
) -> Result<VerifiedPat, String> {
    require_pat(headers, state, required)
        .await
        .map_err(auth_error_message)
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
    options: Value,
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

struct ErrorTemplate(i64, String);

impl ErrorTemplate {
    fn with_id(self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse::error(id, self.0, self.1)
    }
}
