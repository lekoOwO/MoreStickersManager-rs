use serde_json::{json, Value};

use crate::protocol::{
    CallToolResult, ListToolsResult, ToolAnnotations, ToolContent, ToolDefinition,
};

pub const LIST_STICKER_PACKS: &str = "msm.list_sticker_packs";
pub const EXPORT_STICKER_PACK: &str = "msm.export_sticker_pack";
pub const IMPORT_STICKER_PACK: &str = "msm.import_sticker_pack";
pub const UPDATE_STICKER_PACK: &str = "msm.update_sticker_pack";
pub const DELETE_STICKER_PACK: &str = "msm.delete_sticker_pack";
pub const LIST_EXPORT_TARGET_KINDS: &str = "msm.list_export_target_kinds";
pub const LIST_EXPORT_TARGETS: &str = "msm.list_export_targets";
pub const CREATE_EXPORT_TARGET: &str = "msm.create_export_target";
pub const CREATE_EXPORT_JOB: &str = "msm.create_export_job";
pub const GET_EXPORT_JOB: &str = "msm.get_export_job";
pub const LIST_EXPORT_JOB_EVENTS: &str = "msm.list_export_job_events";
pub const LIST_TELEGRAM_PUBLICATIONS: &str = "msm.list_telegram_publications";
pub const GET_TELEGRAM_PUBLICATION: &str = "msm.get_telegram_publication";

#[must_use]
pub fn list_tools_result() -> ListToolsResult {
    ListToolsResult {
        tools: vec![
            list_tool(),
            export_tool(),
            import_tool(),
            update_tool(),
            delete_tool(),
            list_export_target_kinds_tool(),
            list_export_targets_tool(),
            create_export_target_tool(),
            create_export_job_tool(),
            get_export_job_tool(),
            list_export_job_events_tool(),
            list_telegram_publications_tool(),
            get_telegram_publication_tool(),
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

fn list_export_target_kinds_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_EXPORT_TARGET_KINDS,
        title: "List export target kinds",
        description: "List supported MSM export target kinds and capability metadata.",
        input_schema: object_schema(&json!({}), &[]),
        annotations: read_only_annotations(),
    }
}

fn list_export_targets_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_EXPORT_TARGETS,
        title: "List export targets",
        description: "List configured export targets for one MSM tenant.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" }
            }),
            &["tenantId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn create_export_target_tool() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_EXPORT_TARGET,
        title: "Create export target",
        description: "Create a configured MSM export target.",
        input_schema: object_schema(
            &json!({
                "id": { "type": "string" },
                "tenantId": { "type": "string" },
                "kind": { "type": "string" },
                "name": { "type": "string" },
                "config": { "type": "object" },
                "isEnabled": { "type": "boolean" }
            }),
            &["id", "tenantId", "kind", "name", "config", "isEnabled"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: false,
            open_world_hint: false,
        },
    }
}

fn create_export_job_tool() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_EXPORT_JOB,
        title: "Create export job",
        description: "Queue an MSM export job for a source sticker pack and export target.",
        input_schema: object_schema(
            &json!({
                "id": { "type": "string" },
                "tenantId": { "type": "string" },
                "sourcePackId": { "type": "string" },
                "targetId": { "type": "string" },
                "options": { "type": "object" },
                "telegramSetNameSlug": { "type": "string" },
                "telegramDefaultEmoji": { "type": "string" },
                "telegramDryRun": { "type": "boolean" },
                "telegramReconcileMode": {
                    "type": "string",
                    "enum": ["createOnly", "appendMissing", "mirror"]
                },
                "telegramExecuteReconciliation": { "type": "boolean" },
                "telegramAllowDestructiveReconciliation": { "type": "boolean" }
            }),
            &["id", "tenantId", "sourcePackId", "targetId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: false,
            open_world_hint: false,
        },
    }
}

fn get_export_job_tool() -> ToolDefinition {
    ToolDefinition {
        name: GET_EXPORT_JOB,
        title: "Get export job",
        description: "Read one MSM export job by ID.",
        input_schema: object_schema(
            &json!({
                "jobId": { "type": "string" }
            }),
            &["jobId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn list_export_job_events_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_EXPORT_JOB_EVENTS,
        title: "List export job events",
        description: "Read ordered events for one MSM export job.",
        input_schema: object_schema(
            &json!({
                "jobId": { "type": "string" }
            }),
            &["jobId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn list_telegram_publications_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_TELEGRAM_PUBLICATIONS,
        title: "List Telegram publications",
        description: "List persisted Telegram sticker set publications for one source pack.",
        input_schema: object_schema(
            &json!({
                "packId": { "type": "string" }
            }),
            &["packId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn get_telegram_publication_tool() -> ToolDefinition {
    ToolDefinition {
        name: GET_TELEGRAM_PUBLICATION,
        title: "Get Telegram publication",
        description: "Read one persisted Telegram sticker set publication by ID.",
        input_schema: object_schema(
            &json!({
                "publicationId": { "type": "string" }
            }),
            &["publicationId"],
        ),
        annotations: read_only_annotations(),
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
        list_tools_result, CREATE_EXPORT_JOB, CREATE_EXPORT_TARGET, DELETE_STICKER_PACK,
        EXPORT_STICKER_PACK, GET_EXPORT_JOB, GET_TELEGRAM_PUBLICATION, IMPORT_STICKER_PACK,
        LIST_EXPORT_JOB_EVENTS, LIST_EXPORT_TARGETS, LIST_EXPORT_TARGET_KINDS, LIST_STICKER_PACKS,
        LIST_TELEGRAM_PUBLICATIONS, UPDATE_STICKER_PACK,
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
                LIST_EXPORT_TARGET_KINDS,
                LIST_EXPORT_TARGETS,
                CREATE_EXPORT_TARGET,
                CREATE_EXPORT_JOB,
                GET_EXPORT_JOB,
                LIST_EXPORT_JOB_EVENTS,
                LIST_TELEGRAM_PUBLICATIONS,
                GET_TELEGRAM_PUBLICATION,
            ]
        );
    }
}
