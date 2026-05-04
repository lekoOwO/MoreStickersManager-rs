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
    use msm_domain::Sticker;
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
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0]["name"], "msm.list_sticker_packs");
    }

    #[tokio::test]
    async fn tools_call_lists_sticker_packs() {
        let state = seeded_state().await;
        let response = post_mcp(
            state,
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
        let response = post_mcp(
            state,
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
        let pack = sample_pack();
        let response = post_mcp(
            state,
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

    async fn post_mcp(state: msm_api::ApiState, body: Value) -> Value {
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
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
