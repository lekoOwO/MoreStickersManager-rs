# MoreStickersManager-rs

[English](README.md)

**MoreStickersManager-rs（MSM）** 是一套 self-hosted 貼圖庫，適合想集中管理、
整理、分享、訂閱並發佈貼圖包到 MoreStickers/Discord 與 Telegram 的使用者。

你不需要把貼圖包 export 檔案散落在各處。MSM 提供一個由你自己託管的地方：匯入
貼圖包、整理分類、產生訂閱連結、控制私有圖片存取權限，並把選定貼圖包發佈成
Telegram sticker set。

## MSM 能幫你做什麼

### 集中管理貼圖包

- 匯入既有 MoreStickers `.stickerpack` export。
- 從 Telegram、LINE 等已支援 Provider 匯入貼圖包。
- 建立、重新命名、更新、刪除、重新匯出貼圖包。
- 公開貼圖包可以開放存取；私有貼圖包可要求登入、PAT 或訂閱 secret。
- 圖片資源由自己的 MSM instance 託管，也可以設定 CDN URL，例如 Cloudflare。

### 用符合日常使用的方式整理

- 把貼圖包放進資料夾。
- 替貼圖包加 tag。
- 從多個貼圖包建立自訂貼圖包組。
- 在 Web UI 內完成整理，不需要手動改 JSON。

### 分享可自動更新的訂閱

MSM 可以為「單一貼圖包」與「自訂貼圖包組」產生訂閱式 endpoint。

適合用在 moreStickers 這類客戶端定期從 URL 更新最新貼圖包內容：

- **貼圖包訂閱**：每個貼圖包都有自己的預設訂閱式 payload，可直接追蹤該貼圖包。
- **貼圖包組訂閱**：建立一個訂閱群組，加入多個貼圖包，用單一連結分享整個集合。
- **公開訂閱**：知道連結的人可以讀取公開 payload。
- **受保護訂閱**：私有貼圖包/群組 payload 與圖片資源需要符合的訂閱 secret、PAT
  或已登入 Web session。
- **可輪替連結**：訂閱 access token 可以建立、輪替、撤銷，方便處理連結外流或成員
  變動。

### 發佈到 Telegram

MSM 內建受 sticker-bot workflow 啟發的 Telegram 匯出流程：

- 使用 ffmpeg/ffprobe 把圖片/影片準備成 Telegram 貼圖需求格式。
- 透過 bot token 建立 Telegram sticker set。
- 把缺少的貼圖追加到既有 sticker set。
- 用 create-only、append-missing 或受保護 mirror 模式同步 MSM 與 Telegram。
- 保存發佈紀錄與 MSM 貼圖到 Telegram file 的 mapping。
- 可從 Web UI、API、CLI 或 MCP 執行匯出。

### 自用、團隊、社群都能跑

- 多租戶資料模型。
- admin / 一般使用者角色。
- 對貼圖包、圖片資源、匯入、匯出、訂閱、PAT、租戶管理都有細緻權限。
- 本地帳號與 OIDC/SSO 登入。
- Personal Access Token 可供 API、CLI、MCP 使用。
- 可匯出/匯入使用者資料，支援 MSM instance 間遷移。

## Web UI 主要流程

Web UI 是 MSM 的主要管理介面：

- Dashboard 查看貼圖包與系統狀態。
- 貼圖包管理：匯入、重新命名、公開/私有、刪除、匯出。
- Provider 匯入工作區：Telegram / LINE。
- 整理工作區：資料夾、tag、訂閱群組、貼圖包 membership。
- 匯出工作區：MoreStickers 與 Telegram jobs。
- 租戶管理：成員、角色、設定、本地註冊、OIDC providers。
- 遷移工作區：可攜式使用者 export/import。
- 桌面與行動版版面、亮暗模式、英文與台灣繁中。

## 用 Docker Compose 快速啟動

Compose 範例會啟動 MSM 與 PostgreSQL：

```bash
cp examples/docker/.env.example examples/docker/.env
# 編輯 examples/docker/.env
docker compose --env-file examples/docker/.env -f examples/docker/docker-compose.yml up -d --build
curl -fsS http://localhost:3000/readyz
```

接著開啟 `http://localhost:3000`，或你設定的 `_MSM_EXTERNAL_URL`。

第一次用空資料庫啟動時，MSM 會建立預設 tenant/admin，並在
`bootstrap_admin_created` log event 印出 admin 密碼。包含 Authentik SSO
與第一個 admin bootstrap 的完整部署說明在
[`examples/docker/README.md`](examples/docker/README.md)。

## Authentik / OIDC SSO

串接 Authentik：

