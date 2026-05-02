# MSM P7 Web UI Foundation Design

## Scope

P7 creates the frontend foundation for MSM. It does not complete every Web UI
feature. The deliverable is a production-shaped Vue application that can be
developed, tested, built, and later embedded into the Rust service binary.

## Goals

- Use Vue 3, TypeScript, Vite, Shadcn Vue conventions, and Tailwind CSS v4.
- Provide a responsive app shell for desktop, tablet, and mobile management.
- Include light/dark theme switching with persisted preference.
- Include Traditional Chinese and English UI switching with persisted preference.
- Show a sticker-pack management dashboard with realistic mock data.
- Keep UI data access behind a small client boundary so future API integration
  does not rewrite presentation components.
- Add focused frontend tests for preference stores and visible dashboard output.

## Non-Goals

- No authenticated session, OIDC, SSO, PAT management, or RBAC enforcement UI in
  P7.
- No real backend mutation or API integration in P7.
- No frontend embedding into the Rust binary in P7; the build output path is
  prepared for that later phase.
- No complete sticker-pack CRUD implementation in P7.

## Architecture

The frontend lives in `apps/web` and is managed by the root npm workspace. The
Rust workspace remains under `crates`, while the frontend build emits
`apps/web/dist`, which is already ignored by repository hygiene rules.

The Vue app is split into small units:

- `src/app`: top-level application shell and dashboard composition.
- `src/components`: local reusable components.
- `src/components/ui`: Shadcn Vue-compatible primitives used by app components.
- `src/lib`: preference stores, mock API client, utility functions, and i18n.
- `src/styles.css`: Tailwind CSS v4 imports, design tokens, and theme variables.

## UI Direction

The first visual language should feel like an operator console for a sticker
platform, not a default admin dashboard. The layout uses a layered background,
compact command-oriented cards, clear metric panels, and pack rows designed for
quick scanning. It must keep semantic tokens for maintainability and avoid
hard-coded one-off color utilities where design tokens are more appropriate.

## Data Flow

P7 uses a mock `StickerPackSummary` client to avoid blocking on backend auth and
CRUD decisions. Components consume a simple promise-returning API:

```ts
listStickerPacks(): Promise<StickerPackSummary[]>
```

Later phases can replace the mock implementation with OpenAPI-generated or
handwritten HTTP clients while preserving component inputs.

## Accessibility And UX

- Theme and language toggles are real buttons with accessible labels.
- Mobile navigation is available from the top bar.
- Main content uses semantic landmarks.
- Pack rows expose status, visibility, provider, sticker count, and updated time
  in scan-friendly text.

## Testing

P7 uses Vitest with Vue Test Utils. The first tests cover:

- Theme preference defaults, toggling, and DOM class application.
- Locale preference defaults and message lookup.
- Dashboard rendering of pack counts, providers, and visibility labels.

## References

- Shadcn Vue Vite/Tailwind v4 docs: <https://next.shadcn-vue.com/docs/installation/vite>
- Shadcn Vue Tailwind v4 notes: <https://www.shadcn-vue.com/docs/tailwind-v4>
- Shadcn Vue components.json docs: <https://v3.shadcn-vue.com/docs/components-json>
