# Backup and Restore Runbook

This runbook covers production backup/restore for MSM instances. It is intentionally operational rather than migration-only: the portable user export/import API is useful for tenant/user moves, but a complete instance backup must include the database, local assets, and deployment secrets/configuration together.

## What must be backed up

| Component | Why it matters | Backup source |
| --- | --- | --- |
| Database | Tenants, users, PAT hashes, OIDC state/config, pack metadata, provider configs, subscription links, export/import jobs, Telegram mappings. | `MSM_DATABASE_URL` backend. |
| Asset directory | Self-hosted sticker assets and imported provider media. | `MSM_ASSET_DIR`, default `data/assets`. |
| Prepared media cache | Optional but useful to avoid reconversion after restore. | `MSM_PREPARED_MEDIA_DIR`, default `data/prepared-media`. |
| Runtime config/secrets | Bot tokens, OIDC secrets, public/CDN URL choices, bootstrap targets, worker settings. | Deployment environment, secret manager, `.env.*` files if used. |
| Web dist / container image | Usually reproducible from Git/release image, but pin the release version used for restore. | Release artifact, image tag, or build provenance. |

Never back up only the database when using the default local asset store: sticker pack JSON points at MSM-hosted asset paths, and those paths need the asset files to remain valid.

## Backup cadence

- Small personal instance: daily database + asset snapshot, keep at least 7 daily and 4 weekly restore points.
- Team/multi-tenant instance: hourly or continuous database backup plus daily asset snapshots; keep retention aligned with incident response and compliance needs.
- Always take an on-demand backup before changing `MSM_DATABASE_URL`, `MSM_ASSET_DIR`, CDN settings, migrations, or worker/export configuration.

## SQLite backup

SQLite is safe for small deployments, but copy it carefully because WAL files may be active.

Preferred online backup from the same machine:

```bash
sqlite3 data/msm.sqlite3 ".backup 'backups/msm-$(date +%Y%m%d-%H%M%S).sqlite3'"
```

Windows PowerShell example:

```powershell
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
sqlite3 data/msm.sqlite3 ".backup 'backups/msm-$stamp.sqlite3'"
```

If `sqlite3` is unavailable, stop `msm-app` first, then copy the database file and any `-wal`/`-shm` files as one set.

## PostgreSQL backup

Use a custom-format dump so restore can choose ownership/cleaning options:

```bash
pg_dump --format=custom --file backups/msm-$(date +%Y%m%d-%H%M%S).dump "$MSM_DATABASE_URL"
```

For managed PostgreSQL, prefer provider-native PITR/snapshots and keep at least one periodically tested `pg_dump` path as a vendor-neutral fallback.

## Asset and prepared-media backup

Back up the asset directory after or at the same restore point as the database. For local directories:

```bash
tar -C data -czf backups/msm-assets-$(date +%Y%m%d-%H%M%S).tar.gz assets prepared-media
```

Windows PowerShell example:

```powershell
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
Compress-Archive -Path data\assets,data\prepared-media -DestinationPath "backups\msm-assets-$stamp.zip"
```

If you use Cloudflare or another CDN in front of MSM, remember that `MSM_PUBLIC_ASSET_URL` and tenant `publicAssetUrl` are URL rewrite settings, not asset storage by themselves. The origin files still need to be backed up unless your CDN is backed by a durable object store that you also snapshot.

## Restore: SQLite

1. Stop `msm-app` and workers.
2. Move the damaged/current database and asset directories aside; do not overwrite until the restore is verified.
3. Restore the SQLite backup to the path used by `MSM_DATABASE_URL`.
4. Restore `assets` and optionally `prepared-media` to the paths configured by `MSM_ASSET_DIR` and `MSM_PREPARED_MEDIA_DIR`.
5. Restore environment variables/secrets, especially OIDC/client secrets, Telegram bot tokens, CDN/public asset URL settings, and bootstrap target config.
6. Start `msm-app`; it will run migrations for the restored database if the binary is newer.
7. Verify `/readyz`, Web login, pack export, private asset access, subscription links, provider import credentials redaction, and Telegram publication history for a sample tenant.

## Restore: PostgreSQL

1. Stop `msm-app` and workers or point them at maintenance mode.
2. Create a clean target database/user.
3. Restore:

```bash
pg_restore --clean --if-exists --no-owner --dbname "$MSM_DATABASE_URL" backups/msm-YYYYMMDD-HHMMSS.dump
```

4. Restore `assets` and optionally `prepared-media` to the configured directories or object-store mount.
5. Restore environment variables/secrets and start the same or newer MSM binary.
6. Verify `/readyz`, critical API/Web flows, and background workers.

## Cross-instance migration vs full restore

Use portable user export/import when moving selected users or tenants between live MSM instances:

```powershell
cargo run -p msm-cli -- portability export --user-id user_1 --output user_1.msm-export.json
cargo run -p msm-cli -- portability import --tenant-id tenant_2 --input user_1.msm-export.json
```

Portable export/import does not replace a full disaster-recovery backup: it intentionally operates at user-data scope and does not capture all instance-level secrets, worker state, every tenant setting, or every local asset-storage concern.

## Restore verification checklist

- `GET /readyz` returns `200` and both `database` and `assetStore` components are `ok`.
- Web local/OIDC login works for a test user.
- PAT-protected CLI/API calls work with expected scopes.
- A public pack export returns MoreStickers-compatible JSON.
- A private pack asset rejects anonymous access and accepts owner/session/subscription credentials as expected.
- Subscription-group links still include intended packs and exclude private packs from anonymous public reads.
- Provider config list responses redact tokens/secrets.
- Telegram publication records and source-sticker mappings are present for previously published packs.
- CDN/public asset URL settings point at the restored instance or intended CDN origin.

## Backup integrity drills

At least monthly, restore the latest backup into an isolated staging instance, run `/readyz`, import/export a sample pack, and compare a sample user's portable export with production expectations. Record the restore timestamp, backup artifact IDs, MSM version, and verification result in your operations log.
