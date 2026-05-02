# MSM P7 Web UI Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first Vue/Shadcn Vue/Tailwind CSS v4 Web UI foundation for MSM.

**Architecture:** Add an npm workspace at the repository root and a Vite Vue app in `apps/web`. Keep preferences, i18n, mock data access, and UI components in separate files so future API integration can replace only the client boundary.

**Tech Stack:** Vue 3, TypeScript, Vite, Tailwind CSS v4, Shadcn Vue-compatible local UI primitives, Vitest, Vue Test Utils.

---

### Task 1: Frontend Workspace Scaffold

**Files:**
- Create: `apps/web/package.json`
- Create: `apps/web/index.html`
- Create: `apps/web/vite.config.ts`
- Create: `apps/web/tsconfig.json`
- Create: `apps/web/tsconfig.app.json`
- Create: `apps/web/tsconfig.node.json`
- Modify: `package.json`

- [ ] **Step 1: Define npm workspace scripts**

Replace the root `package.json` with:

```json
{
  "private": true,
  "workspaces": [
    "apps/web"
  ],
  "scripts": {
    "web:dev": "npm --workspace apps/web run dev",
    "web:build": "npm --workspace apps/web run build",
    "web:test": "npm --workspace apps/web run test",
    "web:typecheck": "npm --workspace apps/web run typecheck"
  },
  "devDependencies": {
    "shadcn": "^4.6.0",
    "shadcn-vue": "^2.3.0"
  }
}
```

- [ ] **Step 2: Create app package**

Create `apps/web/package.json` with Vue, Vite, Tailwind v4, test tooling, and
Shadcn Vue runtime dependencies.

- [ ] **Step 3: Create Vite and TypeScript config**

Configure the `@/*` alias to `src/*`, enable Vue plugin and Tailwind v4 Vite
plugin, and configure Vitest with `jsdom`.

- [ ] **Step 4: Install dependencies**

Run:

```powershell
npm install
```

Expected: npm updates `package-lock.json` and installs workspace dependencies.

- [ ] **Step 5: Verify empty scaffold**

Run:

```powershell
npm run web:typecheck
```

Expected: PASS after source files are added in later tasks; if this fails before
source files exist, continue to Task 2 and rerun.

- [ ] **Step 6: Commit scaffold**

```powershell
git add package.json package-lock.json apps/web
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "chore: scaffold web workspace"
```

### Task 2: Design Tokens, Preferences, And i18n

**Files:**
- Create: `apps/web/src/styles.css`
- Create: `apps/web/src/main.ts`
- Create: `apps/web/src/App.vue`
- Create: `apps/web/src/lib/utils.ts`
- Create: `apps/web/src/lib/theme.ts`
- Create: `apps/web/src/lib/i18n.ts`
- Create: `apps/web/src/test/setup.ts`
- Create: `apps/web/src/lib/theme.test.ts`
- Create: `apps/web/src/lib/i18n.test.ts`

- [ ] **Step 1: Add Tailwind v4 CSS and semantic tokens**

Create `styles.css` with `@import "tailwindcss";`, a custom `@theme inline`
block, light/dark CSS variables, and a layered app background.

- [ ] **Step 2: Add utility helper**

Create `cn(...inputs)` using `clsx` and `tailwind-merge`.

- [ ] **Step 3: Add theme preference store**

Implement `createThemeController(storage, document)` with `light`, `dark`, and
`system`, persisted to `localStorage`, and applied to `documentElement`.

- [ ] **Step 4: Add i18n preference store**

Implement `createI18nController(storage)` with `zh-TW` and `en`, persisted to
`localStorage`, and a typed `t(key)` lookup.

- [ ] **Step 5: Add tests**

Test theme default/toggle/application and locale message lookup.

- [ ] **Step 6: Run frontend tests**

```powershell
npm run web:test
```

Expected: theme and i18n tests pass.

- [ ] **Step 7: Commit preferences**

```powershell
git add apps/web/src package.json package-lock.json
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add web preferences"
```

### Task 3: Shadcn Vue-Compatible UI Primitives

**Files:**
- Create: `components.json`
- Create: `apps/web/src/components/ui/button/Button.vue`
- Create: `apps/web/src/components/ui/button/index.ts`
- Create: `apps/web/src/components/ui/card/Card.vue`
- Create: `apps/web/src/components/ui/card/CardHeader.vue`
- Create: `apps/web/src/components/ui/card/CardTitle.vue`
- Create: `apps/web/src/components/ui/card/CardDescription.vue`
- Create: `apps/web/src/components/ui/card/CardContent.vue`
- Create: `apps/web/src/components/ui/card/index.ts`
- Create: `apps/web/src/components/ui/badge/Badge.vue`
- Create: `apps/web/src/components/ui/badge/index.ts`

- [ ] **Step 1: Add Shadcn Vue config**

Create `components.json` with the Shadcn Vue schema, `new-york` style,
TypeScript enabled, Tailwind CSS path `apps/web/src/styles.css`, and aliases
pointing to `apps/web/src`.

- [ ] **Step 2: Add button primitive**

Implement a small Shadcn Vue-compatible button with `default`, `secondary`,
`outline`, `ghost`, and `destructive` variants.

- [ ] **Step 3: Add card primitive**

Implement card composition components for shell and dashboard cards.

- [ ] **Step 4: Add badge primitive**

Implement badge variants for provider, visibility, and status labels.

- [ ] **Step 5: Run typecheck**

```powershell
npm run web:typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit primitives**

```powershell
git add components.json apps/web/src/components/ui
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add web UI primitives"
```

### Task 4: Responsive Dashboard Shell

**Files:**
- Create: `apps/web/src/lib/sticker-packs.ts`
- Create: `apps/web/src/app/AppShell.vue`
- Create: `apps/web/src/app/PackDashboard.vue`
- Create: `apps/web/src/app/PackDashboard.test.ts`
- Modify: `apps/web/src/App.vue`

- [ ] **Step 1: Add mock pack client**

Create `StickerPackSummary` and `listStickerPacks()` with Telegram, LINE
sticker, and LINE emoji examples.

- [ ] **Step 2: Add app shell**

Create a responsive shell with top bar, desktop side rail, mobile navigation
toggle, theme toggle, and language toggle.

- [ ] **Step 3: Add dashboard**

Create metric cards, provider coverage cards, and responsive pack rows using the
UI primitives and i18n labels.

- [ ] **Step 4: Add render test**

Test that the dashboard renders pack totals, provider names, and visibility
labels from the mock client.

- [ ] **Step 5: Run frontend verification**

```powershell
npm run web:typecheck
npm run web:test
npm run web:build
```

Expected: all pass and `apps/web/dist` is produced locally but remains ignored.

- [ ] **Step 6: Commit dashboard**

```powershell
git add apps/web/src
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add responsive web dashboard"
```

### Task 5: Documentation And Workspace Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/dev/architecture.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document web commands and scope**

Add Web UI command examples and note that P7 uses mock data only.

- [ ] **Step 2: Update status**

Set current phase to P7 complete and next phase to API integration or app binary
embedding, depending on the selected next task.

- [ ] **Step 3: Run full verification**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run web:typecheck
npm run web:test
npm run web:build
```

Expected: all pass.

- [ ] **Step 4: Commit documentation**

```powershell
git add README.md docs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update web foundation status"
```
