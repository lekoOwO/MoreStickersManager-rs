use msm_app::{build_app_router, initialize_state, log_service_event, AppConfig, AppResult};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::from_env()?;
    log_service_event(
        "service_starting",
        serde_json::json!({
            "bindAddr": config.bind_addr.to_string(),
            "databaseUrlConfigured": !config.database_url.is_empty(),
            "requestBodyLimitBytes": config.request_body_limit_bytes,
            "importRateLimitRequests": config.import_rate_limit_requests,
            "importRateLimitWindowSecs": config.import_rate_limit_window.as_secs(),
        }),
    );
    let state = initialize_state(&config).await?;
    let _export_worker_handle =
        msm_app::spawn_export_worker_if_enabled(state.repository().clone(), config.export_worker);
    let _provider_import_worker_handle = msm_app::spawn_provider_import_worker_if_enabled(
        state.repository().clone(),
        state.asset_store().clone(),
        config.provider_import_worker,
    );
    let router = build_app_router(state, config.web_dist_dir, config.request_body_limit_bytes);
    let listener = TcpListener::bind(config.bind_addr).await?;
    log_service_event(
        "service_listening",
        serde_json::json!({ "bindAddr": config.bind_addr.to_string() }),
    );
    axum::serve(listener, router).await?;
    Ok(())
}
