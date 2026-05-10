#![doc = "OIDC helper types for authorization-code callback hardening."]

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use msm_storage::models::OidcProviderConfigRecord;
use std::{future::Future, pin::Pin};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    #[error("OIDC token response is invalid: {0}")]
    InvalidTokenResponse(String),
    #[error("OIDC ID token is invalid: {0}")]
    InvalidIdToken(String),
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

/// Validated OIDC discovery metadata used by MSM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OidcDiscoveryDocument {
    pub issuer: String,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub userinfo_endpoint: Option<String>,
}

/// Parsed JSON Web Key Set metadata used for ID-token signature validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OidcJwksDocument {
    pub keys: Vec<OidcJwk>,
}

/// Parsed JSON Web Key metadata for a provider signing key.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct OidcJwk {
    pub kty: String,
    pub kid: Option<String>,
    #[serde(rename = "use")]
    pub key_use: Option<String>,
    pub alg: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
}

/// Parsed ID-token header and claims before signature verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OidcIdTokenParts {
    pub header: OidcIdTokenHeader,
    pub claims: OidcIdTokenClaims,
    pub signing_input: String,
    pub signature: Vec<u8>,
}

/// Parsed ID-token JOSE header fields MSM needs for validation.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct OidcIdTokenHeader {
    pub alg: String,
    pub kid: Option<String>,
    pub typ: Option<String>,
}

/// Parsed ID-token claims MSM needs for validation and user linking.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct OidcIdTokenClaims {
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "sub")]
    pub subject: String,
    #[serde(rename = "aud", deserialize_with = "deserialize_audience")]
    pub audience: Vec<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub nonce: Option<String>,
    #[serde(rename = "exp")]
    pub expires_at: i64,
    #[serde(rename = "iat")]
    pub issued_at: Option<i64>,
}

