use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};

use msm_app::{
    ExportWorker, ExportWorkerConfig, ExportWorkerResult, PreparedMediaExecutor,
    PreparedMediaOutput, PreparedMediaRequest,
};
use msm_domain::{Sticker, StickerPack};
use msm_storage::{
    models::{ExportJobStatus, NewExportJob, NewExportTarget, PackVisibility},
    DatabaseConfig, DbPool, StorageRepository,
};

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
        max_concurrent_jobs: 1,
        poll_interval: std::time::Duration::from_millis(100),
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
            }))
        })
    }
}
