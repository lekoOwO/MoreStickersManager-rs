# MSM P16 Web PAT Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Web UI PAT token forwarding and PAT lifecycle management.

**Architecture:** Extend the existing Vite/Vue API client with optional Bearer headers and PAT methods. Keep UI inside the current dashboard shell using existing local Shadcn-compatible Card, Button, and Badge primitives.

**Tech Stack:** Vue 3, Vite, Vitest, Tailwind CSS v4, local Shadcn-compatible primitives.

---

### Task 1: Planning Checkpoint

**Files:**
- Create: `docs/superpowers/specs/2026-05-04-msm-p16-web-pat-management-design.md`
- Create: `docs/superpowers/plans/2026-05-04-msm-p16-web-pat-management.md`

- [ ] **Step 1: Commit planning docs**

```powershell
git add docs/superpowers/specs/2026-05-04-msm-p16-web-pat-management-design.md docs/superpowers/plans/2026-05-04-msm-p16-web-pat-management.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: add P16 Web PAT management plan"
```

### Task 2: Web API Client Tests

**Files:**
- Modify: `apps/web/src/lib/api-client.ts`
- Modify: `apps/web/src/lib/api-client.test.ts`

- [ ] **Step 1: Add failing tests**

Test that pack list sends `Authorization: Bearer <token>` when configured, PAT
create/list/revoke call the correct endpoints, and create exposes the raw token.

- [ ] **Step 2: Verify tests fail**

```powershell
npm run web:test -- api-client
```

Expected: compile failure or assertions because PAT client helpers do not exist.

### Task 3: Web API Client Implementation

**Files:**
- Modify: `apps/web/src/lib/api-client.ts`
- Modify: `apps/web/src/lib/api-client.test.ts`

- [ ] **Step 1: Add auth header helper**

Add optional `authToken` to API client options and apply it to fetch calls.

- [ ] **Step 2: Add PAT DTOs and client**

Add `createPatClient` with `createPersonalAccessToken`,
`listPersonalAccessTokens`, and `revokePersonalAccessToken`.

- [ ] **Step 3: Verify API client focus**

```powershell
npm run web:test -- api-client
```

### Task 4: Web UI PAT Panels

**Files:**
- Modify: `apps/web/src/App.vue`
- Modify: `apps/web/src/app/AppShell.vue`
- Modify: `apps/web/src/app/PackDashboard.vue`
- Modify: `apps/web/src/app/PackDashboard.test.ts`
- Modify: `apps/web/src/lib/i18n.ts`

- [ ] **Step 1: Add local token state**

Store the configured PAT in `localStorage` key `msm.pat`, seeded by
`VITE_MSM_PAT` when available.

- [ ] **Step 2: Add token panel**

Add a responsive Card with a PAT input, save, and clear actions.

- [ ] **Step 3: Add management panel**

Add create/list/revoke controls that use the PAT client. Show created raw token
once after create.

- [ ] **Step 4: Pass token to pack client**

Pass the token into `PackDashboard` so protected pack API calls include Bearer
auth.

- [ ] **Step 5: Verify Web focus**

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

- [ ] **Step 6: Commit implementation**

```powershell
git add apps/web
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add Web PAT management"
```

### Task 5: Documentation And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document Web PAT management**

Document `VITE_MSM_PAT`, local browser storage, and the current security
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
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update Web PAT management status"
```
