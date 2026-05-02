#![doc = "HTTP API and OpenAPI surface for MoreStickersManager-rs."]

pub mod dto;
pub mod error;
pub mod openapi;
pub mod routes;
pub mod state;

pub use error::{ApiError, ApiResult};
pub use state::ApiState;
