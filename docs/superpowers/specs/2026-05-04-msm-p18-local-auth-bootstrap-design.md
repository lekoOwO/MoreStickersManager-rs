# MSM P18 Local Auth Bootstrap Design

## Scope

P18 adds a local-account bootstrap path for MSM. Users can register a local
account with a password and then log in to receive a PAT for API/CLI/Web/MCP use.

## Goals

- Store local password credentials separately from user profile records.
- Hash passwords with Argon2 PHC strings.
- Add storage APIs for local user registration and password verification.
- Add HTTP API routes:
  - `POST /api/v1/auth/local/register`
  - `POST /api/v1/auth/local/login`
- Return a raw PAT only from successful login.
- Keep existing PAT lifecycle endpoints unchanged.

## Non-Goals

- No OIDC/SSO in P18.
- No cookies or browser sessions.
- No password reset flow.
- No account email verification.
- No multi-tenant invitation flow.

## Login Contract

Login request:

```json
{
  "email": "leko@example.com",
  "password": "correct horse battery staple",
  "tokenId": "webui",
  "tokenName": "Web UI",
  "scopes": ["pack.read", "import.run", "pat.manage"],
  "expiresAt": null
}
```

Login response reuses the PAT create response shape and includes `token`.
