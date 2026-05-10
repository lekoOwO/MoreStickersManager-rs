use axum::http::{header, HeaderMap};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ring::digest;

use crate::{ApiError, ApiResult, ApiState};

const FALLBACK_KEY: &str = "anonymous";

pub fn enforce_import_rate_limit(headers: &HeaderMap, state: &ApiState) -> ApiResult<()> {
    let key = import_rate_limit_key(headers);
    if state.check_import_rate_limit(&key) {
        Ok(())
    } else {
        Err(ApiError::TooManyRequests(
            "import rate limit exceeded; retry after the configured window".to_owned(),
        ))
    }
}

fn import_rate_limit_key(headers: &HeaderMap) -> String {
    let identity = header_value(headers, header::AUTHORIZATION.as_str())
        .or_else(|| header_value(headers, "x-forwarded-for"))
        .or_else(|| header_value(headers, "x-real-ip"))
        .unwrap_or(FALLBACK_KEY);
    let digest = digest::digest(&digest::SHA256, identity.as_bytes());
    URL_SAFE_NO_PAD.encode(digest.as_ref())
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(name)?
        .to_str()
        .ok()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
