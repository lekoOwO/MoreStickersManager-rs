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
    use msm_storage::{DatabaseConfig, DbPool, LocalAssetStore, StorageRepository};
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
    async fn tools_list_returns_pack_tools() {
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
        assert_eq!(tools.len(), 5);
        assert_eq!(tools[0]["name"], "msm.list_sticker_packs");
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
