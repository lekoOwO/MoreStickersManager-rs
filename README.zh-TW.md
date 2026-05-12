# MoreStickersManager-rs

[English](README.md)

MoreStickersManager-rs（簡稱 MSM）是一套 self-hosted 貼圖包管理器，支援
MoreStickers 相容格式與 Telegram 等匯出目標。它是 Equicord 的
moreStickers plugin 配套工具 MoreStickersConverter 的 Rust 重寫與擴充。

MSM 維持 MoreStickers `.stickerpack` 匯出格式相容，同時加入 Web UI、HTTP
API、CLI、MCP endpoint、Provider 匯入工作、Telegram 發佈、多租戶權限控管，
以及使用者資料可攜式遷移。

## 功能亮點

- **MoreStickers 相容性**：匯入與匯出既有 `.stickerpack` JSON 結構。
- **Provider 匯入**：已支援 Telegram 與 LINE；Signal、WhatsApp、Kakao、Band、
  OGQ、Viber 目前登錄為未來 Provider 家族。
- **Telegram 匯出**：使用 ffmpeg/ffprobe 準備媒體，規劃貼圖包建立/更新流程，
  透過 teloxide 邊界呼叫 Telegram Bot API，保存發佈紀錄並支援受保護的
  reconciliation。
- **Web UI**：Vue UI，具備桌面/行動版版面、亮暗模式、台灣繁中與英文、貼圖包
  管理、匯出、訂閱連結、租戶管理與遷移流程。
- **API/OpenAPI**：axum API，`/openapi.json` 提供 utoipa OpenAPI schema。
- **CLI 與 MCP**：提供命令列與 JSON-RPC MCP 自動化介面。
- **多租戶安全性**：租戶成員、admin/user 角色、細緻權限、PAT、本地帳號、
  OIDC/SSO、私有資源存取控制。
- **資料可攜性**：可匯出/匯入使用者資料，支援 MSM instance 間遷移。
- **資料庫後端**：支援 SQLite 與 PostgreSQL migrations/repositories。
- **單一二進位**：建置時可把 Web UI dist embed 到 Rust service binary。

## 專案結構

```text
apps/web/                  Vue + Tailwind CSS v4 + shadcn-vue Web UI
crates/msm-app/            all-in-one HTTP service 與 worker orchestration
crates/msm-api/            API routes 與 OpenAPI schema
crates/msm-cli/            CLI client
crates/msm-domain/         貼圖包 domain model 與相容性 helpers
crates/msm-exporters/      匯出目標 registry 與 Telegram planning
crates/msm-mcp/            MCP JSON-RPC endpoint 與 tools
crates/msm-media/          媒體 probing/conversion planning
crates/msm-providers/      Telegram/LINE provider normalization 與 fetch plans
crates/msm-storage/        SQLite/PostgreSQL storage layer
crates/msm-telegram/       teloxide-backed Telegram Bot API boundary
docs/                      使用者、開發、狀態與 release-readiness 文件
examples/docker/           Docker Compose 部署範例
```

## 用 Docker Compose 快速啟動

最簡單的部署方式是使用 PostgreSQL 的 Compose 範例：

```bash
cp examples/docker/.env.example examples/docker/.env
# 編輯 examples/docker/.env
docker compose --env-file examples/docker/.env -f examples/docker/docker-compose.yml up -d --build
curl -fsS http://localhost:3000/readyz
```

接著開啟 `http://localhost:3000`，或你在 `MSM_EXTERNAL_URL` 設定的網址。

若要串接 Authentik SSO，請建立 Authentik OAuth2/OpenID provider，並設定
redirect URI：

```text
${MSM_EXTERNAL_URL}/auth/oidc/callback
```

接著把 issuer URL、client ID、client secret 填進 `examples/docker/.env`，
bootstrap 第一個 tenant admin，並在 MSM 註冊 OIDC provider。完整流程請看
[`examples/docker/README.md`](examples/docker/README.md)。

## 本機開發

需求：

- Rust stable toolchain
- Bun 或 Node.js/npm
- ffmpeg 與 ffprobe，用於媒體轉換流程
- 可選：PostgreSQL，用於後端 parity 測試

安裝 Web dependencies 並執行檢查：

```bash
bun install --frozen-lockfile
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run web:typecheck
npm run web:test
npm run web:build
```

本機啟動 all-in-one service：

