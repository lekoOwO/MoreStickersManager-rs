use msm_domain::{Sticker, StickerPack};
use msm_storage::{
    models::{
        ExportJobStatus, NewExportJob, NewExportJobEvent, NewExportTarget, NewPreparedMediaAsset,
        NewTelegramPublication, NewTelegramStickerMapping, PackVisibility,
    },
    DatabaseConfig, DbPool, StorageRepository,
};

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
