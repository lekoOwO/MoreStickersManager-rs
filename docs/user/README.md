# MSM User Documentation

MSM currently has foundation, storage, authorization, API, CLI, provider
normalization, local auth bootstrap, PAT enforcement, MCP pack operations, and
Web UI pack management slices.

Current usable contract: `.stickerpack` compatibility is documented in `../dev/compatibility.md`.

Provider normalization status is documented in `../dev/providers.md`.

Implemented-versus-planned feature status is documented in
`../status/implementation-matrix.md`.

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
cargo run -p msm-cli -- exports kinds
cargo run -p msm-cli -- exports targets list --tenant-id tenant_1
cargo run -p msm-cli -- exports targets create --id target_telegram --tenant-id tenant_1 --kind telegram --name Telegram --config-json '{"botUsername":"msm_bot","botToken":"123:token"}'
cargo run -p msm-cli -- exports jobs create --id job_1 --tenant-id tenant_1 --source-pack-id pack_1 --target-id target_telegram --options-json '{"setNameSlug":"sample"}'
cargo run -p msm-cli -- exports jobs get --job-id job_1
cargo run -p msm-cli -- exports jobs events --job-id job_1
cargo run -p msm-cli -- exports publications list --pack-id pack_1
cargo run -p msm-cli -- exports publications get --publication-id telegram_pub_1
```

Protected API-backed CLI commands accept a PAT through `--pat` or `MSM_PAT`:

```powershell
cargo run -p msm-cli -- --pat msm_pat_cli1_secret packs list --user-id user_1
$env:MSM_PAT="msm_pat_cli1_secret"
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
```

Current Web UI examples:

```powershell
npm run dev:env -- init development
npm run dev:env -- use development
npm run dev:start
npm run dev:status
npm run dev:stop
npm run web:dev
npm run web:build
```

`npm run dev:start` runs both the Rust API service and the Vite Web server.
`npm run dev:start api` or `npm run dev:start web` starts only one side. The
manager loads `.env.<name>` plus optional `.env.local`, writes stdout/stderr
logs under `tmp/dev-manager/`, and supports `development` and `testing` examples
out of the box through `.env.development.example` and `.env.testing.example`.
`pnpm run` works as well; the repository includes `pnpm-workspace.yaml`, and the
manager launches the local Vite binary directly instead of depending on
package-manager argument forwarding. On Windows, services are launched through
hidden wrappers so Rust/Vite child processes do not open visible console
windows.

The Web UI can run against mock data or the current API. It demonstrates the app
shell, responsive layout, theme toggle, language toggle, PAT management, local
login/register, pack list, pack rename, visibility edit, delete, and pasted
`.stickerpack` import. It also exposes export target settings, Telegram target
token validation, pack export job creation, job refresh, and ordered job event
display when the export API is available. The export wizard also shows persisted
Telegram publication history for the selected pack, including completed sticker
set links from prior non-dry-run jobs.

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

The dashboard can rename packs, change public/private visibility, and delete
packs when the stored PAT has `pack.update` and `pack.delete`.
It can also import a pasted MoreStickers `.stickerpack` JSON export when the
stored PAT has `import.run`.

Telegram sticker set export is partially planned in backend code but not
user-facing yet. P24/P25 document an export pipeline that will convert pack
assets into target-specific media, use a Telegram bot to create sticker sets,
and expose the workflow through Web, API, CLI, and MCP surfaces.

P25 has started the backend media, export planning, and export API foundation.
MSM can select Telegram static/video target profiles, build ffmpeg command
arguments for static, video, and thumbnail outputs, normalize Telegram sticker
set names, split create/append batches, enforce Telegram set size limits, and
prepare teloxide `InputSticker` data. Protected API routes can manage export
targets, queue export jobs, and read job status/events. The app worker can run
queued MoreStickers serialization jobs, Telegram dry-run planning jobs, and
Telegram publication jobs when job options explicitly set `"dryRun": false`.
It can write prepared media cache records through its media executor boundary
and then use those prepared files for teloxide sticker uploads. MSM has a
process-backed ffmpeg executor for prepared media conversion. The CLI can manage
export targets, queue/read export jobs, and list/read Telegram publication
history through the API. The Web UI can configure export targets, queue jobs,
show job status/events, surface completed Telegram sticker set URLs, and reopen
persisted Telegram publication records for the selected pack.

Telegram remote synchronization policy is now defined in backend planner code.
MSM can model create-only, append-missing, and mirror reconciliation operations
against known remote sticker set state, including title updates, sticker
replacement, missing sticker additions, and remote-only sticker deletions. The
Telegram boundary can execute ordered title/add/replace/delete mutation
sequences through teloxide. The worker does not map jobs into those update/delete
operations for live publication yet, but dry-run jobs may include
`reconcileMode` plus a supplied `remoteSet` to return operation and mutation
counts. Non-dry-run append-missing reconciliation can execute supplied-state
mutations only when job options include `"dryRun": false`,
`"reconcileMode": "appendMissing"`, `"executeReconciliation": true`, and a
`remoteSet`. Mirror-mode replace/delete additionally requires
`"allowDestructiveReconciliation": true`. Current normal live Telegram
publication remains create/append-oriented and opt-in via `"dryRun": false`.
The Telegram boundary can fetch remote sticker set metadata, but automatic
worker reconciliation from fetched state is still pending until MSM persists a
per-sticker mapping between MSM sticker IDs and Telegram file IDs.
The storage layer now has that mapping table and repository API; worker
population from post-publication `getStickerSet` results is implemented for
successful non-dry-run publication jobs. Reconciliation mutation jobs still need
mapping refresh after mutation execution.

Service startup can bootstrap configured export targets with
`MSM_BOOTSTRAP_EXPORT_TARGETS_JSON`. This is intended for system or tenant
targets such as a Telegram bot target before Web target settings exist.

Telegram export setup today:

- create or choose a Telegram bot and keep the bot token outside source control;
- create a Telegram export target through the Web UI, CLI, MCP, API, or
  `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON`;
- grant the PAT used by Web/CLI/MCP the `export.read`, `export.run`, and
  `export.target.manage` scopes as needed;
- install `ffmpeg` where `msm-app` can run it, or set `MSM_FFMPEG_PATH` and
  `MSM_FFPROBE_PATH`;
- set `MSM_EXPORT_WORKER_ENABLED=true` when you want the service process to poll
  queued export jobs.
- optionally set `MSM_EXPORT_RETRY_BACKOFF_MS` to control how long retryable
  failed jobs wait before the worker can pick them again. The default is 60000.

The current Telegram worker path defaults to dry-run planning and media
preparation. To execute Telegram upload and sticker set creation, queue the job
with options containing `"dryRun": false` and use a Telegram target config that
contains `botToken`, `botUsername`, and `ownerUserId`.

Export target/job API endpoints and CLI commands now exist for queueing export
jobs and reading their status/events. CLI commands, MCP tools, and Web UI
controls can also read persisted Telegram publication history.

Telegram publication history API endpoints are available for completed
non-dry-run publication records:

- `GET /api/v1/telegram-publications?packId=...`
- `GET /api/v1/telegram-publications/{publication_id}`

Both require `export.read` and the PAT user must own the source pack.

Telegram bot integration now uses `teloxide` internally. Worker tests inject a
fake publisher, so local and CI verification do not call Telegram. Real
publication is only attempted by the worker when dry-run is explicitly disabled.
Transient worker failures are requeued until the job's attempt budget is
exhausted; export job reads expose `attemptCount`, `maxAttempts`, and
`nextAttemptAt` for API/CLI/MCP/Web clients.

Export API endpoints currently available:

- `GET /api/v1/export-target-kinds`
- `GET /api/v1/export-targets?tenantId=...`
- `POST /api/v1/export-targets`
- `PATCH /api/v1/export-targets/{target_id}`
- `DELETE /api/v1/export-targets/{target_id}`
- `POST /api/v1/export-jobs`
- `GET /api/v1/export-jobs/{job_id}`
- `GET /api/v1/export-jobs/{job_id}/events`
- `GET /api/v1/telegram-publications?packId=...`
- `GET /api/v1/telegram-publications/{publication_id}`

Required PAT scopes are `export.read`, `export.run`, and
`export.target.manage`.

Current service binary example:

```powershell
npm run dev:start api
npm run dev:status
npm run dev:stop api
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

