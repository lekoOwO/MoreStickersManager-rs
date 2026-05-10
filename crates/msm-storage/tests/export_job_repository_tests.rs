use msm_domain::{Sticker, StickerPack};
use msm_storage::{
    models::{
        ExportJobStatus, NewExportJob, NewExportJobEvent, NewExportTarget, NewPreparedMediaAsset,
        NewProviderConfig, NewProviderImportJob, NewProviderImportJobEvent, NewTelegramPublication,
        NewTelegramStickerMapping, PackVisibility,
    },
    DatabaseConfig, DbPool, StorageRepository,
};
use std::env;

#[tokio::test]
async fn export_targets_jobs_and_events_roundtrip() {
    let repo = seeded_repo().await;
    let target = repo
        .create_export_target(NewExportTarget {
            id: "target_telegram",
            tenant_id: "tenant_1",
            kind: "telegram.sticker_set",
            name: "Telegram Bot",
            config_json: r#"{"botToken":"<redacted>"}"#,
            is_enabled: true,
        })
        .await
        .unwrap();

    assert_eq!(target.id, "target_telegram");
    assert_eq!(target.kind, "telegram.sticker_set");
    assert!(target.is_enabled);

    let queued = repo
        .create_export_job(NewExportJob {
            id: "job_1",
            tenant_id: "tenant_1",
            owner_user_id: "user_1",
            source_pack_id: "pack_1",
            target_id: "target_telegram",
            request_json: r#"{"mode":"create"}"#,
            max_attempts: 3,
        })
        .await
        .unwrap();
    assert_eq!(queued.status, ExportJobStatus::Queued);
    assert_eq!(queued.attempt_count, 0);
    assert_eq!(queued.max_attempts, 3);
    assert_eq!(queued.next_attempt_at, None);
    assert_eq!(queued.result_json, None);

    let changed = repo
        .update_export_job_status("job_1", ExportJobStatus::Running, None, None)
        .await
        .unwrap();
    assert!(changed);
    assert_eq!(
        repo.find_export_job("job_1").await.unwrap().unwrap().status,
        ExportJobStatus::Running
    );

    repo.append_export_job_event(NewExportJobEvent {
        job_id: "job_1",
        sequence: 2,
        level: "info",
        stage: "upload",
        message: "uploaded sticker",
        metadata_json: r#"{"count":1}"#,
    })
    .await
    .unwrap();
    repo.append_export_job_event(NewExportJobEvent {
        job_id: "job_1",
        sequence: 1,
        level: "info",
        stage: "convert",
        message: "converted sticker",
        metadata_json: "{}",
    })
    .await
    .unwrap();

    let events = repo.list_export_job_events("job_1").await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].sequence, 1);
    assert_eq!(events[1].sequence, 2);
}