impl OidcJwksDocument {
    /// Selects a signature key matching an optional JWT `kid` and `alg`.
    #[must_use]
    pub fn select_signature_key(&self, kid: Option<&str>, alg: Option<&str>) -> Option<&OidcJwk> {
        self.keys.iter().find(|key| {
            key.key_use.as_deref().is_none_or(|usage| usage == "sig")
                && kid.is_none_or(|expected| key.kid.as_deref() == Some(expected))
                && alg.is_none_or(|expected| {
                    key.alg.as_deref().is_none_or(|key_alg| key_alg == expected)
                })
        })
    }
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

#[derive(serde::Deserialize)]
struct RawDiscoveryDocument {
    issuer: Option<String>,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
    jwks_uri: Option<String>,
    userinfo_endpoint: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawJwksDocument {
    keys: Vec<OidcJwk>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum AudienceClaim {
    Single(String),
    Multiple(Vec<String>),
}

/// Boxed future returned by an OIDC token exchanger implementation.
pub type OidcTokenExchangeFuture =
    Pin<Box<dyn Future<Output = Result<OidcTokenResponse, OidcError>> + Send>>;

/// Boxed future returned by an OIDC discovery fetcher implementation.
pub type OidcDiscoveryFuture =
    Pin<Box<dyn Future<Output = Result<OidcDiscoveryDocument, OidcError>> + Send>>;

/// Exchanges an authorization code with a provider token endpoint.
pub trait OidcTokenExchanger: Send + Sync {
    fn exchange(&self, request: OidcTokenExchangeRequest) -> OidcTokenExchangeFuture;
}

/// Fetches and validates OIDC discovery metadata for an issuer.
pub trait OidcDiscoveryFetcher: Send + Sync {
    fn discover(&self, issuer_url: String) -> OidcDiscoveryFuture;
}

/// HTTP implementation of [`OidcTokenExchanger`].
#[derive(Clone, Debug)]
pub struct HttpOidcTokenExchanger {
    client: reqwest::Client,
}

/// HTTP implementation of [`OidcDiscoveryFetcher`].
#[derive(Clone, Debug)]
pub struct HttpOidcDiscoveryFetcher {
    client: reqwest::Client,
}

impl Default for HttpOidcTokenExchanger {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HttpOidcDiscoveryFetcher {
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

impl HttpOidcDiscoveryFetcher {
    /// Creates an HTTP OIDC discovery fetcher with the default reqwest client.
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

impl OidcDiscoveryFetcher for HttpOidcDiscoveryFetcher {
    fn discover(&self, issuer_url: String) -> OidcDiscoveryFuture {
        let client = self.client.clone();
        Box::pin(async move {
            let discovery_url = discovery_document_url(&issuer_url)?;
            let response = client.get(discovery_url).send().await?;
            let status = response.status();
            if !status.is_success() {
                return Err(OidcError::TokenEndpointStatus(status));
            }
            let body = response.text().await?;
            parse_discovery_document(&body, &issuer_url)
        })
    }
}

/// Builds the standard OIDC discovery document URL for an issuer.
///
/// # Errors
///
/// Returns an error when the issuer URL is not a valid absolute URL.
pub fn discovery_document_url(issuer_url: &str) -> Result<String, OidcError> {
    let mut url = Url::parse(issuer_url)
        .map_err(|error| OidcError::InvalidTokenEndpoint(error.to_string()))?;
    let discovery_path = format!(
        "{}/.well-known/openid-configuration",
        url.path().trim_end_matches('/')
    );
    url.set_path(&discovery_path);
    url.set_query(None);
    Ok(url.to_string())
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

/// Parses and validates an OIDC discovery document.
///
/// # Errors
///
/// Returns an error when the document is not JSON, the issuer does not match the
/// expected issuer, or required endpoint URLs are missing or invalid.
pub fn parse_discovery_document(
    body: &str,
    expected_issuer: &str,
) -> Result<OidcDiscoveryDocument, OidcError> {
    let raw: RawDiscoveryDocument = serde_json::from_str(body)?;
    let issuer = required_url_field(raw.issuer, "issuer")?;
    if normalize_url_claim(&issuer) != normalize_url_claim(expected_issuer) {
        return Err(OidcError::InvalidTokenResponse(
            "discovery issuer mismatch".to_owned(),
        ));
    }
    Ok(OidcDiscoveryDocument {
        issuer,
        authorization_endpoint: optional_url_field(
            raw.authorization_endpoint,
            "authorization_endpoint",
        )?,
        token_endpoint: required_url_field(raw.token_endpoint, "token_endpoint")?,
        jwks_uri: required_url_field(raw.jwks_uri, "jwks_uri")?,
        userinfo_endpoint: optional_url_field(raw.userinfo_endpoint, "userinfo_endpoint")?,
    })
}

/// Parses and validates a provider JWKS document.
///
/// # Errors
///
/// Returns an error when the document is not JSON, contains no keys, or does not
/// include any usable RSA signature keys.
pub fn parse_jwks_document(body: &str) -> Result<OidcJwksDocument, OidcError> {
    let raw: RawJwksDocument = serde_json::from_str(body)?;
    if raw.keys.is_empty() {
        return Err(OidcError::InvalidTokenResponse(
            "JWKS contains no keys".to_owned(),
        ));
    }
    let keys = raw
        .keys
        .into_iter()
        .filter(|key| {
            key.kty == "RSA"
                && key.key_use.as_deref().is_none_or(|usage| usage == "sig")
                && key.n.as_deref().is_some_and(|value| !value.is_empty())
                && key.e.as_deref().is_some_and(|value| !value.is_empty())
        })
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return Err(OidcError::InvalidTokenResponse(
            "JWKS contains no RSA signature keys".to_owned(),
        ));
    }
    Ok(OidcJwksDocument { keys })
}

/// Parses the header and claims of a compact ID token before signature verification.
///
/// # Errors
///
/// Returns an error when the token is not compact JWT-shaped, contains invalid
/// base64url segments, or has JSON header/claims that do not include required
/// OIDC fields.
pub fn parse_id_token_unverified(token: &str) -> Result<OidcIdTokenParts, OidcError> {
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(OidcError::InvalidIdToken(
            "expected compact JWT with three segments".to_owned(),
        ));
    }
    let header_bytes = decode_jwt_segment(parts[0], "header")?;
    let claims_bytes = decode_jwt_segment(parts[1], "claims")?;
    let signature = decode_jwt_segment(parts[2], "signature")?;
    let header = serde_json::from_slice::<OidcIdTokenHeader>(&header_bytes)?;
    let claims = serde_json::from_slice::<OidcIdTokenClaims>(&claims_bytes)?;
    if header.alg.trim().is_empty() {
        return Err(OidcError::InvalidIdToken("missing alg".to_owned()));
    }
    if claims.issuer.trim().is_empty() || claims.subject.trim().is_empty() {
        return Err(OidcError::InvalidIdToken(
            "missing issuer or subject".to_owned(),
        ));
    }
    if claims.audience.is_empty() || claims.audience.iter().any(|value| value.trim().is_empty()) {
        return Err(OidcError::InvalidIdToken("missing audience".to_owned()));
    }
    Ok(OidcIdTokenParts {
        header,
        claims,
        signing_input: format!("{}.{}", parts[0], parts[1]),
        signature,
    })
}

/// Validates ID-token claims after signature verification has selected a trusted token.
///
/// # Errors
///
/// Returns an error when issuer, audience, nonce, or expiration do not match
/// the expected callback context.
pub fn validate_id_token_claims(
    claims: &OidcIdTokenClaims,
    expected_issuer: &str,
    expected_audience: &str,
    expected_nonce: &str,
    now_unix_seconds: i64,
) -> Result<(), OidcError> {
    if normalize_url_claim(&claims.issuer) != normalize_url_claim(expected_issuer) {
        return Err(OidcError::InvalidIdToken("issuer mismatch".to_owned()));
    }
    if !claims
        .audience
        .iter()
        .any(|audience| audience == expected_audience)
    {
        return Err(OidcError::InvalidIdToken("audience mismatch".to_owned()));
    }
    if claims.nonce.as_deref() != Some(expected_nonce) {
        return Err(OidcError::InvalidIdToken("nonce mismatch".to_owned()));
    }
    if claims.expires_at <= now_unix_seconds {
        return Err(OidcError::InvalidIdToken("token expired".to_owned()));
    }
    Ok(())
}

fn decode_jwt_segment(segment: &str, label: &str) -> Result<Vec<u8>, OidcError> {
    URL_SAFE_NO_PAD
        .decode(segment.as_bytes())
        .map_err(|error| OidcError::InvalidIdToken(format!("invalid {label}: {error}")))
}

fn deserialize_audience<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let audience = serde::Deserialize::deserialize(deserializer)?;
    Ok(match audience {
        AudienceClaim::Single(value) => vec![value],
        AudienceClaim::Multiple(values) => values,
    })
}

fn required_url_field(value: Option<String>, field: &str) -> Result<String, OidcError> {
    let value = value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| OidcError::InvalidTokenResponse(format!("missing {field}")))?;
    Url::parse(&value)
        .map_err(|error| OidcError::InvalidTokenResponse(format!("invalid {field}: {error}")))?;
    Ok(value)
}

fn optional_url_field(value: Option<String>, field: &str) -> Result<Option<String>, OidcError> {
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            Url::parse(&value).map_err(|error| {
                OidcError::InvalidTokenResponse(format!("invalid {field}: {error}"))
            })?;
            Ok(value)
        })
        .transpose()
}

fn normalize_url_claim(value: &str) -> String {
    value.trim_end_matches('/').to_owned()
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

    #[test]
    fn discovery_parser_requires_matching_issuer_and_required_endpoints() {
        let parsed = super::parse_discovery_document(
            r#"{
                "issuer":"https://accounts.google.com",
                "authorization_endpoint":"https://accounts.google.com/o/oauth2/v2/auth",
                "token_endpoint":"https://oauth2.googleapis.com/token",
                "jwks_uri":"https://www.googleapis.com/oauth2/v3/certs",
                "userinfo_endpoint":"https://openidconnect.googleapis.com/v1/userinfo"
            }"#,
            "https://accounts.google.com/",
        )
        .expect("matching discovery document should parse");

        assert_eq!(parsed.issuer, "https://accounts.google.com");
        assert_eq!(parsed.token_endpoint, "https://oauth2.googleapis.com/token");
        assert_eq!(
            parsed.jwks_uri,
            "https://www.googleapis.com/oauth2/v3/certs"
        );
        assert!(super::parse_discovery_document(
            r#"{"issuer":"https://evil.example","token_endpoint":"https://oauth2.googleapis.com/token","jwks_uri":"https://www.googleapis.com/oauth2/v3/certs"}"#,
            "https://accounts.google.com"
        )
        .is_err());
        assert!(super::parse_discovery_document(
            r#"{"issuer":"https://accounts.google.com","token_endpoint":"","jwks_uri":"https://www.googleapis.com/oauth2/v3/certs"}"#,
            "https://accounts.google.com"
        )
        .is_err());
    }

