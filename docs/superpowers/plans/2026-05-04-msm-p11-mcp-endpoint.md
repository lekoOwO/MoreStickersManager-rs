# MSM P11 MCP Endpoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first MCP endpoint exposing current MSM pack operations as tools.

**Architecture:** Create `msm-mcp` as an Axum/JSON-RPC crate that reuses `msm-api::ApiState` and storage repositories. Mount it from `msm-app` at `/mcp`.

**Tech Stack:** Rust, Axum, serde, JSON-RPC 2.0, MCP 2025-06-18 tool message shapes.

---

### Task 1: MCP Crate And Protocol Types

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/msm-mcp/Cargo.toml`
- Create: `crates/msm-mcp/src/lib.rs`
- Create: `crates/msm-mcp/src/protocol.rs`
- Create: `crates/msm-mcp/src/tools.rs`

- [ ] **Step 1: Add workspace member**

Add `crates/msm-mcp` to root workspace members.

- [ ] **Step 2: Add protocol types**

Implement JSON-RPC request/response, MCP initialize result, tool definition, and
tool result structs.

- [ ] **Step 3: Add tool registry**

Implement static definitions for list/export/import sticker pack tools.

- [ ] **Step 4: Add protocol tests**

Test `tools/list` output includes all three tool names.

- [ ] **Step 5: Commit**

```powershell
git add Cargo.toml Cargo.lock crates/msm-mcp
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add MCP protocol tools"
```

### Task 2: MCP Tool Execution And Route

**Files:**
- Modify: `crates/msm-mcp/src/lib.rs`
- Create: `crates/msm-mcp/src/handler.rs`
- Modify: `crates/msm-app/Cargo.toml`
- Modify: `crates/msm-app/src/lib.rs`

- [ ] **Step 1: Add JSON-RPC handler**

Support `initialize`, `ping`, `tools/list`, and `tools/call`.

- [ ] **Step 2: Execute pack tools**

Call storage repository methods for list/export/import.

- [ ] **Step 3: Mount `/mcp`**

Merge the MCP router into `msm-app`.

- [ ] **Step 4: Add tests**

Test initialize, tools/list, list tool call, export tool call, import tool call,
and unknown method error.

- [ ] **Step 5: Verify**

```powershell
cargo test -p msm-mcp
cargo clippy -p msm-mcp --all-targets -- -D warnings
cargo test -p msm-app
```

- [ ] **Step 6: Commit**

```powershell
git add Cargo.lock crates/msm-mcp crates/msm-app
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add MCP endpoint"
```

### Task 3: Documentation And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/dev/architecture.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`
- Modify: `docs/user/README.md`

- [ ] **Step 1: Document endpoint and tools**

Document `/mcp`, supported methods, and tool names.

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
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update MCP endpoint status"
```
