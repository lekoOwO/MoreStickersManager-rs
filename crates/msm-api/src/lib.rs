#![doc = "HTTP API and `OpenAPI` surface for MoreStickersManager-rs."]

pub mod dto;
pub mod error;
pub mod openapi;
pub mod routes;
pub mod state;

use axum::{routing::get, Router};

pub use error::{ApiError, ApiResult};
pub use state::ApiState;

pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route("/openapi.json", get(openapi::openapi_json))
        .route(
            "/assets/packs/{pack_public_id}/{filename}",
            get(routes::assets::read_asset),
        )
        .route("/api/v1/packs", get(routes::packs::list_packs))
        .route(
            "/api/v1/packs/import",
            axum::routing::post(routes::packs::import_pack),
        )
        .route(
            "/api/v1/packs/{pack_id}/stickerpack",
            get(routes::packs::export_pack),
        )
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
    use tower::ServiceExt;

    use crate::{build_router, ApiState};

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let response = build_router(test_state().await)
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], br#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn openapi_endpoint_contains_health_path() {
        let response = build_router(test_state().await)
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["paths"].get("/healthz").is_some());
    }

    #[tokio::test]
    async fn imports_lists_and_exports_pack() {
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

        let pack = sample_pack();
        let import_body = serde_json::json!({
            "tenantId": "tenant_1",
            "ownerUserId": "user_1",
            "packId": "pack_1",
            "visibility": "private",
            "pack": pack,
        });

        let import_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/packs/import")
                    .header("content-type", "application/json")
                    .body(Body::from(import_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(import_response.status(), StatusCode::CREATED);

        let list_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs?userId=user_1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let export_response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs/pack_1/stickerpack")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(export_response.status(), StatusCode::OK);
        let body = to_bytes(export_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let exported: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(exported["id"], "MoreStickers:Telegram:Pack:sample");
    }

    #[tokio::test]
    async fn reads_asset_bytes() {
        let state = test_state().await;
        let key = msm_storage::AssetKey::new("pack_1", "sticker.webp").unwrap();
        state
            .asset_store()
            .write(&key, b"webp-bytes")
            .await
            .unwrap();

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/assets/packs/pack_1/sticker.webp")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "image/webp"
        );
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"webp-bytes");
    }

    async fn test_state() -> ApiState {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        let repository = StorageRepository::new(pool);
        let assets = LocalAssetStore::new(tempfile::tempdir().unwrap().keep());
        ApiState::new(repository, assets)
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
