# Product Data API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first API slice for user-managed folders, tags, subscription groups, and pack access metadata.

**Architecture:** Reuse the existing storage schema in `0001_initial.sql` and existing authorization primitives in `msm-domain`. Keep this slice API-first: storage repository methods, API DTO/routes/OpenAPI, then CLI/MCP/Web can follow in separate slices.

**Tech Stack:** Rust, Axum, SQLx SQLite repository, utoipa OpenAPI, PAT scope enforcement, TDD with package tests.

---

## File Map

- Modify `crates/msm-storage/src/models.rs` for folder/tag/subscription record structs.
- Modify `crates/msm-storage/src/repositories.rs` for list/create/update/delete/link methods.
- Modify `crates/msm-api/src/dto.rs` for request/response DTOs.
- Create or modify `crates/msm-api/src/routes/metadata.rs` for folder/tag/subscription routes.
- Modify `crates/msm-api/src/lib.rs` to mount routes and add tests.
- Modify `crates/msm-api/src/openapi.rs` to register paths and schemas.
- Update `docs/status/current.md`, `docs/status/implementation-matrix.md`, `docs/status/checkpoints.md`, and `docs/status/roadmap.md`.

## Task 1: Storage Listing And CRUD Primitives

**Files:**
- Modify: `crates/msm-storage/src/models.rs`
- Modify: `crates/msm-storage/src/repositories.rs`

- [x] **Step 1: Write failing storage tests**

Add tests proving:

```rust
#[tokio::test]
async fn folders_tags_and_subscriptions_can_be_managed() {
    let repo = test_repo().await;
    repo.create_tenant("tenant_1", "Tenant").await.unwrap();
    repo.create_user("user_1", "leko@example.com", "Leko").await.unwrap();

    repo.create_folder("folder_1", "tenant_1", "user_1", "Favorites").await.unwrap();
    repo.rename_folder("folder_1", "Pinned").await.unwrap();
    assert_eq!(repo.list_folders("tenant_1", "user_1").await.unwrap()[0].name, "Pinned");

    repo.create_tag("tag_1", "tenant_1", "cute", "#1677ff").await.unwrap();
    assert_eq!(repo.list_tags("tenant_1").await.unwrap()[0].slug, "cute");

    repo.create_subscription_group("sub_1", "tenant_1", "user_1", "Weekly").await.unwrap();
    assert_eq!(repo.list_subscription_groups("tenant_1", "user_1").await.unwrap()[0].title, "Weekly");
}
```

- [x] **Step 2: Run the failing tests**

Run:

```powershell
cargo test -p msm-storage folders_tags_and_subscriptions_can_be_managed --locked
```

Result: failed because `NewTag` plus folder/tag/subscription repository methods
were missing and `create_subscription_group` returned `()`.

- [x] **Step 3: Implement minimal storage methods**

Add record structs with exact DB fields, then add methods using the existing SQLx style. Keep method names simple:

```rust
pub async fn list_folders(&self, tenant_id: &str, owner_user_id: &str) -> StorageResult<Vec<FolderRecord>>;
pub async fn create_folder(&self, id: &str, tenant_id: &str, owner_user_id: &str, name: &str) -> StorageResult<()>;
pub async fn rename_folder(&self, id: &str, name: &str) -> StorageResult<()>;
pub async fn delete_folder(&self, id: &str) -> StorageResult<()>;
```

Result: implemented `FolderRecord`, `TagRecord`, `NewTag`, folder CRUD,
tag create/list/delete, subscription group list/rename/delete, and changed
`create_subscription_group` to return `SubscriptionGroupRecord`.

- [x] **Step 4: Verify storage**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p msm-storage folders_tags_and_subscriptions_can_be_managed --locked
cargo clippy -p msm-storage --all-targets --locked -- -D warnings
```

Result: passed with `cargo fmt --all -- --check`,
`cargo test -p msm-storage --locked`,
`cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and
`git diff --check`.

## Task 2: API Routes And OpenAPI

**Files:**
- Create or modify: `crates/msm-api/src/routes/metadata.rs`
- Modify: `crates/msm-api/src/dto.rs`
- Modify: `crates/msm-api/src/lib.rs`
- Modify: `crates/msm-api/src/openapi.rs`

- [x] **Step 1: Write failing API tests**

Add tests for:

```rust
#[tokio::test]
async fn metadata_routes_manage_folders_tags_and_subscriptions() {
    let state = seeded_state().await;
    let token = create_pat(&state, "manage", "user_1", [
        Permission::PackUpdate,
        Permission::SubscriptionCreate,
        Permission::SubscriptionRead,
    ]).await;

    // POST /api/v1/folders creates a folder.
    // GET /api/v1/folders?tenantId=tenant_1&ownerUserId=user_1 lists it.
    // POST /api/v1/tags creates a tag.
    // GET /api/v1/tags?tenantId=tenant_1 lists it.
    // POST /api/v1/subscription-groups creates a group.
    // GET /api/v1/subscription-groups?tenantId=tenant_1&ownerUserId=user_1 lists it.
}
```

- [x] **Step 2: Run the failing API tests**

Run:

