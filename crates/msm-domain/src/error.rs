use std::path::PathBuf;

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("invalid sticker pack path extension: {path}")]
    InvalidStickerPackExtension { path: PathBuf },

    #[error("invalid provider id component `{component}`: {reason}")]
    InvalidProviderIdComponent {
        component: String,
        reason: &'static str,
    },

    #[error("invalid base URL `{url}`: {source}")]
    InvalidBaseUrl {
        url: String,
        source: url::ParseError,
    },

    #[error("invalid asset URL base: {url}")]
    InvalidAssetUrlBase { url: String },
}
