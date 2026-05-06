use msm_domain::{Sticker, StickerPack};
use msm_storage::{
    models::{
        ExportJobStatus, NewExportJob, NewExportJobEvent, NewExportTarget, NewPreparedMediaAsset,
        PackVisibility,
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
        })
        .await
        .unwrap();
    assert_eq!(queued.status, ExportJobStatus::Queued);
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
