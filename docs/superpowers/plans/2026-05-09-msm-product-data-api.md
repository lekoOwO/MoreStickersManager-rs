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

- [ ] **Step 1: Write failing API tests**

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

- [ ] **Step 2: Run the failing API tests**

Run:

```powershell
cargo test -p msm-api metadata_routes_manage_folders_tags_and_subscriptions --locked
```

Expected: fail because routes are not mounted.

- [ ] **Step 3: Implement API route module**

Use existing PAT helpers:

- folder and tag management require `pack.update` for this first slice;
- subscription group create requires `subscription.create`;
- subscription group list requires `subscription.read`;
- all routes must verify tenant/user ownership through query/body fields.

- [ ] **Step 4: Register OpenAPI**

Register paths and DTO schemas in `crates/msm-api/src/openapi.rs`. Add a test asserting `/api/v1/folders`, `/api/v1/tags`, and `/api/v1/subscription-groups` exist in `/openapi.json`.

- [ ] **Step 5: Verify API**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p msm-api --locked
cargo clippy -p msm-api --all-targets --locked -- -D warnings
```

Expected: all pass.

## Task 3: Status And Handoff Documentation

**Files:**
- Modify: `docs/status/current.md`
- Modify: `docs/status/implementation-matrix.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/status/roadmap.md`
- Modify: `docs/user/README.md`

- [ ] **Step 1: Update user-facing docs**

Document that folder/tag/subscription APIs exist, while CLI/MCP/Web controls remain future slices.

- [ ] **Step 2: Update status docs**

Move the current focus from P33 Telegram reconciliation to the new product-data API slice after the route work lands.

- [ ] **Step 3: Run final verification**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p msm-storage -p msm-api --locked
cargo clippy -p msm-storage -p msm-api --all-targets --locked -- -D warnings
git diff --check
```

Expected: all pass.