#[tokio::test]
async fn postgres_provider_configs_roundtrip_when_configured() {
    let Some(repo) = postgres_repo().await else {
        return;
    };
    let suffix = unique_suffix();
    let tenant_id = format!("tenant_provider_config_{suffix}");
    let provider_config_id = format!("provider_telegram_{suffix}");
    repo.create_tenant(&tenant_id, "Tenant").await.unwrap();

    let created = repo
        .upsert_provider_config(NewProviderConfig {
            id: &provider_config_id,
            tenant_id: &tenant_id,
            provider_id: "telegram",
            name: "Telegram Import Bot",
            config_json: r#"{"botToken":"123456:secret"}"#,
            is_enabled: true,
        })
        .await
        .unwrap();
    assert_eq!(created.provider_id, "telegram");
    assert!(created.is_enabled);

    let updated = repo
        .upsert_provider_config(NewProviderConfig {
            id: &provider_config_id,
            tenant_id: &tenant_id,
            provider_id: "telegram",
            name: "Telegram Import Bot Updated",
            config_json: r#"{"botToken":"456:rotated"}"#,
            is_enabled: false,
        })
        .await
        .unwrap();
    assert_eq!(updated.name, "Telegram Import Bot Updated");
    assert!(!updated.is_enabled);
    assert_eq!(
        repo.list_provider_configs(&tenant_id).await.unwrap(),
        vec![updated.clone()]
    );
    assert_eq!(
        repo.find_provider_config(&provider_config_id)
            .await
            .unwrap(),
        Some(updated)
    );
    assert!(repo
        .delete_provider_config(&provider_config_id)
        .await
        .unwrap());
    assert!(repo
        .find_provider_config(&provider_config_id)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn postgres_provider_import_jobs_events_and_retries_roundtrip_when_configured() {
    let Some(context) = postgres_export_context().await else {
        return;
    };
    let job_id = format!("provider_job_{}", context.suffix);
    let retry_at = "2026-05-08T11:00:00Z";

    let queued = context
        .repo
        .create_provider_import_job(NewProviderImportJob {
            id: &job_id,
            tenant_id: &context.tenant_id,
            owner_user_id: &context.user_id,
            provider_id: "telegram",
            remote_id: "remote-pack",
            target_pack_id: Some(&context.pack_id),
            request_json: r#"{"mode":"import"}"#,
            max_attempts: 3,
        })
        .await
        .unwrap();
    assert_eq!(queued.status, ExportJobStatus::Queued);

    assert!(context
        .repo
        .update_provider_import_job_status(&job_id, ExportJobStatus::Running, None, None)
        .await
        .unwrap());
    assert_eq!(
        context
            .repo
            .find_provider_import_job(&job_id)
            .await
            .unwrap()
            .unwrap()
            .status,
        ExportJobStatus::Running
    );
    context
        .repo
        .append_provider_import_job_event(NewProviderImportJobEvent {
            job_id: &job_id,
            sequence: 1,
            level: "info",
            stage: "fetch",
            message: "fetched remote pack",
            metadata_json: "{}",
        })
        .await
        .unwrap();
    assert_eq!(
        context
            .repo
            .list_provider_import_job_events(&job_id)
            .await
            .unwrap()
            .len(),
        1
    );
    let retry = context
        .repo
        .record_provider_import_job_retry(&job_id, "rate limited", retry_at)
        .await
        .unwrap()
        .expect("provider import job should exist");
    assert_eq!(retry.attempt_count, 1);
    assert_eq!(
        context
            .repo
            .find_next_due_provider_import_job(retry_at)
            .await
            .unwrap()
            .unwrap()
            .id,
        job_id
    );
}

#[tokio::test]
async fn postgres_export_targets_roundtrip_when_configured() {
    let Some(repo) = postgres_repo().await else {
        return;
    };
    let suffix = unique_suffix();
    let tenant_id = format!("tenant_export_target_{suffix}");
    let target_id = format!("target_telegram_{suffix}");

    repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
    let created = repo
        .create_export_target(NewExportTarget {
            id: &target_id,
            tenant_id: &tenant_id,
            kind: "telegram.sticker_set",
            name: "Telegram Bot",
            config_json: r#"{"botToken":"<redacted>"}"#,
            is_enabled: true,
        })
        .await
        .unwrap();
    assert_eq!(created.id, target_id);
    assert!(created.is_enabled);

    let updated = repo
        .update_export_target(
            &target_id,
            "Telegram Bot Updated",
            r#"{"botToken":"<rotated>"}"#,
            false,
        )
        .await
        .unwrap()
        .expect("target should be updated");
    assert_eq!(updated.name, "Telegram Bot Updated");
    assert!(!updated.is_enabled);

    assert_eq!(
        repo.find_export_target(&target_id).await.unwrap(),
        Some(updated.clone())
    );
    assert_eq!(
        repo.list_export_targets(&tenant_id).await.unwrap(),
        vec![updated]
    );
    assert!(repo.delete_export_target(&target_id).await.unwrap());
    assert!(repo.find_export_target(&target_id).await.unwrap().is_none());
}

#[tokio::test]
async fn postgres_export_jobs_events_and_retries_roundtrip_when_configured() {
    let Some(context) = postgres_export_context().await else {
        return;
    };
    let job_id = format!("job_{}", context.suffix);

    let queued = context
        .repo
        .create_export_job(NewExportJob {
            id: &job_id,
            tenant_id: &context.tenant_id,
            owner_user_id: &context.user_id,
            source_pack_id: &context.pack_id,
            target_id: &context.target_id,
            request_json: r#"{"mode":"create"}"#,
            max_attempts: 3,
        })
        .await
        .unwrap();
    assert_eq!(queued.status, ExportJobStatus::Queued);
    assert_eq!(queued.attempt_count, 0);
    assert_eq!(
        context
            .repo
            .find_next_export_job_by_status(ExportJobStatus::Queued)
            .await
            .unwrap()
            .unwrap()
            .id,
        job_id
    );

    assert!(context
        .repo
        .update_export_job_status(&job_id, ExportJobStatus::Running, None, None)
        .await
        .unwrap());
    assert_eq!(
        context
            .repo
            .find_export_job(&job_id)
            .await
            .unwrap()
            .unwrap()
            .status,
        ExportJobStatus::Running
    );

    context
        .repo
        .append_export_job_event(NewExportJobEvent {
            job_id: &job_id,
            sequence: 2,
            level: "info",
            stage: "upload",
            message: "uploaded sticker",
            metadata_json: r#"{"count":1}"#,
        })
        .await
        .unwrap();
    context
        .repo
        .append_export_job_event(NewExportJobEvent {
            job_id: &job_id,
            sequence: 1,
            level: "info",
            stage: "convert",
            message: "converted sticker",
            metadata_json: "{}",
        })
        .await
        .unwrap();
    let events = context.repo.list_export_job_events(&job_id).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].sequence, 1);

    let retry_at = "2026-05-08T10:00:00Z";
    let retry = context
        .repo
        .record_export_job_retry(&job_id, "telegram api down", retry_at)
        .await
        .unwrap()
        .expect("job should exist");
    assert_eq!(retry.status, ExportJobStatus::Queued);
    assert_eq!(retry.attempt_count, 1);
    assert_eq!(retry.next_attempt_at.as_deref(), Some(retry_at));
    assert_eq!(
        context
            .repo
            .find_next_due_export_job(retry_at)
            .await
            .unwrap()
            .unwrap()
            .id,
        job_id
    );
}

