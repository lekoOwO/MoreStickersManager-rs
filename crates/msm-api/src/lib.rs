#![doc = "HTTP API and `OpenAPI` surface for MoreStickersManager-rs."]

pub mod auth;
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
            "/api/v1/auth/local/register",
            axum::routing::post(routes::auth::register_local_user),
        )
        .route(
            "/api/v1/auth/local/login",
            axum::routing::post(routes::auth::login_local_user),
        )
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
        .route(
            "/api/v1/pats",
            get(routes::pats::list_pats).post(routes::pats::create_pat),
        )
        .route(
            "/api/v1/pats/{token_id}",
            axum::routing::delete(routes::pats::revoke_pat),
        )
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
        assert!(json["paths"].get("/api/v1/pats").is_some());
        assert!(json["paths"].get("/api/v1/auth/local/login").is_some());
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
        let token = create_pat(
            &state,
            "patfull",
            "user_1",
            [Permission::ImportRun, Permission::PackRead],
        )
        .await;

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
                    .header("authorization", format!("Bearer {token}"))
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
                    .header("authorization", format!("Bearer {token}"))
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
                    .header("authorization", format!("Bearer {token}"))
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

    #[tokio::test]
    async fn pat_enforcement_requires_bearer_for_pack_list() {
        let response = build_router(test_state().await)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs?userId=user_1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn pat_enforcement_allows_owner_pack_list_with_pack_read() {
        let state = seeded_state().await;
        let token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs?userId=user_1")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn pat_enforcement_rejects_pack_list_missing_scope() {
        let state = seeded_state().await;
        let token = create_pat(&state, "patasset", "user_1", [Permission::AssetRead]).await;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs?userId=user_1")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn pat_enforcement_rejects_pack_list_user_mismatch() {
        let state = seeded_state().await;
        state
            .repository()
            .create_user("user_2", "other@example.com", "Other")
            .await
            .unwrap();
        let token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs?userId=user_2")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn pat_enforcement_import_requires_import_run_and_matching_owner() {
        let state = empty_state_with_owner().await;
        let pack = sample_pack();
        let import_body = serde_json::json!({
            "tenantId": "tenant_1",
            "ownerUserId": "user_1",
            "packId": "pack_1",
            "visibility": "private",
            "pack": pack,
        });
        let read_only_token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;

        let forbidden = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/packs/import")
                    .header("authorization", format!("Bearer {read_only_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(import_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

        let import_token = create_pat(&state, "patimport", "user_1", [Permission::ImportRun]).await;
        let created = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/packs/import")
                    .header("authorization", format!("Bearer {import_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(import_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(created.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn pat_enforcement_export_requires_pack_read() {
        let state = seeded_state().await;
        let asset_token = create_pat(&state, "patasset", "user_1", [Permission::AssetRead]).await;
        let forbidden = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs/pack_1/stickerpack")
                    .header("authorization", format!("Bearer {asset_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

        let read_token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;
        let ok = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs/pack_1/stickerpack")
                    .header("authorization", format!("Bearer {read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(ok.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn creates_lists_and_revokes_personal_access_token() {
        let state = test_state().await;
        state
            .repository()
            .create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();

        let create_body = serde_json::json!({
            "id": "cli1",
            "userId": "user_1",
            "name": "CLI",
            "scopes": ["pack.read", "asset.read"],
            "expiresAt": null,
        });
        let create_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/pats")
                    .header("content-type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = created["token"].as_str().unwrap().to_owned();
        assert!(token.starts_with("msm_pat_cli1_"));
        assert!(created.get("tokenHash").is_none());

        let list_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/pats?userId=user_1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let listed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(listed[0]["id"], "cli1");
        assert!(listed[0].get("token").is_none());
        assert!(listed[0].get("tokenHash").is_none());

        let revoke_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/pats/cli1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(revoke_response.status(), StatusCode::NO_CONTENT);
        assert!(state
            .repository()
            .verify_personal_access_token(&token)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn create_personal_access_token_rejects_unknown_scopes() {
        let state = test_state().await;
        let create_body = serde_json::json!({
            "id": "cli1",
            "userId": "user_1",
            "name": "CLI",
            "scopes": ["pack.unknown"],
            "expiresAt": null,
        });

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/pats")
                    .header("content-type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn local_auth_registers_and_logs_in_with_pat() {
        let state = test_state().await;
        let register_body = serde_json::json!({
            "id": "user_1",
            "email": "leko@example.com",
            "displayName": "Leko",
            "password": "correct horse battery staple",
        });

        let register_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/local/register")
                    .header("content-type", "application/json")
                    .body(Body::from(register_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(register_response.status(), StatusCode::CREATED);

        let login_body = serde_json::json!({
            "email": "leko@example.com",
            "password": "correct horse battery staple",
            "tokenId": "webui",
            "tokenName": "Web UI",
            "scopes": ["pack.read", "pat.manage"],
            "expiresAt": null,
        });
        let login_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/local/login")
                    .header("content-type", "application/json")
                    .body(Body::from(login_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(login_response.status(), StatusCode::OK);
        let body = to_bytes(login_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let logged_in: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = logged_in["token"].as_str().unwrap();
        assert!(token.starts_with("msm_pat_webui_"));
        assert!(state
            .repository()
            .verify_personal_access_token(token)
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn local_auth_rejects_wrong_password() {
        let state = test_state().await;
        state
            .repository()
            .create_local_user_with_password("user_1", "leko@example.com", "Leko", "password")
            .await
            .unwrap();

        let login_body = serde_json::json!({
            "email": "leko@example.com",
            "password": "wrong",
            "tokenId": "webui",
            "tokenName": "Web UI",
            "scopes": ["pack.read"],
            "expiresAt": null,
        });
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/local/login")
                    .header("content-type", "application/json")
                    .body(Body::from(login_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    async fn test_state() -> ApiState {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        let repository = StorageRepository::new(pool);
        let assets = LocalAssetStore::new(tempfile::tempdir().unwrap().keep());
        ApiState::new(repository, assets)
    }

    async fn seeded_state() -> ApiState {
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

    async fn empty_state_with_owner() -> ApiState {
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

    async fn create_pat<const N: usize>(
        state: &ApiState,
        id: &str,
        user_id: &str,
        scopes: [Permission; N],
    ) -> String {
        state
            .repository()
            .create_personal_access_token(id, user_id, "Test PAT", &BTreeSet::from(scopes), None)
            .await
            .unwrap()
            .token
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
