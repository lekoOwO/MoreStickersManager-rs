# Provider Normalization

P6 adds a provider interface for converting provider-specific JSON into
`MoreStickers`-compatible sticker packs.

## Implemented Providers

- `telegram`: normalizes a Telegram sticker set fixture into a pack with
  `MoreStickers:Telegram:Pack:{stickerSetName}` and
  `MoreStickers:Telegram:Sticker:{stickerSetName}:{fileUniqueId}` IDs.
- `line-stickers`: normalizes a LINE sticker fixture into
  `MoreStickers:Line:Pack:{packId}` and
  `MoreStickers:Line:Sticker:{packId}:{stickerId}` IDs.
- `line-emojis`: normalizes a LINE emoji fixture into
  `MoreStickers:Line:Emoji-Pack:{packId}` and
  `MoreStickers:Line-Emoji:{packId}:{emojiId}` IDs.

## Planned Providers

Signal, WhatsApp, Kakao, Band, OGQ, and Viber are registered as planned
providers only. They must not appear as implemented capabilities until they can
normalize payloads and preserve MoreStickers-compatible export output.

## Current Scope

The provider crate accepts already-fetched JSON and now exposes testable remote
fetch plan boundaries. Provider import credentials are modeled as tenant-scoped
configuration records rather than export targets, because providers are input-side
normalizers and tenant admins need a separate lifecycle for provider secrets. `telegram_sticker_set_fetch_plan` describes the Telegram
Bot API `getStickerSet` metadata request without embedding a bot token and marks
assets as requiring Telegram `getFile`/file download resolution.
`line_sticker_pack_fetch_plan` describes a LINE sticker-shop product metadata
request and marks assets as direct remote URLs once parsed. The crate still does
not execute network requests, download assets, store assets, or import
normalized packs into the database. Runtime crates should execute these plans,
feed resulting JSON to `StickerProvider`, then internalize assets.

`msm-app` provides the first runtime-side boundary for this flow:

- `fetch_provider_metadata` executes a `ProviderRemoteFetchPlan` through an
  injected `ProviderMetadataFetcher` so tests and callers can supply the actual
  HTTP implementation.
- `internalize_direct_remote_pack_assets` downloads direct remote sticker URLs
  through an injected `ProviderAssetDownloader`, writes them to
  `LocalAssetStore`, sets sticker filenames, rewrites image URLs to MSM-hosted
  `/assets/packs/{pack_id}/{filename}` URLs, and updates the pack logo.

`ProviderImportWorker` can now pick a queued provider import job, mark it
running, execute the injected metadata fetch, normalize LINE fixture-schema
metadata, download direct remote sticker assets into `LocalAssetStore`, upsert a
private MSM pack, record success/failure events, and schedule retryable failures
with backoff. Service startup can poll jobs when
`MSM_PROVIDER_IMPORT_WORKER_ENABLED=true`; the loop uses
`MSM_PROVIDER_IMPORT_WORKER_POLL_INTERVAL_MS`,
`MSM_PROVIDER_IMPORT_RETRY_BACKOFF_MS`, and `MSM_PUBLIC_ASSET_BASE_URL` for
polling, retry, and rewritten asset URLs.

Telegram still needs runtime `getFile` resolution before it can reuse the asset
internalization path. LINE still needs a runtime parser that converts fetched
product data into the existing LINE fixture schema.

The API exposes the first protected workflow surface at
`POST /api/v1/provider-imports/plan`. It requires `provider.import`, validates
same-user tenant access, and returns Telegram or LINE fetch plans suitable for
runtime execution. CLI can call the same surface with
`msm providers plan --tenant-id ... --owner-user-id ... --provider-id ... --remote-id ...`.
MCP can call the same planning boundary with `msm.create_provider_import_plan`.
MCP can also create/read provider import jobs and list events with
`msm.create_provider_import_job`, `msm.get_provider_import_job`, and
`msm.list_provider_import_job_events`.
Web has a Providers workspace planner that calls the same endpoint, displays the
metadata request and asset strategy, queues provider import jobs, refreshes job
status, and lists job events.
`POST /api/v1/provider-import-jobs` now persists a queued provider import job
with the same protected planning payload and records an initial queued event.
`GET /api/v1/provider-import-jobs/{job_id}` and
`GET /api/v1/provider-import-jobs/{job_id}/events` expose status/event reads.
A tested app worker foundation can execute LINE fixture-schema/direct-asset jobs,
LINE product pages with embedded metadata, and Telegram `getFile` asset
downloads; service startup can run that worker loop when enabled. Provider
credential/config MCP/Web UI remains pending. CLI can list/upsert/delete
tenant-scoped provider configs, and CLI, MCP, and Web can create provider
import jobs and read job status/events.

## Provider Versus Export Target

Providers are input-side normalizers. Export targets are output-side publishers
or serializers. Keep these boundaries separate even when the same external
service appears on both sides.

- Telegram as a provider means importing or normalizing an existing Telegram
  sticker set into an MSM pack.
- Telegram as an export target means converting MSM assets and using a Telegram
  bot to create or update a Telegram sticker set.
- MoreStickers is currently an export format and compatibility contract, not a
  remote provider.

P24 plans a target-neutral export pipeline inspired by moe-sticker-bot. The
pipeline should put format conversion in `msm-media`, target orchestration in
`msm-exporters`, and Telegram Bot API calls in `msm-telegram`.

## Export Target Status

Implemented export target foundations:

- `morestickers`: serializes canonical MSM packs as `.stickerpack` JSON while
  preserving the existing MoreStickers compatibility contract.
- `telegram`: has target capability metadata, Telegram set planning, media
  profile selection, `teloxide` bot construction, Web/API/CLI/MCP target/job
  management, worker dry-run planning, opt-in worker publication for
  `"dryRun": false`, and Web result URL display.

Not implemented yet:

- Provider-side remote fetch/download orchestration.
- Reconciliation policies for updating or deleting remote Telegram sticker sets.
