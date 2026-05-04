# MSM P15 API/MCP PAT Enforcement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforce Bearer PAT scopes on pack API routes and MCP tool calls.

**Architecture:** Add a focused API auth helper that verifies `Authorization: Bearer ...` headers against `StorageRepository`. API routes and MCP tools call this helper with required `msm_domain::Permission` values; CLI forwards `--pat` or `MSM_PAT` as Bearer auth.

**Tech Stack:** Rust, Axum extractors, reqwest, clap, async-trait.

---

### Task 1: Planning Checkpoint

**Files:**
- Create: `docs/superpowers/specs/2026-05-04-msm-p15-api-mcp-pat-enforcement-design.md`
- Create: `docs/superpowers/plans/2026-05-04-msm-p15-api-mcp-pat-enforcement.md`

- [ ] **Step 1: Commit planning docs**

```powershell
git add docs/superpowers/specs/2026-05-04-msm-p15-api-mcp-pat-enforcement-design.md docs/superpowers/plans/2026-05-04-msm-p15-api-mcp-pat-enforcement.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: add P15 API MCP PAT enforcement plan"
```

### Task 2: API Enforcement Tests

**Files:**
- Create: `crates/msm-api/src/auth.rs`
- Modify: `crates/msm-api/src/error.rs`
- Modify: `crates/msm-api/src/lib.rs`
- Modify: `crates/msm-api/src/routes/packs.rs`

- [ ] **Step 1: Add failing tests in `crates/msm-api/src/lib.rs`**

Add tests for:

- unauthenticated `GET /api/v1/packs?userId=user_1` returns `401`;
- PAT with `pack.read` can list its own packs;
- PAT with `asset.read` but no `pack.read` returns `403`;
- PAT for `user_1` cannot list `user_2`;
- import requires `import.run` and matching `ownerUserId`;
- export requires `pack.read`.

- [ ] **Step 2: Verify API tests fail**

```powershell
cargo test -p msm-api pat_enforcement
```

Expected: compile or assertion failure because auth helper and handler
extraction do not exist yet.

### Task 3: API Enforcement Implementation

**Files:**
- Create: `crates/msm-api/src/auth.rs`
- Modify: `crates/msm-api/src/error.rs`
- Modify: `crates/msm-api/src/lib.rs`
- Modify: `crates/msm-api/src/routes/packs.rs`

- [ ] **Step 1: Add auth helper**

Create `VerifiedPat` and `require_pat(headers, state, required_permission)`.
The helper validates the Bearer header, verifies storage PATs, checks scopes, and
returns `ApiError::Unauthorized` or `ApiError::Forbidden` where appropriate.

- [ ] **Step 2: Protect pack handlers**

Add `HeaderMap` extraction to list/import/export handlers. Check:

- list: `Permission::PackRead` and PAT user equals query `user_id`;
- import: `Permission::ImportRun` and PAT user equals request `owner_user_id`;
- export: `Permission::PackRead`.

- [ ] **Step 3: Verify API focus**

```powershell
cargo fmt --all -- --check
cargo test -p msm-api pat_enforcement
cargo clippy -p msm-api --all-targets -- -D warnings
```

### Task 4: MCP Enforcement Tests And Implementation

**Files:**
- Modify: `crates/msm-mcp/src/handler.rs`
- Modify: `crates/msm-mcp/src/lib.rs`

- [ ] **Step 1: Add failing MCP tests**

Add tests for:

- `tools/call` without PAT returns an MCP tool error;
- `msm.list_sticker_packs` succeeds with `pack.read`;
- `msm.import_sticker_pack` fails without `import.run`;
- user mismatch returns an MCP tool error.

- [ ] **Step 2: Implement MCP auth context**

Extract headers in `mcp_post`, verify Bearer PAT once per request, and pass an
optional verified PAT into `handle_mcp_message`. Enforce scopes and user match
inside tool calls.

- [ ] **Step 3: Verify MCP focus**

```powershell
cargo fmt --all -- --check
cargo test -p msm-mcp pat_enforcement
cargo clippy -p msm-mcp --all-targets -- -D warnings
```

### Task 5: CLI Bearer Forwarding

**Files:**
- Modify: `crates/msm-cli/src/command.rs`
- Modify: `crates/msm-cli/src/client.rs`

- [ ] **Step 1: Add failing CLI tests**

Test that global `--pat` parses and that reqwest requests can be constructed
with a configured token.

- [ ] **Step 2: Implement token forwarding**

Add global `--pat` and `MSM_PAT` fallback. Store the token in
`ReqwestMsmClient` and attach Bearer auth to API requests when present.

- [ ] **Step 3: Verify CLI focus**

```powershell
cargo fmt --all -- --check
cargo test -p msm-cli pat
cargo clippy -p msm-cli --all-targets -- -D warnings
```

- [ ] **Step 4: Commit implementation**

```powershell
git add crates/msm-api crates/msm-mcp crates/msm-cli
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: enforce PAT scopes on API and MCP"
```

### Task 6: Documentation And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document PAT enforcement**

Document protected routes, CLI `--pat`, `MSM_PAT`, and the current bootstrap
limitations.

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
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update PAT enforcement status"
```
