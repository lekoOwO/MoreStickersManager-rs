#![doc = "Runnable service composition for MoreStickersManager-rs."]

use std::{
    collections::BTreeMap,
    env,
    net::{AddrParseError, SocketAddr},
    path::{Path, PathBuf},
};

use axum::{
    body::Body,
    extract::OriginalUri,
    http::{header::CONTENT_TYPE, Response, StatusCode, Uri},
    routing::get,
    Router,
};
use bytes::Bytes;
use include_dir::{include_dir, Dir};
use msm_api::{build_router, ApiState};
use msm_storage::{DatabaseConfig, DbPool, LocalAssetStore, StorageRepository};

pub mod export_worker;

pub use export_worker::{ExportWorker, ExportWorkerConfig, ExportWorkerError, ExportWorkerResult};

static EMBEDDED_WEB_DIR: Dir<'_> = include_dir!("$OUT_DIR/web-dist-embed");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub asset_dir: PathBuf,
    pub web_dist_dir: PathBuf,
    pub export_worker: ExportWorkerConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid bind address `{value}`: {source}")]
    InvalidBindAddress {
        value: String,
        source: AddrParseError,
    },

    #[error("invalid numeric environment value `{key}` = `{value}`")]
    InvalidNumber { key: String, value: String },

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
    pub const DEFAULT_FFMPEG_PATH: &'static str = "ffmpeg";
    pub const DEFAULT_FFPROBE_PATH: &'static str = "ffprobe";
    pub const DEFAULT_EXPORT_MAX_CONCURRENT_JOBS: usize = 1;

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
        let max_concurrent_jobs = read_usize(
            vars,
            "MSM_EXPORT_MAX_CONCURRENT_JOBS",
            Self::DEFAULT_EXPORT_MAX_CONCURRENT_JOBS,
        )?;
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
            export_worker: ExportWorkerConfig {
                ffmpeg_path: PathBuf::from(read(
                    vars,
                    "MSM_FFMPEG_PATH",
                    Self::DEFAULT_FFMPEG_PATH,
                )),
                ffprobe_path: PathBuf::from(read(
                    vars,
                    "MSM_FFPROBE_PATH",
                    Self::DEFAULT_FFPROBE_PATH,
                )),
                max_concurrent_jobs,
            },
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
    let assets = WebAssets::new(web_dist_dir.into());
    build_router(state.clone())
        .merge(msm_mcp::build_router(state))
        .fallback(get(move |OriginalUri(uri): OriginalUri| {
            serve_web_asset(uri, assets.clone())
        }))
}

#[derive(Clone, Debug)]
struct WebAssets {
    disk_dir: PathBuf,
}

impl WebAssets {
    fn new(disk_dir: PathBuf) -> Self {
        Self { disk_dir }
    }
}

async fn serve_web_asset(uri: Uri, assets: WebAssets) -> Response<Body> {
    let Some(path) = normalized_web_path(uri.path()) else {
        return status_response(StatusCode::BAD_REQUEST);
    };

    if let Some(response) = disk_asset_response(&assets.disk_dir, &path).await {
        return response;
    }

    embedded_asset_response(&path)
        .or_else(|| embedded_asset_response("index.html"))
        .unwrap_or_else(|| status_response(StatusCode::NOT_FOUND))
}

fn read(vars: &BTreeMap<String, String>, key: &str, default: &str) -> String {
    vars.get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| default.to_owned())
}

fn read_usize(vars: &BTreeMap<String, String>, key: &str, default: usize) -> AppResult<usize> {
    let Some(value) = vars.get(key).filter(|value| !value.trim().is_empty()) else {
        return Ok(default);
    };
    let parsed = value
        .parse::<usize>()
        .map_err(|_| AppError::InvalidNumber {
            key: key.to_owned(),
            value: value.to_owned(),
        })?;
    if parsed == 0 {
        Err(AppError::InvalidNumber {
            key: key.to_owned(),
            value: value.to_owned(),
        })
    } else {
        Ok(parsed)
    }
}

