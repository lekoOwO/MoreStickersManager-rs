#![doc = "OIDC helper types for authorization-code callback hardening."]

use msm_storage::models::OidcProviderConfigRecord;
use std::{future::Future, pin::Pin};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    #[error("OIDC token response is invalid: {0}")]
    InvalidTokenResponse(String),
    #[error("OIDC token endpoint is invalid: {0}")]
    InvalidTokenEndpoint(String),
    #[error("OIDC token endpoint rejected the exchange: {0}")]
    TokenEndpointStatus(reqwest::StatusCode),
    #[error("OIDC token exchange HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("OIDC JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Owned request data for exchanging an OIDC authorization code for tokens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OidcTokenExchangeRequest {
    pub token_endpoint_url: String,
    pub form: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OidcTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub id_token: Option<String>,
    pub expires_in: Option<u64>,
}

#[derive(serde::Deserialize)]
struct RawTokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    id_token: Option<String>,
    expires_in: Option<u64>,
}

/// Boxed future returned by an OIDC token exchanger implementation.
pub type OidcTokenExchangeFuture =
    Pin<Box<dyn Future<Output = Result<OidcTokenResponse, OidcError>> + Send>>;

/// Exchanges an authorization code with a provider token endpoint.
pub trait OidcTokenExchanger: Send + Sync {
    fn exchange(&self, request: OidcTokenExchangeRequest) -> OidcTokenExchangeFuture;
}

/// HTTP implementation of [`OidcTokenExchanger`].
#[derive(Clone, Debug)]
pub struct HttpOidcTokenExchanger {
    client: reqwest::Client,
}

impl Default for HttpOidcTokenExchanger {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpOidcTokenExchanger {
    /// Creates an HTTP OIDC token exchanger with the default reqwest client.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl OidcTokenExchanger for HttpOidcTokenExchanger {
    fn exchange(&self, request: OidcTokenExchangeRequest) -> OidcTokenExchangeFuture {
        let client = self.client.clone();
        Box::pin(async move {
            let response = client
                .post(&request.token_endpoint_url)
                .form(&request.form)
                .send()
                .await?;
            let status = response.status();
            if !status.is_success() {
                return Err(OidcError::TokenEndpointStatus(status));
            }
            let body = response.text().await?;
            parse_token_response(&body)
        })
    }
}

/// Builds the token endpoint URL from the configured issuer URL.
///
/// # Errors
///
/// Returns an error when the issuer URL is not a valid absolute URL.
pub fn token_endpoint_url(issuer_url: &str) -> Result<String, OidcError> {
    let mut url = Url::parse(issuer_url)
        .map_err(|error| OidcError::InvalidTokenEndpoint(error.to_string()))?;
    let token_path = format!("{}/token", url.path().trim_end_matches('/'));
    url.set_path(&token_path);
    url.set_query(None);
    Ok(url.to_string())
}

/// Builds the `application/x-www-form-urlencoded` body for an OIDC token exchange.
#[must_use]
pub fn build_token_exchange_form(
    provider: &OidcProviderConfigRecord,
    code: &str,
    redirect_uri: &str,
) -> Vec<(String, String)> {
    vec![
        ("grant_type".to_owned(), "authorization_code".to_owned()),
        ("code".to_owned(), code.to_owned()),
        ("redirect_uri".to_owned(), redirect_uri.to_owned()),
        ("client_id".to_owned(), provider.client_id.clone()),
        ("client_secret".to_owned(), provider.client_secret.clone()),
    ]
}

/// Parses and validates the subset of an OIDC token response MSM currently needs.
///
/// # Errors
///
/// Returns an error when the response body is not JSON, does not include a non-empty
/// access token, or reports a token type other than `Bearer`.
pub fn parse_token_response(body: &str) -> Result<OidcTokenResponse, OidcError> {
    let raw: RawTokenResponse = serde_json::from_str(body)?;
    let access_token = raw
        .access_token
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| OidcError::InvalidTokenResponse("missing access_token".to_owned()))?;
    let token_type = raw
        .token_type
        .filter(|value| value.eq_ignore_ascii_case("Bearer"))
        .ok_or_else(|| OidcError::InvalidTokenResponse("token_type must be Bearer".to_owned()))?;

    Ok(OidcTokenResponse {
        access_token,
        token_type,
        id_token: raw.id_token,
        expires_in: raw.expires_in,
    })
}

#[cfg(test)]
mod tests {
    use msm_storage::models::OidcProviderConfigRecord;
    use std::collections::BTreeSet;

    use super::{build_token_exchange_form, parse_token_response};

    #[test]
    fn token_exchange_form_contains_authorization_code_parameters() {
        let provider = sample_provider();

        let form = build_token_exchange_form(
            &provider,
            "code-123",
            "https://msm.example/auth/oidc/callback",
        );

        assert_eq!(
            form,
            vec![
                ("grant_type".to_owned(), "authorization_code".to_owned()),
                ("code".to_owned(), "code-123".to_owned()),
                (
                    "redirect_uri".to_owned(),
                    "https://msm.example/auth/oidc/callback".to_owned(),
                ),
                ("client_id".to_owned(), "client-id".to_owned()),
                ("client_secret".to_owned(), "client-secret".to_owned()),
            ]
        );
    }

    #[test]
    fn token_response_parser_requires_access_token_and_bearer_type() {
        let parsed = parse_token_response(
            r#"{"access_token":"access-123","token_type":"Bearer","id_token":"id-123","expires_in":3600}"#,
        )
        .expect("valid token response should parse");

        assert_eq!(parsed.access_token, "access-123");
        assert_eq!(parsed.id_token.as_deref(), Some("id-123"));
        assert_eq!(parsed.expires_in, Some(3600));
        assert!(parse_token_response(r#"{"token_type":"Bearer"}"#).is_err());
        assert!(parse_token_response(r#"{"access_token":"access","token_type":"mac"}"#).is_err());
    }

    fn sample_provider() -> OidcProviderConfigRecord {
        OidcProviderConfigRecord {
            id: "google".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            display_name: "Google".to_owned(),
            issuer_url: "https://accounts.google.com".to_owned(),
            client_id: "client-id".to_owned(),
            client_secret: "client-secret".to_owned(),
            scopes: BTreeSet::from(["openid".to_owned(), "email".to_owned()]),
            is_enabled: true,
            allow_registration: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
