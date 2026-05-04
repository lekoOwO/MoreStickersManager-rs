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
npm run web:dev
```

When `VITE_MSM_API_BASE_URL` is omitted, the dashboard falls back to mock data.

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
Current tools are `msm.list_sticker_packs`, `msm.export_sticker_pack`, and
`msm.import_sticker_pack`.
