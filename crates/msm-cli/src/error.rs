use std::path::PathBuf;

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("invalid base URL `{url}`: {source}")]
    InvalidBaseUrl {
        url: String,
        source: url::ParseError,
    },

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("I/O error for `{path}`: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("domain error: {0}")]
    Domain(#[from] msm_domain::DomainError),

    #[error("client error: {0}")]
    Client(String),
}