```bash
npm run web:build
cargo run -p msm-app
```

預設 service 監聽 `127.0.0.1:3000`，使用 `sqlite:data/msm.sqlite3`，本機資源目錄為
`data/assets`，並提供：

- `GET /healthz`
- `GET /readyz`
- `GET /openapi.json`
- `POST /mcp`

## 設定

常用環境變數：

| 變數 | 預設值 | 說明 |
| --- | --- | --- |
| `MSM_BIND_ADDR` | `127.0.0.1:3000` | Service bind address。 |
| `MSM_DATABASE_URL` | `sqlite:data/msm.sqlite3` | `sqlite:<path>` 或 PostgreSQL URL。 |
| `MSM_ASSET_DIR` | `data/assets` | 本機來源資源目錄。 |
| `MSM_PREPARED_MEDIA_DIR` | `data/prepared-media` | 轉換後媒體 cache/output 目錄。 |
| `MSM_WEB_DIST_DIR` | `apps/web/dist` | 可選的 runtime Web dist override。 |
| `MSM_PUBLIC_ASSET_URL` | 未設定 | system-wide CDN/public asset URL fallback。 |
| `MSM_PUBLIC_ASSET_BASE_URL` | 由 bind addr 推導 | Provider import worker 使用的 public base。 |
| `MSM_REQUEST_BODY_LIMIT_BYTES` | `10485760` | API request body 上限。 |
| `MSM_IMPORT_RATE_LIMIT_REQUESTS` | `60` | 每個 identity 的匯入類 request 限制。 |
| `MSM_IMPORT_RATE_LIMIT_WINDOW_SECS` | `60` | rate-limit window 秒數。 |
| `MSM_FFMPEG_PATH` | `ffmpeg` | ffmpeg 執行檔路徑。 |
| `MSM_FFPROBE_PATH` | `ffprobe` | ffprobe 執行檔路徑。 |
| `MSM_EXPORT_WORKER_ENABLED` | `false` | 是否啟用 export worker polling。 |
| `MSM_PROVIDER_IMPORT_WORKER_ENABLED` | `false` | 是否啟用 provider import worker polling。 |
| `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON` | 未設定 | 可選的 startup export-target bootstrap JSON。 |

## 驗證與 SSO

MSM 支援本地帳號註冊/登入與 tenant-scoped OIDC provider。成功的本地或 OIDC
登入會回傳 PAT，並設定 HttpOnly `msm_session` cookie，供 Web-session protected
reads 使用。

OIDC provider 管理可透過 Web Tenant admin、API、CLI、MCP 完成。使用者 SSO 文件：
[`docs/user/oidc-sso.md`](docs/user/oidc-sso.md)。

## CLI

CLI binary 名稱為 `msm`：

```bash
cargo run -p msm-cli -- --help
cargo run -p msm-cli -- --base-url http://127.0.0.1:3000 --pat "$MSM_PAT" packs list --user-id user_1
```

## MCP

MCP endpoint 是 `/mcp` 上的 stateless JSON-RPC over HTTP POST。公開 metadata
methods 包含 `initialize`、`ping`、`tools/list`。受保護的 tool calls 需要 HTTP
`Authorization: Bearer msm_pat_...` header。

Transport contract 請看
[`docs/dev/mcp-transport-contract.md`](docs/dev/mcp-transport-contract.md)。

## 文件

- [`docs/user/README.md`](docs/user/README.md)：使用者指南與 API/CLI 範例
- [`docs/user/oidc-sso.md`](docs/user/oidc-sso.md)：OIDC/SSO 指南
- [`docs/user/backup-restore-runbook.md`](docs/user/backup-restore-runbook.md)：備份與還原
- [`docs/dev/architecture.md`](docs/dev/architecture.md)：架構筆記
- [`docs/dev/compatibility.md`](docs/dev/compatibility.md)：MoreStickers 相容性
- [`docs/status/completion-audit.md`](docs/status/completion-audit.md)：release-readiness audit
- [`docs/PRD.md`](docs/PRD.md)：產品需求與完成狀態

## 目前狀態

目前 PRD contract 已完成，release-readiness verification 紀錄於
[`docs/status/completion-audit.md`](docs/status/completion-audit.md)。未來新增產品範圍前，
建議先建立新的 PRD revision。

## License

此 workspace 在 `Cargo.toml` 宣告 `MIT OR Apache-2.0`。
