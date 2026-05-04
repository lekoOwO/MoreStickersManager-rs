use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApiErrorBody {
    pub error: ApiErrorPayload,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApiErrorPayload {
    pub code: &'static str,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", message),
        };

        (
            status,
            Json(ApiErrorBody {
                error: ApiErrorPayload { code, message },
            }),
        )
            .into_response()
    }
}

impl From<msm_storage::StorageError> for ApiError {
    fn from(error: msm_storage::StorageError) -> Self {
        match error {
            msm_storage::StorageError::InvalidPersonalAccessToken { .. } => {
                Self::BadRequest(error.to_string())
            }
            _ => Self::Internal(error.to_string()),
        }
    }
}

impl From<msm_domain::DomainError> for ApiError {
    fn from(error: msm_domain::DomainError) -> Self {
        Self::BadRequest(error.to_string())
    }
}
