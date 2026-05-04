# MSM P20 Admin Bootstrap Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let local registration optionally create a tenant and add the new user as admin.

**Architecture:** Extend `RegisterLocalUserRequest` and `auth::register_local_user`. Keep storage unchanged by composing existing repository methods.

**Tech Stack:** Rust, Axum, sqlx-backed repository methods, utoipa.

---

### Task 1: Planning Checkpoint

**Files:**
- Create: `docs/superpowers/specs/2026-05-04-msm-p20-admin-bootstrap-policy-design.md`
- Create: `docs/superpowers/plans/2026-05-04-msm-p20-admin-bootstrap-policy.md`

- [ ] **Step 1: Commit planning docs**

```powershell
git add docs/superpowers/specs/2026-05-04-msm-p20-admin-bootstrap-policy-design.md docs/superpowers/plans/2026-05-04-msm-p20-admin-bootstrap-policy.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: add P20 admin bootstrap plan"
```

### Task 2: API Test And Implementation

**Files:**
- Modify: `crates/msm-api/src/dto.rs`
- Modify: `crates/msm-api/src/routes/auth.rs`
- Modify: `crates/msm-api/src/lib.rs`

- [ ] **Step 1: Add failing test**

Add a test that local register with `tenantId` creates a tenant admin who can
import a pack with a login-issued `import.run` PAT.

- [ ] **Step 2: Implement optional tenant bootstrap**

If `tenantId` is present, call `create_tenant` and `add_tenant_member` after
creating the local user.

- [ ] **Step 3: Verify**

```powershell
cargo test -p msm-api admin_bootstrap
cargo clippy -p msm-api --all-targets --locked -- -D warnings
```

- [ ] **Step 4: Commit implementation**

```powershell
git add crates/msm-api
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add admin bootstrap registration"
```

### Task 3: Documentation And Status

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document tenant bootstrap fields**

- [ ] **Step 2: Run full verification**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run web:typecheck
npm run web:test
npm run web:build
```

- [ ] **Step 3: Commit documentation**

```powershell
git add README.md docs/user/README.md docs/status/current.md docs/status/checkpoints.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update admin bootstrap status"
```
