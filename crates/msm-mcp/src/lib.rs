#![doc = "MCP endpoint for MoreStickersManager-rs."]

pub mod handler;
pub mod protocol;
pub mod tools;

use axum::{routing::post, Router};
use msm_api::ApiState;

pub use handler::handle_mcp_message;

pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/mcp", post(handler::mcp_post))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use std::collections::BTreeSet;

    use msm_domain::{Permission, Sticker};
    use msm_storage::{
        models::{NewExportJobEvent, NewExportTarget, NewTag, NewTelegramPublication},
        DatabaseConfig, DbPool, LocalAssetStore, StorageRepository,
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use crate::build_router;

    #[tokio::test]
    async fn initialize_returns_capabilities() {
        let response = post_mcp(
            test_state().await,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": { "name": "test", "version": "0.0.0" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["protocolVersion"], "2025-06-18");
        assert!(response["result"]["capabilities"].get("tools").is_some());
    }

    #[tokio::test]
    async fn tools_list_returns_pack_and_export_tools() {
        let response = post_mcp(
            test_state().await,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }),
        )
        .await;

        let tools = response["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 44);
        assert_eq!(tools[0]["name"], "msm.list_sticker_packs");
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.get_pat_scope_policy"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
        assert!(tools.iter().any(|tool| tool["name"] == "msm.create_folder"
            && tool["inputSchema"]["required"].as_array().unwrap().len() == 4));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.add_pack_to_folder"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 3));
        assert!(tools.iter().any(|tool| tool["name"] == "msm.list_tags"
            && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.add_tag_to_pack"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 2));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.create_subscription_group"
                && tool["inputSchema"]["properties"]["visibility"]["enum"]
                    .as_array()
                    .unwrap()
                    .len()
                    == 2));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.add_pack_to_subscription_group"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 3));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.create_subscription_link"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 3));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.rotate_subscription_link"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.list_tenant_members"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.set_tenant_member_role"
                && tool["inputSchema"]["properties"]["role"]["enum"]
                    .as_array()
                    .unwrap()
                    .len()
                    == 2));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.get_tenant_settings"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.update_tenant_settings"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 2));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.set_tenant_user_status"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 3));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.list_tenant_roles"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.upsert_tenant_role"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 4));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.create_export_job"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 4
                && tool["inputSchema"]["properties"]["telegramReconcileMode"]["enum"]
                    .as_array()
                    .unwrap()
                    .len()
                    == 3));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.list_telegram_publications"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.get_telegram_publication"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
    }

    #[tokio::test]
    async fn tools_list_returns_oidc_provider_tools() {
        let response = post_mcp(
            test_state().await,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }),
        )
        .await;

        let tools = response["result"]["tools"].as_array().unwrap();
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.list_oidc_providers"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 1));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.upsert_oidc_provider"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 8));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.delete_oidc_provider"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 2));
    }

    #[tokio::test]
    async fn tools_list_returns_provider_import_plan_tool() {
        let response = post_mcp(
            test_state().await,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }),
        )
        .await;

        let tools = response["result"]["tools"].as_array().unwrap();
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.create_provider_import_plan"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 4
                && tool["inputSchema"]["properties"]["providerId"]["enum"]
                    .as_array()
                    .unwrap()
                    .len()
                    == 2));
    }

    #[tokio::test]
    async fn tools_call_lists_sticker_packs() {
        let state = seeded_state().await;
        let token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_sticker_packs",
                    "arguments": { "userId": "user_1" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(
            response["result"]["structuredContent"]["packs"][0]["title"],
            "Sample"
        );
    }

    #[tokio::test]
    async fn tools_call_gets_pat_scope_policy() {
        let state = empty_state_with_owner().await;
        let token = create_pat(&state, "patmanage", "user_1", [Permission::PatManage]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.get_pat_scope_policy",
                    "arguments": { "userId": "user_1" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        let scopes = response["result"]["structuredContent"]["allowedScopes"]
            .as_array()
            .unwrap();
        assert!(scopes.iter().any(|scope| scope == "pat.manage"));
        assert!(scopes.iter().any(|scope| scope == "tenant.manage_members"));
        assert!(!scopes.iter().any(|scope| scope == "system.configure"));
    }

    #[tokio::test]
    async fn tools_call_exports_sticker_pack() {
        let state = seeded_state().await;
        let token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.export_sticker_pack",
                    "arguments": { "packId": "pack_1" }
                }
            }),
        )
        .await;

        assert_eq!(
            response["result"]["structuredContent"]["pack"]["id"],
            "MoreStickers:Telegram:Pack:sample"
        );
    }

    #[tokio::test]
    async fn tools_call_imports_sticker_pack() {
        let state = empty_state_with_owner().await;
        let token = create_pat(&state, "patimport", "user_1", [Permission::ImportRun]).await;
        let pack = sample_pack();
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.import_sticker_pack",
                    "arguments": {
                        "tenantId": "tenant_1",
                        "ownerUserId": "user_1",
                        "packId": "pack_1",
                        "visibility": "private",
                        "pack": pack
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(response["result"]["structuredContent"]["imported"], true);
    }

    #[tokio::test]
    async fn tools_call_creates_provider_import_plan() {
        let state = empty_state_with_owner().await;
        let token = create_pat(
            &state,
            "patproviderimport",
            "user_1",
            [Permission::ProviderImport],
        )
        .await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_provider_import_plan",
                    "arguments": {
                        "tenantId": "tenant_1",
                        "ownerUserId": "user_1",
                        "providerId": "line-stickers",
                        "remoteId": "12345",
                        "baseUrl": "https://store.line.me"
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(
            response["result"]["structuredContent"]["plan"]["providerId"],
            "line-stickers"
        );
        assert_eq!(
            response["result"]["structuredContent"]["plan"]["assetStrategy"],
            "directRemoteUrls"
        );
        assert_eq!(
            response["result"]["structuredContent"]["plan"]["metadataRequest"]["url"],
            "https://store.line.me/stickershop/product/12345/en"
        );
    }

    #[tokio::test]
    async fn tools_call_updates_sticker_pack() {
        let state = seeded_state().await;
        let token = create_pat(&state, "patupdate", "user_1", [Permission::PackUpdate]).await;
        let response = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.update_sticker_pack",
                    "arguments": {
                        "packId": "pack_1",
                        "title": "Renamed Pack",
                        "visibility": "public"
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(response["result"]["structuredContent"]["updated"], true);
        assert_eq!(
            state
                .repository()
                .find_sticker_pack("pack_1")
                .await
                .unwrap()
                .unwrap()
                .title,
            "Renamed Pack"
        );
    }

    #[tokio::test]
    async fn tools_call_deletes_sticker_pack() {
        let state = seeded_state().await;
        let token = create_pat(&state, "patdelete", "user_1", [Permission::PackDelete]).await;
        let response = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.delete_sticker_pack",
                    "arguments": { "packId": "pack_1" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(response["result"]["structuredContent"]["deleted"], true);
        assert!(state
            .repository()
            .find_sticker_pack("pack_1")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn tools_call_lists_export_target_kinds() {
        let state = seeded_state().await;
        let token = create_pat(&state, "exportread", "user_1", [Permission::ExportRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_export_target_kinds",
                    "arguments": {}
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(
            response["result"]["structuredContent"]["targetKinds"][0]["kind"],
            "morestickers"
        );
    }

    #[tokio::test]
    async fn tools_call_creates_export_target_with_redacted_response() {
        let state = empty_state_with_owner().await;
        let token = create_pat(
            &state,
            "targetmanage",
            "user_1",
            [Permission::ExportTargetManage],
        )
        .await;
        let response = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_export_target",
                    "arguments": {
                        "id": "target_telegram",
                        "tenantId": "tenant_1",
                        "kind": "telegram",
                        "name": "Telegram",
                        "config": { "botToken": "123:secret" },
                        "isEnabled": true
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(
            response["result"]["structuredContent"]["target"]["config"]["botToken"],
            "<redacted>"
        );
        assert_eq!(
            state
                .repository()
                .find_export_target("target_telegram")
                .await
                .unwrap()
                .unwrap()
                .kind,
            "telegram"
        );
    }

    #[tokio::test]
    async fn tools_call_creates_export_job() {
        let state = seeded_state_with_export_target().await;
        let token = create_pat(&state, "exportrun", "user_1", [Permission::ExportRun]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_export_job",
                    "arguments": {
                        "id": "job_1",
                        "tenantId": "tenant_1",
                        "sourcePackId": "pack_1",
                        "targetId": "target_morestickers",
                        "options": { "format": "stickerpack" }
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(
            response["result"]["structuredContent"]["job"]["status"],
            "queued"
        );
        assert_eq!(
            response["result"]["structuredContent"]["job"]["attemptCount"],
            0
        );
        assert_eq!(
            response["result"]["structuredContent"]["job"]["maxAttempts"],
            3
        );
        assert_eq!(
            response["result"]["structuredContent"]["job"]["request"]["options"]["format"],
            "stickerpack"
        );
    }

    #[tokio::test]
    async fn tools_call_creates_telegram_reconciliation_job_without_raw_options() {
        let state = seeded_state_with_export_target().await;
        let token = create_pat(&state, "exportrun", "user_1", [Permission::ExportRun]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_export_job",
                    "arguments": {
                        "id": "job_telegram_reconcile",
                        "tenantId": "tenant_1",
                        "sourcePackId": "pack_1",
                        "targetId": "target_morestickers",
                        "telegramDryRun": false,
                        "telegramReconcileMode": "appendMissing",
                        "telegramExecuteReconciliation": true,
                        "telegramSetNameSlug": "sample",
                        "telegramDefaultEmoji": "ok"
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        let options = &response["result"]["structuredContent"]["job"]["request"]["options"];
        assert_eq!(options["dryRun"], false);
        assert_eq!(options["reconcileMode"], "appendMissing");
        assert_eq!(options["executeReconciliation"], true);
        assert_eq!(options["setNameSlug"], "sample");
        assert_eq!(options["defaultEmoji"], "ok");
    }

    #[tokio::test]
    async fn tools_call_reads_export_job_events() {
        let state = seeded_state_with_export_job().await;
        let token = create_pat(&state, "exportread", "user_1", [Permission::ExportRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_export_job_events",
                    "arguments": { "jobId": "job_1" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
        assert_eq!(
            response["result"]["structuredContent"]["events"][0]["message"],
            "job queued"
        );
    }

    #[tokio::test]
    async fn tools_call_reads_telegram_publications() {
        let state = seeded_state_with_publication().await;
        let token = create_pat(&state, "exportread", "user_1", [Permission::ExportRead]).await;
        let list_response = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_telegram_publications",
                    "arguments": { "packId": "pack_1" }
                }
            }),
        )
        .await;

        assert_eq!(list_response["result"]["isError"], false);
        assert_eq!(
            list_response["result"]["structuredContent"]["publications"][0]["stickerSetUrl"],
            "https://t.me/addstickers/sample_by_msm_bot"
        );

        let get_response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "msm.get_telegram_publication",
                    "arguments": { "publicationId": "telegram_pub_1" }
                }
            }),
        )
        .await;

        assert_eq!(get_response["result"]["isError"], false);
        assert_eq!(
            get_response["result"]["structuredContent"]["publication"]["stickerSetName"],
            "sample_by_msm_bot"
        );
    }

    #[tokio::test]
    async fn tools_call_manages_folders() {
        let state = empty_state_with_owner().await;
        let token = create_pat(&state, "foldermeta", "user_1", [Permission::PackUpdate]).await;

        let created = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_folder",
                    "arguments": {
                        "id": "folder_1",
                        "tenantId": "tenant_1",
                        "ownerUserId": "user_1",
                        "name": "Favorites"
                    }
                }
            }),
        )
        .await;
        assert_eq!(created["result"]["isError"], false);
        assert_eq!(
            created["result"]["structuredContent"]["folder"]["name"],
            "Favorites"
        );

        let listed = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_folders",
                    "arguments": { "tenantId": "tenant_1", "ownerUserId": "user_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            listed["result"]["structuredContent"]["folders"][0]["id"],
            "folder_1"
        );
    }

    #[tokio::test]
    async fn tools_call_manages_tags() {
        let state = empty_state_with_owner().await;
        let token = create_pat(&state, "tagmeta", "user_1", [Permission::PackUpdate]).await;

        let created = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_tag",
                    "arguments": { "id": "tag_1", "tenantId": "tenant_1", "name": "cute" }
                }
            }),
        )
        .await;
        assert_eq!(
            created["result"]["structuredContent"]["tag"]["name"],
            "cute"
        );

        let listed = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_tags",
                    "arguments": { "tenantId": "tenant_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            listed["result"]["structuredContent"]["tags"][0]["id"],
            "tag_1"
        );
    }

    #[tokio::test]
    async fn tools_call_manages_subscription_groups() {
        let state = empty_state_with_owner().await;
        let token = create_pat(
            &state,
            "submeta",
            "user_1",
            [Permission::SubscriptionCreate, Permission::SubscriptionRead],
        )
        .await;

        let created = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_subscription_group",
                    "arguments": {
                        "id": "sub_1",
                        "tenantId": "tenant_1",
                        "ownerUserId": "user_1",
                        "title": "Weekly",
                        "visibility": "private"
                    }
                }
            }),
        )
        .await;
        assert_eq!(
            created["result"]["structuredContent"]["subscriptionGroup"]["visibility"],
            "private"
        );

        let listed = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_subscription_groups",
                    "arguments": { "tenantId": "tenant_1", "ownerUserId": "user_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            listed["result"]["structuredContent"]["subscriptionGroups"][0]["title"],
            "Weekly"
        );
    }

    #[tokio::test]
    async fn tools_call_manages_folder_pack_memberships() {
        let state = seeded_state_with_product_metadata().await;
        let token = create_pat(&state, "foldermember", "user_1", [Permission::PackUpdate]).await;

        let folder_add = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.add_pack_to_folder",
                    "arguments": {
                        "folderId": "folder_1",
                        "packId": "pack_1",
                        "sortOrder": 10
                    }
                }
            }),
        )
        .await;
        assert_eq!(folder_add["result"]["isError"], false);
        assert_eq!(
            folder_add["result"]["structuredContent"]["folderPack"]["sortOrder"],
            10
        );

        let folder_list = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_folder_packs",
                    "arguments": { "folderId": "folder_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            folder_list["result"]["structuredContent"]["packIds"],
            json!(["pack_1"])
        );
    }

    #[tokio::test]
    async fn tools_call_manages_pack_tag_memberships() {
        let state = seeded_state_with_product_metadata().await;
        let token = create_pat(&state, "tagmember", "user_1", [Permission::PackUpdate]).await;

        let tag_add = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "msm.add_tag_to_pack",
                    "arguments": { "packId": "pack_1", "tagId": "tag_1" }
                }
            }),
        )
        .await;
        assert_eq!(tag_add["result"]["isError"], false);
        assert_eq!(
            tag_add["result"]["structuredContent"]["packTag"]["tagId"],
            "tag_1"
        );

        let tag_remove = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "msm.remove_tag_from_pack",
                    "arguments": { "packId": "pack_1", "tagId": "tag_1" }
                }
            }),
        )
        .await;
        assert_eq!(tag_remove["result"]["structuredContent"]["removed"], true);
    }

    #[tokio::test]
    async fn tools_call_manages_subscription_group_pack_memberships() {
        let state = seeded_state_with_product_metadata().await;
        let token = create_pat(
            &state,
            "groupmember",
            "user_1",
            [Permission::SubscriptionCreate, Permission::SubscriptionRead],
        )
        .await;

        let group_add = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "msm.add_pack_to_subscription_group",
                    "arguments": {
                        "subscriptionGroupId": "sub_1",
                        "packId": "pack_1",
                        "sortOrder": 20
                    }
                }
            }),
        )
        .await;
        assert_eq!(
            group_add["result"]["structuredContent"]["subscriptionGroupPack"]["sortOrder"],
            20
        );

        let group_remove = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {
                    "name": "msm.remove_pack_from_subscription_group",
                    "arguments": { "subscriptionGroupId": "sub_1", "packId": "pack_1" }
                }
            }),
        )
        .await;
        assert_eq!(group_remove["result"]["structuredContent"]["removed"], true);
    }

    #[tokio::test]
    async fn tools_call_manages_subscription_links() {
        let state = seeded_state_with_product_metadata().await;
        let token = create_pat(
            &state,
            "linkmanage",
            "user_1",
            [
                Permission::PackManageAccess,
                Permission::SubscriptionManageAccess,
            ],
        )
        .await;

        let create = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_subscription_link",
                    "arguments": {
                        "id": "packlink",
                        "resourceType": "pack",
                        "resourceId": "pack_1"
                    }
                }
            }),
        )
        .await;
        assert_eq!(create["result"]["isError"], false);
        assert!(create["result"]["structuredContent"]["token"]
            .as_str()
            .unwrap()
            .starts_with("msm_sub_packlink_"));
        assert_eq!(
            create["result"]["structuredContent"]["subscriptionLink"]["resourceType"],
            "pack"
        );
        assert!(create["result"]["structuredContent"]["subscriptionLink"]
            .get("tokenHash")
            .is_none());

        let list = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 8,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_subscription_links",
                    "arguments": { "userId": "user_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            list["result"]["structuredContent"]["subscriptionLinks"][0]["id"],
            "packlink"
        );
        assert!(list["result"]["structuredContent"]["subscriptionLinks"][0]
            .get("token")
            .is_none());

        let rotate = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 9,
                "method": "tools/call",
                "params": {
                    "name": "msm.rotate_subscription_link",
                    "arguments": { "tokenId": "packlink" }
                }
            }),
        )
        .await;
        assert_eq!(rotate["result"]["isError"], false);
        assert!(rotate["result"]["structuredContent"]["token"]
            .as_str()
            .unwrap()
            .starts_with("msm_sub_packlink_"));

        let revoke = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 10,
                "method": "tools/call",
                "params": {
                    "name": "msm.revoke_subscription_link",
                    "arguments": { "tokenId": "packlink" }
                }
            }),
        )
        .await;
        assert_eq!(revoke["result"]["structuredContent"]["revoked"], true);
    }

    #[tokio::test]
    async fn tools_call_manages_tenant_members() {
        let state = empty_state_with_owner().await;
        state
            .repository()
            .create_user("user_2", "member@example.com", "Member")
            .await
            .unwrap();
        let token = create_pat(
            &state,
            "tenantmembers",
            "user_1",
            [Permission::TenantManageMembers],
        )
        .await;

        let set_role = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 11,
                "method": "tools/call",
                "params": {
                    "name": "msm.set_tenant_member_role",
                    "arguments": { "tenantId": "tenant_1", "userId": "user_2", "role": "user" }
                }
            }),
        )
        .await;
        assert_eq!(set_role["result"]["isError"], false);
        assert_eq!(
            set_role["result"]["structuredContent"]["member"]["role"],
            "user"
        );

        let listed = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 12,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_tenant_members",
                    "arguments": { "tenantId": "tenant_1" }
                }
            }),
        )
        .await;
        assert_eq!(listed["result"]["isError"], false);
        assert_eq!(
            listed["result"]["structuredContent"]["members"][1]["userId"],
            "user_2"
        );
    }

    #[tokio::test]
    async fn pat_enforcement_tenant_member_tools_require_admin_membership() {
        let state = empty_state_with_owner().await;
        state
            .repository()
            .create_user("user_2", "member@example.com", "Member")
            .await
            .unwrap();
        state
            .repository()
            .add_tenant_member("tenant_1", "user_2", "user")
            .await
            .unwrap();
        let token = create_pat(
            &state,
            "tenantmembers",
            "user_2",
            [Permission::TenantManageMembers],
        )
        .await;

        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 13,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_tenant_members",
                    "arguments": { "tenantId": "tenant_1" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("tenant admin"));
    }

    #[tokio::test]
    async fn tools_call_manages_tenant_settings() {
        let state = empty_state_with_owner().await;
        let token = create_pat(
            &state,
            "tenantsettings",
            "user_1",
            [Permission::TenantManageSettings],
        )
        .await;

        let update_settings = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 14,
                "method": "tools/call",
                "params": {
                    "name": "msm.update_tenant_settings",
                    "arguments": {
                        "tenantId": "tenant_1",
                        "name": "Production",
                        "publicAssetUrl": "https://cdn.example.test/msm",
                        "localRegistrationEnabled": false
                    }
                }
            }),
        )
        .await;
        assert_eq!(
            update_settings["result"]["structuredContent"]["settings"]["publicAssetUrl"],
            "https://cdn.example.test/msm"
        );
        assert_eq!(
            update_settings["result"]["structuredContent"]["settings"]["localRegistrationEnabled"],
            false
        );

        let get_settings = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 15,
                "method": "tools/call",
                "params": {
                    "name": "msm.get_tenant_settings",
                    "arguments": { "tenantId": "tenant_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            get_settings["result"]["structuredContent"]["settings"]["name"],
            "Production"
        );
    }

    #[tokio::test]
    async fn tools_call_manages_oidc_providers() {
        let state = empty_state_with_owner().await;
        let token = create_pat(
            &state,
            "oidcproviders",
            "user_1",
            [Permission::TenantManageSettings],
        )
        .await;

        let upsert = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 40,
                "method": "tools/call",
                "params": {
                    "name": "msm.upsert_oidc_provider",
                    "arguments": {
                        "tenantId": "tenant_1",
                        "providerId": "google",
                        "displayName": "Google Workspace",
                        "issuerUrl": "https://accounts.google.com",
                        "clientId": "client_1",
                        "clientSecret": "secret_1",
                        "scopes": ["openid", "email"],
                        "isEnabled": true,
                        "allowRegistration": false
                    }
                }
            }),
        )
        .await;
        assert_eq!(upsert["result"]["isError"], false);
        assert_eq!(
            upsert["result"]["structuredContent"]["provider"]["clientSecret"],
            "[redacted]"
        );
        assert_eq!(
            upsert["result"]["structuredContent"]["provider"]["allowRegistration"],
            false
        );

        let list = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 41,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_oidc_providers",
                    "arguments": { "tenantId": "tenant_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            list["result"]["structuredContent"]["providers"][0]["id"],
            "google"
        );
        assert_eq!(
            list["result"]["structuredContent"]["providers"][0]["clientSecret"],
            "[redacted]"
        );

        let deleted = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 42,
                "method": "tools/call",
                "params": {
                    "name": "msm.delete_oidc_provider",
                    "arguments": { "tenantId": "tenant_1", "providerId": "google" }
                }
            }),
        )
        .await;
        assert_eq!(deleted["result"]["structuredContent"]["deleted"], true);

        let list_after_delete = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 43,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_oidc_providers",
                    "arguments": { "tenantId": "tenant_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            list_after_delete["result"]["structuredContent"]["providers"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn tools_call_sets_tenant_user_status() {
        let state = empty_state_with_owner().await;
        state
            .repository()
            .create_user("user_2", "member@example.com", "Member")
            .await
            .unwrap();
        state
            .repository()
            .add_tenant_member("tenant_1", "user_2", "user")
            .await
            .unwrap();
        let token = create_pat(
            &state,
            "tenantuserstatus",
            "user_1",
            [Permission::TenantManageUsers],
        )
        .await;

        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 16,
                "method": "tools/call",
                "params": {
                    "name": "msm.set_tenant_user_status",
                    "arguments": { "tenantId": "tenant_1", "userId": "user_2", "isDisabled": true }
                }
            }),
        )
        .await;
        assert_eq!(
            response["result"]["structuredContent"]["user"]["isDisabled"],
            true
        );
    }

    #[tokio::test]
    async fn tools_call_manages_tenant_roles() {
        let state = empty_state_with_owner().await;
        let token = create_pat(
            &state,
            "tenantroles",
            "user_1",
            [Permission::TenantManageRoles],
        )
        .await;

        let upsert_role = post_mcp_with_auth(
            state.clone(),
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 17,
                "method": "tools/call",
                "params": {
                    "name": "msm.upsert_tenant_role",
                    "arguments": {
                        "tenantId": "tenant_1",
                        "roleId": "role_editor",
                        "name": "Editors",
                        "permissions": ["pack.read", "pack.update"]
                    }
                }
            }),
        )
        .await;
        assert_eq!(
            upsert_role["result"]["structuredContent"]["role"]["permissions"][1],
            "pack.update"
        );

        let list_roles = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 18,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_tenant_roles",
                    "arguments": { "tenantId": "tenant_1" }
                }
            }),
        )
        .await;
        assert_eq!(
            list_roles["result"]["structuredContent"]["roles"][0]["id"],
            "role_editor"
        );
    }

    #[tokio::test]
    async fn pat_enforcement_metadata_tools_require_expected_scopes() {
        let state = empty_state_with_owner().await;
        let token = create_pat(&state, "packread", "user_1", [Permission::PackRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_folder",
                    "arguments": {
                        "id": "folder_1",
                        "tenantId": "tenant_1",
                        "ownerUserId": "user_1",
                        "name": "Favorites"
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("pack.update"));
    }

    #[tokio::test]
    async fn pat_enforcement_membership_tools_require_expected_scopes() {
        let state = seeded_state_with_product_metadata().await;
        let token = create_pat(&state, "packread", "user_1", [Permission::PackRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.add_pack_to_folder",
                    "arguments": {
                        "folderId": "folder_1",
                        "packId": "pack_1",
                        "sortOrder": 10
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("pack.update"));
    }

    #[tokio::test]
    async fn pat_enforcement_telegram_publications_require_export_read() {
        let state = seeded_state_with_publication().await;
        let token = create_pat(&state, "packread", "user_1", [Permission::PackRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_telegram_publications",
                    "arguments": { "packId": "pack_1" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("export.read"));
    }

    #[tokio::test]
    async fn pat_enforcement_export_job_requires_export_run() {
        let state = seeded_state_with_export_target().await;
        let token = create_pat(&state, "exportread", "user_1", [Permission::ExportRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.create_export_job",
                    "arguments": {
                        "id": "job_1",
                        "tenantId": "tenant_1",
                        "sourcePackId": "pack_1",
                        "targetId": "target_morestickers",
                        "options": {}
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("export.run"));
    }

    #[tokio::test]
    async fn unknown_method_returns_json_rpc_error() {
        let response = post_mcp(
            test_state().await,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "unknown"
            }),
        )
        .await;

        assert_eq!(response["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn pat_enforcement_tools_call_requires_bearer() {
        let response = post_mcp(
            seeded_state().await,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_sticker_packs",
                    "arguments": { "userId": "user_1" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Personal Access Token"));
    }

    #[tokio::test]
    async fn pat_enforcement_tools_call_allows_pack_read_list() {
        let state = seeded_state().await;
        let token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_sticker_packs",
                    "arguments": { "userId": "user_1" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], false);
    }

    #[tokio::test]
    async fn pat_enforcement_tools_call_import_requires_import_run() {
        let state = empty_state_with_owner().await;
        let token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.import_sticker_pack",
                    "arguments": {
                        "tenantId": "tenant_1",
                        "ownerUserId": "user_1",
                        "packId": "pack_1",
                        "visibility": "private",
                        "pack": sample_pack()
                    }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("import.run"));
    }

    #[tokio::test]
    async fn pat_enforcement_tools_call_rejects_user_mismatch() {
        let state = seeded_state().await;
        let token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;
        let response = post_mcp_with_auth(
            state,
            &token,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "msm.list_sticker_packs",
                    "arguments": { "userId": "user_2" }
                }
            }),
        )
        .await;

        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("user mismatch"));
    }

    async fn post_mcp(state: msm_api::ApiState, body: Value) -> Value {
        post_mcp_request(state, None, body).await
    }

    async fn post_mcp_with_auth(state: msm_api::ApiState, token: &str, body: Value) -> Value {
        post_mcp_request(state, Some(token), body).await
    }

    async fn post_mcp_request(state: msm_api::ApiState, token: Option<&str>, body: Value) -> Value {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("content-type", "application/json");
        if let Some(token) = token {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }

        let response = build_router(state)
            .oneshot(builder.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    async fn create_pat<const N: usize>(
        state: &msm_api::ApiState,
        id: &str,
        user_id: &str,
        scopes: [Permission; N],
    ) -> String {
        state
            .repository()
            .create_personal_access_token(id, user_id, "MCP", &BTreeSet::from(scopes), None)
            .await
            .unwrap()
            .token
    }

    async fn seeded_state() -> msm_api::ApiState {
        let state = empty_state_with_owner().await;
        state
            .repository()
            .upsert_sticker_pack(
                "pack_1",
                "tenant_1",
                "user_1",
                msm_storage::models::PackVisibility::Private,
                Some("telegram"),
                &sample_pack(),
            )
            .await
            .unwrap();
        state
    }

    async fn seeded_state_with_product_metadata() -> msm_api::ApiState {
        let state = seeded_state().await;
        state
            .repository()
            .create_folder("folder_1", "tenant_1", "user_1", "Favorites")
            .await
            .unwrap();
        state
            .repository()
            .create_tag(NewTag {
                id: "tag_1",
                tenant_id: "tenant_1",
                name: "cute",
            })
            .await
            .unwrap();
        state
            .repository()
            .create_subscription_group(
                "sub_1",
                "tenant_1",
                "user_1",
                "Weekly",
                msm_storage::models::PackVisibility::Private,
            )
            .await
            .unwrap();
        state
    }

    async fn seeded_state_with_export_target() -> msm_api::ApiState {
        let state = seeded_state().await;
        state
            .repository()
            .create_export_target(NewExportTarget {
                id: "target_morestickers",
                tenant_id: "tenant_1",
                kind: "morestickers",
                name: "MoreStickers",
                config_json: "{}",
                is_enabled: true,
            })
            .await
            .unwrap();
        state
    }

    async fn seeded_state_with_export_job() -> msm_api::ApiState {
        let state = seeded_state_with_export_target().await;
        state
            .repository()
            .create_export_job(msm_storage::models::NewExportJob {
                id: "job_1",
                tenant_id: "tenant_1",
                owner_user_id: "user_1",
                source_pack_id: "pack_1",
                target_id: "target_morestickers",
                request_json: r#"{"id":"job_1","tenantId":"tenant_1","sourcePackId":"pack_1","targetId":"target_morestickers","options":{}}"#,
                max_attempts: 3,
            })
            .await
            .unwrap();
        state
            .repository()
            .append_export_job_event(NewExportJobEvent {
                job_id: "job_1",
                sequence: 1,
                level: "info",
                stage: "queued",
                message: "job queued",
                metadata_json: "{}",
            })
            .await
            .unwrap();
        state
    }

    async fn seeded_state_with_publication() -> msm_api::ApiState {
        let state = seeded_state_with_export_job().await;
        state
            .repository()
            .upsert_telegram_publication(NewTelegramPublication {
                id: "telegram_pub_1",
                pack_id: "pack_1",
                target_id: "target_morestickers",
                job_id: "job_1",
                sticker_set_name: "sample_by_msm_bot",
                sticker_set_url: "https://t.me/addstickers/sample_by_msm_bot",
                sticker_count: 1,
                sticker_type: "regular",
            })
            .await
            .unwrap();
        state
    }

    async fn empty_state_with_owner() -> msm_api::ApiState {
        let state = test_state().await;
        state
            .repository()
            .create_tenant("tenant_1", "Tenant")
            .await
            .unwrap();
        state
            .repository()
            .create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();
        state
            .repository()
            .add_tenant_member("tenant_1", "user_1", "admin")
            .await
            .unwrap();
        state
    }

    async fn test_state() -> msm_api::ApiState {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        let repository = StorageRepository::new(pool);
        let assets = LocalAssetStore::new(tempfile::tempdir().unwrap().keep());
        msm_api::ApiState::new(repository, assets)
    }

    fn sample_pack() -> msm_domain::StickerPack {
        let sticker = Sticker {
            id: "MoreStickers:Telegram:Sticker:sample:file".to_owned(),
            image: "https://msm.example/assets/packs/sample/file.webp".to_owned(),
            title: "file".to_owned(),
            sticker_pack_id: "MoreStickers:Telegram:Pack:sample".to_owned(),
            filename: Some("file.webp".to_owned()),
            is_animated: Some(false),
        };

        msm_domain::StickerPack {
            id: "MoreStickers:Telegram:Pack:sample".to_owned(),
            title: "Sample".to_owned(),
            author: None,
            logo: sticker.clone(),
            stickers: vec![sticker],
        }
    }
}
