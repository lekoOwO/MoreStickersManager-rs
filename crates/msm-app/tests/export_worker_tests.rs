use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};

use msm_app::{
    ConversionCommandOutput, ConversionCommandRunner, ExportWorker, ExportWorkerConfig,
    ExportWorkerResult, PreparedMediaExecutor, PreparedMediaOutput, PreparedMediaRequest,
    ProcessPreparedMediaExecutor, TelegramMutationExecutor, TelegramMutationRequest,
    TelegramPublicationExecutor, TelegramPublicationRequest, TelegramRemoteStateExecutor,
    TelegramRemoteStateRequest,
};
use msm_domain::{Sticker, StickerPack};
use msm_media::ConversionCommand;
use msm_storage::{
    models::{
        ExportJobStatus, NewExportJob, NewExportTarget, NewPreparedMediaAsset, PackVisibility,
    },
    DatabaseConfig, DbPool, StorageRepository,
};
use msm_telegram::{
    TelegramFetchedSticker, TelegramFetchedStickerSet, TelegramPublishError, TelegramPublishedSet,
};
use sha2::{Digest, Sha256};
use teloxide::types::StickerType;

#[tokio::test]
async fn worker_runs_moresticker_export_job_without_remote_calls() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_morestickers",
        tenant_id: "tenant_1",
        kind: "morestickers",
        name: "MoreStickers",
        config_json: "{}",
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_1",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_morestickers",
        request_json: r#"{"options":{}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let worker = ExportWorker::new(repo.clone(), worker_config());

    let completed = worker.run_next_queued().await.unwrap().unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();
    assert_eq!(result["kind"], "moreStickers");
    assert_eq!(result["targetKind"], "morestickers");
    assert_eq!(result["mimeType"], "application/json");
    assert!(result["byteLen"].as_u64().unwrap() > 0);
    let events = repo.list_export_job_events("job_1").await.unwrap();
    assert_eq!(events[0].stage, "running");
    assert_eq!(events[1].stage, "succeeded");
}

#[tokio::test]
async fn worker_plans_telegram_export_job_without_network_calls() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"😀"}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let worker = ExportWorker::with_media_executor(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
    );

    let completed = worker.run_job("job_telegram").await.unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();
    assert_eq!(result["kind"], "telegramDryRun");
    assert_eq!(result["targetKind"], "telegram");
    assert_eq!(result["stickerSetName"], "sample_pack_by_msm_bot");
    assert_eq!(result["initialStickerCount"], 1);
    assert_eq!(result["appendStickerCount"], 0);
    assert_eq!(result["dryRun"], true);
    assert_eq!(result["mediaProfileKeys"][0], "telegram.sticker.static.v1");
    let cached = repo
        .find_prepared_media_asset(
            result["preparedMedia"][0]["sourceAssetHash"]
                .as_str()
                .unwrap(),
            "telegram.sticker.static.v1",
        )
        .await
        .unwrap()
        .expect("prepared media should be cached");
    assert_eq!(cached.output_asset_key, "prepared/file.png");
    assert_eq!(cached.mime_type, "image/png");
}

#[tokio::test]
async fn worker_reuses_prepared_media_cache_without_executor_call() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_cached_media",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"😀"}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let source_hash = format!(
        "sha256:{:x}",
        Sha256::digest("https://msm.example/assets/packs/sample/file.webp".as_bytes())
    );
    repo.upsert_prepared_media_asset(NewPreparedMediaAsset {
        source_asset_hash: &source_hash,
        profile_key: "telegram.sticker.static.v1",
        output_asset_key: "telegram.sticker.static.v1/cached.png",
        mime_type: "image/png",
        width_px: 512,
        height_px: 512,
        duration_ms: None,
        file_size_bytes: 1234,
    })
    .await
    .unwrap();
    let worker = ExportWorker::with_media_executor(
        repo.clone(),
        worker_config(),
        Arc::new(PanicPreparedMediaExecutor),
    );

    let completed = worker.run_job("job_telegram_cached_media").await.unwrap();
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    assert_eq!(
        result["preparedMedia"][0]["outputAssetKey"],
        "telegram.sticker.static.v1/cached.png"
    );
    assert_eq!(result["preparedMedia"][0]["fileSizeBytes"], 1234);
}

