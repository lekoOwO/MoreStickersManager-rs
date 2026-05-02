# MSM P10 Embedded Web Assets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Embed Web UI assets in the Rust service binary with disk override support.

**Architecture:** Add an `msm-app` build script that prepares an embed directory, use `include_dir` in the app crate, and replace the disk-only `ServeDir` fallback with a safe disk-first, embedded-second asset handler.

**Tech Stack:** Rust, Axum, include_dir, mime_guess.

---

### Task 1: Embed Asset Pipeline

**Files:**
- Modify: `crates/msm-app/Cargo.toml`
- Create: `crates/msm-app/build.rs`
- Create: `crates/msm-app/web-dist-placeholder/index.html`
- Modify: `crates/msm-app/src/lib.rs`

- [ ] **Step 1: Add dependencies**

Add `include_dir`, `bytes`, and `mime_guess` to `msm-app`.

- [ ] **Step 2: Add build script**

Copy `apps/web/dist` to `$OUT_DIR/web-dist-embed` when it exists; otherwise copy
`crates/msm-app/web-dist-placeholder`.

- [ ] **Step 3: Add placeholder**

Create a minimal HTML file explaining that the real frontend dist was not built.

- [ ] **Step 4: Replace fallback service**

Implement a disk-first and embedded-second fallback handler with safe path
normalization and SPA `index.html` fallback.

- [ ] **Step 5: Add tests**

Test path normalization rejects traversal and embedded index exists.

- [ ] **Step 6: Verify**

```powershell
cargo test -p msm-app
cargo clippy -p msm-app --all-targets -- -D warnings
```

- [ ] **Step 7: Commit**

```powershell
git add Cargo.lock crates/msm-app
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: embed web assets"
```

### Task 2: Documentation And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/dev/architecture.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document build order**

Document that `npm run web:build` before `cargo build -p msm-app` embeds the real
frontend dist, while missing dist embeds the placeholder.

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
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update embedded web asset status"
```
