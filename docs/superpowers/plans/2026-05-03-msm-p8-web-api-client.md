# MSM P8 Web API Client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the Web UI dashboard data boundary to the P4 pack listing API shape.

**Architecture:** Keep `StickerPackSummary` as the dashboard-facing model, add a separate `api-client.ts` for HTTP and mapping, and keep mock data as a fallback client mode for tests and static preview.

**Tech Stack:** TypeScript, Vue 3, Fetch API, Vitest.

---

### Task 1: Typed Pack API Client

**Files:**
- Modify: `apps/web/src/lib/sticker-packs.ts`
- Create: `apps/web/src/lib/api-client.ts`
- Create: `apps/web/src/lib/api-client.test.ts`

- [ ] **Step 1: Move mock fixture export**

Export `mockStickerPacks` from `sticker-packs.ts` and keep `StickerPackSummary`
unchanged for dashboard components.

- [ ] **Step 2: Add API DTO and mapper**

Create `ApiStickerPackRecord`, `createPackClient`, and `mapApiPackRecord` in
`api-client.ts`.

- [ ] **Step 3: Add tests**

Test mock fallback, URL construction with encoded `userId`, provider inference,
visibility mapping, sticker counts, and updated date extraction.

- [ ] **Step 4: Run frontend tests**

```powershell
npm run web:test
```

Expected: PASS with the new API client tests.

- [ ] **Step 5: Commit client**

```powershell
git add apps/web/src/lib
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add web pack API client"
```

### Task 2: Dashboard Client Integration

**Files:**
- Modify: `apps/web/src/app/PackDashboard.vue`
- Modify: `apps/web/src/app/PackDashboard.test.ts`
- Create: `apps/web/src/env.d.ts`

- [ ] **Step 1: Use API client in dashboard**

Create a client from `import.meta.env.VITE_MSM_API_BASE_URL` and
`VITE_MSM_USER_ID`, defaulting to mock fallback when unset.

- [ ] **Step 2: Preserve dashboard test determinism**

Keep the dashboard test independent of network by relying on unset env fallback.

- [ ] **Step 3: Run frontend verification**

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

Expected: all pass.

- [ ] **Step 4: Commit dashboard integration**

```powershell
git add apps/web/src
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: connect dashboard data client"
```

### Task 3: Documentation And Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document environment variables**

Document `VITE_MSM_API_BASE_URL` and `VITE_MSM_USER_ID`.

- [ ] **Step 2: Run full verification**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run web:typecheck
npm run web:test
npm run web:build
```

Expected: all pass.

- [ ] **Step 3: Commit documentation**

```powershell
git add README.md docs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update web API client status"
```