```powershell
cargo test -p msm-api metadata_routes_manage_folders_tags_and_subscriptions --locked
```

Result: failed with `404` because routes were not mounted.

- [x] **Step 3: Implement API route module**

Use existing PAT helpers:

- folder and tag management require `pack.update` for this first slice;
- subscription group create requires `subscription.create`;
- subscription group list requires `subscription.read`;
- all routes must verify tenant/user ownership through query/body fields.

- [x] **Step 4: Register OpenAPI**

Register paths and DTO schemas in `crates/msm-api/src/openapi.rs`. Add a test asserting `/api/v1/folders`, `/api/v1/tags`, and `/api/v1/subscription-groups` exist in `/openapi.json`.

- [x] **Step 5: Verify API**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p msm-api --locked
cargo clippy -p msm-api --all-targets --locked -- -D warnings
```

Result: passed with `cargo fmt --all -- --check`,
`cargo test -p msm-storage -p msm-api --locked`,
`cargo clippy -p msm-storage -p msm-api --all-targets --locked -- -D warnings`,
and `git diff --check`.

## Task 3: Status And Handoff Documentation

**Files:**
- Modify: `docs/status/current.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/roadmap.md`
- Modify: `docs/user/README.md`

- [x] **Step 1: Update user-facing docs**

Documented folder/tag/subscription APIs plus CLI/MCP/Web create/list controls.
Updated later after membership API routes landed, noting that CLI/MCP/Web
membership controls remain follow-up slices.

- [x] **Step 2: Update status docs**

Moved the active focus from P33 Telegram reconciliation to product-data
management, then from base metadata routes to membership controls after the API
membership slice landed.

- [ ] **Step 3: Run final verification**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p msm-storage -p msm-api --locked
cargo clippy -p msm-storage -p msm-api --all-targets --locked -- -D warnings
git diff --check
```

Expected: all pass.

Result: passed during the base API route slice and updated again during
membership API documentation.

## Task 6: Web Product Metadata Management

**Files:**
- Modify: `apps/web/src/lib/api-client.ts`
- Modify: `apps/web/src/lib/api-client.test.ts`
- Add: `apps/web/src/components/ProductMetadataPanel.vue`
- Add: `apps/web/src/components/__tests__/product-metadata-ui.test.ts`
- Modify: `apps/web/src/app/AppShell.vue`
- Modify: `apps/web/src/app/PackDashboard.vue`
- Modify: `apps/web/src/lib/i18n.ts`
- Modify: `apps/web/src/lib/workspace.ts`
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: status and agent handoff docs.

- [x] **Step 1: Add failing Web client and component tests**

Added RED tests for metadata API client URL/request behavior and a
`ProductMetadataPanel` workflow.

```powershell
npm run web:test -- api-client
npm run web:test -- product-metadata-ui
```

Result: failed because the metadata API client helpers and Web component did
not exist.

- [x] **Step 2: Implement metadata client and Organize workspace**

Added `createProductMetadataClient`, folder/tag/subscription group DTOs, list
URL helpers, an Organize workspace section, navigation entry, i18n labels, and
subscription PAT scope options.

- [x] **Step 3: Verify Web surface**

```powershell
npm run web:test -- api-client
npm run web:test -- product-metadata-ui
npm run web:typecheck
npm run web:test
npm run web:build
```

Result: all passed.

## Task 7: Storage Pack Membership Links

**Files:**
- Modify: `crates/msm-storage/src/models.rs`
- Modify: `crates/msm-storage/src/repositories.rs`
- Modify: `crates/msm-storage/tests/product_data_repository_tests.rs`
- Modify: status and agent handoff docs.

- [x] **Step 1: Add failing storage test**

Added RED coverage for folder-pack, pack-tag, and subscription-group pack link
add/list/remove behavior.

```powershell
cargo test -p msm-storage pack_memberships_can_be_managed --locked
```

Result: failed because the repository did not expose folder/tag membership
methods and subscription group pack insertion returned no link record.

- [x] **Step 2: Implement membership primitives**

Added link records plus repository methods for add/list/remove operations. The
folder and subscription group pack insert paths are duplicate-safe and update
sort order on conflict.

- [x] **Step 3: Verify storage slice**

```powershell
cargo test -p msm-storage pack_memberships_can_be_managed --locked
cargo fmt --all -- --check
cargo test -p msm-storage --test product_data_repository_tests --locked
cargo clippy -p msm-storage --all-targets --locked -- -D warnings
git diff --check
```

Expected: all pass.

Result: passed with `cargo fmt --all -- --check`,
`cargo test -p msm-storage --test product_data_repository_tests --locked`,
`cargo clippy -p msm-storage --all-targets --locked -- -D warnings`, and
`git diff --check`.

## Task 8: API Pack Membership Links

**Files:**
- Modify: `crates/msm-api/src/dto.rs`
- Modify: `crates/msm-api/src/routes/metadata.rs`
- Modify: `crates/msm-api/src/lib.rs`
- Modify: `crates/msm-api/src/openapi.rs`
- Modify: `crates/msm-storage/src/repositories.rs`
- Modify: `docs/user/README.md`
- Modify: status and agent handoff docs.