fn normalized_web_path(path: &str) -> Option<String> {
    let trimmed = path.trim_start_matches('/');
    let candidate = if trimmed.is_empty() || trimmed.ends_with('/') {
        "index.html"
    } else {
        trimmed
    };

    if candidate.contains('\\')
        || candidate
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return None;
    }

    Some(candidate.to_owned())
}

async fn disk_asset_response(disk_dir: &Path, path: &str) -> Option<Response<Body>> {
    let disk_path = disk_dir.join(path);
    let bytes = tokio::fs::read(&disk_path).await.ok()?;
    Some(asset_response(path, Bytes::from(bytes)))
}

fn embedded_asset_response(path: &str) -> Option<Response<Body>> {
    let file = EMBEDDED_WEB_DIR.get_file(path)?;
    Some(asset_response(
        path,
        Bytes::copy_from_slice(file.contents()),
    ))
}

fn asset_response(path: &str, bytes: Bytes) -> Response<Body> {
    Response::builder()
        .header(
            CONTENT_TYPE,
            mime_guess::from_path(path)
                .first_or_octet_stream()
                .essence_str(),
        )
        .body(Body::from(bytes))
        .expect("asset response should be buildable")
}

fn status_response(status: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::empty())
        .expect("status response should be buildable")
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
        assert_eq!(config.export_worker.ffmpeg_path, PathBuf::from("ffmpeg"));
        assert_eq!(config.export_worker.ffprobe_path, PathBuf::from("ffprobe"));
        assert_eq!(config.export_worker.max_concurrent_jobs, 1);
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
        vars.insert("MSM_FFMPEG_PATH".to_owned(), "bin/ffmpeg".to_owned());
        vars.insert("MSM_FFPROBE_PATH".to_owned(), "bin/ffprobe".to_owned());
        vars.insert("MSM_EXPORT_MAX_CONCURRENT_JOBS".to_owned(), "4".to_owned());

        let config = AppConfig::from_env_map(&vars).unwrap();

        assert_eq!(
            config.bind_addr,
            "0.0.0.0:8080".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.database_url, "sqlite:data/test.sqlite3");
        assert_eq!(config.asset_dir, PathBuf::from("tmp/assets"));
        assert_eq!(config.web_dist_dir, PathBuf::from("tmp/web"));
        assert_eq!(
            config.export_worker.ffmpeg_path,
            PathBuf::from("bin/ffmpeg")
        );
        assert_eq!(
            config.export_worker.ffprobe_path,
            PathBuf::from("bin/ffprobe")
        );
        assert_eq!(config.export_worker.max_concurrent_jobs, 4);
    }

    #[test]
    fn config_rejects_invalid_bind_addr() {
        let mut vars = BTreeMap::new();
        vars.insert("MSM_BIND_ADDR".to_owned(), "not-a-socket".to_owned());

        let error = AppConfig::from_env_map(&vars).expect_err("invalid bind address must fail");

        assert!(error.to_string().contains("invalid bind address"));
    }

    #[test]
    fn config_rejects_invalid_export_worker_concurrency() {
        let mut vars = BTreeMap::new();
        vars.insert("MSM_EXPORT_MAX_CONCURRENT_JOBS".to_owned(), "0".to_owned());

        let error = AppConfig::from_env_map(&vars).expect_err("zero concurrency must fail");

        assert!(error.to_string().contains("MSM_EXPORT_MAX_CONCURRENT_JOBS"));
    }

    #[test]
    fn normalizes_safe_web_paths() {
        assert_eq!(
            super::normalized_web_path("/").as_deref(),
            Some("index.html")
        );
        assert_eq!(
            super::normalized_web_path("/assets/index.js").as_deref(),
            Some("assets/index.js")
        );
        assert!(super::normalized_web_path("/../secret").is_none());
        assert!(super::normalized_web_path("/assets/../secret").is_none());
        assert!(super::normalized_web_path("/assets\\secret").is_none());
    }

    #[test]
    fn embedded_index_exists() {
        assert!(super::EMBEDDED_WEB_DIR.get_file("index.html").is_some());
    }
}