#[tokio::test]
async fn worker_includes_prepared_media_diagnostics_in_job_result() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_media_diagnostics",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"😀"}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let worker = ExportWorker::with_media_executor(
        repo.clone(),
        worker_config(),
        Arc::new(DiagnosticPreparedMediaExecutor),
    );

    let completed = worker
        .run_job("job_telegram_media_diagnostics")
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    assert_eq!(result["preparedMedia"][0]["converterExitCode"], 0);
    assert_eq!(
        result["preparedMedia"][0]["converterStdout"],
        "ffmpeg stdout summary"
    );
    assert_eq!(
        result["preparedMedia"][0]["converterStderr"],
        "frame=1 size=14kB"
    );
}

#[tokio::test]
async fn telegram_dry_run_does_not_call_publication_executor() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_dry_run",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok"}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let publisher = Arc::new(FakeTelegramPublicationExecutor::default());
    let worker = ExportWorker::with_media_telegram_mutation_and_remote_executors(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        publisher.clone(),
        Arc::new(FakeTelegramMutationExecutor::default()),
        Arc::new(FakeTelegramRemoteStateExecutor::with_set(remote_set(
            "sample_pack_by_msm_bot",
            vec![remote_sticker("tg_file_1", "tg_unique_1")],
        ))),
    );

    let completed = worker.run_job("job_telegram_dry_run").await.unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();
    assert_eq!(result["kind"], "telegramDryRun");
    assert!(publisher.calls.lock().unwrap().is_empty());
    assert!(repo
        .list_telegram_publications_for_pack("pack_1")
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn telegram_dry_run_reports_reconciliation_mutation_plan() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_reconcile_dry_run",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","reconcileMode":"appendMissing","remoteSet":{"stickerSetName":"sample_pack_by_msm_bot","title":"Sample","stickers":[]}}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let publisher = Arc::new(FakeTelegramPublicationExecutor::default());
    let worker = ExportWorker::with_media_and_telegram_executors(
        repo,
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        publisher.clone(),
    );

    let completed = worker
        .run_job("job_telegram_reconcile_dry_run")
        .await
        .unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();
    assert_eq!(result["kind"], "telegramDryRun");
    assert_eq!(result["reconciliation"]["mode"], "appendMissing");
    assert_eq!(result["reconciliation"]["operationCount"], 1);
    assert_eq!(result["reconciliation"]["mutationCount"], 1);
    assert_eq!(result["reconciliation"]["operations"][0], "addSticker");
    assert!(publisher.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn telegram_export_job_can_publish_through_injected_executor() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_publish",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let publisher = Arc::new(FakeTelegramPublicationExecutor::default());
    let worker = ExportWorker::with_media_telegram_mutation_and_remote_executors(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        publisher.clone(),
        Arc::new(FakeTelegramMutationExecutor::default()),
        Arc::new(FakeTelegramRemoteStateExecutor::with_set(remote_set(
            "sample_pack_by_msm_bot",
            vec![remote_sticker("tg_file_1", "tg_unique_1")],
        ))),
    );

    let completed = worker.run_job("job_telegram_publish").await.unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();
    assert_eq!(result["kind"], "telegramPublished");
    assert_eq!(result["targetKind"], "telegram");
    assert_eq!(result["stickerSetName"], "sample_pack_by_msm_bot");
    assert_eq!(
        result["stickerSetUrl"],
        "https://t.me/addstickers/sample_pack_by_msm_bot"
    );
    assert_eq!(result["stickerCount"], 1);
    assert_eq!(result["dryRun"], false);

    let publication = repo
        .find_telegram_publication_by_target_set("target_telegram", "sample_pack_by_msm_bot")
        .await
        .unwrap()
        .expect("successful publication should be persisted");
    assert_eq!(publication.pack_id, "pack_1");
    assert_eq!(publication.target_id, "target_telegram");
    assert_eq!(publication.job_id, "job_telegram_publish");
    assert_eq!(publication.sticker_count, 1);
    assert_eq!(publication.sticker_type, "regular");
    assert_eq!(
        publication.sticker_set_url,
        "https://t.me/addstickers/sample_pack_by_msm_bot"
    );

    {
        let calls = publisher.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].bot_token, "123:secret");
        assert_eq!(calls[0].sticker_set_name, "sample_pack_by_msm_bot");
        assert_eq!(calls[0].initial_count, 1);
        assert_eq!(calls[0].append_count, 0);
    }

    let stages = repo
        .list_export_job_events("job_telegram_publish")
        .await
        .unwrap()
        .into_iter()
        .map(|event| event.stage)
        .collect::<Vec<_>>();
    assert!(stages.contains(&"telegram.prepare".to_owned()));
    assert!(stages.contains(&"telegram.publish.create".to_owned()));
    assert!(stages.contains(&"telegram.publish.append".to_owned()));
    assert!(stages.contains(&"succeeded".to_owned()));
}

