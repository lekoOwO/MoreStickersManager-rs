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
