# MSM User Documentation

MSM currently has foundation, storage, authorization, API, CLI, provider
normalization, and Web UI foundation slices.

Current usable contract: `.stickerpack` compatibility is documented in `../dev/compatibility.md`.

Provider normalization status is documented in `../dev/providers.md`.

Current CLI examples:

```powershell
cargo run -p msm-cli -- health
cargo run -p msm-cli -- packs list --user-id user_1
cargo run -p msm-cli -- packs export --pack-id pack_1 --output -
```

Current Web UI examples:

```powershell
npm run web:dev
npm run web:build
```

The P7 Web UI uses mock data. It demonstrates the app shell, responsive layout,
theme toggle, language toggle, and sticker-pack dashboard before backend API
integration is wired in.
