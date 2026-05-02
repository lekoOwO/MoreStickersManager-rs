# MoreStickersManager-rs

MoreStickersManager-rs, abbreviated MSM, is a Rust self-hosted manager for MoreStickers-compatible sticker packs.

Current phase: P0/P1 foundation and format compatibility.

## Compatibility Target

MSM preserves the `.stickerpack` JSON shape used by Equicord moreStickers and MoreStickersConverter. The compatibility source of truth is documented in `docs/dev/compatibility.md`.

## Development

Run the current baseline checks:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Before the Rust workspace exists, use:

```powershell
git status --short
```

## CLI Slice

The current CLI is an HTTP client for the P4 API slice:

```powershell
cargo run -p msm-cli -- health
cargo run -p msm-cli -- packs list --user-id user_1
cargo run -p msm-cli -- packs import --tenant-id tenant_1 --owner-user-id user_1 --pack-id pack_1 --visibility private --file pack.stickerpack
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
```

## Project Docs

- `docs/dev/architecture.md`: architecture and crate boundaries.
- `docs/dev/compatibility.md`: sticker pack format compatibility.
- `docs/user/README.md`: user-facing documentation index.
- `docs/agents/README.md`: agent handoff entrypoint.
- `docs/status/current.md`: current development state.
