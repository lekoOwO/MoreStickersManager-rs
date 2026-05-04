# MSM P19 Web Local Login Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Web local register/login UI backed by P18 auth APIs.

**Architecture:** Extend `apps/web/src/lib/api-client.ts` with a local auth client. Add a local auth card in `AppShell.vue` that reuses current Card/Button primitives and emits the login-issued PAT through the existing token storage event.

**Tech Stack:** Vue 3, Vite, Vitest, Tailwind CSS v4, local Shadcn-compatible primitives.

---

### Task 1: Planning Checkpoint

**Files:**
- Create: `docs/superpowers/specs/2026-05-04-msm-p19-web-local-login-design.md`
- Create: `docs/superpowers/plans/2026-05-04-msm-p19-web-local-login.md`

- [ ] **Step 1: Commit planning docs**

```powershell
git add docs/superpowers/specs/2026-05-04-msm-p19-web-local-login-design.md docs/superpowers/plans/2026-05-04-msm-p19-web-local-login.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: add P19 Web local login plan"
```

### Task 2: Web Auth Client Tests And Implementation

**Files:**
- Modify: `apps/web/src/lib/api-client.ts`
- Modify: `apps/web/src/lib/api-client.test.ts`

- [ ] **Step 1: Add failing tests**

Test `createLocalAuthClient().registerLocalUser` and `.loginLocalUser`.

- [ ] **Step 2: Implement auth client**

Use:

- `POST /api/v1/auth/local/register`
- `POST /api/v1/auth/local/login`

- [ ] **Step 3: Verify client tests**

```powershell
npm run web:test -- api-client
```

### Task 3: Web Login UI

**Files:**
- Modify: `apps/web/src/app/AppShell.vue`
- Modify: `apps/web/src/lib/i18n.ts`

- [ ] **Step 1: Add local auth state**

Add fields for user ID, display name, email, password, token ID, and scopes.

- [ ] **Step 2: Add register/login actions**

Register creates a user. Login stores the returned PAT via `updatePatToken` and
shows it once.

- [ ] **Step 3: Verify Web**

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

- [ ] **Step 4: Commit implementation**

```powershell
git add apps/web
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add Web local login"
```

### Task 4: Documentation And Status

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document Web local login**

Document browser-local token behavior and bootstrap limitations.

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
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update Web local login status"
```
