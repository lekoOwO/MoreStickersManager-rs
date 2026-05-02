#![doc = "HTTP API and OpenAPI surface for MoreStickersManager-rs."]

pub mod dto;
pub mod error;
pub mod openapi;
pub mod routes;
pub mod state;

use axum::{Router, routing::get};

pub use error::{ApiError, ApiResult};
pub use state::ApiState;

#[must_use]
pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route("/openapi.json", get(openapi::openapi_json))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use msm_storage::{DatabaseConfig, DbPool, LocalAssetStore, StorageRepository};
    use tower::ServiceExt;

    use crate::{ApiState, build_router};

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let response = build_router(test_state().await)
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
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

    async fn test_state() -> ApiState {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        let repository = StorageRepository::new(pool);
        let assets = LocalAssetStore::new(tempfile::tempdir().unwrap().keep());
        ApiState::new(repository, assets)
    }
}
