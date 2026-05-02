# MSM P9 Service Binary Design

## Scope

P9 adds the first runnable Rust service binary. It wires the existing storage,
API router, local asset store, and Web UI build output directory into one
process.

## Goals

- Add `msm-app` binary crate.
- Read service configuration from environment variables.
- Initialize database connection and migrations on startup.
- Serve existing API routes and local assets through `msm-api`.
- Serve Web UI `dist` files from disk with SPA fallback to `index.html`.
- Add tests for environment config parsing.

## Non-Goals

- No production auth/session handling in P9.
- No generated frontend build during `cargo build` in P9.
- No final embedded frontend bytes in P9; this slice creates the service
  boundary that a later embedding phase can replace with embedded assets.

## Configuration

- `MSM_BIND_ADDR`: bind address, default `127.0.0.1:3000`.
- `MSM_DATABASE_URL`: database URL, default `sqlite:data/msm.sqlite3`.
- `MSM_ASSET_DIR`: local asset storage directory, default `data/assets`.
- `MSM_WEB_DIST_DIR`: Web UI dist directory, default `apps/web/dist`.

## Runtime Flow

1. Parse config.
2. Connect storage and run migrations.
3. Build `ApiState`.
4. Build the API router.
5. Add static Web UI fallback service.
6. Bind TCP listener and serve the app.
