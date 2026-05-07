# MSM Telegram Export Pipeline Design

## Summary

MSM should integrate moe-sticker-bot-style Telegram sticker creation as a modular export pipeline. The design keeps provider import, media conversion, and remote publication as separate concerns so MoreStickers export, Telegram export, and future Signal/WhatsApp/Kakao/Band/OGQ/Viber exporters can share the same orchestration model without leaking target-specific rules into `msm-domain`.

References:

- moe-sticker-bot: <https://github.com/star-39/moe-sticker-bot>
- Telegram Bot API `InputSticker`: <https://core.telegram.org/bots/api#inputsticker>
- Telegram Bot API `uploadStickerFile`: <https://core.telegram.org/bots/api#uploadstickerfile>
- Telegram Bot API `createNewStickerSet`: <https://core.telegram.org/bots/api#createnewstickerset>
- Telegram Bot API `addStickerToSet`: <https://core.telegram.org/bots/api#addstickertoset>

## Reference Analysis

moe-sticker-bot proves three capabilities MSM should adopt:

- Import or download LINE/Kakao/Telegram sticker assets and normalize them into a managed pack.
- Convert arbitrary image, animation, and video inputs into Telegram-compatible static, animated, or video sticker files.
- Use a Telegram bot to create and manage sticker sets through a Web-managed workflow.

MSM should not copy moe-sticker-bot's bot-first architecture. MSM is a server/API-first manager with Web UI, API, CLI, and MCP surfaces. Telegram bot operations should be one export target behind the same job, auth, audit, and progress model used by other targets.

## Terms

- Provider: an input source that normalizes external payloads into MSM canonical sticker packs. Existing examples are Telegram import fixtures and LINE fixture normalizers.
- Export target: an output destination that publishes or serializes a canonical MSM pack. Existing MoreStickers export is a target. Telegram sticker set creation is a remote publication target.
- Media profile: target-specific output requirements such as static, animated, video, thumbnail, dimensions, file type, duration, and size budget.
- Conversion plan: deterministic per-asset steps from original MSM asset to a target media profile.
- Publication job: durable async operation that performs conversion, upload, remote create/update calls, progress events, and result recording.

## Architecture

Add three bounded layers after P24:

- `msm-media`: probes media, creates conversion plans, executes configured converter commands, and caches prepared outputs. It has no Telegram or MoreStickers API code.
- `msm-exporters`: owns target-neutral exporter traits, capability metadata, export planning, and exporter registration. It can include a MoreStickers serializer adapter and a Telegram exporter implementation.
- `msm-telegram`: owns the teloxide bot boundary and Telegram bot configuration. It has no database, Web, CLI, or MCP logic.

`msm-app` composes storage, media, exporters, background jobs, and route surfaces. `msm-api`, `msm-cli`, `msm-mcp`, and `apps/web` call the same export/job use cases rather than implementing target-specific behavior independently.

## Data Flow

1. A user imports or creates a canonical MSM sticker pack.
2. A user configures an export target such as MoreStickers or Telegram sticker set.
3. The Web UI, API, CLI, or MCP creates an export job with a pack ID, target ID, target options, and acting principal.
4. The worker verifies RBAC/PAT scope, loads the pack and assets, and asks the target to build an export plan.
5. `msm-media` prepares required media outputs and writes them to asset storage or a conversion cache.
6. The exporter publishes the prepared assets or serializes an output artifact.
7. The job stores events, errors, and final publication metadata.

## Telegram Export Target

The Telegram exporter creates a sticker set or custom emoji set owned by a Telegram user through a configured bot token.

Initial behavior:

- Validate the bot token with `getMe` and cache the bot username.
- Generate and validate sticker set names that end with `_by_<bot_username>`.
- Build `InputSticker` values with emoji lists, keywords, optional mask positions, and static/animated/video formats.
- Use `uploadStickerFile` for prepared files that need Telegram-hosted file IDs.
- Use `createNewStickerSet` for the first 1-50 stickers.
- Use `addStickerToSet` for remaining stickers, respecting Telegram's set size limits.
- Record `t.me/addstickers/<set_name>`, sticker count, target type, and target format in the export result.

