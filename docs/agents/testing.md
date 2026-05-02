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

## P3 Authorization Tests

Run:

```powershell
cargo test -p msm-domain --test authorization
```

These tests prove:

- tenant admins can manage in-tenant resources but cannot cross tenants;
- owners can manage owned private packs;
- tenant members can only read member-access private packs;
- anonymous access is limited to public resources;
- PAT access requires matching scopes;
- pack secrets and subscription secrets only grant their narrow resource access;
- public subscription groups do not globally expose private pack assets.
