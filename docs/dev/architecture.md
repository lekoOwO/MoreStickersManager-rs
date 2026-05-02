# Architecture

MSM is built as a Rust workspace. The domain crate owns MoreStickers compatibility types and provider-neutral logic. Later crates add storage, API, CLI, MCP, providers, and the final app binary.

## Crate Boundaries

- `msm-domain`: compatibility models, pure ID helpers, pure URL resolution, and import/export helpers.
- `msm-domain::authz`: pure authorization policy evaluation for pack and subscription access.
- `msm-storage`: database repositories and asset storage, added in P2.
- `msm-api`: HTTP API and OpenAPI, added in P4.
- `msm-cli`: command-line client, added in P5.
- `msm-mcp`: MCP endpoint, added in P9.
- `msm-providers`: provider registry and provider-specific normalization into `MoreStickers` packs, added in P6.
- `msm-app`: final service binary and embedded frontend, added after API and Web UI foundations exist.

## Dependency Rule

`msm-domain` must not depend on Axum, SQLx, provider SDKs, frontend code, or runtime-specific infrastructure.

Authorization policies stay in `msm-domain` so API, CLI, MCP, and Web UI assumptions can share the same rules.

## Provider Boundary

`msm-providers` converts provider-specific payloads into `msm-domain::StickerPack`.
It must keep output IDs and `.stickerpack` field names compatible with upstream
moreStickers conventions. Network fetching and asset downloading are not part of
the P6 provider boundary; they should be added behind explicit provider
capabilities so API, CLI, MCP, and Web UI can expose the same feature set.