Initial sync policy is create-only. If the set already exists, MSM returns an actionable conflict error with the existing target name. Replace, append-only, reorder, delete, and emoji-edit sync policies are a later phase because they need remote state reconciliation and sharper UX.

## Data Model

Add storage tables in a later implementation phase:

- `export_targets`: tenant, kind, display name, redacted config, enabled flag, timestamps.
- `export_target_secrets`: secret references or encrypted secrets for bot tokens and future target credentials.
- `export_jobs`: tenant, owner user, source pack, target, status, request JSON, result JSON, error summary, timestamps.
- `export_job_events`: ordered durable progress and diagnostic events.
- `prepared_media_assets`: source asset hash, media profile key, output asset key, MIME type, dimensions, duration, file size, timestamps.
- `telegram_publications`: pack, target, set name, set URL, sticker type, sticker count, last job, timestamps.

## Permissions And Security

Add or reserve these permission keys:

- `export.read`: read export targets, jobs, and publication metadata visible to the principal.
- `export.run`: start export jobs for accessible packs and enabled targets.
- `export.target.manage`: create, update, disable, and delete export target configuration.

Telegram bot tokens must not be returned by API, CLI, MCP, or Web responses. If encrypted DB-backed secrets are not ready when the first Telegram export phase starts, the safe bootstrap mode is environment-configured system targets only. DB-backed tenant targets can follow once secret encryption or an external secret provider exists.

Private pack assets are readable by the server-side worker if the acting principal has pack access. Exporting to Telegram publishes copies into Telegram's infrastructure, so the UI and API must make that privacy boundary explicit before creating a job.

Remote asset download and conversion must validate content type, size, and URL safety before passing files to converters. Converter execution should use bounded timeouts, explicit output paths, and no shell interpolation.

## Web, API, CLI, And MCP Surface

All surfaces should expose the same capabilities:

- List export target kinds and capability metadata.
- Configure or validate Telegram targets, with token redaction.
- Create an export job from a pack to a target.
- Read job progress, events, result URL, and errors.
- Cancel queued or running jobs once worker cancellation is available.

The Web UI should add a target settings page and a pack-level export wizard. The wizard should show target constraints, conversion preview, privacy notice, job progress, and final Telegram link.

## Testing Strategy

- Unit-test media profile selection, sticker set name validation, Telegram DTO serialization, and exporter planning without network or ffmpeg.
- Integration-test Telegram HTTP calls through mocked HTTP responses, including create, append, conflict, bad token, and rate-limit-like errors.
- Repository-test export target, job, event, prepared asset, and publication persistence for SQLite and PostgreSQL-compatible SQL paths.
- API-test RBAC/PAT enforcement, OpenAPI schemas, token redaction, and job creation/status behavior.
- CLI/MCP-test command/tool parity with fake clients.
- Web-test target settings, export wizard, job state rendering, and error display with injected clients.
- End-to-end smoke-test a local ffmpeg conversion fixture and mocked Telegram publication in CI without calling real Telegram.

## Phased Scope

- P24 documents this design and implementation plan.
- P25 builds the media profile, conversion planning, export persistence, exporter registry, MoreStickers target, teloxide bot boundary, and Telegram export planner foundation.
- P26 adds asset internalization and conversion cache storage.
- P27 adds remaining target-neutral exporter execution details.
- P28 adds protected export API/OpenAPI routes.
- P29 adds Telegram publication execution through teloxide and durable worker orchestration.
- P30 adds worker execution, retries, progress events, and publication recording.
- P31 adds CLI and MCP export job parity.
- P32 adds Web target configuration, pack export wizard, and job progress UI.
- P33 adds Telegram sync/update/delete policies after create-only publication is stable.

## Design Review Notes

This spec intentionally treats Telegram as both a possible import provider and an export target, but those roles remain separate modules. MoreStickers-compatible `.stickerpack` import/export stays unchanged. Media conversion is designed as target-neutral infrastructure because future output targets will need the same probe, transform, cache, and validation stages.
