use std::collections::BTreeSet;

use axum::http::{
    header::{AUTHORIZATION, COOKIE},
    HeaderMap,
};
use msm_domain::Permission;

use crate::{ApiError, ApiResult, ApiState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedPat {
    pub user_id: String,
    pub scopes: BTreeSet<Permission>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedWebSession {
    pub user_id: String,
}

impl VerifiedWebSession {
    /// Requires the verified Web session to belong to the requested user.
    ///
    /// # Errors
    ///
    /// Returns forbidden when the session user differs from the requested user.
    pub fn require_user(&self, user_id: &str) -> ApiResult<()> {
        if self.user_id == user_id {
            Ok(())
        } else {
            Err(ApiError::Forbidden("Web session user mismatch".to_owned()))
        }
    }
}

impl VerifiedPat {
    /// Requires the verified PAT to belong to the requested user.
    ///
    /// # Errors
    ///
    /// Returns forbidden when the token user differs from the requested user.
    pub fn require_user(&self, user_id: &str) -> ApiResult<()> {
        if self.user_id == user_id {
            Ok(())
        } else {
            Err(ApiError::Forbidden("PAT user mismatch".to_owned()))
        }
    }
}

/// Verifies a Bearer PAT and requires a scope.
///
/// # Errors
///
/// Returns unauthorized for missing or invalid tokens, forbidden for missing scopes, and internal
/// errors when storage verification fails.
pub async fn require_pat(
    headers: &HeaderMap,
    state: &ApiState,
    required: Permission,
) -> ApiResult<VerifiedPat> {
    let token = bearer_token(headers)?;
    let record = state
        .repository()
        .verify_personal_access_token(token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid Personal Access Token".to_owned()))?;

    if !record.scopes.contains(&required) {
        return Err(ApiError::Forbidden(format!(
            "missing PAT scope `{}`",
            required.as_key()
        )));
    }

    Ok(VerifiedPat {
        user_id: record.user_id,
        scopes: record.scopes,
    })
}

/// Verifies an optional `msm_session` cookie.
///
/// # Errors
///
/// Returns unauthorized for an invalid session cookie and internal errors when storage
/// verification fails.
pub async fn optional_web_session(
    headers: &HeaderMap,
    state: &ApiState,
) -> ApiResult<Option<VerifiedWebSession>> {
    let Some(token) = web_session_token(headers)? else {
        return Ok(None);
    };
    let record = state
        .repository()
        .verify_web_session(token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid Web session".to_owned()))?;

    Ok(Some(VerifiedWebSession {
        user_id: record.user_id,
    }))
}

pub(crate) fn bearer_token(headers: &HeaderMap) -> ApiResult<&str> {
    let value = headers
        .get(AUTHORIZATION)
        .ok_or_else(|| ApiError::Unauthorized("missing Bearer token".to_owned()))?
        .to_str()
        .map_err(|_| ApiError::Unauthorized("invalid Authorization header".to_owned()))?;

    value
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .ok_or_else(|| ApiError::Unauthorized("missing Bearer token".to_owned()))
}

fn web_session_token(headers: &HeaderMap) -> ApiResult<Option<&str>> {
    let Some(value) = headers.get(COOKIE) else {
        return Ok(None);
    };
    let cookie = value
        .to_str()
        .map_err(|_| ApiError::Unauthorized("invalid Cookie header".to_owned()))?;

    Ok(cookie.split(';').find_map(|part| {
        let trimmed = part.trim();
        trimmed
            .strip_prefix("msm_session=")
            .filter(|token| !token.is_empty())
    }))
}
