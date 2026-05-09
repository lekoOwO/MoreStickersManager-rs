#![doc = "HTTP API and `OpenAPI` surface for MoreStickersManager-rs."]

pub mod auth;
pub mod dto;
pub mod error;
pub mod openapi;
pub mod routes;
pub mod state;

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

pub use error::{ApiError, ApiResult};
pub use state::ApiState;

pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .merge(system_routes())
        .merge(auth_routes())
        .merge(pack_routes())
        .merge(metadata_routes())
        .merge(export_routes())
        .merge(pat_routes())
        .with_state(state)
}

fn system_routes() -> Router<ApiState> {
    Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route("/openapi.json", get(openapi::openapi_json))
        .route(
            "/assets/packs/{pack_public_id}/{filename}",
            get(routes::assets::read_asset),
        )
        .route(
            "/api/public/packs/{pack_id}/stickerpack",
            get(routes::subscriptions::public_pack_stickerpack),
        )
        .route(
            "/api/public/packs/{pack_id}/subscription",
            get(routes::subscriptions::public_pack_subscription),
        )
        .route(
            "/api/public/subscriptions/{subscription_group_id}",
            get(routes::subscriptions::public_subscription_group),
        )
}

fn auth_routes() -> Router<ApiState> {
    Router::new()
        .route(
            "/api/v1/auth/local/register",
            post(routes::auth::register_local_user),
        )
        .route(
            "/api/v1/auth/local/login",
            post(routes::auth::login_local_user),
        )
}

fn pack_routes() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/packs", get(routes::packs::list_packs))
        .route(
            "/api/v1/packs/{pack_id}",
            patch(routes::packs::update_pack).delete(routes::packs::delete_pack),
        )
        .route("/api/v1/packs/import", post(routes::packs::import_pack))
        .route(
            "/api/v1/packs/{pack_id}/stickerpack",
            get(routes::packs::export_pack),
        )
}

fn metadata_routes() -> Router<ApiState> {
    Router::new()
        .route(
            "/api/v1/folders",
            get(routes::metadata::list_folders).post(routes::metadata::create_folder),
        )
        .route(
            "/api/v1/folders/{folder_id}/packs",
            get(routes::metadata::list_folder_pack_ids),
        )
        .route(
            "/api/v1/folders/{folder_id}/packs/{pack_id}",
            put(routes::metadata::add_pack_to_folder)
                .delete(routes::metadata::remove_pack_from_folder),
        )
        .route(
            "/api/v1/tags",
            get(routes::metadata::list_tags).post(routes::metadata::create_tag),
        )
        .route(
            "/api/v1/packs/{pack_id}/tags",
            get(routes::metadata::list_pack_tag_ids),
        )
        .route(
            "/api/v1/packs/{pack_id}/tags/{tag_id}",
            put(routes::metadata::add_tag_to_pack).delete(routes::metadata::remove_tag_from_pack),
        )
        .route(
            "/api/v1/subscription-groups",
            get(routes::metadata::list_subscription_groups)
                .post(routes::metadata::create_subscription_group),
        )
        .route(
            "/api/v1/subscription-groups/{subscription_group_id}/packs",
            get(routes::metadata::list_subscription_group_pack_ids),
        )
        .route(
            "/api/v1/subscription-groups/{subscription_group_id}/packs/{pack_id}",
            put(routes::metadata::add_pack_to_subscription_group)
                .delete(routes::metadata::remove_pack_from_subscription_group),
        )
}

fn export_routes() -> Router<ApiState> {
    Router::new()
        .route(
            "/api/v1/export-target-kinds",
            get(routes::exports::list_target_kinds),
        )
        .route(
            "/api/v1/export-targets",
            get(routes::exports::list_targets).post(routes::exports::create_target),
        )
        .route(
            "/api/v1/export-targets/{target_id}",
            patch(routes::exports::update_target).delete(routes::exports::delete_target),
        )
        .route("/api/v1/export-jobs", post(routes::exports::create_job))
        .route(
            "/api/v1/export-jobs/{job_id}",
            get(routes::exports::get_job),
        )
        .route(
            "/api/v1/export-jobs/{job_id}/events",
            get(routes::exports::list_job_events),
        )
        .route(
            "/api/v1/telegram-publications",
            get(routes::exports::list_telegram_publications),
        )
        .route(
            "/api/v1/telegram-publications/{publication_id}",
            get(routes::exports::get_telegram_publication),
        )
}