#[tokio::test]
async fn telegram_publication_fetches_remote_state_and_persists_sticker_mappings() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_mapping",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let remote_state = Arc::new(FakeTelegramRemoteStateExecutor::with_set(remote_set(
        "sample_pack_by_msm_bot",
        vec![remote_sticker("tg_file_1", "tg_unique_1")],
    )));
    let worker = ExportWorker::with_media_telegram_mutation_and_remote_executors(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        Arc::new(FakeTelegramPublicationExecutor::default()),
        Arc::new(FakeTelegramMutationExecutor::default()),
        remote_state.clone(),
    );

    let completed = worker.run_job("job_telegram_mapping").await.unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    assert_eq!(
        remote_state.calls.lock().unwrap().as_slice(),
        ["fetch:123:secret:sample_pack_by_msm_bot"]
    );
    let publication = repo
        .find_telegram_publication_by_target_set("target_telegram", "sample_pack_by_msm_bot")
        .await
        .unwrap()
        .expect("publication should be persisted before mappings");
    let mappings = repo
        .list_telegram_sticker_mappings_for_publication(&publication.id)
        .await
        .unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(
        mappings[0].source_sticker_id,
        "MoreStickers:Telegram:Sticker:sample:file"
    );
    assert_eq!(mappings[0].telegram_file_id, "tg_file_1");
    assert_eq!(mappings[0].telegram_file_unique_id, "tg_unique_1");
    assert_eq!(mappings[0].position, 0);
}

#[tokio::test]
async fn telegram_append_missing_reconciliation_can_execute_mutations_when_explicitly_enabled() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_reconcile_execute",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false,"reconcileMode":"appendMissing","executeReconciliation":true,"remoteSet":{"stickerSetName":"sample_pack_by_msm_bot","title":"Sample","stickers":[]}}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let publisher = Arc::new(FakeTelegramPublicationExecutor::default());
    let mutations = Arc::new(FakeTelegramMutationExecutor::default());
    let worker = ExportWorker::with_media_telegram_mutation_and_remote_executors(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        publisher.clone(),
        mutations.clone(),
        Arc::new(FakeTelegramRemoteStateExecutor::with_set(remote_set(
            "sample_pack_by_msm_bot",
            vec![remote_sticker("tg_file_1", "tg_unique_1")],
        ))),
    );

    let completed = worker
        .run_job("job_telegram_reconcile_execute")
        .await
        .unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();
    assert_eq!(result["kind"], "telegramReconciled");
    assert_eq!(result["stickerSetName"], "sample_pack_by_msm_bot");
    assert_eq!(result["reconciliation"]["mode"], "appendMissing");
    assert_eq!(result["reconciliation"]["mutationCount"], 1);
    assert_eq!(result["dryRun"], false);
    assert!(publisher.calls.lock().unwrap().is_empty());
    let calls = mutations.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].bot_token, "123:secret");
    assert_eq!(calls[0].mutation_count, 1);
    assert_eq!(calls[0].sticker_set_name, "sample_pack_by_msm_bot");
}

#[tokio::test]
async fn telegram_reconciliation_refreshes_remote_state_and_persists_sticker_mappings() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_reconcile_mapping",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false,"reconcileMode":"appendMissing","executeReconciliation":true,"remoteSet":{"stickerSetName":"sample_pack_by_msm_bot","title":"Sample","stickers":[]}}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let remote_state = Arc::new(FakeTelegramRemoteStateExecutor::with_set(remote_set(
        "sample_pack_by_msm_bot",
        vec![remote_sticker(
            "tg_file_after_reconcile",
            "tg_unique_after_reconcile",
        )],
    )));
    let worker = ExportWorker::with_media_telegram_mutation_and_remote_executors(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        Arc::new(FakeTelegramPublicationExecutor::default()),
        Arc::new(FakeTelegramMutationExecutor::default()),
        remote_state.clone(),
    );

    let completed = worker
        .run_job("job_telegram_reconcile_mapping")
        .await
        .unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    assert_eq!(
        remote_state.calls.lock().unwrap().as_slice(),
        ["fetch:123:secret:sample_pack_by_msm_bot"]
    );
    let publication = repo
        .find_telegram_publication_by_target_set("target_telegram", "sample_pack_by_msm_bot")
        .await
        .unwrap()
        .expect("reconciled publication should be persisted before mappings");
    let mappings = repo
        .list_telegram_sticker_mappings_for_publication(&publication.id)
        .await
        .unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(
        mappings[0].source_sticker_id,
        "MoreStickers:Telegram:Sticker:sample:file"
    );
    assert_eq!(mappings[0].telegram_file_id, "tg_file_after_reconcile");
    assert_eq!(
        mappings[0].telegram_file_unique_id,
        "tg_unique_after_reconcile"
    );
}

