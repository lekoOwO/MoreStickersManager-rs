use std::path::PathBuf;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("invalid database URL `{url}`: {reason}")]
    InvalidDatabaseUrl { url: String, reason: &'static str },

    #[error("database kind `{kind}` is not supported by this operation")]
    UnsupportedDatabaseKind { kind: String },

    #[error("invalid asset key component `{component}`: {reason}")]
    InvalidAssetKey {
        component: String,
        reason: &'static str,
    },

    #[error("invalid personal access token: {reason}")]
    InvalidPersonalAccessToken { reason: &'static str },

    #[error("invalid timestamp `{value}`: {message}")]
    InvalidTimestamp { value: String, message: String },

    #[error("password hash error: {message}")]
    PasswordHash { message: String },

    #[error("random generation error: {message}")]
    Random { message: String },

    #[error("asset not found: {path}")]
    AssetNotFound { path: PathBuf },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("domain error: {0}")]
    Domain(#[from] msm_domain::DomainError),
}