1. 在 Authentik 建立 MSM 專用 OAuth2/OpenID Provider 與 Application。
2. Allowed redirect URI 設為：

   ```text
   ${_MSM_EXTERNAL_URL}/auth/oidc/callback
   ```

3. 把 issuer URL、client ID、client secret 填入部署 env。
4. 用第一次啟動產生的 bootstrap admin 或其他 tenant admin 登入。
5. 從 Web UI Tenant admin、CLI、API 或 MCP 新增 OIDC provider。

請參考 [`examples/docker/README.md`](examples/docker/README.md) 與
[`docs/user/oidc-sso.md`](docs/user/oidc-sso.md)。

## MoreStickers 相容性

MSM 維持既有 MoreStickers `.stickerpack` export shape，讓貼圖包仍可被了解目前
moreStickers 格式的客戶端使用。同時，MSM 也支援讓客戶端用 URL 更新的 dynamic
pack 與貼圖包組訂閱 payload。

相容性說明在 [`docs/dev/compatibility.md`](docs/dev/compatibility.md)。

## 支援的使用介面

同一套產品功能可從多種介面使用：

| 介面 | 用途 |
| --- | --- |
| Web UI | 日常貼圖包管理、訂閱、匯出、租戶管理。 |
| HTTP API | 腳本、服務、反向代理、自訂客戶端整合。 |
| OpenAPI | `/openapi.json` 提供 machine-readable API schema。 |
| CLI | 終端機操作與自動化。 |
| MCP | 透過 `/mcp` 提供 MCP-capable clients tool access。 |

## 匯入與匯出目標

目前已實作：

- MoreStickers 匯入/匯出。
- Telegram provider 匯入。
- LINE provider 匯入。
- Telegram sticker-set 匯出與受保護 reconciliation。

目前登錄為未來計畫的 Provider 家族：

- Signal
- WhatsApp
- Kakao
- Band
- OGQ
- Viber

## 部署筆記

常用 runtime 設定：

| 變數 | 用途 |
| --- | --- |
| `MSM_BIND_ADDR` | Service bind address。 |
| `MSM_DATABASE_URL` | SQLite 或 PostgreSQL database URL。 |
| `MSM_ASSET_DIR` | 本機貼圖圖片儲存位置。 |
| `MSM_PREPARED_MEDIA_DIR` | Telegram prepared media output/cache。 |
| `MSM_PUBLIC_ASSET_URL` | 可選的 system-wide CDN/public asset URL。 |
| `MSM_EXPORT_WORKER_ENABLED` | 啟用 export job polling。 |
| `MSM_PROVIDER_IMPORT_WORKER_ENABLED` | 啟用 provider import job polling。 |
| `MSM_BOOTSTRAP_EXPORT_TARGETS_JSON` | 可選的 startup export target bootstrap。 |

Docker image 內含 ffmpeg/ffprobe，可直接執行媒體轉換。多人使用建議 PostgreSQL；
小型或單人 instance 可用 SQLite。

備份與還原請看
[`docs/user/backup-restore-runbook.md`](docs/user/backup-restore-runbook.md)。

## 本機開發

需求：

- Rust stable toolchain
- Bun 或 Node.js/npm
- ffmpeg 與 ffprobe
- 可選：PostgreSQL，用於後端 parity 測試

常用檢查：

```bash
bun install --frozen-lockfile
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run web:typecheck
npm run web:test
npm run web:build
```

本機啟動：

```bash
npm run web:build
cargo run -p msm-app
```

預設 endpoints：

- `GET /healthz`
- `GET /readyz`
- `GET /openapi.json`
- `POST /mcp`

## 文件

- [`examples/docker/README.md`](examples/docker/README.md)：Docker Compose 與
  Authentik 設定
- [`docs/user/README.md`](docs/user/README.md)：詳細使用者指南與 API/CLI 範例
- [`docs/user/oidc-sso.md`](docs/user/oidc-sso.md)：SSO 指南
- [`docs/user/backup-restore-runbook.md`](docs/user/backup-restore-runbook.md)：備份與還原
- [`docs/dev/compatibility.md`](docs/dev/compatibility.md)：MoreStickers 格式相容性
- [`docs/dev/mcp-transport-contract.md`](docs/dev/mcp-transport-contract.md)：MCP transport 行為
- [`docs/status/completion-audit.md`](docs/status/completion-audit.md)：release-readiness audit

## 專案狀態

目前 PRD contract 已完成。Release-readiness verification 紀錄於
[`docs/status/completion-audit.md`](docs/status/completion-audit.md)。未來新增產品範圍前，
建議先建立新的 PRD revision。

## License

本專案使用 GNU General Public License v3.0 or later（`GPL-3.0-or-later`）授權。