#[tokio::test]
async fn telegram_reconciliation_can_build_remote_set_from_stored_mappings() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_initial_mapping",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_auto_remote_reconcile",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false,"reconcileMode":"appendMissing","executeReconciliation":true}}"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let remote_state = Arc::new(FakeTelegramRemoteStateExecutor::with_set(remote_set(
        "sample_pack_by_msm_bot",
        vec![remote_sticker("tg_existing_file", "tg_existing_unique")],
    )));
    let mutations = Arc::new(FakeTelegramMutationExecutor::default());
    let worker = ExportWorker::with_media_telegram_mutation_and_remote_executors(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        Arc::new(FakeTelegramPublicationExecutor::default()),
        mutations.clone(),
        remote_state.clone(),
    );

    worker
        .run_job("job_telegram_initial_mapping")
        .await
        .unwrap();
    let completed = worker
        .run_job("job_telegram_auto_remote_reconcile")
        .await
        .unwrap();

    assert_eq!(completed.status, ExportJobStatus::Succeeded);
    let result: serde_json::Value = serde_json::from_str(&completed.result_json.unwrap()).unwrap();
    assert_eq!(result["kind"], "telegramReconciled");
    assert_eq!(result["reconciliation"]["mode"], "appendMissing");
    assert_eq!(result["reconciliation"]["mutationCount"], 0);
    assert!(mutations.calls.lock().unwrap().is_empty());
    assert_eq!(
        remote_state.calls.lock().unwrap().as_slice(),
        [
            "fetch:123:secret:sample_pack_by_msm_bot",
            "fetch:123:secret:sample_pack_by_msm_bot",
        ]
    );
}

#[tokio::test]
async fn telegram_mirror_reconciliation_refuses_destructive_mutations_without_explicit_allowance() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_mirror_blocked",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false,"reconcileMode":"mirror","executeReconciliation":true,"remoteSet":{"stickerSetName":"sample_pack_by_msm_bot","title":"Sample","stickers":[{"stickerId":"remote-only","telegramFileId":"tg_remote_only","targetProfileKey":"telegram.sticker.static.v1","emojiList":["ok"],"keywords":[]}]}}}"#,
        max_attempts: 1,
    })
    .await
    .unwrap();
    let mutations = Arc::new(FakeTelegramMutationExecutor::default());
    let worker = ExportWorker::with_media_telegram_and_mutation_executors(
        repo,
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        Arc::new(FakeTelegramPublicationExecutor::default()),
        mutations.clone(),
    );

    let error = worker
        .run_job("job_telegram_mirror_blocked")
        .await
        .expect_err("mirror delete must require explicit destructive opt-in");

    assert!(error.to_string().contains("allowDestructiveReconciliation"));
    assert!(mutations.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn telegram_publication_failure_marks_job_failed() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_publish_failure",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false}}"#,
        max_attempts: 1,
    })
    .await
    .unwrap();
    let worker = ExportWorker::with_media_and_telegram_executors(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        Arc::new(FakeTelegramPublicationExecutor::failing()),
    );

    let error = worker
        .run_job("job_telegram_publish_failure")
        .await
        .expect_err("failed Telegram publication must fail the job");

    assert!(error.to_string().contains("telegram api down"));
    let stored = repo
        .find_export_job("job_telegram_publish_failure")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, ExportJobStatus::Failed);
    assert!(stored.error_summary.unwrap().contains("telegram api down"));
}