- [x] **Step 1: Add failing API tests**

Added RED coverage for folder-pack, pack-tag, and subscription-group pack
membership add/list/remove behavior through HTTP routes, plus OpenAPI path
registration.

```powershell
cargo test -p msm-api metadata_routes_manage_pack_memberships --locked
cargo test -p msm-api openapi_endpoint_contains_health_path --locked
```

Result: failed with route `404` and missing `/openapi.json` paths.

- [x] **Step 2: Implement membership routes**

Added:

- `GET /api/v1/folders/{folder_id}/packs`
- `PUT /api/v1/folders/{folder_id}/packs/{pack_id}`
- `DELETE /api/v1/folders/{folder_id}/packs/{pack_id}`
- `GET /api/v1/packs/{pack_id}/tags`
- `PUT /api/v1/packs/{pack_id}/tags/{tag_id}`
- `DELETE /api/v1/packs/{pack_id}/tags/{tag_id}`
- `GET /api/v1/subscription-groups/{subscription_group_id}/packs`
- `PUT /api/v1/subscription-groups/{subscription_group_id}/packs/{pack_id}`
- `DELETE /api/v1/subscription-groups/{subscription_group_id}/packs/{pack_id}`

Routes require PAT scopes, pack ownership, folder/subscription-group ownership,
and same-tenant resources before mutating links.

- [x] **Step 3: Register OpenAPI**

Registered membership DTO schemas and routes in `ApiDoc`.

- [x] **Step 4: Update handoff docs**

Updated user docs, roadmap, implementation matrix, current status, checkpoints,
project map, and testing handoff.

- [x] **Step 5: Run final verification**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p msm-api --locked
cargo clippy -p msm-api -p msm-storage --all-targets --locked -- -D warnings
git diff --check
```

Expected: all pass.

Result: passed with `cargo fmt --all -- --check`,
`cargo test -p msm-api --locked`,
`cargo test -p msm-storage --test product_data_repository_tests --locked`,
`cargo clippy -p msm-api -p msm-storage --all-targets --locked -- -D warnings`,
and `git diff --check`.

## Task 5: MCP Product Metadata Tools

**Files:**
- Modify: `crates/msm-mcp/src/tools.rs`
- Modify: `crates/msm-mcp/src/handler.rs`
- Modify: `crates/msm-mcp/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/roadmap.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`

- [x] **Step 1: Add failing MCP tests**

Added tests for `tools/list`, metadata tool execution, and PAT scope
enforcement.

```powershell
cargo test -p msm-mcp tools_call_manages_product_metadata --locked
```

Result: failed because MCP tool registry and handler dispatch did not yet know
the product metadata tools.

- [x] **Step 2: Implement MCP tools**

Added `msm.list_folders`, `msm.create_folder`, `msm.list_tags`,
`msm.create_tag`, `msm.list_subscription_groups`, and
`msm.create_subscription_group` tool definitions and handler implementations.

- [x] **Step 3: Verify focused tests**

```powershell
cargo test -p msm-mcp tools_call_manages_folders --locked
cargo test -p msm-mcp tools_call_manages_tags --locked
cargo test -p msm-mcp tools_call_manages_subscription_groups --locked
cargo test -p msm-mcp tools_list_returns_pack_and_export_tools --locked
cargo test -p msm-mcp tool_registry_contains_pack_tools --locked
cargo test -p msm-mcp pat_enforcement_metadata_tools_require_expected_scopes --locked
```

Result: all passed.

- [x] **Step 4: Update handoff and user docs**

Documented the MCP tools and updated status/agent handoff docs.

- [x] **Step 5: Run full MCP verification**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p msm-mcp --locked
cargo clippy -p msm-mcp --all-targets --locked -- -D warnings
git diff --check
```

Expected: all pass.

## Task 4: CLI Product Metadata Commands

**Files:**
- Modify: `crates/msm-cli/src/client.rs`
- Modify: `crates/msm-cli/src/command.rs`
- Modify: `crates/msm-cli/src/output.rs`
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/roadmap.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`

- [x] **Step 1: Add failing CLI parser/execution tests**

Added RED tests for `msm metadata folders`, `msm metadata tags`, and
`msm metadata subscription-groups`.

```powershell
cargo test -p msm-cli parses_metadata_commands --locked
```

Result: failed because metadata CLI payloads, response models, command enums,
and client methods did not exist yet.

- [x] **Step 2: Implement CLI metadata client and commands**

Added folder/tag/subscription-group payload and response DTOs, reqwest client
methods, Clap command groups, command dispatch, fake-client coverage, and
human/JSON output formatting.

- [x] **Step 3: Verify focused tests**

```powershell
cargo test -p msm-cli parses_metadata_commands --locked
cargo test -p msm-cli executes_metadata_commands --locked
```

Result: both passed.

- [x] **Step 4: Update handoff and user docs**

Documented the new metadata CLI commands and updated status/agent handoff docs.

- [x] **Step 5: Run full CLI verification**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p msm-cli --locked
cargo clippy -p msm-cli --all-targets --locked -- -D warnings
git diff --check
```

Expected: all pass.