#[tokio::test]
async fn export_job_retry_state_tracks_attempts_and_due_time() {
    let repo = seeded_export_job_repo().await;
    let next_attempt_at = "2026-05-08T10:00:00Z";

    let retry = repo
        .record_export_job_retry("job_1", "telegram api down", next_attempt_at)
        .await
        .unwrap()
        .expect("job should exist");

    assert_eq!(retry.status, ExportJobStatus::Queued);
    assert_eq!(retry.attempt_count, 1);
    assert_eq!(retry.max_attempts, 3);
    assert_eq!(retry.error_summary.as_deref(), Some("telegram api down"));
    assert_eq!(retry.next_attempt_at.as_deref(), Some(next_attempt_at));
    assert!(repo
        .find_next_due_export_job("2026-05-08T09:59:59Z")
        .await
        .unwrap()
        .is_none());
    assert_eq!(
        repo.find_next_due_export_job(next_attempt_at)
            .await
            .unwrap()
            .unwrap()
            .id,
        "job_1"
    );
}

#[tokio::test]
async fn export_job_status_records_success_and_failure_payloads() {
    let repo = seeded_export_job_repo().await;

    repo.update_export_job_status(
        "job_1",
        ExportJobStatus::Succeeded,
        None,
        Some(r#"{"url":"https://t.me/addstickers/sample"}"#),
    )
    .await
    .unwrap();
    let succeeded = repo.find_export_job("job_1").await.unwrap().unwrap();
    assert_eq!(succeeded.status, ExportJobStatus::Succeeded);
    assert_eq!(succeeded.next_attempt_at, None);
    assert_eq!(
        succeeded.result_json.as_deref(),
        Some(r#"{"url":"https://t.me/addstickers/sample"}"#)
    );
    assert_eq!(succeeded.error_summary, None);

    repo.update_export_job_status(
        "job_1",
        ExportJobStatus::Failed,
        Some("telegram set already exists"),
        None,
    )
    .await
    .unwrap();
    let failed = repo.find_export_job("job_1").await.unwrap().unwrap();
    assert_eq!(failed.status, ExportJobStatus::Failed);
    assert_eq!(
        failed.error_summary.as_deref(),
        Some("telegram set already exists")
    );
}

#[tokio::test]
async fn prepared_media_assets_upsert_by_source_hash_and_profile_key() {
    let repo = seeded_repo().await;

    let created = repo
        .upsert_prepared_media_asset(NewPreparedMediaAsset {
            source_asset_hash: "sha256:source",
            profile_key: "telegram.sticker.static.v1",
            output_asset_key: "prepared/first.png",
            mime_type: "image/png",
            width_px: 512,
            height_px: 512,
            duration_ms: None,
            file_size_bytes: 1024,
        })
        .await
        .unwrap();
    assert_eq!(created.output_asset_key, "prepared/first.png");

    let updated = repo
        .upsert_prepared_media_asset(NewPreparedMediaAsset {
            source_asset_hash: "sha256:source",
            profile_key: "telegram.sticker.static.v1",
            output_asset_key: "prepared/second.png",
            mime_type: "image/png",
            width_px: 512,
            height_px: 512,
            duration_ms: None,
            file_size_bytes: 2048,
        })
        .await
        .unwrap();

    assert_eq!(updated.output_asset_key, "prepared/second.png");
    assert_eq!(updated.file_size_bytes, 2048);
    assert_eq!(
        repo.find_prepared_media_asset("sha256:source", "telegram.sticker.static.v1")
            .await
            .unwrap()
            .unwrap()
            .output_asset_key,
        "prepared/second.png"
    );
}

#[tokio::test]
async fn postgres_prepared_media_assets_upsert_when_configured() {
    let Some(repo) = postgres_repo().await else {
        return;
    };
    let suffix = unique_suffix();
    let source_hash = format!("sha256:source:{suffix}");
    let profile_key = format!("telegram.sticker.static.v1.{suffix}");

    let created = repo
        .upsert_prepared_media_asset(NewPreparedMediaAsset {
            source_asset_hash: &source_hash,
            profile_key: &profile_key,
            output_asset_key: "prepared/first.png",
            mime_type: "image/png",
            width_px: 512,
            height_px: 512,
            duration_ms: None,
            file_size_bytes: 1024,
        })
        .await
        .unwrap();
    assert_eq!(created.output_asset_key, "prepared/first.png");

    let updated = repo
        .upsert_prepared_media_asset(NewPreparedMediaAsset {
            source_asset_hash: &source_hash,
            profile_key: &profile_key,
            output_asset_key: "prepared/second.png",
            mime_type: "image/png",
            width_px: 512,
            height_px: 512,
            duration_ms: None,
            file_size_bytes: 2048,
        })
        .await
        .unwrap();
    assert_eq!(updated.output_asset_key, "prepared/second.png");
    assert_eq!(updated.file_size_bytes, 2048);
    assert_eq!(
        repo.find_prepared_media_asset(&source_hash, &profile_key)
            .await
            .unwrap(),
        Some(updated)
    );
}

#[tokio::test]
async fn telegram_publications_can_be_found_and_listed() {
    let repo = seeded_export_job_repo().await;

    let publication = repo
        .upsert_telegram_publication(NewTelegramPublication {
            id: "telegram_pub_1",
            pack_id: "pack_1",
            target_id: "target_telegram",
            job_id: "job_1",
            sticker_set_name: "sample_by_msm_bot",
            sticker_set_url: "https://t.me/addstickers/sample_by_msm_bot",
            sticker_count: 1,
            sticker_type: "regular",
        })
        .await
        .unwrap();

    assert_eq!(publication.id, "telegram_pub_1");
    assert_eq!(publication.pack_id, "pack_1");
    assert_eq!(publication.target_id, "target_telegram");
    assert_eq!(publication.job_id, "job_1");
    assert_eq!(publication.sticker_set_name, "sample_by_msm_bot");
    assert_eq!(
        publication.sticker_set_url,
        "https://t.me/addstickers/sample_by_msm_bot"
    );
    assert_eq!(publication.sticker_count, 1);
    assert_eq!(publication.sticker_type, "regular");

    let by_id = repo
        .find_telegram_publication("telegram_pub_1")
        .await
        .unwrap()
        .expect("publication should be found by ID");
    assert_eq!(by_id, publication);

    let by_target_set = repo
        .find_telegram_publication_by_target_set("target_telegram", "sample_by_msm_bot")
        .await
        .unwrap()
        .expect("publication should be found by target and set");
    assert_eq!(by_target_set.id, "telegram_pub_1");

    let list = repo
        .list_telegram_publications_for_pack("pack_1")
        .await
        .unwrap();
    assert_eq!(list, vec![publication]);
}

#[tokio::test]
async fn telegram_publication_upsert_updates_existing_target_set() {
    let repo = seeded_export_job_repo().await;
    repo.upsert_telegram_publication(NewTelegramPublication {
        id: "telegram_pub_1",
        pack_id: "pack_1",
        target_id: "target_telegram",
        job_id: "job_1",
        sticker_set_name: "sample_by_msm_bot",
        sticker_set_url: "https://t.me/addstickers/sample_by_msm_bot",
        sticker_count: 1,
        sticker_type: "regular",
    })
    .await
    .unwrap();
    repo.create_export_job(NewExportJob {
        id: "job_2",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        source_pack_id: "pack_1",
        target_id: "target_telegram",
        request_json: "{}",
        max_attempts: 3,
    })
    .await
    .unwrap();

    let updated = repo
        .upsert_telegram_publication(NewTelegramPublication {
            id: "telegram_pub_replacement",
            pack_id: "pack_1",
            target_id: "target_telegram",
            job_id: "job_2",
            sticker_set_name: "sample_by_msm_bot",
            sticker_set_url: "https://t.me/addstickers/sample_by_msm_bot_v2",
            sticker_count: 2,
            sticker_type: "customEmoji",
        })
        .await
        .unwrap();

    assert_eq!(updated.id, "telegram_pub_1");
    assert_eq!(updated.job_id, "job_2");
    assert_eq!(
        updated.sticker_set_url,
        "https://t.me/addstickers/sample_by_msm_bot_v2"
    );
    assert_eq!(updated.sticker_count, 2);
    assert_eq!(updated.sticker_type, "customEmoji");
    assert_eq!(
        repo.list_telegram_publications_for_pack("pack_1")
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn telegram_sticker_mappings_upsert_and_list_for_publication() {
    let repo = seeded_export_job_repo().await;
    repo.upsert_telegram_publication(NewTelegramPublication {
        id: "telegram_pub_1",
        pack_id: "pack_1",
        target_id: "target_telegram",
        job_id: "job_1",
        sticker_set_name: "sample_by_msm_bot",
        sticker_set_url: "https://t.me/addstickers/sample_by_msm_bot",
        sticker_count: 1,
        sticker_type: "regular",
    })
    .await
    .unwrap();

    let created = repo
        .upsert_telegram_sticker_mapping(NewTelegramStickerMapping {
            publication_id: "telegram_pub_1",
            target_id: "target_telegram",
            sticker_set_name: "sample_by_msm_bot",
            source_sticker_id: "MoreStickers:Telegram:Sticker:sample:file",
            telegram_file_id: "tg_file_1",
            telegram_file_unique_id: "tg_unique_1",
            position: 0,
        })
        .await
        .unwrap();

    assert_eq!(created.publication_id, "telegram_pub_1");
    assert_eq!(
        created.source_sticker_id,
        "MoreStickers:Telegram:Sticker:sample:file"
    );
    assert_eq!(created.telegram_file_id, "tg_file_1");
    assert_eq!(created.telegram_file_unique_id, "tg_unique_1");
    assert_eq!(created.position, 0);

    let updated = repo
        .upsert_telegram_sticker_mapping(NewTelegramStickerMapping {
            publication_id: "telegram_pub_1",
            target_id: "target_telegram",
            sticker_set_name: "sample_by_msm_bot",
            source_sticker_id: "MoreStickers:Telegram:Sticker:sample:file",
            telegram_file_id: "tg_file_2",
            telegram_file_unique_id: "tg_unique_2",
            position: 2,
        })
        .await
        .unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.telegram_file_id, "tg_file_2");
    assert_eq!(updated.telegram_file_unique_id, "tg_unique_2");
    assert_eq!(updated.position, 2);

    let found = repo
        .find_telegram_sticker_mapping_by_source(
            "target_telegram",
            "sample_by_msm_bot",
            "MoreStickers:Telegram:Sticker:sample:file",
        )
        .await
        .unwrap()
        .expect("mapping should be found by source sticker");
    assert_eq!(found, updated);

    let list = repo
        .list_telegram_sticker_mappings_for_publication("telegram_pub_1")
        .await
        .unwrap();
    assert_eq!(list, vec![updated]);
}

#[tokio::test]
async fn postgres_telegram_publications_and_mappings_roundtrip_when_configured() {
    let Some(context) = postgres_export_context().await else {
        return;
    };
    let job_id = format!("job_{}", context.suffix);
    let publication_id = format!("telegram_pub_{}", context.suffix);
    let set_name = format!("sample_{}_by_msm_bot", context.suffix);

    context
        .repo
        .create_export_job(NewExportJob {
            id: &job_id,
            tenant_id: &context.tenant_id,
            owner_user_id: &context.user_id,
            source_pack_id: &context.pack_id,
            target_id: &context.target_id,
            request_json: "{}",
            max_attempts: 3,
        })
        .await
        .unwrap();

    assert_postgres_telegram_publication_roundtrip(&context, &job_id, &publication_id, &set_name)
        .await;
    assert_postgres_telegram_mapping_roundtrip(&context, &publication_id, &set_name).await;
}

async fn assert_postgres_telegram_publication_roundtrip(
    context: &PgExportContext,
    job_id: &str,
    publication_id: &str,
    set_name: &str,
) {
    let publication = context
        .repo
        .upsert_telegram_publication(NewTelegramPublication {
            id: publication_id,
            pack_id: &context.pack_id,
            target_id: &context.target_id,
            job_id,
            sticker_set_name: set_name,
            sticker_set_url: "https://t.me/addstickers/sample_by_msm_bot",
            sticker_count: 1,
            sticker_type: "regular",
        })
        .await
        .unwrap();
    assert_eq!(publication.id, publication_id);
    assert_eq!(
        context
            .repo
            .find_telegram_publication(publication_id)
            .await
            .unwrap(),
        Some(publication.clone())
    );
    assert_eq!(
        context
            .repo
            .find_telegram_publication_by_target_set(&context.target_id, set_name)
            .await
            .unwrap(),
        Some(publication.clone())
    );
    assert_eq!(
        context
            .repo
            .list_telegram_publications_for_pack(&context.pack_id)
            .await
            .unwrap(),
        vec![publication]
    );
}

async fn assert_postgres_telegram_mapping_roundtrip(
    context: &PgExportContext,
    publication_id: &str,
    set_name: &str,
) {
    let created = context
        .repo
        .upsert_telegram_sticker_mapping(NewTelegramStickerMapping {
            publication_id,
            target_id: &context.target_id,
            sticker_set_name: set_name,
            source_sticker_id: "MoreStickers:Telegram:Sticker:sample:file",
            telegram_file_id: "tg_file_1",
            telegram_file_unique_id: "tg_unique_1",
            position: 0,
        })
        .await
        .unwrap();
    let updated = context
        .repo
        .upsert_telegram_sticker_mapping(NewTelegramStickerMapping {
            publication_id,
            target_id: &context.target_id,
            sticker_set_name: set_name,
            source_sticker_id: "MoreStickers:Telegram:Sticker:sample:file",
            telegram_file_id: "tg_file_2",
            telegram_file_unique_id: "tg_unique_2",
            position: 2,
        })
        .await
        .unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.telegram_file_id, "tg_file_2");
    assert_eq!(
        context
            .repo
            .find_telegram_sticker_mapping_by_source(
                &context.target_id,
                set_name,
                "MoreStickers:Telegram:Sticker:sample:file"
            )
            .await
            .unwrap(),
        Some(updated.clone())
    );
    assert_eq!(
        context
            .repo
            .list_telegram_sticker_mappings_for_publication(publication_id)
            .await
            .unwrap(),
        vec![updated]
    );
}

async fn seeded_export_job_repo() -> StorageRepository {
    let repo = seeded_repo().await;
    repo.create_export_target(NewExportTarget {
        id: "target_telegram",
        tenant_id: "tenant_1",
        kind: "telegram.sticker_set",
        name: "Telegram Bot",
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
        target_id: "target_telegram",
        request_json: "{}",
        max_attempts: 3,
    })
    .await
    .unwrap();
    repo
}

async fn seeded_repo() -> StorageRepository {
    let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
    let pool = DbPool::connect(&config).await.unwrap();
    pool.run_migrations().await.unwrap();
    let repo = StorageRepository::new(pool);
    repo.create_tenant("tenant_1", "Tenant").await.unwrap();
    repo.create_user("user_1", "leko@example.com", "Leko")
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

async fn postgres_repo() -> Option<StorageRepository> {
    let url = env::var("MSM_TEST_POSTGRES_URL").ok()?;
    let config = DatabaseConfig::parse(&url).unwrap();
    let pool = DbPool::connect(&config).await.unwrap();
    pool.run_migrations().await.unwrap();
    Some(StorageRepository::new(pool))
}

struct PgExportContext {
    repo: StorageRepository,
    suffix: String,
    tenant_id: String,
    user_id: String,
    pack_id: String,
    target_id: String,
}

async fn postgres_export_context() -> Option<PgExportContext> {
    let repo = postgres_repo().await?;
    let suffix = unique_suffix();
    let tenant_id = format!("tenant_export_job_{suffix}");
    let user_id = format!("user_export_job_{suffix}");
    let pack_id = format!("pack_export_job_{suffix}");
    let target_id = format!("target_export_job_{suffix}");

    repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
    repo.create_user(&user_id, &format!("{user_id}@example.com"), "Leko")
        .await
        .unwrap();
    repo.upsert_sticker_pack(
        &pack_id,
        &tenant_id,
        &user_id,
        PackVisibility::Private,
        Some("telegram"),
        &sample_pack(),
    )
    .await
    .unwrap();
    repo.create_export_target(NewExportTarget {
        id: &target_id,
        tenant_id: &tenant_id,
        kind: "telegram.sticker_set",
        name: "Telegram Bot",
        config_json: "{}",
        is_enabled: true,
    })
    .await
    .unwrap();

    Some(PgExportContext {
        repo,
        suffix,
        tenant_id,
        user_id,
        pack_id,
        target_id,
    })
}

fn unique_suffix() -> String {
    format!(
        "{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
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