#[tokio::test]
async fn telegram_publication_failure_requeues_until_attempt_budget_is_exhausted() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram",
        name: "Telegram",
        config_json: r#"{"botUsername":"msm_bot","ownerUserId":42,"botToken":"123:secret"}"#,
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_telegram_retry",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: r#"{"options":{"setNameSlug":"Sample Pack","defaultEmoji":"ok","dryRun":false}}"#,
        max_attempts: 2,
    })
    .await
    .unwrap();
    let worker = ExportWorker::with_media_and_telegram_executors(
        repo.clone(),
        worker_config(),
        Arc::new(FakePreparedMediaExecutor),
        Arc::new(FakeTelegramPublicationExecutor::failing()),
    );

    let retry = worker
        .run_job("job_telegram_retry")
        .await
        .expect("first failed attempt should be requeued for retry");

    assert_eq!(retry.status, ExportJobStatus::Queued);
    assert_eq!(retry.attempt_count, 1);
    assert_eq!(retry.max_attempts, 2);
    assert!(retry.next_attempt_at.is_some());
    assert!(retry.error_summary.unwrap().contains("telegram api down"));
    let stages = repo
        .list_export_job_events("job_telegram_retry")
        .await
        .unwrap()
        .into_iter()
        .map(|event| event.stage)
        .collect::<Vec<_>>();
    assert!(stages.contains(&"retry_scheduled".to_owned()));

    let exhausted = worker
        .run_job("job_telegram_retry")
        .await
        .expect_err("second failed attempt should exhaust the retry budget");

    assert!(exhausted.to_string().contains("telegram api down"));
    let stored = repo
        .find_export_job("job_telegram_retry")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, ExportJobStatus::Failed);
    assert_eq!(stored.attempt_count, 2);
}

#[tokio::test]
async fn worker_skips_queued_jobs_until_retry_backoff_is_due() {
    let repo = seeded_repository().await;
    repo.create_export_target(NewExportTarget {
        id: "target_morestickers",
        tenant_id: "tenant_1",
        kind: "morestickers",
        name: "MoreStickers",
        config_json: "{}",
        is_enabled: true,
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_retry_backoff",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_morestickers",
        request_json: r#"{"options":{}}"#,
        max_attempts: 2,
    })
    .await
    .unwrap();
    repo.record_export_job_retry(
        "job_retry_backoff",
        "transient failure",
        "2999-01-01T00:00:00Z",
    )
    .await
    .unwrap();
    let worker = ExportWorker::new(repo, worker_config());

    assert!(worker.run_next_queued().await.unwrap().is_none());
}

#[tokio::test]
async fn process_prepared_media_executor_runs_command_and_returns_output_metadata() {
    let output_dir = tempfile::tempdir().unwrap();
    let executor = ProcessPreparedMediaExecutor::with_runner(
        PathBuf::from("ffmpeg-test"),
        output_dir.path().to_path_buf(),
        Arc::new(WritingCommandRunner),
    );

    let output = executor
        .prepare(PreparedMediaRequest {
            sticker_id: "sticker_1".to_owned(),
            source_uri: "source.webp".to_owned(),
            source_asset_hash: "sha256:source".to_owned(),
            profile_key: "telegram.sticker.static.v1".to_owned(),
            mime_type: "image/png".to_owned(),
            extension: "png".to_owned(),
            width_px: 512,
            height_px: 512,
            duration_ms: None,
        })
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        output.output_asset_key,
        "telegram.sticker.static.v1/sha256_source.png"
    );
    assert_eq!(output.mime_type, "image/png");
    assert_eq!(output.file_size_bytes, 14);
}

#[tokio::test]
async fn process_prepared_media_executor_returns_command_diagnostics() {
    let output_dir = tempfile::tempdir().unwrap();
    let executor = ProcessPreparedMediaExecutor::with_runner(
        PathBuf::from("ffmpeg-test"),
        output_dir.path().to_path_buf(),
        Arc::new(DiagnosticCommandRunner),
    );

    let output = executor
        .prepare(PreparedMediaRequest {
            sticker_id: "sticker_1".to_owned(),
            source_uri: "source.webp".to_owned(),
            source_asset_hash: "sha256:source".to_owned(),
            profile_key: "telegram.sticker.static.v1".to_owned(),
            mime_type: "image/png".to_owned(),
            extension: "png".to_owned(),
            width_px: 512,
            height_px: 512,
            duration_ms: None,
        })
        .await
        .unwrap()
        .unwrap();

    assert_eq!(output.converter_stdout, "ffmpeg stdout summary");
    assert_eq!(output.converter_stderr, "frame=1 size=14kB");
    assert_eq!(output.converter_exit_code, Some(0));
}

