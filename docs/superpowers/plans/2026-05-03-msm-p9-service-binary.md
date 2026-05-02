# MSM P9 Service Binary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first runnable Rust service binary for MSM.

**Architecture:** Create `msm-app` as the composition crate. It depends on `msm-api` and `msm-storage`, parses environment config, initializes storage, builds `ApiState`, and layers static Web UI serving behind the API router.

**Tech Stack:** Rust, Axum, Tokio, tower-http static file services, SQLx through `msm-storage`.

---

### Task 1: App Crate Scaffold

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/msm-app/Cargo.toml`
- Create: `crates/msm-app/src/lib.rs`
- Create: `crates/msm-app/src/main.rs`

- [ ] **Step 1: Add workspace member**

Add `crates/msm-app` to root workspace members.

- [ ] **Step 2: Add crate dependencies**

Use `axum`, `tokio`, `tower-http`, `msm-api`, and `msm-storage`.

- [ ] **Step 3: Add config parser**

Implement `AppConfig::from_env_map` and `AppConfig::from_env`.

- [ ] **Step 4: Add router composition**

Implement `build_app_router(state, web_dist_dir)` using `msm_api::build_router`
and `ServeDir` with `index.html` fallback.

- [ ] **Step 5: Add main**

Implement async startup with database migrations, asset store creation, and
`axum::serve`.

- [ ] **Step 6: Add tests**

Test default config and environment override parsing.

- [ ] **Step 7: Verify**

```powershell
cargo test -p msm-app
cargo clippy -p msm-app --all-targets -- -D warnings
```

- [ ] **Step 8: Commit**

```powershell
git add Cargo.toml Cargo.lock crates/msm-app
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add service binary"
```

### Task 2: Documentation And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/dev/architecture.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document service command**

Document `cargo run -p msm-app` and environment variables.

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
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update service binary status"
```
