#![doc = "Runnable service composition for MoreStickersManager-rs."]

use std::{
    collections::BTreeMap,
    env,
    net::{AddrParseError, SocketAddr},
    path::PathBuf,
};

use axum::Router;
use msm_api::{build_router, ApiState};
use msm_storage::{DatabaseConfig, DbPool, LocalAssetStore, StorageRepository};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub asset_dir: PathBuf,
    pub web_dist_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid bind address `{value}`: {source}")]
    InvalidBindAddress {
        value: String,
        source: AddrParseError,
    },

    #[error("storage error: {0}")]
    Storage(#[from] msm_storage::StorageError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppConfig {
    pub const DEFAULT_BIND_ADDR: &'static str = "127.0.0.1:3000";
    pub const DEFAULT_DATABASE_URL: &'static str = "sqlite:data/msm.sqlite3";
    pub const DEFAULT_ASSET_DIR: &'static str = "data/assets";
    pub const DEFAULT_WEB_DIST_DIR: &'static str = "apps/web/dist";

    /// Reads service configuration from process environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error when `MSM_BIND_ADDR` is not a valid socket address.
    pub fn from_env() -> AppResult<Self> {
        let vars = env::vars().collect::<BTreeMap<_, _>>();
        Self::from_env_map(&vars)
    }

    /// Reads service configuration from an environment-like map.
    ///
    /// # Errors
    ///
    /// Returns an error when `MSM_BIND_ADDR` is not a valid socket address.
    pub fn from_env_map(vars: &BTreeMap<String, String>) -> AppResult<Self> {
        let bind_addr = read(vars, "MSM_BIND_ADDR", Self::DEFAULT_BIND_ADDR);
        Ok(Self {
            bind_addr: bind_addr
                .parse()
                .map_err(|source| AppError::InvalidBindAddress {
                    value: bind_addr,
                    source,
                })?,
            database_url: read(vars, "MSM_DATABASE_URL", Self::DEFAULT_DATABASE_URL),
            asset_dir: PathBuf::from(read(vars, "MSM_ASSET_DIR", Self::DEFAULT_ASSET_DIR)),
            web_dist_dir: PathBuf::from(read(vars, "MSM_WEB_DIST_DIR", Self::DEFAULT_WEB_DIST_DIR)),
        })
    }
}

/// Initializes API state from app configuration.
///
/// # Errors
///
/// Returns an error when database config parsing, database connection, migrations, or asset
/// directory creation fails.
pub async fn initialize_state(config: &AppConfig) -> AppResult<ApiState> {
    let database = DatabaseConfig::parse(config.database_url.clone())?;
    let pool = DbPool::connect(&database).await?;
    pool.run_migrations().await?;
    std::fs::create_dir_all(&config.asset_dir)?;
    let repository = StorageRepository::new(pool);
    let asset_store = LocalAssetStore::new(config.asset_dir.clone());
    Ok(ApiState::new(repository, asset_store))
}

pub fn build_app_router(state: ApiState, web_dist_dir: impl Into<PathBuf>) -> Router {
    let web_dist_dir = web_dist_dir.into();
    let index_file = web_dist_dir.join("index.html");
    let static_service = ServeDir::new(web_dist_dir).not_found_service(ServeFile::new(index_file));
    build_router(state).fallback_service(static_service)
}

fn read(vars: &BTreeMap<String, String>, key: &str, default: &str) -> String {
    vars.get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| default.to_owned())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};

    use crate::AppConfig;

    #[test]
    fn config_uses_defaults() {
        let config = AppConfig::from_env_map(&BTreeMap::new()).unwrap();

        assert_eq!(
            config.bind_addr,
            "127.0.0.1:3000".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.database_url, "sqlite:data/msm.sqlite3");
        assert_eq!(config.asset_dir, PathBuf::from("data/assets"));
        assert_eq!(config.web_dist_dir, PathBuf::from("apps/web/dist"));
    }

    #[test]
    fn config_reads_overrides() {
        let mut vars = BTreeMap::new();
        vars.insert("MSM_BIND_ADDR".to_owned(), "0.0.0.0:8080".to_owned());
        vars.insert(
            "MSM_DATABASE_URL".to_owned(),
            "sqlite:data/test.sqlite3".to_owned(),
        );
        vars.insert("MSM_ASSET_DIR".to_owned(), "tmp/assets".to_owned());
        vars.insert("MSM_WEB_DIST_DIR".to_owned(), "tmp/web".to_owned());

        let config = AppConfig::from_env_map(&vars).unwrap();

        assert_eq!(
            config.bind_addr,
            "0.0.0.0:8080".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.database_url, "sqlite:data/test.sqlite3");
        assert_eq!(config.asset_dir, PathBuf::from("tmp/assets"));
        assert_eq!(config.web_dist_dir, PathBuf::from("tmp/web"));
    }

    #[test]
    fn config_rejects_invalid_bind_addr() {
        let mut vars = BTreeMap::new();
        vars.insert("MSM_BIND_ADDR".to_owned(), "not-a-socket".to_owned());

        let error = AppConfig::from_env_map(&vars).expect_err("invalid bind address must fail");

        assert!(error.to_string().contains("invalid bind address"));
    }
}
