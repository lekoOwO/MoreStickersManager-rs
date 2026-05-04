# MSM P18 Local Auth Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add local password registration and login that issues PATs.

**Architecture:** Extend `msm-storage` with a `local_user_credentials` table and Argon2 password hashing. Add `msm-api` auth DTOs/routes that call storage and return PAT response DTOs.

**Tech Stack:** Rust, sqlx migrations, argon2, Axum, utoipa.

---

### Task 1: Planning Checkpoint

**Files:**
- Create: `docs/superpowers/specs/2026-05-04-msm-p18-local-auth-bootstrap-design.md`
- Create: `docs/superpowers/plans/2026-05-04-msm-p18-local-auth-bootstrap.md`

- [ ] **Step 1: Commit planning docs**

```powershell
git add docs/superpowers/specs/2026-05-04-msm-p18-local-auth-bootstrap-design.md docs/superpowers/plans/2026-05-04-msm-p18-local-auth-bootstrap.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: add P18 local auth bootstrap plan"
```

### Task 2: Storage Local Credential Tests And Implementation

**Files:**
- Modify: `crates/msm-storage/Cargo.toml`
- Create: `crates/msm-storage/migrations/0002_local_credentials.sql`
- Modify: `crates/msm-storage/src/models.rs`
- Modify: `crates/msm-storage/src/repositories.rs`

- [ ] **Step 1: Add failing tests**

Add tests proving registration stores an Argon2 hash, correct passwords verify,
wrong passwords fail, and duplicate credentials fail.

- [ ] **Step 2: Implement storage**

Add `create_local_user_with_password` and `verify_local_user_password`.

- [ ] **Step 3: Verify storage**

```powershell
cargo test -p msm-storage local_credentials
cargo clippy -p msm-storage --all-targets --locked -- -D warnings
```

### Task 3: API Local Auth Tests And Implementation

**Files:**
- Modify: `crates/msm-api/src/dto.rs`
- Create: `crates/msm-api/src/routes/auth.rs`
- Modify: `crates/msm-api/src/routes/mod.rs`
- Modify: `crates/msm-api/src/openapi.rs`
- Modify: `crates/msm-api/src/lib.rs`

- [ ] **Step 1: Add failing tests**

Add API tests proving local register returns `201`, local login returns a raw
PAT, wrong passwords return `401`, and OpenAPI includes local auth paths.

- [ ] **Step 2: Implement API routes**

Register creates a local user and password credential. Login verifies the
password, parses requested scope keys, creates a PAT, and returns the PAT create
response DTO.

- [ ] **Step 3: Verify API**

```powershell
cargo test -p msm-api local_auth
cargo clippy -p msm-api --all-targets --locked -- -D warnings
```

- [ ] **Step 4: Commit implementation**

```powershell
git add crates/msm-storage crates/msm-api Cargo.lock
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add local auth bootstrap"
```

### Task 4: Documentation And Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document local auth**

Document register/login endpoints and the PAT response behavior.

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
git add README.md docs/user/README.md docs/agents/testing.md docs/status/current.md docs/status/checkpoints.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update local auth bootstrap status"
```