Export MCP tools:

- `msm.list_export_target_kinds`
- `msm.list_export_targets`
- `msm.create_export_target`
- `msm.create_export_job`
- `msm.get_export_job`
- `msm.list_export_job_events`
- `msm.list_telegram_publications`
- `msm.get_telegram_publication`

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

Export CLI commands:

- `msm exports kinds`
- `msm exports targets list --tenant-id <tenant_id>`
- `msm exports targets create --id <target_id> --tenant-id <tenant_id> --kind <kind> --name <name> --config-json <json>`
- `msm exports jobs create --id <job_id> --tenant-id <tenant_id> --source-pack-id <pack_id> --target-id <target_id> --options-json <json>`
- `msm exports jobs get --job-id <job_id>`
- `msm exports jobs events --job-id <job_id>`
- `msm exports publications list --pack-id <pack_id>`
- `msm exports publications get --publication-id <publication_id>`

`msm pats create` prints the raw token once. Store it immediately outside MSM if
you need to use it later.

PAT enforcement status:

- `pack.read` is required for pack list/export API routes and MCP tools.
- `pack.update` is required for pack rename/visibility update API routes and
  MCP tools.
- `pack.delete` is required for pack delete API routes and MCP tools.
- `import.run` is required for pack import API routes and MCP tools.
- `export.read` is required for export target/job/publication read API routes
  and MCP tools.
- `export.run` is required for export job creation API routes and MCP tools.
- `export.target.manage` is required for export target management API routes and
  MCP tools.
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
