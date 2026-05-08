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
        models::{NewExportJobEvent, NewExportTarget, NewTelegramPublication},
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
        assert_eq!(tools.len(), 13);
        assert_eq!(tools[0]["name"], "msm.list_sticker_packs");
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "msm.create_export_job"
                && tool["inputSchema"]["required"].as_array().unwrap().len() == 5));
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
            response["result"]["structuredContent"]["job"]["request"]["options"]["format"],
            "stickerpack"
        );
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
