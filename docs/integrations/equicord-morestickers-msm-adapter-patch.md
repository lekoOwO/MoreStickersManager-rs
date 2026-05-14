# Equicord moreStickers MSM Patch Artifact

Patch file:

- `docs/integrations/patches/equicord-morestickers-msm-adapter.patch`

Upstream base inspected:

- Repository: `https://github.com/Equicord/Equicord`
- Base commit: `3a45528`
- Plugin path: `src/equicordplugins/moreStickers`

## Scope

The patch prepares the Equicord `moreStickers` plugin for MSM-managed sticker
packs and subscription groups.

It adds:

- an MSM subscription tab in the plugin settings modal;
- direct MSM subscription fetches without the public LINE CORS proxy;
- validation for MSM dynamic pack-set payloads;
- initial add, startup refresh, manual refresh, removal, and version-aware
  update behavior for dynamic pack sets;
- header-aware asset fetching for protected MSM-hosted sticker images;
- a small blob URL cache so protected assets can render in `<img>` contexts;
- `sendSticker` asset fetch support for protected MSM assets.

## Verification

Evidence collected in this repository:

- `git -C tmp/Equicord-moreStickers diff --check` passed.
- `git diff --check -- docs/integrations/patches/equicord-morestickers-msm-adapter.patch` passed.
- `git -C tmp/Equicord-patch-check apply --check <patch>` passed against a
  clean detached worktree at `3a45528`.
- `pnpm testTsc` passed in `tmp/Equicord-patch-work` after applying the patch.
- Targeted ESLint passed for the changed moreStickers TypeScript/TSX files.
- Targeted stylelint passed for `src/equicordplugins/moreStickers/style.css`.
- `pnpm buildStandalone` passed.
- Patch content includes new `assetCache.tsx` and `msm.ts` files plus settings,
  picker, upload, dynamic-sync, type, and style updates.

Not yet verified:

- Full `pnpm lint` still fails on upstream-unrelated files:
  `src/equicordplugins/userpfp/index.tsx`, `src/main/applyHostPatch.ts`,
  `src/main/hostUpdateHook.ts`, and `src/main/patchWin32Updater.ts`.
- Runtime behavior inside Discord/Equicord.
- Public and protected MSM pack/group subscriptions end-to-end.

## Apply Command

From an Equicord checkout at the compatible base:

```powershell
git apply D:\DATA\Codes\MoreStickersManager-rs\docs\integrations\patches\equicord-morestickers-msm-adapter.patch
```

After applying, run Equicord's normal verification:

```powershell
pnpm install --frozen-lockfile
pnpm testTsc
pnpm lint
pnpm lint-styles
pnpm buildStandalone
```
