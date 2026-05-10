# OIDC / SSO User Guide

Last updated: 2026-05-11.

MSM supports tenant-scoped OIDC provider configuration and Web/API login flows.
After a successful callback, SSO-backed users use the same PAT, Web session,
RBAC, and scope-policy model as local users.

## Provider administration

Administrators need a PAT with `tenant.manage_settings` and an admin membership
in the target tenant.

API endpoints:

- `GET /api/v1/tenants/{tenant_id}/oidc-providers`
- `PUT /api/v1/tenants/{tenant_id}/oidc-providers/{provider_id}`
- `DELETE /api/v1/tenants/{tenant_id}/oidc-providers/{provider_id}`

CLI commands:

```powershell
msm tenants oidc-providers list --tenant-id <tenant_id>
msm tenants oidc-providers upsert --tenant-id <tenant_id> --provider-id <provider_id> --display-name <name> --issuer-url <issuer_url> --client-id <client_id> --client-secret <client_secret> --scope openid --scope email
msm tenants oidc-providers delete --tenant-id <tenant_id> --provider-id <provider_id>
```

MCP tools:

- `msm.list_oidc_providers`
- `msm.upsert_oidc_provider`
- `msm.delete_oidc_provider`

Provider responses redact `clientSecret`; upsert requests replace the stored
secret with the submitted value. Providers can be disabled, and
provider-backed registration can be denied per provider.

## Web login flow

1. Configure an enabled OIDC provider for the tenant.
2. Open the Web auth dialog.
3. Enter tenant ID, provider ID, redirect URI, desired token ID/name, and scopes.
4. Start OIDC login. MSM calls
   `GET /api/v1/auth/oidc/{tenant_id}/{provider_id}/login?redirectUri=...`,
   stores a one-time state/nonce, and returns the provider authorization URL.
5. Complete the provider login in the browser.
6. When the provider redirects back to `/auth/oidc/callback?code=...&state=...`,
   the Web UI pre-fills the callback form from the URL and pending state/nonce.
7. Submit callback completion. MSM exchanges the authorization code, verifies
   ID-token issuer/audience/nonce/expiration and RS256 JWKS signature, fetches
   userinfo when profile claims are missing, links or creates the tenant user,
   returns a raw PAT once, and sets an HttpOnly `msm_session` cookie.

## API callback request

```json
{
  "state": "opaque_state_from_login_start",
  "nonce": "opaque_nonce_from_login_start",
  "authorizationCode": "provider_authorization_code",
  "issuer": "https://accounts.example.test",
  "audience": "client-id",
  "providerSubject": "fallback-subject",
  "email": "fallback@example.test",
  "displayName": "Fallback Name",
  "tokenId": "webui-oidc",
  "tokenName": "Web OIDC",
  "scopes": ["pack.read"],
  "expiresAt": null
}
```

When `authorizationCode` is present and the provider returns a valid ID token,
MSM derives subject, email, and display name from verified provider claims.
Fallback claims are kept for compatibility with pre-validated provider flows and
are not preferred over verified token/userinfo data.

## PAT, CLI, and MCP usage after SSO

Use the returned raw PAT exactly like a local-login PAT:

```powershell
$env:MSM_PAT="msm_pat_from_oidc_callback"
msm packs list --user-id <linked_user_id>
```

For MCP, send it in the HTTP header:

```http
Authorization: Bearer msm_pat_from_oidc_callback
```

Token scopes are capped by the linked user's tenant role/template policy. Use
`GET /api/v1/pats/scope-policy`, the Web scope picker, or
`msm pats scope-policy --user-id <linked_user_id>` to choose valid scopes.
