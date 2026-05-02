pub type ProviderResult<T> = Result<T, ProviderError>;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("invalid provider JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("domain error: {0}")]
    Domain(#[from] msm_domain::DomainError),

    #[error("invalid provider payload: {0}")]
    InvalidPayload(String),
}
