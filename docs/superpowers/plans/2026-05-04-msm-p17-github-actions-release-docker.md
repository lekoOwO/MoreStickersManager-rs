# MSM P17 GitHub Actions Release And Docker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CI, Docker publish, prerelease, and release workflows matching the referenced workflow family while adapting them for MSM's Rust + Web build.

**Architecture:** Keep four workflow files under `.github/workflows`. Add a root `Dockerfile` and `.dockerignore` for GHCR publishing. Release jobs build `apps/web/dist` first, then `msm-app` so the binary embeds the Web UI.

**Tech Stack:** GitHub Actions, Rust stable, Node.js 24, npm, Docker Buildx, cross, `softprops/action-gh-release`.

---

### Task 1: Planning Checkpoint

**Files:**
- Create: `docs/superpowers/specs/2026-05-04-msm-p17-github-actions-release-docker-design.md`
- Create: `docs/superpowers/plans/2026-05-04-msm-p17-github-actions-release-docker.md`

- [ ] **Step 1: Commit planning docs**

```powershell
git add docs/superpowers/specs/2026-05-04-msm-p17-github-actions-release-docker-design.md docs/superpowers/plans/2026-05-04-msm-p17-github-actions-release-docker.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: add P17 release workflow plan"
```

### Task 2: Workflow And Docker Implementation

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `.github/workflows/docker.yml`
- Create: `.github/workflows/prerelease.yml`
- Create: `.github/workflows/release.yml`
- Create: `Dockerfile`
- Create: `.dockerignore`

- [ ] **Step 1: Update CI**

Make CI trigger on pull requests and main pushes, then run:

- Rust fmt/clippy/test on Ubuntu.
- Web typecheck/test/build on Ubuntu.
- Cross-platform `cargo build --locked -p msm-app` on Ubuntu, macOS, Windows.

- [ ] **Step 2: Add Docker workflow**

Publish `ghcr.io/${{ github.repository }}` on `main` and `v*` tags using
Buildx, QEMU, metadata, and GHA cache.

- [ ] **Step 3: Add prerelease and release workflows**

Build Web dist and `msm-app` binaries for Linux, macOS, and Windows. Publish
`prerelease` on main and normal releases on `v*` tags.

- [ ] **Step 4: Add Dockerfile**

Use a Node stage for Web dist, a Rust stage for `msm-app`, and a Debian runtime
stage with persistent `/data`.

### Task 3: Verification And Commit

**Files:**
- Workflow and Docker files from Task 2.

- [ ] **Step 1: Validate file diffs**

```powershell
git diff --check -- .github/workflows Dockerfile .dockerignore
```

- [ ] **Step 2: Run local workflow equivalents**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run web:typecheck
npm run web:test
npm run web:build
cargo build --locked -p msm-app
```

- [ ] **Step 3: Commit implementation**

```powershell
git add .github/workflows Dockerfile .dockerignore
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "ci: add release and Docker workflows"
```

### Task 4: Documentation And Status

**Files:**
- Modify: `README.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document workflow coverage**

Document CI, Docker, prerelease, and release workflows plus local equivalents.

- [ ] **Step 2: Commit documentation**

```powershell
git add README.md docs/agents/testing.md docs/status/current.md docs/status/checkpoints.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update workflow status"
```
