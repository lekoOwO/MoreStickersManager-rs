use serde_json::{json, Value};

use crate::protocol::{
    CallToolResult, ListToolsResult, ToolAnnotations, ToolContent, ToolDefinition,
};

pub const LIST_STICKER_PACKS: &str = "msm.list_sticker_packs";
pub const EXPORT_STICKER_PACK: &str = "msm.export_sticker_pack";
pub const IMPORT_STICKER_PACK: &str = "msm.import_sticker_pack";
pub const UPDATE_STICKER_PACK: &str = "msm.update_sticker_pack";
pub const DELETE_STICKER_PACK: &str = "msm.delete_sticker_pack";

#[must_use]
pub fn list_tools_result() -> ListToolsResult {
    ListToolsResult {
        tools: vec![
            list_tool(),
            export_tool(),
            import_tool(),
            update_tool(),
            delete_tool(),
        ],
    }
}

pub fn success_result(text: impl Into<String>, structured_content: Value) -> CallToolResult {
    CallToolResult {
        content: vec![ToolContent::Text { text: text.into() }],
        structured_content: Some(structured_content),
        is_error: false,
    }
}

pub fn execution_error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult {
        content: vec![ToolContent::Text {
            text: message.into(),
        }],
        structured_content: None,
        is_error: true,
    }
}

fn list_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_STICKER_PACKS,
        title: "List sticker packs",
        description: "List MoreStickers-compatible sticker packs owned by a user.",
        input_schema: object_schema(
            &json!({
                "userId": {
                    "type": "string",
                    "description": "MSM user ID whose sticker packs should be listed."
                }
            }),
            &["userId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn export_tool() -> ToolDefinition {
    ToolDefinition {
        name: EXPORT_STICKER_PACK,
        title: "Export sticker pack",
        description: "Export one MSM sticker pack as MoreStickers-compatible .stickerpack JSON.",
        input_schema: object_schema(
            &json!({
                "packId": {
                    "type": "string",
                    "description": "Internal MSM sticker pack ID."
                }
            }),
            &["packId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn import_tool() -> ToolDefinition {
    ToolDefinition {
        name: IMPORT_STICKER_PACK,
        title: "Import sticker pack",
        description: "Import a MoreStickers-compatible sticker pack into MSM storage.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "ownerUserId": { "type": "string" },
                "packId": { "type": "string" },
                "visibility": { "type": "string", "enum": ["public", "private"] },
                "pack": { "type": "object" }
            }),
            &["tenantId", "ownerUserId", "packId", "visibility", "pack"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn update_tool() -> ToolDefinition {
    ToolDefinition {
        name: UPDATE_STICKER_PACK,
        title: "Update sticker pack",
        description: "Rename an owned MSM sticker pack and update its visibility.",
        input_schema: object_schema(
            &json!({
                "packId": { "type": "string" },
                "title": { "type": "string" },
                "visibility": { "type": "string", "enum": ["public", "private"] }
            }),
            &["packId", "title", "visibility"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn delete_tool() -> ToolDefinition {
    ToolDefinition {
        name: DELETE_STICKER_PACK,
        title: "Delete sticker pack",
        description: "Delete an owned MSM sticker pack.",
        input_schema: object_schema(
            &json!({
                "packId": { "type": "string" }
            }),
            &["packId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn object_schema(properties: &Value, required: &[&'static str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

fn read_only_annotations() -> ToolAnnotations {
    ToolAnnotations {
        read_only_hint: true,
        destructive_hint: false,
        idempotent_hint: true,
        open_world_hint: false,
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::{
        list_tools_result, DELETE_STICKER_PACK, EXPORT_STICKER_PACK, IMPORT_STICKER_PACK,
        LIST_STICKER_PACKS, UPDATE_STICKER_PACK,
    };

    #[test]
    fn tool_registry_contains_pack_tools() {
        let tools = list_tools_result();
        let names = tools.tools.iter().map(|tool| tool.name).collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                LIST_STICKER_PACKS,
                EXPORT_STICKER_PACK,
                IMPORT_STICKER_PACK,
                UPDATE_STICKER_PACK,
                DELETE_STICKER_PACK,
            ]
        );
    }
}
