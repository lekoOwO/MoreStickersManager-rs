# MSM P13 PAT Management API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add HTTP API and OpenAPI coverage for PAT create/list/revoke.

**Architecture:** Keep PAT lifecycle in `msm-storage`; `msm-api` converts request scope keys to domain permissions and maps records into hash-free response DTOs.

**Tech Stack:** Rust, Axum, utoipa, serde.

---

### Task 1: PAT DTOs And Routes

**Files:**
- Modify: `crates/msm-api/src/dto.rs`
- Create: `crates/msm-api/src/routes/pats.rs`
- Modify: `crates/msm-api/src/routes/mod.rs`
- Modify: `crates/msm-api/src/lib.rs`
- Modify: `crates/msm-api/src/openapi.rs`

- [ ] **Step 1: Add DTOs**

Add create/list response DTOs that never expose `token_hash`.

- [ ] **Step 2: Add routes**

Implement create/list/revoke by calling storage repository PAT lifecycle methods.

- [ ] **Step 3: Wire routes and OpenAPI**

Mount routes and add them to utoipa paths/schemas.

- [ ] **Step 4: Add tests**

Test create returns token once, list hides token hash, revoke makes storage
verification fail, and OpenAPI contains `/api/v1/pats`.

- [ ] **Step 5: Verify**

```powershell
cargo test -p msm-api pats
cargo clippy -p msm-api --all-targets -- -D warnings
```

- [ ] **Step 6: Commit**

```powershell
git add crates/msm-api
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add PAT management API"
```

### Task 2: Documentation And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document PAT API endpoints**

Document create/list/revoke endpoints and non-enforcement status.

- [ ] **Step 2: Run full verification**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run web:typecheck
npm run web:test
npm run web:build
```

- [ ] **Step 3: Commit documentation**

```powershell
git add README.md docs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update PAT management API status"
```
