# Docker Compose deployment example

This directory contains a runnable Docker Compose example for MSM with
PostgreSQL and an Authentik OIDC/SSO provider.

## Files

- `docker-compose.yml`: builds the local repository image and runs MSM plus
  PostgreSQL.
- `.env.example`: copy to `.env` and fill deployment-specific values.

## Start MSM

From the repository root:

```bash
cp examples/docker/.env.example examples/docker/.env
# edit examples/docker/.env
docker compose --env-file examples/docker/.env -f examples/docker/docker-compose.yml up -d --build
```

Check readiness:

```bash
curl -fsS http://localhost:3000/readyz
```

Open the Web UI at the value of `MSM_EXTERNAL_URL`, for example
`http://localhost:3000`.

## Authentik application/provider settings

Create an Authentik OAuth2/OpenID provider and application for MSM.

Recommended Authentik provider values:

| Authentik field | Value |
| --- | --- |
| Provider type | OAuth2/OpenID Provider |
| Signing key | Your normal Authentik signing key |
| Client type | Confidential |
| Authorization flow | Your normal explicit-consent or default authorization flow |
| Redirect URI / allowed redirect URI | `${MSM_EXTERNAL_URL}/auth/oidc/callback` |
| Scopes | `openid`, `email`, `profile` |
| Subject mode | stable per-user subject, usually Authentik default |

Then copy these Authentik values into `examples/docker/.env`:

- `MSM_OIDC_ISSUER_URL` — usually
  `https://<authentik-host>/application/o/<slug>/`.
- `MSM_OIDC_CLIENT_ID`.
- `MSM_OIDC_CLIENT_SECRET`.
- `MSM_OIDC_SCOPES` — normally `openid email profile`.

The OIDC values in `.env` are setup inputs. `msm-app` stores OIDC providers in
its database, so you still need to register the provider once through Web Tenant
admin, CLI, MCP, or API.

## Bootstrap the first tenant admin

Register a local admin once. This creates the tenant and admin membership; the
login response returns a PAT that can configure the OIDC provider.

PowerShell example:

```powershell
$envFile = Get-Content examples/docker/.env | Where-Object { $_ -match '^[^#].+=' }
$vars = @{}
foreach ($line in $envFile) {
  $key, $value = $line -split '=', 2
  $vars[$key] = $value
}

$baseUrl = $vars.MSM_EXTERNAL_URL
$registerBody = @{
  id = $vars.MSM_ADMIN_USER_ID
  email = $vars.MSM_ADMIN_EMAIL
  displayName = $vars.MSM_ADMIN_DISPLAY_NAME
  password = $vars.MSM_ADMIN_PASSWORD
  tenantId = $vars.MSM_TENANT_ID
  tenantName = $vars.MSM_TENANT_NAME
  tenantRole = 'admin'
} | ConvertTo-Json

Invoke-RestMethod -Method Post -Uri "$baseUrl/api/v1/auth/local/register" -ContentType 'application/json' -Body $registerBody

$loginBody = @{
  email = $vars.MSM_ADMIN_EMAIL
  password = $vars.MSM_ADMIN_PASSWORD
  tokenId = 'admin-bootstrap'
  tokenName = 'Admin bootstrap'
  scopes = @('tenant.manage_settings','tenant.manage_members','tenant.manage_users','tenant.manage_roles','pat.manage','pack.create','pack.read','pack.update','pack.delete','pack.manage_access','asset.read','import.run','provider.import','export.read','export.run','export.target.manage','subscription.create','subscription.read')
  expiresAt = $null
} | ConvertTo-Json -Depth 4

$login = Invoke-RestMethod -Method Post -Uri "$baseUrl/api/v1/auth/local/login" -ContentType 'application/json' -Body $loginBody
$pat = $login.rawToken
$pat
```

If the user already exists, skip registration and run only the login step.

## Register the Authentik provider in MSM

PowerShell example using the PAT from the previous step:

```powershell
$oidcBody = @{
  displayName = $vars.MSM_OIDC_DISPLAY_NAME
  issuerUrl = $vars.MSM_OIDC_ISSUER_URL
  clientId = $vars.MSM_OIDC_CLIENT_ID
  clientSecret = $vars.MSM_OIDC_CLIENT_SECRET
  scopes = $vars.MSM_OIDC_SCOPES.Split(' ', [System.StringSplitOptions]::RemoveEmptyEntries)
  isEnabled = $true
  allowRegistration = [System.Convert]::ToBoolean($vars.MSM_OIDC_ALLOW_REGISTRATION)
} | ConvertTo-Json -Depth 4

Invoke-RestMethod `
  -Method Put `
  -Uri "$baseUrl/api/v1/tenants/$($vars.MSM_TENANT_ID)/oidc-providers/$($vars.MSM_OIDC_PROVIDER_ID)" `
  -Headers @{ Authorization = "Bearer $pat" } `
  -ContentType 'application/json' `
  -Body $oidcBody
```

You can also configure the same provider in the Web UI:

1. Log in with the local admin.
2. Open **Tenant admin**.
3. Fill OIDC provider ID, display name, issuer URL, client ID, client secret,
   scopes, enabled, and allow-registration.
4. Save the provider.
5. Open the auth dialog and start SSO with provider `authentik`.

## Required information from you

To make the example concrete for your deployment, provide:

1. Public MSM URL, for example `https://stickers.example.com`.
2. Authentik issuer URL, usually
   `https://auth.example.com/application/o/<application-slug>/`.
3. Authentik client ID and client secret.
4. Whether SSO auto-registration should be enabled.
5. Whether you want SQLite instead of PostgreSQL.
6. Whether export/provider workers should be enabled immediately. The example
   image includes `ffmpeg`/`ffprobe`, but workers are disabled by default so you
   can enable them deliberately after configuring Telegram/provider secrets.
