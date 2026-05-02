# MSM User Documentation

MSM currently has foundation, storage, authorization, API, CLI, and provider
normalization slices.

Current usable contract: `.stickerpack` compatibility is documented in `../dev/compatibility.md`.

Provider normalization status is documented in `../dev/providers.md`.

Current CLI examples:

```powershell
cargo run -p msm-cli -- health
cargo run -p msm-cli -- packs list --user-id user_1
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
```
