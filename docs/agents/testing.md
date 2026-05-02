# Testing Guide

## P1 Domain Tests

Run:

```powershell
cargo test -p msm-domain
```

These tests prove:

- `.stickerpack` fixtures parse.
- serialized JSON uses camelCase field names.
- optional fields are skipped when absent.
- provider ID helpers match upstream conventions.
- asset URL resolution chooses CDN URL before app URL.

## Workspace Tests

Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## P2 Storage Tests

Run:

```powershell
cargo test -p msm-storage
```

These tests prove:

- database URL parsing rejects unsupported schemes;
- local asset keys reject traversal attempts;
- local asset bytes can be written, read, and deleted;
- SQLite migrations create the P2 schema;
- repository operations can create tenant, user, pack, sticker, and subscription records;
- portable user data can be exported from one SQLite database and imported into another.
