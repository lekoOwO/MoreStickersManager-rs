use serde_json::{json, Value};

use crate::protocol::{
    CallToolResult, ListToolsResult, ToolAnnotations, ToolContent, ToolDefinition,
};

pub const LIST_STICKER_PACKS: &str = "msm.list_sticker_packs";
pub const EXPORT_STICKER_PACK: &str = "msm.export_sticker_pack";
pub const IMPORT_STICKER_PACK: &str = "msm.import_sticker_pack";
pub const CREATE_PROVIDER_IMPORT_PLAN: &str = "msm.create_provider_import_plan";
pub const CREATE_PROVIDER_IMPORT_JOB: &str = "msm.create_provider_import_job";
pub const GET_PROVIDER_IMPORT_JOB: &str = "msm.get_provider_import_job";
pub const LIST_PROVIDER_IMPORT_JOB_EVENTS: &str = "msm.list_provider_import_job_events";
pub const LIST_PROVIDER_CONFIGS: &str = "msm.list_provider_configs";
pub const UPSERT_PROVIDER_CONFIG: &str = "msm.upsert_provider_config";
pub const DELETE_PROVIDER_CONFIG: &str = "msm.delete_provider_config";
pub const UPDATE_STICKER_PACK: &str = "msm.update_sticker_pack";
pub const DELETE_STICKER_PACK: &str = "msm.delete_sticker_pack";
pub const LIST_FOLDERS: &str = "msm.list_folders";
pub const CREATE_FOLDER: &str = "msm.create_folder";
pub const LIST_FOLDER_PACKS: &str = "msm.list_folder_packs";
pub const ADD_PACK_TO_FOLDER: &str = "msm.add_pack_to_folder";
pub const REMOVE_PACK_FROM_FOLDER: &str = "msm.remove_pack_from_folder";
pub const LIST_TAGS: &str = "msm.list_tags";
pub const CREATE_TAG: &str = "msm.create_tag";
pub const LIST_PACK_TAGS: &str = "msm.list_pack_tags";
pub const ADD_TAG_TO_PACK: &str = "msm.add_tag_to_pack";
pub const REMOVE_TAG_FROM_PACK: &str = "msm.remove_tag_from_pack";
pub const LIST_SUBSCRIPTION_GROUPS: &str = "msm.list_subscription_groups";
pub const CREATE_SUBSCRIPTION_GROUP: &str = "msm.create_subscription_group";
pub const LIST_SUBSCRIPTION_GROUP_PACKS: &str = "msm.list_subscription_group_packs";
pub const ADD_PACK_TO_SUBSCRIPTION_GROUP: &str = "msm.add_pack_to_subscription_group";
pub const REMOVE_PACK_FROM_SUBSCRIPTION_GROUP: &str = "msm.remove_pack_from_subscription_group";
pub const CREATE_SUBSCRIPTION_LINK: &str = "msm.create_subscription_link";
pub const LIST_SUBSCRIPTION_LINKS: &str = "msm.list_subscription_links";
pub const ROTATE_SUBSCRIPTION_LINK: &str = "msm.rotate_subscription_link";
pub const REVOKE_SUBSCRIPTION_LINK: &str = "msm.revoke_subscription_link";
pub const GET_PAT_SCOPE_POLICY: &str = "msm.get_pat_scope_policy";
pub const LIST_TENANT_MEMBERS: &str = "msm.list_tenant_members";
pub const SET_TENANT_MEMBER_ROLE: &str = "msm.set_tenant_member_role";
pub const GET_TENANT_SETTINGS: &str = "msm.get_tenant_settings";
pub const UPDATE_TENANT_SETTINGS: &str = "msm.update_tenant_settings";
pub const SET_TENANT_USER_STATUS: &str = "msm.set_tenant_user_status";
pub const LIST_TENANT_ROLES: &str = "msm.list_tenant_roles";
pub const UPSERT_TENANT_ROLE: &str = "msm.upsert_tenant_role";
pub const LIST_OIDC_PROVIDERS: &str = "msm.list_oidc_providers";
pub const UPSERT_OIDC_PROVIDER: &str = "msm.upsert_oidc_provider";
pub const DELETE_OIDC_PROVIDER: &str = "msm.delete_oidc_provider";
pub const LIST_EXPORT_TARGET_KINDS: &str = "msm.list_export_target_kinds";
pub const LIST_EXPORT_TARGETS: &str = "msm.list_export_targets";
pub const CREATE_EXPORT_TARGET: &str = "msm.create_export_target";
pub const CREATE_EXPORT_JOB: &str = "msm.create_export_job";
pub const GET_EXPORT_JOB: &str = "msm.get_export_job";
pub const REQUEUE_EXPORT_JOB: &str = "msm.requeue_export_job";
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
            create_provider_import_plan_tool(),
            create_provider_import_job_tool(),
            get_provider_import_job_tool(),
            list_provider_import_job_events_tool(),
            list_provider_configs_tool(),
            upsert_provider_config_tool(),
            delete_provider_config_tool(),
            update_tool(),
            delete_tool(),
            list_folders_tool(),
            create_folder_tool(),
            list_folder_packs_tool(),
            add_pack_to_folder_tool(),
            remove_pack_from_folder_tool(),
            list_tags_tool(),
            create_tag_tool(),
            list_pack_tags_tool(),
            add_tag_to_pack_tool(),
            remove_tag_from_pack_tool(),
            list_subscription_groups_tool(),
            create_subscription_group_tool(),
            list_subscription_group_packs_tool(),
            add_pack_to_subscription_group_tool(),
            remove_pack_from_subscription_group_tool(),
            create_subscription_link_tool(),
            list_subscription_links_tool(),
            rotate_subscription_link_tool(),
            revoke_subscription_link_tool(),
            get_pat_scope_policy_tool(),
            list_tenant_members_tool(),
            set_tenant_member_role_tool(),
            get_tenant_settings_tool(),
            update_tenant_settings_tool(),
            set_tenant_user_status_tool(),
            list_tenant_roles_tool(),
            upsert_tenant_role_tool(),
            list_oidc_providers_tool(),
            upsert_oidc_provider_tool(),
            delete_oidc_provider_tool(),
            list_export_target_kinds_tool(),
            list_export_targets_tool(),
            create_export_target_tool(),
            create_export_job_tool(),
            get_export_job_tool(),
            requeue_export_job_tool(),
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

