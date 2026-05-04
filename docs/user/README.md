# MSM User Documentation

MSM currently has foundation, storage, authorization, API, CLI, provider
normalization, and Web UI foundation slices.

Current usable contract: `.stickerpack` compatibility is documented in `../dev/compatibility.md`.

Provider normalization status is documented in `../dev/providers.md`.

Current CLI examples:

```powershell
cargo run -p msm-cli -- health
cargo run -p msm-cli -- packs list --user-id user_1
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
cargo run -p msm-cli -- packs rename --pack-id pack_1 --title "Renamed Pack" --visibility public
cargo run -p msm-cli -- packs delete --pack-id pack_1
cargo run -p msm-cli -- pats create --id cli1 --user-id user_1 --name CLI --scope pack.read --scope asset.read
cargo run -p msm-cli -- pats list --user-id user_1
cargo run -p msm-cli -- pats revoke --token-id cli1
```

Protected API-backed CLI commands accept a PAT through `--pat` or `MSM_PAT`:

```powershell
cargo run -p msm-cli -- --pat msm_pat_cli1_secret packs list --user-id user_1
$env:MSM_PAT="msm_pat_cli1_secret"
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
```

Current Web UI examples:

```powershell
npm run web:dev
npm run web:build
```

The P7 Web UI uses mock data. It demonstrates the app shell, responsive layout,
theme toggle, language toggle, and sticker-pack dashboard before backend API
integration is wired in.

To point the dashboard at the current pack-list API:

```powershell
$env:VITE_MSM_API_BASE_URL="http://localhost:3000"
$env:VITE_MSM_USER_ID="user_1"
$env:VITE_MSM_PAT="msm_pat_cli1_secret"
npm run web:dev
```

When `VITE_MSM_API_BASE_URL` is omitted, the dashboard falls back to mock data.
When it is configured, the Web UI can store a PAT in browser localStorage and
send it to protected pack API calls. `VITE_MSM_PAT` can seed the token during
development.

Current service binary example:

```powershell
npm run web:build
cargo run -p msm-app
```

By default the service listens on `127.0.0.1:3000`, uses
`sqlite:data/msm.sqlite3`, stores assets under `data/assets`, and serves the Web
UI from `apps/web/dist` when present. The binary also embeds Web assets at
compile time; run `npm run web:build` before `cargo build -p msm-app` to embed
the full UI instead of the placeholder.

Current MCP endpoint:

```text
POST /mcp
```

Supported methods are `initialize`, `ping`, `tools/list`, and `tools/call`.
Current tools are `msm.list_sticker_packs`, `msm.export_sticker_pack`,
`msm.import_sticker_pack`, `msm.update_sticker_pack`, and
`msm.delete_sticker_pack`.

PAT foundation status:

- token format is `msm_pat_<token_id>_<random_secret>`;
- only the secret hash is stored;
- scope keys include values such as `pack.read`, `asset.read`, and `pat.manage`;
- API/CLI/MCP pack operations use Bearer PAT enforcement.

PAT API endpoints:

- `POST /api/v1/pats`
- `GET /api/v1/pats?userId=...`
- `DELETE /api/v1/pats/{token_id}`

PAT CLI commands:

- `msm pats create --id <token_id> --user-id <user_id> --name <name> --scope <scope>`
- `msm pats list --user-id <user_id>`
- `msm pats revoke --token-id <token_id>`

`msm pats create` prints the raw token once. Store it immediately outside MSM if
you need to use it later.

PAT enforcement status:

- `pack.read` is required for pack list/export API routes and MCP tools.
- `pack.update` is required for pack rename/visibility update API routes and
  MCP tools.
- `pack.delete` is required for pack delete API routes and MCP tools.
- `import.run` is required for pack import API routes and MCP tools.
- user-scoped list/import/update/delete operations reject PATs belonging to
  another user.
- PAT lifecycle endpoints are still bootstrap/admin placeholders until the
  login and admin model is implemented.
- asset privacy enforcement is not wired yet.

Local auth bootstrap endpoints:

- `POST /api/v1/auth/local/register`
- `POST /api/v1/auth/local/login`

Register creates a local user and Argon2 password credential. Login verifies
the password and returns a raw PAT once.

The Web UI can call these endpoints when `VITE_MSM_API_BASE_URL` is configured.
Successful Web login stores the returned PAT in browser localStorage.

Local register can also bootstrap a tenant admin:

```json
{
  "id": "user_1",
  "email": "leko@example.com",
  "displayName": "Leko",
  "password": "password",
  "tenantId": "tenant_1",
  "tenantName": "Tenant",
  "tenantRole": "admin"
}
```
