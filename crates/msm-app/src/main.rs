use msm_app::{build_app_router, initialize_state, AppConfig, AppResult};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::from_env()?;
    let state = initialize_state(&config).await?;
    let _export_worker_handle =
        msm_app::spawn_export_worker_if_enabled(state.repository().clone(), config.export_worker);
    let _provider_import_worker_handle = msm_app::spawn_provider_import_worker_if_enabled(
        state.repository().clone(),
        state.asset_store().clone(),
        config.provider_import_worker,
    );
    let router = build_app_router(state, config.web_dist_dir);
    let listener = TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