async fn seeded_repository() -> StorageRepository {
    let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
    let pool = DbPool::connect(&config).await.unwrap();
    pool.run_migrations().await.unwrap();
    let repo = StorageRepository::new(pool);
    repo.create_tenant("tenant_1", "Tenant").await.unwrap();
    repo.create_user("user_1", "leko@example.com", "Leko")
        .await
        .unwrap();
    repo.add_tenant_member("tenant_1", "user_1", "admin")
        .await
        .unwrap();
    repo.upsert_sticker_pack(
        "pack_1",
        "tenant_1",
        "user_1",
        PackVisibility::Private,
        Some("telegram"),
        &sample_pack(),
    )
    .await
    .unwrap();
    repo
}

fn worker_config() -> ExportWorkerConfig {
    ExportWorkerConfig {
        enabled: false,
        ffmpeg_path: PathBuf::from("ffmpeg-test"),
        ffprobe_path: PathBuf::from("ffprobe-test"),
        prepared_media_dir: PathBuf::from("prepared-test"),
        max_concurrent_jobs: 1,
        poll_interval: std::time::Duration::from_millis(100),
        retry_backoff: std::time::Duration::from_mins(1),
    }
}

fn sample_pack() -> StickerPack {
    let sticker = Sticker {
        id: "MoreStickers:Telegram:Sticker:sample:file".to_owned(),
        image: "https://msm.example/assets/packs/sample/file.webp".to_owned(),
        title: "file".to_owned(),
        sticker_pack_id: "MoreStickers:Telegram:Pack:sample".to_owned(),
        filename: Some("file.webp".to_owned()),
        is_animated: Some(false),
    };

    StickerPack {
        id: "MoreStickers:Telegram:Pack:sample".to_owned(),
        title: "Sample".to_owned(),
        author: None,
        logo: sticker.clone(),
        stickers: vec![sticker],
    }
}

#[derive(Debug, Default)]
struct FakeTelegramPublicationExecutor {
    calls: std::sync::Mutex<Vec<RecordedTelegramPublication>>,
    fail: bool,
}

impl FakeTelegramPublicationExecutor {
    fn failing() -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
            fail: true,
        }
    }
}

impl TelegramPublicationExecutor for FakeTelegramPublicationExecutor {
    fn publish(
        &self,
        request: TelegramPublicationRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<TelegramPublishedSet>> + Send + '_>> {
        Box::pin(async move {
            self.calls
                .lock()
                .unwrap()
                .push(RecordedTelegramPublication {
                    bot_token: request.bot_token.clone(),
                    sticker_set_name: request.publish_request.sticker_set_name.clone(),
                    initial_count: request.publish_request.initial_stickers.len(),
                    append_count: request.publish_request.append_stickers.len(),
                });
            if self.fail {
                Err(TelegramPublishError::Api {
                    message: "telegram api down".to_owned(),
                }
                .into())
            } else {
                Ok(TelegramPublishedSet {
                    sticker_set_name: request.publish_request.sticker_set_name.clone(),
                    title: request.publish_request.title.clone(),
                    url: format!(
                        "https://t.me/addstickers/{}",
                        request.publish_request.sticker_set_name
                    ),
                    sticker_count: request.publish_request.initial_stickers.len()
                        + request.publish_request.append_stickers.len(),
                })
            }
        })
    }
}

#[derive(Debug)]
struct RecordedTelegramPublication {
    bot_token: String,
    sticker_set_name: String,
    initial_count: usize,
    append_count: usize,
}

#[derive(Debug, Default)]
struct FakeTelegramMutationExecutor {
    calls: std::sync::Mutex<Vec<RecordedTelegramMutation>>,
}

impl TelegramMutationExecutor for FakeTelegramMutationExecutor {
    fn apply(
        &self,
        request: TelegramMutationRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<usize>> + Send + '_>> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(RecordedTelegramMutation {
                bot_token: request.bot_token,
                sticker_set_name: request.sticker_set_name,
                mutation_count: request.mutations.len(),
            });
            Ok(1)
        })
    }
}

#[derive(Debug)]
struct RecordedTelegramMutation {
    bot_token: String,
    sticker_set_name: String,
    mutation_count: usize,
}

#[derive(Debug)]
struct FakeTelegramRemoteStateExecutor {
    calls: std::sync::Mutex<Vec<String>>,
    remote_set: TelegramFetchedStickerSet,
}

