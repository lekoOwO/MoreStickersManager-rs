use axum::{extract::State, http::StatusCode, Json};
use msm_api::ApiState;
use msm_domain::StickerPack;
use msm_storage::models::PackVisibility;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    protocol::{initialize_result, CallToolResult, JsonRpcRequest, JsonRpcResponse},
    tools::{
        execution_error_result, list_tools_result, success_result, EXPORT_STICKER_PACK,
        IMPORT_STICKER_PACK, LIST_STICKER_PACKS,
    },
};

pub async fn mcp_post(
    State(state): State<ApiState>,
    Json(request): Json<JsonRpcRequest>,
) -> (StatusCode, Json<JsonRpcResponse>) {
    let response = handle_mcp_message(state, request).await;
    (StatusCode::OK, Json(response))
}

pub async fn handle_mcp_message(state: ApiState, request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.unwrap_or(Value::Null);
    if request.jsonrpc != "2.0" {
        return JsonRpcResponse::error(id, -32600, "Invalid JSON-RPC version");
    }

    match request.method.as_str() {
        "initialize" => serialize_success(id, initialize_result()),
        "ping" => JsonRpcResponse::success(id, json!({})),
        "tools/list" => serialize_success(id, list_tools_result()),
        "tools/call" => call_tool(state, id, request.params).await,
        _ => JsonRpcResponse::error(id, -32601, "Method not found"),
    }
}

async fn call_tool(state: ApiState, id: Value, params: Option<Value>) -> JsonRpcResponse {
    let params = match parse_params::<CallToolParams>(params) {
        Ok(params) => params,
        Err(response) => return response.with_id(id),
    };
    let arguments = params.arguments.unwrap_or_else(|| json!({}));

    let result = match params.name.as_str() {
        LIST_STICKER_PACKS => list_sticker_packs(&state, arguments).await,
        EXPORT_STICKER_PACK => export_sticker_pack(&state, arguments).await,
        IMPORT_STICKER_PACK => import_sticker_pack(&state, arguments).await,
        _ => Err("Unknown tool".to_owned()),
    };

    match result {
        Ok(result) => serialize_success(id, result),
        Err(message) => serialize_success(id, execution_error_result(message)),
    }
}

async fn list_sticker_packs(state: &ApiState, arguments: Value) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ListStickerPacksArgs>(arguments)?;
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

async fn export_sticker_pack(state: &ApiState, arguments: Value) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ExportStickerPackArgs>(arguments)?;
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

async fn import_sticker_pack(state: &ApiState, arguments: Value) -> Result<CallToolResult, String> {
    let args = parse_arguments::<ImportStickerPackArgs>(arguments)?;
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

struct ErrorTemplate(i64, String);

impl ErrorTemplate {
    fn with_id(self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse::error(id, self.0, self.1)
    }
}
