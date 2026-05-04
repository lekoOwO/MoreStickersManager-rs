# MSM P12 PAT Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Personal Access Token creation, hashing, verification, listing, and revocation foundations.

**Architecture:** Keep permission key mapping in `msm-domain`, token lifecycle persistence in `msm-storage`, and route enforcement for later API/MCP slices.

**Tech Stack:** Rust, SQLx SQLite, SHA-256, OS random bytes.

---

### Task 1: Permission Scope Keys

**Files:**
- Modify: `crates/msm-domain/src/authz.rs`

- [ ] **Step 1: Add `Permission::as_key`**

Map every permission enum variant to a stable lowercase dot-separated key.

- [ ] **Step 2: Add `Permission::from_key`**

Parse stable keys back into permission enum variants.

- [ ] **Step 3: Add tests**

Roundtrip every permission key and reject unknown keys.

- [ ] **Step 4: Commit**

```powershell
git add crates/msm-domain/src/authz.rs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add permission scope keys"
```

### Task 2: PAT Storage Lifecycle

**Files:**
- Modify: `crates/msm-storage/Cargo.toml`
- Modify: `crates/msm-storage/src/models.rs`
- Modify: `crates/msm-storage/src/repositories.rs`

- [ ] **Step 1: Add dependencies**

Add `getrandom`, `sha2`, and `hex`.

- [ ] **Step 2: Add PAT models**

Add `PersonalAccessTokenRecord` and `CreatedPersonalAccessToken`.

- [ ] **Step 3: Add repository methods**

Implement create/list/verify/revoke methods.

- [ ] **Step 4: Add tests**

Test token creation only returns raw token once, verification succeeds, invalid
secret fails, revocation fails verification, and expired tokens fail
verification.

- [ ] **Step 5: Verify**

```powershell
cargo test -p msm-storage personal_access_tokens
cargo clippy -p msm-storage --all-targets -- -D warnings
```

- [ ] **Step 6: Commit**

```powershell
git add Cargo.lock crates/msm-storage
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add PAT storage lifecycle"
```

### Task 3: Documentation And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/dev/architecture.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document PAT foundation scope**

Document token format, scope keys, and non-enforcement status.

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
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update PAT foundation status"
```
