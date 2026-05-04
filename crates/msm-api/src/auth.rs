use std::collections::BTreeSet;

use axum::http::{header::AUTHORIZATION, HeaderMap};
use msm_domain::Permission;

use crate::{ApiError, ApiResult, ApiState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedPat {
    pub user_id: String,
    pub scopes: BTreeSet<Permission>,
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

fn bearer_token(headers: &HeaderMap) -> ApiResult<&str> {
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
