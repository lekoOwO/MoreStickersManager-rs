# MSM P10 Embedded Web Assets Design

## Scope

P10 embeds Web UI assets into the Rust service binary while preserving a disk
override for local development.

## Goals

- Add compile-time embedded Web asset support to `msm-app`.
- Prefer `MSM_WEB_DIST_DIR` files when they exist.
- Fall back to embedded assets when disk files are absent.
- Support SPA fallback to `index.html`.
- Keep clean Rust builds working even before `npm run web:build` by embedding a
  small placeholder asset set.

## Non-Goals

- No automatic npm build from Cargo in P10.
- No production cache-control policy yet.
- No frontend API/auth expansion in P10.

## Design

`crates/msm-app/build.rs` prepares `$OUT_DIR/web-dist-embed`. If
`apps/web/dist/index.html` exists, it copies that dist directory. Otherwise it
copies `crates/msm-app/web-dist-placeholder`. The crate embeds the prepared
directory with `include_dir`.

At runtime `msm-app` serves:

1. Disk file from `MSM_WEB_DIST_DIR` when present and safe.
2. Embedded matching asset when disk file is absent.
3. Embedded `index.html` for SPA routes.

This keeps development flexible while satisfying the binary-embedded asset
requirement for builds that run `npm run web:build` before `cargo build`.