fn pat_routes() -> Router<ApiState> {
    Router::new()
        .route(
            "/api/v1/pats",
            get(routes::pats::list_pats).post(routes::pats::create_pat),
        )
        .route("/api/v1/pats/{token_id}", delete(routes::pats::revoke_pat))
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
        models::{
            NewExportJobEvent, NewExportTarget, NewTag, NewTelegramPublication,
            SubscriptionAccessResourceType,
        },
        DatabaseConfig, DbPool, LocalAssetStore, StorageRepository,
    };
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
        assert!(json["paths"]
            .get("/api/public/packs/{pack_id}/stickerpack")
            .is_some());
        assert!(json["paths"]
            .get("/api/public/packs/{pack_id}/subscription")
            .is_some());
        assert!(json["paths"]
            .get("/api/public/subscriptions/{subscription_group_id}")
            .is_some());
        assert!(json["paths"].get("/api/v1/pats").is_some());
        assert!(json["paths"].get("/api/v1/auth/local/login").is_some());
        assert!(json["paths"].get("/api/v1/export-target-kinds").is_some());
        assert!(json["paths"].get("/api/v1/export-targets").is_some());
        assert!(json["paths"].get("/api/v1/export-jobs").is_some());
        assert!(json["paths"].get("/api/v1/folders").is_some());
        assert!(json["paths"]
            .get("/api/v1/folders/{folder_id}/packs")
            .is_some());
        assert!(json["paths"]
            .get("/api/v1/folders/{folder_id}/packs/{pack_id}")
            .is_some());
        assert!(json["paths"].get("/api/v1/tags").is_some());
        assert!(json["paths"].get("/api/v1/packs/{pack_id}/tags").is_some());
        assert!(json["paths"]
            .get("/api/v1/packs/{pack_id}/tags/{tag_id}")
            .is_some());
        assert!(json["paths"].get("/api/v1/subscription-groups").is_some());
        assert!(json["paths"]
            .get("/api/v1/subscription-groups/{subscription_group_id}/packs")
            .is_some());
        assert!(json["paths"]
            .get("/api/v1/subscription-groups/{subscription_group_id}/packs/{pack_id}")
            .is_some());
        assert!(json["paths"].get("/api/v1/telegram-publications").is_some());
        assert!(json["paths"]
            .get("/api/v1/telegram-publications/{publication_id}")
            .is_some());
    }

    #[tokio::test]
    async fn openapi_documents_telegram_export_job_options() {
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

        assert_eq!(
            json["components"]["schemas"]["TelegramReconcileModeOption"]["enum"],
            serde_json::json!(["createOnly", "appendMissing", "mirror"])
        );
        let option_schema = &json["components"]["schemas"]["TelegramExportJobOptions"];
        assert_eq!(
            option_schema["properties"]["reconcileMode"]["oneOf"][1]["enum"],
            serde_json::json!(["createOnly", "appendMissing", "mirror"])
        );
        assert_eq!(
            option_schema["properties"]["allowDestructiveReconciliation"]["type"],
            serde_json::json!(["boolean", "null"])
        );
        assert_eq!(
            json["components"]["schemas"]["CreateExportJobRequest"]["properties"]["options"]
                ["$ref"],
            "#/components/schemas/TelegramExportJobOptions"
        );
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
    async fn updates_pack_title_and_visibility_with_pack_update_scope() {
        let state = seeded_state().await;
        let update_token =
            create_pat(&state, "patupdate", "user_1", [Permission::PackUpdate]).await;
        let read_token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;
        let update_body = serde_json::json!({
            "title": "Renamed Pack",
            "visibility": "public",
        });

        let update_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/packs/pack_1")
                    .header("authorization", format!("Bearer {update_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(update_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(update_response.status(), StatusCode::OK);

        let export_response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs/pack_1/stickerpack")
                    .header("authorization", format!("Bearer {read_token}"))
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
        assert_eq!(exported["title"], "Renamed Pack");
    }

    #[tokio::test]
    async fn deletes_pack_with_pack_delete_scope() {
        let state = seeded_state().await;
        let delete_token =
            create_pat(&state, "patdelete", "user_1", [Permission::PackDelete]).await;
        let read_token = create_pat(&state, "patread", "user_1", [Permission::PackRead]).await;

        let delete_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/packs/pack_1")
                    .header("authorization", format!("Bearer {delete_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        let export_response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs/pack_1/stickerpack")
                    .header("authorization", format!("Bearer {read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(export_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn export_target_routes_redact_tokens_and_require_scopes() {
        let state = empty_state_with_owner().await;
        let read_token = create_pat(&state, "exportread", "user_1", [Permission::ExportRead]).await;
        let manage_token = create_pat(
            &state,
            "exportmanage",
            "user_1",
            [Permission::ExportTargetManage],
        )
        .await;

        let kinds_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/export-target-kinds")
                    .header("authorization", format!("Bearer {read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(kinds_response.status(), StatusCode::OK);
        let kinds_body = to_bytes(kinds_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let kinds: serde_json::Value = serde_json::from_slice(&kinds_body).unwrap();
        assert!(kinds
            .as_array()
            .unwrap()
            .iter()
            .any(|kind| kind["kind"] == "telegram"));

        let create_body = serde_json::json!({
            "id": "target_telegram",
            "tenantId": "tenant_1",
            "kind": "telegram",
            "name": "Telegram",
            "config": {
                "botUsername": "msm_bot",
                "botToken": "123456:secret"
            },
            "isEnabled": true
        });

        let forbidden_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/export-targets")
                    .header("authorization", format!("Bearer {read_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);

        let create_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/export-targets")
                    .header("authorization", format!("Bearer {manage_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
        assert_eq!(created["config"]["botToken"], "<redacted>");
        assert!(!contains_bytes(&create_body, b"123456:secret"));

        let list_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/export-targets?tenantId=tenant_1")
                    .header("authorization", format!("Bearer {read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(!contains_bytes(&list_body, b"123456:secret"));

        let update_body = serde_json::json!({
            "name": "Telegram Updated",
            "config": {
                "botUsername": "msm_bot",
                "botToken": "456:rotated"
            },
            "isEnabled": false
        });
        let update_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/export-targets/target_telegram")
                    .header("authorization", format!("Bearer {manage_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(update_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(update_response.status(), StatusCode::OK);

        let delete_response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/export-targets/target_telegram")
                    .header("authorization", format!("Bearer {manage_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn export_job_routes_require_export_run_and_pack_owner() {
        let state = seeded_state().await;
        state
            .repository()
            .create_export_target(NewExportTarget {
                id: "target_telegram",
                tenant_id: "tenant_1",
                kind: "telegram",
                name: "Telegram",
                config_json: r#"{"botUsername":"msm_bot","botToken":"123456:secret"}"#,
                is_enabled: true,
            })
            .await
            .unwrap();
        state
            .repository()
            .create_user("user_2", "other@example.com", "Other")
            .await
            .unwrap();
        let read_token = create_pat(&state, "exportread", "user_1", [Permission::ExportRead]).await;
        let run_token = create_pat(&state, "exportrun", "user_1", [Permission::ExportRun]).await;
        let other_run_token =
            create_pat(&state, "otherrun", "user_2", [Permission::ExportRun]).await;
        let create_body = serde_json::json!({
            "id": "job_1",
            "tenantId": "tenant_1",
            "sourcePackId": "pack_1",
            "targetId": "target_telegram",
            "options": { "setNameSlug": "sample" }
        });

        let missing_scope = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/export-jobs")
                    .header("authorization", format!("Bearer {read_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing_scope.status(), StatusCode::FORBIDDEN);

        let owner_mismatch = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/export-jobs")
                    .header("authorization", format!("Bearer {other_run_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(owner_mismatch.status(), StatusCode::FORBIDDEN);

        let created_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/export-jobs")
                    .header("authorization", format!("Bearer {run_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(created_response.status(), StatusCode::CREATED);
        let job_response_bytes = to_bytes(created_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&job_response_bytes).unwrap();
        assert_eq!(created["status"], "queued");
        assert_eq!(created["ownerUserId"], "user_1");
        assert_eq!(created["attemptCount"], 0);
        assert_eq!(created["maxAttempts"], 3);
        assert_eq!(created["nextAttemptAt"], serde_json::Value::Null);

        state
            .repository()
            .append_export_job_event(NewExportJobEvent {
                job_id: "job_1",
                sequence: 1,
                level: "info",
                stage: "queued",
                message: "job queued",
                metadata_json: r#"{"target":"telegram"}"#,
            })
            .await
            .unwrap();

        let get_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/export-jobs/job_1")
                    .header("authorization", format!("Bearer {read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let events_response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/export-jobs/job_1/events")
                    .header("authorization", format!("Bearer {read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(events_response.status(), StatusCode::OK);
        let events_body = to_bytes(events_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let events: serde_json::Value = serde_json::from_slice(&events_body).unwrap();
        assert_eq!(events[0]["message"], "job queued");
        assert_eq!(events[0]["metadata"]["target"], "telegram");
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn metadata_routes_manage_folders_tags_and_subscriptions() {
        let state = seeded_state().await;
        let pack_token = create_pat(&state, "packupdate", "user_1", [Permission::PackUpdate]).await;
        let subscription_create_token = create_pat(
            &state,
            "subcreate",
            "user_1",
            [Permission::SubscriptionCreate],
        )
        .await;
        let subscription_read_token =
            create_pat(&state, "subread", "user_1", [Permission::SubscriptionRead]).await;

        let folder_body = serde_json::json!({
            "id": "folder_1",
            "tenantId": "tenant_1",
            "ownerUserId": "user_1",
            "name": "Favorites"
        });
        let folder_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/folders")
                    .header("authorization", format!("Bearer {pack_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(folder_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(folder_response.status(), StatusCode::CREATED);

        let folder_list = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/folders?tenantId=tenant_1&ownerUserId=user_1")
                    .header("authorization", format!("Bearer {pack_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(folder_list.status(), StatusCode::OK);
        let folder_body = to_bytes(folder_list.into_body(), usize::MAX).await.unwrap();
        let folders: serde_json::Value = serde_json::from_slice(&folder_body).unwrap();
        assert_eq!(folders[0]["name"], "Favorites");

        let tag_body = serde_json::json!({
            "id": "tag_1",
            "tenantId": "tenant_1",
            "name": "cute"
        });
        let tag_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tags")
                    .header("authorization", format!("Bearer {pack_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(tag_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(tag_response.status(), StatusCode::CREATED);

        let tag_list = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tags?tenantId=tenant_1")
                    .header("authorization", format!("Bearer {pack_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(tag_list.status(), StatusCode::OK);
        let tag_body = to_bytes(tag_list.into_body(), usize::MAX).await.unwrap();
        let tags: serde_json::Value = serde_json::from_slice(&tag_body).unwrap();
        assert_eq!(tags[0]["name"], "cute");

        let subscription_body = serde_json::json!({
            "id": "sub_1",
            "tenantId": "tenant_1",
            "ownerUserId": "user_1",
            "title": "Weekly",
            "visibility": "private"
        });
        let subscription_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/subscription-groups")
                    .header(
                        "authorization",
                        format!("Bearer {subscription_create_token}"),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(subscription_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(subscription_response.status(), StatusCode::CREATED);

        let subscription_list = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/subscription-groups?tenantId=tenant_1&ownerUserId=user_1")
                    .header("authorization", format!("Bearer {subscription_read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(subscription_list.status(), StatusCode::OK);
        let subscription_body = to_bytes(subscription_list.into_body(), usize::MAX)
            .await
            .unwrap();
        let subscriptions: serde_json::Value = serde_json::from_slice(&subscription_body).unwrap();
        assert_eq!(subscriptions[0]["title"], "Weekly");
        assert_eq!(subscriptions[0]["visibility"], "private");
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn metadata_routes_manage_pack_memberships() {
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

        let token = create_pat(
            &state,
            "metadata",
            "user_1",
            [
                Permission::PackUpdate,
                Permission::SubscriptionCreate,
                Permission::SubscriptionRead,
            ],
        )
        .await;

        let folder_add_body = serde_json::json!({ "sortOrder": 10 });
        let folder_add = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/folders/folder_1/packs/pack_1")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(folder_add_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(folder_add.status(), StatusCode::OK);
        let folder_add_body = to_bytes(folder_add.into_body(), usize::MAX).await.unwrap();
        let folder_link: serde_json::Value = serde_json::from_slice(&folder_add_body).unwrap();
        assert_eq!(folder_link["folderId"], "folder_1");
        assert_eq!(folder_link["packId"], "pack_1");
        assert_eq!(folder_link["sortOrder"], 10);

        let folder_list = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/folders/folder_1/packs")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(folder_list.status(), StatusCode::OK);
        let folder_list_body = to_bytes(folder_list.into_body(), usize::MAX).await.unwrap();
        let folder_pack_ids: serde_json::Value = serde_json::from_slice(&folder_list_body).unwrap();
        assert_eq!(folder_pack_ids, serde_json::json!(["pack_1"]));

        let tag_add = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/packs/pack_1/tags/tag_1")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(tag_add.status(), StatusCode::OK);
        let tag_add_body = to_bytes(tag_add.into_body(), usize::MAX).await.unwrap();
        let tag_link: serde_json::Value = serde_json::from_slice(&tag_add_body).unwrap();
        assert_eq!(tag_link["packId"], "pack_1");
        assert_eq!(tag_link["tagId"], "tag_1");

        let tag_list = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/packs/pack_1/tags")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(tag_list.status(), StatusCode::OK);
        let tag_list_body = to_bytes(tag_list.into_body(), usize::MAX).await.unwrap();
        let tag_ids: serde_json::Value = serde_json::from_slice(&tag_list_body).unwrap();
        assert_eq!(tag_ids, serde_json::json!(["tag_1"]));

        let subscription_add_body = serde_json::json!({ "sortOrder": 20 });
        let subscription_add = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/subscription-groups/sub_1/packs/pack_1")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(subscription_add_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(subscription_add.status(), StatusCode::OK);
        let subscription_add_body = to_bytes(subscription_add.into_body(), usize::MAX)
            .await
            .unwrap();
        let subscription_link: serde_json::Value =
            serde_json::from_slice(&subscription_add_body).unwrap();
        assert_eq!(subscription_link["subscriptionGroupId"], "sub_1");
        assert_eq!(subscription_link["packId"], "pack_1");
        assert_eq!(subscription_link["sortOrder"], 20);

        let subscription_list = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/subscription-groups/sub_1/packs")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(subscription_list.status(), StatusCode::OK);
        let subscription_list_body = to_bytes(subscription_list.into_body(), usize::MAX)
            .await
            .unwrap();
        let subscription_pack_ids: serde_json::Value =
            serde_json::from_slice(&subscription_list_body).unwrap();
        assert_eq!(subscription_pack_ids, serde_json::json!(["pack_1"]));

        for uri in [
            "/api/v1/folders/folder_1/packs/pack_1",
            "/api/v1/packs/pack_1/tags/tag_1",
            "/api/v1/subscription-groups/sub_1/packs/pack_1",
        ] {
            let response = build_router(state.clone())
                .oneshot(
                    Request::builder()
                        .method("DELETE")
                        .uri(uri)
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::NO_CONTENT);
        }
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn public_subscription_routes_emit_accessible_dynamic_payloads() {
        let state = empty_state_with_owner().await;
        state
            .repository()
            .upsert_sticker_pack(
                "pack_public",
                "tenant_1",
                "user_1",
                msm_storage::models::PackVisibility::Public,
                Some("telegram"),
                &sample_pack_with_suffix("public"),
            )
            .await
            .unwrap();
        state
            .repository()
            .upsert_sticker_pack(
                "pack_private",
                "tenant_1",
                "user_1",
                msm_storage::models::PackVisibility::Private,
                Some("telegram"),
                &sample_pack_with_suffix("private"),
            )
            .await
            .unwrap();
        state
            .repository()
            .create_subscription_group(
                "sub_public",
                "tenant_1",
                "user_1",
                "Public Feed",
                msm_storage::models::PackVisibility::Public,
            )
            .await
            .unwrap();
        state
            .repository()
            .create_subscription_group(
                "sub_private",
                "tenant_1",
                "user_1",
                "Private Feed",
                msm_storage::models::PackVisibility::Private,
            )
            .await
            .unwrap();
        state
            .repository()
            .add_pack_to_subscription_group("sub_public", "pack_public", 0)
            .await
            .unwrap();
        state
            .repository()
            .add_pack_to_subscription_group("sub_public", "pack_private", 1)
            .await
            .unwrap();
        state
            .repository()
            .add_pack_to_subscription_group("sub_private", "pack_private", 0)
            .await
            .unwrap();
        let token = create_pat(
            &state,
            "subscription-read",
            "user_1",
            [Permission::PackRead, Permission::SubscriptionRead],
        )
        .await;

        let public_pack = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/packs/pack_public/stickerpack")
                    .header("host", "msm.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(public_pack.status(), StatusCode::OK);

        let private_pack = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/packs/pack_private/stickerpack")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(private_pack.status(), StatusCode::UNAUTHORIZED);

        let private_pack_with_pat = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/packs/pack_private/stickerpack")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(private_pack_with_pat.status(), StatusCode::OK);

        let public_pack_subscription = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/packs/pack_public/subscription")
                    .header("host", "msm.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(public_pack_subscription.status(), StatusCode::OK);
        let body = to_bytes(public_pack_subscription.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["id"], "pack_public");
        assert_eq!(payload["packs"].as_array().unwrap().len(), 1);
        assert_eq!(
            payload["packs"][0]["dynamic"]["refreshUrl"],
            "http://msm.example/api/public/packs/pack_public/stickerpack"
        );

        let private_pack_subscription = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/packs/pack_private/subscription")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(private_pack_subscription.status(), StatusCode::UNAUTHORIZED);

        let subscription = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/subscriptions/sub_public")
                    .header("host", "msm.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(subscription.status(), StatusCode::OK);
        let body = to_bytes(subscription.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["id"], "sub_public");
        assert_eq!(payload["title"], "Public Feed");
        assert!(payload.get("authHeaders").is_none());
        assert_eq!(payload["packs"].as_array().unwrap().len(), 1);
        assert_eq!(
            payload["packs"][0]["dynamic"]["refreshUrl"],
            "http://msm.example/api/public/packs/pack_public/stickerpack"
        );

        let private_subscription = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/subscriptions/sub_private")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(private_subscription.status(), StatusCode::UNAUTHORIZED);

        let protected_subscription = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/public/subscriptions/sub_private")
                    .header("host", "msm.example")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(protected_subscription.status(), StatusCode::OK);
        let body = to_bytes(protected_subscription.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["id"], "sub_private");
        assert_eq!(payload["packs"].as_array().unwrap().len(), 1);
        assert_eq!(
            payload["packs"][0]["dynamic"]["refreshUrl"],
            "http://msm.example/api/public/packs/pack_private/stickerpack"
        );
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn public_subscription_routes_accept_subscription_access_tokens() {
        let state = empty_state_with_owner().await;
        state
            .repository()
            .upsert_sticker_pack(
                "pack_private",
                "tenant_1",
                "user_1",
                msm_storage::models::PackVisibility::Private,
                Some("telegram"),
                &sample_pack_with_suffix("private"),
            )
            .await
            .unwrap();
        state
            .repository()
            .create_subscription_group(
                "sub_private",
                "tenant_1",
                "user_1",
                "Private Feed",
                msm_storage::models::PackVisibility::Private,
            )
            .await
            .unwrap();
        state
            .repository()
            .add_pack_to_subscription_group("sub_private", "pack_private", 0)
            .await
            .unwrap();

        let pack_token = state
            .repository()
            .create_subscription_access_token(
                "packlink",
                "tenant_1",
                "user_1",
                SubscriptionAccessResourceType::Pack,
                "pack_private",
            )
            .await
            .unwrap()
            .token;
        let group_token = state
            .repository()
            .create_subscription_access_token(
                "grouplink",
                "tenant_1",
                "user_1",
                SubscriptionAccessResourceType::SubscriptionGroup,
                "sub_private",
            )
            .await
            .unwrap()
            .token;

        let private_pack = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/packs/pack_private/stickerpack")
                    .header("authorization", format!("Bearer {pack_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(private_pack.status(), StatusCode::OK);

        let private_pack_subscription = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/packs/pack_private/subscription")
                    .header("host", "msm.example")
                    .header("authorization", format!("Bearer {pack_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(private_pack_subscription.status(), StatusCode::OK);
        let body = to_bytes(private_pack_subscription.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            payload["authHeaders"]["Authorization"],
            format!("Bearer {pack_token}")
        );
        assert_eq!(
            payload["packs"][0]["dynamic"]["authHeaders"]["Authorization"],
            format!("Bearer {pack_token}")
        );

        let private_subscription = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/public/subscriptions/sub_private")
                    .header("host", "msm.example")
                    .header("authorization", format!("Bearer {group_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(private_subscription.status(), StatusCode::OK);
        let body = to_bytes(private_subscription.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["id"], "sub_private");
        assert_eq!(payload["packs"].as_array().unwrap().len(), 1);
        assert_eq!(
            payload["authHeaders"]["Authorization"],
            format!("Bearer {group_token}")
        );
        assert_eq!(
            payload["packs"][0]["dynamic"]["authHeaders"]["Authorization"],
            format!("Bearer {group_token}")
        );

        let wrong_resource_token = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/public/subscriptions/sub_private")
                    .header("authorization", format!("Bearer {pack_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wrong_resource_token.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn telegram_publication_routes_require_export_read_and_pack_owner() {
        let state = seeded_state_with_publication().await;
        state
            .repository()
            .create_user("user_2", "other@example.com", "Other")
            .await
            .unwrap();
        let read_token = create_pat(&state, "pubread", "user_1", [Permission::ExportRead]).await;
        let run_token = create_pat(&state, "pubrun", "user_1", [Permission::ExportRun]).await;
        let other_read_token =
            create_pat(&state, "otherpubread", "user_2", [Permission::ExportRead]).await;

        let missing_scope = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/telegram-publications?packId=pack_1")
                    .header("authorization", format!("Bearer {run_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing_scope.status(), StatusCode::FORBIDDEN);

        let owner_mismatch = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/telegram-publications?packId=pack_1")
                    .header("authorization", format!("Bearer {other_read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(owner_mismatch.status(), StatusCode::FORBIDDEN);

        let list_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/telegram-publications?packId=pack_1")
                    .header("authorization", format!("Bearer {read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list: serde_json::Value = serde_json::from_slice(&list_body).unwrap();
        assert_eq!(list[0]["id"], "telegram_pub_1");
        assert_eq!(list[0]["packId"], "pack_1");
        assert_eq!(list[0]["targetId"], "target_telegram");
        assert_eq!(list[0]["jobId"], "job_1");
        assert_eq!(list[0]["stickerSetName"], "sample_by_msm_bot");
        assert_eq!(
            list[0]["stickerSetUrl"],
            "https://t.me/addstickers/sample_by_msm_bot"
        );
        assert_eq!(list[0]["stickerCount"], 1);
        assert_eq!(list[0]["stickerType"], "regular");

        let get_response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/telegram-publications/telegram_pub_1")
                    .header("authorization", format!("Bearer {read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_body = to_bytes(get_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let publication: serde_json::Value = serde_json::from_slice(&get_body).unwrap();
        assert_eq!(publication["id"], "telegram_pub_1");

        let get_owner_mismatch = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/telegram-publications/telegram_pub_1")
                    .header("authorization", format!("Bearer {other_read_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_owner_mismatch.status(), StatusCode::FORBIDDEN);
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

    #[tokio::test]
    async fn admin_bootstrap_registers_tenant_admin() {
        let state = test_state().await;
        let register_body = serde_json::json!({
            "id": "user_1",
            "email": "leko@example.com",
            "displayName": "Leko",
            "password": "password",
            "tenantId": "tenant_1",
            "tenantName": "Tenant",
            "tenantRole": "admin",
        });

        let response = build_router(state.clone())
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

        assert_eq!(response.status(), StatusCode::CREATED);

        let token = state
            .repository()
            .create_personal_access_token(
                "patimport",
                "user_1",
                "Import",
                &BTreeSet::from([Permission::ImportRun]),
                None,
            )
            .await
            .unwrap()
            .token;
        let import_body = serde_json::json!({
            "tenantId": "tenant_1",
            "ownerUserId": "user_1",
            "packId": "pack_1",
            "visibility": "private",
            "pack": sample_pack(),
        });
        let import_response = build_router(state)
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

    async fn seeded_state_with_publication() -> ApiState {
        let state = seeded_state().await;
        state
            .repository()
            .create_export_target(NewExportTarget {
                id: "target_telegram",
                tenant_id: "tenant_1",
                kind: "telegram",
                name: "Telegram",
                config_json: r#"{"botUsername":"msm_bot","botToken":"123456:secret"}"#,
                is_enabled: true,
            })
            .await
            .unwrap();
        state
            .repository()
            .create_export_job(msm_storage::models::NewExportJob {
                id: "job_1",
                tenant_id: "tenant_1",
                owner_user_id: "user_1",
                source_pack_id: "pack_1",
                target_id: "target_telegram",
                request_json: r#"{"options":{"dryRun":false}}"#,
                max_attempts: 3,
            })
            .await
            .unwrap();
        state
            .repository()
            .upsert_telegram_publication(NewTelegramPublication {
                id: "telegram_pub_1",
                pack_id: "pack_1",
                target_id: "target_telegram",
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

    fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }

    fn sample_pack() -> msm_domain::StickerPack {
        sample_pack_with_suffix("sample")
    }

    fn sample_pack_with_suffix(suffix: &str) -> msm_domain::StickerPack {
        let sticker = Sticker {
            id: format!("MoreStickers:Telegram:Sticker:{suffix}:file"),
            image: format!("https://msm.example/assets/packs/{suffix}/file.webp"),
            title: "file".to_owned(),
            sticker_pack_id: format!("MoreStickers:Telegram:Pack:{suffix}"),
            filename: Some("file.webp".to_owned()),
            is_animated: Some(false),
        };

        msm_domain::StickerPack {
            id: format!("MoreStickers:Telegram:Pack:{suffix}"),
            title: format!("Sample {suffix}"),
            author: None,
            logo: sticker.clone(),
            stickers: vec![sticker],
        }
    }
}