    #[test]
    fn jwks_parser_selects_signature_key_by_kid_and_algorithm() {
        let jwks = super::parse_jwks_document(
            r#"{
                "keys": [
                    {"kty":"RSA","kid":"old-key","use":"sig","alg":"RS256","n":"old-modulus","e":"AQAB"},
                    {"kty":"RSA","kid":"active-key","use":"sig","alg":"RS256","n":"active-modulus","e":"AQAB"},
                    {"kty":"RSA","kid":"enc-key","use":"enc","alg":"RS256","n":"enc-modulus","e":"AQAB"}
                ]
            }"#,
        )
        .expect("JWKS should parse");

        let key = jwks
            .select_signature_key(Some("active-key"), Some("RS256"))
            .expect("matching signature key should be selected");
        assert_eq!(key.kid.as_deref(), Some("active-key"));
        assert_eq!(key.n.as_deref(), Some("active-modulus"));
        assert!(jwks
            .select_signature_key(Some("enc-key"), Some("RS256"))
            .is_none());
        assert!(super::parse_jwks_document(r#"{"keys":[]}"#).is_err());
    }

    #[test]
    fn id_token_parser_decodes_header_and_claims_without_trusting_signature() {
        let token = [
            r#"{"alg":"RS256","kid":"active-key","typ":"JWT"}"#,
            r#"{"iss":"https://issuer.example","sub":"subject-1","aud":["client-id","other-client"],"email":"leko@example.com","name":"Leko","nonce":"nonce-1","exp":1893456000,"iat":1704067200}"#,
            "signature",
        ]
        .into_iter()
        .map(base64_url_no_pad)
        .collect::<Vec<_>>()
        .join(".");

        let parsed = super::parse_id_token_unverified(&token)
            .expect("well-formed ID token should parse before signature validation");

        assert_eq!(parsed.header.alg, "RS256");
        assert_eq!(parsed.header.kid.as_deref(), Some("active-key"));
        assert_eq!(parsed.claims.issuer, "https://issuer.example");
        assert_eq!(parsed.claims.subject, "subject-1");
        assert_eq!(parsed.claims.audience, vec!["client-id", "other-client"]);
        assert_eq!(parsed.claims.email.as_deref(), Some("leko@example.com"));
        assert_eq!(parsed.claims.name.as_deref(), Some("Leko"));
        assert_eq!(parsed.claims.nonce.as_deref(), Some("nonce-1"));
        assert_eq!(parsed.claims.expires_at, 1_893_456_000);
        assert!(super::parse_id_token_unverified("not.a.jwt.with.extra.parts").is_err());
    }

    #[test]
    fn id_token_claim_validator_rejects_provider_claim_mismatches() {
        let token = [
            r#"{"alg":"RS256","kid":"active-key","typ":"JWT"}"#,
            r#"{"iss":"https://issuer.example","sub":"subject-1","aud":"client-id","email":"leko@example.com","nonce":"nonce-1","exp":1893456000,"iat":1704067200}"#,
            "signature",
        ]
        .into_iter()
        .map(base64_url_no_pad)
        .collect::<Vec<_>>()
        .join(".");
        let parsed = super::parse_id_token_unverified(&token).unwrap();

        super::validate_id_token_claims(
            &parsed.claims,
            "https://issuer.example/",
            "client-id",
            "nonce-1",
            1_704_067_300,
        )
        .expect("matching claims should validate");
        assert!(super::validate_id_token_claims(
            &parsed.claims,
            "https://evil.example",
            "client-id",
            "nonce-1",
            1_704_067_300,
        )
        .is_err());
        assert!(super::validate_id_token_claims(
            &parsed.claims,
            "https://issuer.example",
            "other-client",
            "nonce-1",
            1_704_067_300,
        )
        .is_err());
        assert!(super::validate_id_token_claims(
            &parsed.claims,
            "https://issuer.example",
            "client-id",
            "wrong-nonce",
            1_704_067_300,
        )
        .is_err());
        assert!(super::validate_id_token_claims(
            &parsed.claims,
            "https://issuer.example",
            "client-id",
            "nonce-1",
            1_893_456_001,
        )
        .is_err());
    }

    fn base64_url_no_pad(value: &str) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

        URL_SAFE_NO_PAD.encode(value.as_bytes())
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
