# Telegram Reconciliation Runbook

This runbook is for operators who publish MSM sticker packs into Telegram
sticker sets and need to keep an existing Telegram set synchronized.

## Modes

- `createOnly`: create a missing set, reject existing remote sets.
- `appendMissing`: add MSM stickers that are missing remotely and leave
  remote-only stickers untouched.
- `mirror`: make the remote Telegram set match MSM. This can replace changed
  stickers and delete remote-only stickers.

Use `appendMissing` for normal maintenance. Use `mirror` only when the MSM pack
is the authoritative source and remote-only Telegram stickers should be removed.

## Safe Operator Flow

1. Run a dry-run reconciliation first.
2. Review the job result, especially operation counts and mutation counts.
3. Confirm the target sticker set name is the intended Telegram set.
4. Confirm the MSM pack is the source of truth for that set.
5. Execute `appendMissing` if only additions are expected.
6. Execute `mirror` only after reviewing replace/delete operations.

Dry-run is the default. Live Telegram mutation requires `dryRun:false`.
Reconciliation mutation additionally requires `executeReconciliation:true`.
Mirror replacement/deletion additionally requires
`allowDestructiveReconciliation:true`.

## Append-Missing Example

CLI:

```powershell
cargo run -p msm-cli -- exports jobs create `
  --id job_tg_append `
  --tenant-id tenant_1 `
  --source-pack-id pack_1 `
  --target-id target_telegram `
  --telegram-live `
  --telegram-reconcile-mode append-missing `
  --execute-reconciliation `
  --telegram-set-name-slug sample `
  --telegram-default-emoji ok
```

API options:

```json
{
  "dryRun": false,
  "reconcileMode": "appendMissing",
  "executeReconciliation": true,
  "setNameSlug": "sample",
  "defaultEmoji": "ok"
}
```

## Guarded Mirror Example

CLI:

```powershell
cargo run -p msm-cli -- exports jobs create `
  --id job_tg_mirror `
  --tenant-id tenant_1 `
  --source-pack-id pack_1 `
  --target-id target_telegram `
  --telegram-live `
  --telegram-reconcile-mode mirror `
  --execute-reconciliation `
  --allow-destructive-reconciliation `
  --telegram-set-name-slug sample `
  --telegram-default-emoji ok
```

API options:

```json
{
  "dryRun": false,
  "reconcileMode": "mirror",
  "executeReconciliation": true,
  "allowDestructiveReconciliation": true,
  "setNameSlug": "sample",
  "defaultEmoji": "ok"
}
```

## Review Checklist

- The selected export target is the intended Telegram bot target.
- The target config contains the expected `botUsername`, `botToken`, and
  `ownerUserId`.
- The job result references the expected Telegram sticker set URL.
- Dry-run output has no unexpected `delete` or `replace` operations.
- Mirror jobs include `allowDestructiveReconciliation:true` only after review.
- The operator keeps the previous Telegram publication record so MSM can map
  source sticker IDs to Telegram file IDs during future reconciliation.

## Recovery Notes

MSM records successful Telegram publications and refreshes sticker mappings
after publication and reconciliation mutation jobs. If a live mirror operation
removes a sticker unexpectedly, restore the MSM pack contents if needed and run
another live publication or reconciliation job. If the remote Telegram set was
manually changed outside MSM, run dry-run reconciliation before the next live
job so operation summaries expose the drift.