fn create_provider_import_plan_tool() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_PROVIDER_IMPORT_PLAN,
        title: "Create provider import plan",
        description:
            "Plan remote metadata and asset fetches for importing a provider sticker pack into MSM.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "ownerUserId": { "type": "string" },
                "providerId": {
                    "type": "string",
                    "enum": ["telegram", "line-stickers"]
                },
                "remoteId": {
                    "type": "string",
                    "description": "Provider-native sticker set or pack identifier."
                },
                "baseUrl": {
                    "type": "string",
                    "description": "Optional provider API/store base URL override for tests or mirrors."
                }
            }),
            &["tenantId", "ownerUserId", "providerId", "remoteId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn create_provider_import_job_tool() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_PROVIDER_IMPORT_JOB,
        title: "Create provider import job",
        description: "Queue a provider sticker pack import job for Telegram or LINE.",
        input_schema: object_schema(
            &json!({
                "id": { "type": "string" },
                "tenantId": { "type": "string" },
                "ownerUserId": { "type": "string" },
                "providerId": { "type": "string", "enum": ["telegram", "line-stickers"] },
                "remoteId": { "type": "string" },
                "targetPackId": { "type": "string" },
                "baseUrl": { "type": "string" }
            }),
            &["id", "tenantId", "ownerUserId", "providerId", "remoteId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn get_provider_import_job_tool() -> ToolDefinition {
    ToolDefinition {
        name: GET_PROVIDER_IMPORT_JOB,
        title: "Get provider import job",
        description: "Read a queued or completed provider import job.",
        input_schema: object_schema(&json!({ "jobId": { "type": "string" } }), &["jobId"]),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn list_provider_import_job_events_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_PROVIDER_IMPORT_JOB_EVENTS,
        title: "List provider import job events",
        description: "List ordered event records for a provider import job.",
        input_schema: object_schema(&json!({ "jobId": { "type": "string" } }), &["jobId"]),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn list_provider_configs_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_PROVIDER_CONFIGS,
        title: "List provider configs",
        description: "List tenant-scoped provider import credentials with secrets redacted.",
        input_schema: object_schema(&json!({ "tenantId": { "type": "string" } }), &["tenantId"]),
        annotations: read_only_annotations(),
    }
}

fn upsert_provider_config_tool() -> ToolDefinition {
    ToolDefinition {
        name: UPSERT_PROVIDER_CONFIG,
        title: "Upsert provider config",
        description: "Create or replace a tenant-scoped provider import credential/config.",
        input_schema: object_schema(
            &json!({
                "id": { "type": "string" },
                "tenantId": { "type": "string" },
                "providerId": { "type": "string", "enum": ["telegram", "line-stickers"] },
                "name": { "type": "string" },
                "config": { "type": "object" },
                "isEnabled": { "type": "boolean" }
            }),
            &[
                "id",
                "tenantId",
                "providerId",
                "name",
                "config",
                "isEnabled",
            ],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn delete_provider_config_tool() -> ToolDefinition {
    ToolDefinition {
        name: DELETE_PROVIDER_CONFIG,
        title: "Delete provider config",
        description: "Delete one tenant-scoped provider import credential/config.",
        input_schema: object_schema(&json!({ "id": { "type": "string" } }), &["id"]),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
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

fn list_folders_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_FOLDERS,
        title: "List folders",
        description: "List user-created sticker pack folders for one tenant owner.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "ownerUserId": { "type": "string" }
            }),
            &["tenantId", "ownerUserId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn create_folder_tool() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_FOLDER,
        title: "Create folder",
        description: "Create a user folder for sticker pack organization.",
        input_schema: object_schema(
            &json!({
                "id": { "type": "string" },
                "tenantId": { "type": "string" },
                "ownerUserId": { "type": "string" },
                "name": { "type": "string" }
            }),
            &["id", "tenantId", "ownerUserId", "name"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: false,
            open_world_hint: false,
        },
    }
}

fn list_folder_packs_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_FOLDER_PACKS,
        title: "List folder packs",
        description: "List sticker pack IDs assigned to one folder.",
        input_schema: object_schema(
            &json!({
                "folderId": { "type": "string" }
            }),
            &["folderId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn add_pack_to_folder_tool() -> ToolDefinition {
    ToolDefinition {
        name: ADD_PACK_TO_FOLDER,
        title: "Add pack to folder",
        description: "Assign an owned sticker pack to an owned folder.",
        input_schema: object_schema(
            &json!({
                "folderId": { "type": "string" },
                "packId": { "type": "string" },
                "sortOrder": { "type": "integer" }
            }),
            &["folderId", "packId", "sortOrder"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn remove_pack_from_folder_tool() -> ToolDefinition {
    ToolDefinition {
        name: REMOVE_PACK_FROM_FOLDER,
        title: "Remove pack from folder",
        description: "Remove a sticker pack assignment from a folder.",
        input_schema: object_schema(
            &json!({
                "folderId": { "type": "string" },
                "packId": { "type": "string" }
            }),
            &["folderId", "packId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn list_tags_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_TAGS,
        title: "List tags",
        description: "List sticker pack tags configured for one tenant.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" }
            }),
            &["tenantId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn create_tag_tool() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_TAG,
        title: "Create tag",
        description: "Create a sticker pack tag in one tenant.",
        input_schema: object_schema(
            &json!({
                "id": { "type": "string" },
                "tenantId": { "type": "string" },
                "name": { "type": "string" }
            }),
            &["id", "tenantId", "name"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: false,
            open_world_hint: false,
        },
    }
}

fn list_pack_tags_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_PACK_TAGS,
        title: "List pack tags",
        description: "List tag IDs assigned to one sticker pack.",
        input_schema: object_schema(
            &json!({
                "packId": { "type": "string" }
            }),
            &["packId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn add_tag_to_pack_tool() -> ToolDefinition {
    ToolDefinition {
        name: ADD_TAG_TO_PACK,
        title: "Add tag to pack",
        description: "Assign a tenant tag to an owned sticker pack.",
        input_schema: object_schema(
            &json!({
                "packId": { "type": "string" },
                "tagId": { "type": "string" }
            }),
            &["packId", "tagId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn remove_tag_from_pack_tool() -> ToolDefinition {
    ToolDefinition {
        name: REMOVE_TAG_FROM_PACK,
        title: "Remove tag from pack",
        description: "Remove a tag assignment from an owned sticker pack.",
        input_schema: object_schema(
            &json!({
                "packId": { "type": "string" },
                "tagId": { "type": "string" }
            }),
            &["packId", "tagId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn list_subscription_groups_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_SUBSCRIPTION_GROUPS,
        title: "List subscription groups",
        description: "List subscription groups owned by one user in one tenant.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "ownerUserId": { "type": "string" }
            }),
            &["tenantId", "ownerUserId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn create_subscription_group_tool() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_SUBSCRIPTION_GROUP,
        title: "Create subscription group",
        description: "Create a subscription group for selected sticker packs.",
        input_schema: object_schema(
            &json!({
                "id": { "type": "string" },
                "tenantId": { "type": "string" },
                "ownerUserId": { "type": "string" },
                "title": { "type": "string" },
                "visibility": { "type": "string", "enum": ["public", "private"] }
            }),
            &["id", "tenantId", "ownerUserId", "title", "visibility"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: false,
            open_world_hint: false,
        },
    }
}

fn list_subscription_group_packs_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_SUBSCRIPTION_GROUP_PACKS,
        title: "List subscription group packs",
        description: "List sticker pack IDs assigned to one subscription group.",
        input_schema: object_schema(
            &json!({
                "subscriptionGroupId": { "type": "string" }
            }),
            &["subscriptionGroupId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn add_pack_to_subscription_group_tool() -> ToolDefinition {
    ToolDefinition {
        name: ADD_PACK_TO_SUBSCRIPTION_GROUP,
        title: "Add pack to subscription group",
        description: "Assign an owned sticker pack to an owned subscription group.",
        input_schema: object_schema(
            &json!({
                "subscriptionGroupId": { "type": "string" },
                "packId": { "type": "string" },
                "sortOrder": { "type": "integer" }
            }),
            &["subscriptionGroupId", "packId", "sortOrder"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn remove_pack_from_subscription_group_tool() -> ToolDefinition {
    ToolDefinition {
        name: REMOVE_PACK_FROM_SUBSCRIPTION_GROUP,
        title: "Remove pack from subscription group",
        description: "Remove a sticker pack assignment from a subscription group.",
        input_schema: object_schema(
            &json!({
                "subscriptionGroupId": { "type": "string" },
                "packId": { "type": "string" }
            }),
            &["subscriptionGroupId", "packId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn create_subscription_link_tool() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_SUBSCRIPTION_LINK,
        title: "Create subscription link",
        description:
            "Create a pack or subscription-group access link and return its raw secret once.",
        input_schema: object_schema(
            &json!({
                "id": { "type": "string" },
                "resourceType": { "type": "string", "enum": ["pack", "subscriptionGroup"] },
                "resourceId": { "type": "string" }
            }),
            &["id", "resourceType", "resourceId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: false,
            open_world_hint: false,
        },
    }
}

fn list_subscription_links_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_SUBSCRIPTION_LINKS,
        title: "List subscription links",
        description:
            "List subscription access link metadata for a user without raw secrets or hashes.",
        input_schema: object_schema(
            &json!({
                "userId": { "type": "string" }
            }),
            &["userId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn rotate_subscription_link_tool() -> ToolDefinition {
    ToolDefinition {
        name: ROTATE_SUBSCRIPTION_LINK,
        title: "Rotate subscription link",
        description: "Rotate a subscription access link and return the new raw secret once.",
        input_schema: object_schema(
            &json!({
                "tokenId": { "type": "string" }
            }),
            &["tokenId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: false,
            open_world_hint: false,
        },
    }
}

fn revoke_subscription_link_tool() -> ToolDefinition {
    ToolDefinition {
        name: REVOKE_SUBSCRIPTION_LINK,
        title: "Revoke subscription link",
        description: "Revoke a subscription access link.",
        input_schema: object_schema(
            &json!({
                "tokenId": { "type": "string" }
            }),
            &["tokenId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn get_pat_scope_policy_tool() -> ToolDefinition {
    ToolDefinition {
        name: GET_PAT_SCOPE_POLICY,
        title: "Get PAT scope policy",
        description:
            "List role-allowed Personal Access Token scopes for one user. Requires pat.manage.",
        input_schema: object_schema(
            &json!({
                "userId": { "type": "string" }
            }),
            &["userId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn list_tenant_members_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_TENANT_MEMBERS,
        title: "List tenant members",
        description: "List members and roles for one MSM tenant. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" }
            }),
            &["tenantId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn set_tenant_member_role_tool() -> ToolDefinition {
    ToolDefinition {
        name: SET_TENANT_MEMBER_ROLE,
        title: "Set tenant member role",
        description: "Add or update a tenant member role. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "userId": { "type": "string" },
                "role": { "type": "string", "enum": ["admin", "user"] }
            }),
            &["tenantId", "userId", "role"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn get_tenant_settings_tool() -> ToolDefinition {
    ToolDefinition {
        name: GET_TENANT_SETTINGS,
        title: "Get tenant settings",
        description: "Read editable tenant settings. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" }
            }),
            &["tenantId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn update_tenant_settings_tool() -> ToolDefinition {
    ToolDefinition {
        name: UPDATE_TENANT_SETTINGS,
        title: "Update tenant settings",
        description: "Replace editable tenant settings. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "name": { "type": "string" },
                "publicAssetUrl": { "type": ["string", "null"] },
                "localRegistrationEnabled": { "type": "boolean" }
            }),
            &["tenantId", "name"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn set_tenant_user_status_tool() -> ToolDefinition {
    ToolDefinition {
        name: SET_TENANT_USER_STATUS,
        title: "Set tenant user status",
        description: "Enable or disable a tenant user. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "userId": { "type": "string" },
                "isDisabled": { "type": "boolean" }
            }),
            &["tenantId", "userId", "isDisabled"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn list_tenant_roles_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_TENANT_ROLES,
        title: "List tenant role templates",
        description: "List role templates for one MSM tenant. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" }
            }),
            &["tenantId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn upsert_tenant_role_tool() -> ToolDefinition {
    ToolDefinition {
        name: UPSERT_TENANT_ROLE,
        title: "Upsert tenant role template",
        description: "Add or update a tenant role template. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "roleId": { "type": "string" },
                "name": { "type": "string" },
                "permissions": { "type": "array", "items": { "type": "string" } }
            }),
            &["tenantId", "roleId", "name", "permissions"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn list_oidc_providers_tool() -> ToolDefinition {
    ToolDefinition {
        name: LIST_OIDC_PROVIDERS,
        title: "List OIDC providers",
        description:
            "List OIDC provider configurations for one tenant. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" }
            }),
            &["tenantId"],
        ),
        annotations: read_only_annotations(),
    }
}

fn upsert_oidc_provider_tool() -> ToolDefinition {
    ToolDefinition {
        name: UPSERT_OIDC_PROVIDER,
        title: "Upsert OIDC provider",
        description:
            "Add or update a tenant OIDC provider configuration. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "providerId": { "type": "string" },
                "displayName": { "type": "string" },
                "issuerUrl": { "type": "string" },
                "clientId": { "type": "string" },
                "clientSecret": { "type": "string" },
                "scopes": { "type": "array", "items": { "type": "string" } },
                "isEnabled": { "type": "boolean" },
                "allowRegistration": { "type": "boolean" }
            }),
            &[
                "tenantId",
                "providerId",
                "displayName",
                "issuerUrl",
                "clientId",
                "clientSecret",
                "scopes",
                "isEnabled",
            ],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
    }
}

fn delete_oidc_provider_tool() -> ToolDefinition {
    ToolDefinition {
        name: DELETE_OIDC_PROVIDER,
        title: "Delete OIDC provider",
        description:
            "Delete one tenant OIDC provider configuration. Requires tenant admin membership.",
        input_schema: object_schema(
            &json!({
                "tenantId": { "type": "string" },
                "providerId": { "type": "string" }
            }),
            &["tenantId", "providerId"],
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

fn requeue_export_job_tool() -> ToolDefinition {
    ToolDefinition {
        name: REQUEUE_EXPORT_JOB,
        title: "Requeue export job",
        description: "Requeue a failed or cancelled MSM export job for operator recovery.",
        input_schema: object_schema(
            &json!({
                "jobId": { "type": "string" }
            }),
            &["jobId"],
        ),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            idempotent_hint: false,
            open_world_hint: false,
        },
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
        list_tools_result, ADD_PACK_TO_FOLDER, ADD_PACK_TO_SUBSCRIPTION_GROUP, ADD_TAG_TO_PACK,
        CREATE_EXPORT_JOB, CREATE_EXPORT_TARGET, CREATE_FOLDER, CREATE_PROVIDER_IMPORT_JOB,
        CREATE_PROVIDER_IMPORT_PLAN, CREATE_SUBSCRIPTION_GROUP, CREATE_SUBSCRIPTION_LINK,
        CREATE_TAG, DELETE_OIDC_PROVIDER, DELETE_PROVIDER_CONFIG, DELETE_STICKER_PACK,
        EXPORT_STICKER_PACK, GET_EXPORT_JOB, GET_PAT_SCOPE_POLICY, GET_PROVIDER_IMPORT_JOB,
        GET_TELEGRAM_PUBLICATION, GET_TENANT_SETTINGS, IMPORT_STICKER_PACK, LIST_EXPORT_JOB_EVENTS,
        LIST_EXPORT_TARGETS, LIST_EXPORT_TARGET_KINDS, LIST_FOLDERS, LIST_FOLDER_PACKS,
        LIST_OIDC_PROVIDERS, LIST_PACK_TAGS, LIST_PROVIDER_CONFIGS,
        LIST_PROVIDER_IMPORT_JOB_EVENTS, LIST_STICKER_PACKS, LIST_SUBSCRIPTION_GROUPS,
        LIST_SUBSCRIPTION_GROUP_PACKS, LIST_SUBSCRIPTION_LINKS, LIST_TAGS,
        LIST_TELEGRAM_PUBLICATIONS, LIST_TENANT_MEMBERS, LIST_TENANT_ROLES,
        REMOVE_PACK_FROM_FOLDER, REMOVE_PACK_FROM_SUBSCRIPTION_GROUP, REMOVE_TAG_FROM_PACK,
        REQUEUE_EXPORT_JOB, REVOKE_SUBSCRIPTION_LINK, ROTATE_SUBSCRIPTION_LINK,
        SET_TENANT_MEMBER_ROLE, SET_TENANT_USER_STATUS, UPDATE_STICKER_PACK,
        UPDATE_TENANT_SETTINGS, UPSERT_OIDC_PROVIDER, UPSERT_PROVIDER_CONFIG, UPSERT_TENANT_ROLE,
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
                CREATE_PROVIDER_IMPORT_PLAN,
                CREATE_PROVIDER_IMPORT_JOB,
                GET_PROVIDER_IMPORT_JOB,
                LIST_PROVIDER_IMPORT_JOB_EVENTS,
                LIST_PROVIDER_CONFIGS,
                UPSERT_PROVIDER_CONFIG,
                DELETE_PROVIDER_CONFIG,
                UPDATE_STICKER_PACK,
                DELETE_STICKER_PACK,
                LIST_FOLDERS,
                CREATE_FOLDER,
                LIST_FOLDER_PACKS,
                ADD_PACK_TO_FOLDER,
                REMOVE_PACK_FROM_FOLDER,
                LIST_TAGS,
                CREATE_TAG,
                LIST_PACK_TAGS,
                ADD_TAG_TO_PACK,
                REMOVE_TAG_FROM_PACK,
                LIST_SUBSCRIPTION_GROUPS,
                CREATE_SUBSCRIPTION_GROUP,
                LIST_SUBSCRIPTION_GROUP_PACKS,
                ADD_PACK_TO_SUBSCRIPTION_GROUP,
                REMOVE_PACK_FROM_SUBSCRIPTION_GROUP,
                CREATE_SUBSCRIPTION_LINK,
                LIST_SUBSCRIPTION_LINKS,
                ROTATE_SUBSCRIPTION_LINK,
                REVOKE_SUBSCRIPTION_LINK,
                GET_PAT_SCOPE_POLICY,
                LIST_TENANT_MEMBERS,
                SET_TENANT_MEMBER_ROLE,
                GET_TENANT_SETTINGS,
                UPDATE_TENANT_SETTINGS,
                SET_TENANT_USER_STATUS,
                LIST_TENANT_ROLES,
                UPSERT_TENANT_ROLE,
                LIST_OIDC_PROVIDERS,
                UPSERT_OIDC_PROVIDER,
                DELETE_OIDC_PROVIDER,
                LIST_EXPORT_TARGET_KINDS,
                LIST_EXPORT_TARGETS,
                CREATE_EXPORT_TARGET,
                CREATE_EXPORT_JOB,
                GET_EXPORT_JOB,
                REQUEUE_EXPORT_JOB,
                LIST_EXPORT_JOB_EVENTS,
                LIST_TELEGRAM_PUBLICATIONS,
                GET_TELEGRAM_PUBLICATION,
            ]
        );
    }
}