impl FakeTelegramRemoteStateExecutor {
    fn with_set(remote_set: TelegramFetchedStickerSet) -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
            remote_set,
        }
    }
}

impl TelegramRemoteStateExecutor for FakeTelegramRemoteStateExecutor {
    fn fetch(
        &self,
        request: TelegramRemoteStateRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<TelegramFetchedStickerSet>> + Send + '_>>
    {
        Box::pin(async move {
            self.calls.lock().unwrap().push(format!(
                "fetch:{}:{}",
                request.bot_token, request.sticker_set_name
            ));
            Ok(self.remote_set.clone())
        })
    }
}

#[derive(Debug)]
struct FakePreparedMediaExecutor;

impl PreparedMediaExecutor for FakePreparedMediaExecutor {
    fn prepare(
        &self,
        request: PreparedMediaRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<Option<PreparedMediaOutput>>> + Send + '_>>
    {
        Box::pin(async move {
            Ok(Some(PreparedMediaOutput {
                source_asset_hash: request.source_asset_hash,
                profile_key: request.profile_key,
                output_asset_key: "prepared/file.png".to_owned(),
                mime_type: request.mime_type,
                width_px: request.width_px,
                height_px: request.height_px,
                duration_ms: request.duration_ms,
                file_size_bytes: 512,
                converter_stdout: String::new(),
                converter_stderr: String::new(),
                converter_exit_code: Some(0),
            }))
        })
    }
}

#[derive(Debug)]
struct DiagnosticPreparedMediaExecutor;

impl PreparedMediaExecutor for DiagnosticPreparedMediaExecutor {
    fn prepare(
        &self,
        request: PreparedMediaRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<Option<PreparedMediaOutput>>> + Send + '_>>
    {
        Box::pin(async move {
            Ok(Some(PreparedMediaOutput {
                source_asset_hash: request.source_asset_hash,
                profile_key: request.profile_key,
                output_asset_key: "prepared/diagnostic.png".to_owned(),
                mime_type: request.mime_type,
                width_px: request.width_px,
                height_px: request.height_px,
                duration_ms: request.duration_ms,
                file_size_bytes: 2048,
                converter_stdout: "ffmpeg stdout summary".to_owned(),
                converter_stderr: "frame=1 size=14kB".to_owned(),
                converter_exit_code: Some(0),
            }))
        })
    }
}

#[derive(Debug)]
struct PanicPreparedMediaExecutor;

impl PreparedMediaExecutor for PanicPreparedMediaExecutor {
    fn prepare(
        &self,
        _request: PreparedMediaRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<Option<PreparedMediaOutput>>> + Send + '_>>
    {
        Box::pin(async move { panic!("prepared media executor should not be called on cache hit") })
    }
}

fn remote_set(
    sticker_set_name: &str,
    stickers: Vec<TelegramFetchedSticker>,
) -> TelegramFetchedStickerSet {
    TelegramFetchedStickerSet {
        sticker_set_name: sticker_set_name.to_owned(),
        title: "Sample".to_owned(),
        sticker_type: StickerType::Regular,
        stickers,
    }
}

fn remote_sticker(file_id: &str, unique_id: &str) -> TelegramFetchedSticker {
    TelegramFetchedSticker {
        telegram_file_id: file_id.to_owned(),
        telegram_file_unique_id: unique_id.to_owned(),
        emoji: Some("ok".to_owned()),
        is_animated: false,
        is_video: false,
    }
}

#[derive(Debug)]
struct WritingCommandRunner;

impl ConversionCommandRunner for WritingCommandRunner {
    fn run(
        &self,
        command: ConversionCommand,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<ConversionCommandOutput>> + Send + '_>>
    {
        Box::pin(async move {
            tokio::fs::write(command.output_path(), b"prepared-bytes").await?;
            Ok(ConversionCommandOutput::success(
                String::new(),
                String::new(),
                Some(0),
            ))
        })
    }
}

#[derive(Debug)]
struct DiagnosticCommandRunner;

impl ConversionCommandRunner for DiagnosticCommandRunner {
    fn run(
        &self,
        command: ConversionCommand,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<ConversionCommandOutput>> + Send + '_>>
    {
        Box::pin(async move {
            tokio::fs::write(command.output_path(), b"prepared-bytes").await?;
            Ok(ConversionCommandOutput::success(
                "ffmpeg stdout summary".to_owned(),
                "frame=1 size=14kB".to_owned(),
                Some(0),
            ))
        })
    }
}
