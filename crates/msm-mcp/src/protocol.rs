use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    Success(JsonRpcSuccess),
    Error(JsonRpcErrorResponse),
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonRpcSuccess {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub result: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub error: JsonRpcErrorBody,
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonRpcErrorBody {
    pub code: i64,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: &'static str,
    pub capabilities: ServerCapabilities,
    pub server_info: Implementation,
    pub instructions: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct ServerCapabilities {
    pub tools: ToolsCapability,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    pub list_changed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Implementation {
    pub name: &'static str,
    pub title: &'static str,
    pub version: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsResult {
    pub tools: Vec<ToolDefinition>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    pub annotations: ToolAnnotations,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
    pub is_error: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
}

impl JsonRpcResponse {
    #[must_use]
    pub fn success(id: Value, result: Value) -> Self {
        Self::Success(JsonRpcSuccess {
            jsonrpc: "2.0",
            id,
            result,
        })
    }

    #[must_use]
    pub fn error(id: Value, code: i64, message: impl Into<String>) -> Self {
        Self::Error(JsonRpcErrorResponse {
            jsonrpc: "2.0",
            id,
            error: JsonRpcErrorBody {
                code,
                message: message.into(),
            },
        })
    }
}

#[must_use]
pub fn initialize_result() -> InitializeResult {
    InitializeResult {
        protocol_version: PROTOCOL_VERSION,
        capabilities: ServerCapabilities {
            tools: ToolsCapability {
                list_changed: false,
            },
        },
        server_info: Implementation {
            name: "morestickersmanager-rs",
            title: "MoreStickersManager",
            version: env!("CARGO_PKG_VERSION"),
        },
        instructions:
            "Use MSM tools to list, import, and export MoreStickers-compatible sticker packs.",
    }
}
