use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use msm_app::{
    ProviderAssetDownloader, ProviderImportResult, ProviderImportWorker,
    ProviderImportWorkerConfig, ProviderMetadataFetcher,
};
use msm_providers::ProviderRemoteFetchPlan;
use msm_storage::{
    models::{ExportJobStatus, NewProviderImportJob},
    DatabaseConfig, DbPool, LocalAssetStore, StorageRepository,
};

#[tokio::test]
async fn provider_import_worker_internalizes_line_pack_assets() {
    let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
    let pool = DbPool::connect(&config).await.unwrap();
    pool.run_migrations().await.unwrap();
    let repo = StorageRepository::new(pool);
    repo.create_tenant("tenant_1", "Tenant").await.unwrap();
    repo.create_user("user_1", "leko@example.com", "Leko")
        .await
        .unwrap();
    repo.create_provider_import_job(NewProviderImportJob {
        id: "provider_job_1",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        provider_id: "line-stickers",
        remote_id: "line_cats",
        target_pack_id: Some("pack_line_cats"),
        request_json: r#"{
            "providerId": "line-stickers",
            "remoteId": "line_cats",
            "targetPackId": "pack_line_cats",
            "baseUrl": "https://store.line.me"
        }"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let asset_store = LocalAssetStore::new(temp.path());
    let worker = ProviderImportWorker::new(
        repo.clone(),
        asset_store,
        ProviderImportWorkerConfig {
            enabled: false,
            public_asset_base_url: "https://msm.example.test".to_owned(),
            poll_interval: Duration::from_millis(5),
            retry_backoff: Duration::from_millis(5),
        },
        Arc::new(FakeFetcher::new(line_metadata())),
        Arc::new(FakeDownloader::new(BTreeMap::from([
            (
                "https://cdn.example.test/001.png".to_owned(),
                b"one".to_vec(),
            ),
            (
                "https://cdn.example.test/002.apng".to_owned(),
                b"two".to_vec(),
            ),
        ]))),
    );

    let job = worker.run_job("provider_job_1").await.unwrap();
    let pack = repo
        .find_sticker_pack("pack_line_cats")
        .await
        .unwrap()
        .unwrap();
    let events = repo
        .list_provider_import_job_events("provider_job_1")
        .await
        .unwrap();

    assert_eq!(job.status, ExportJobStatus::Succeeded);
    assert_eq!(pack.title, "LINE Cats");
    assert_eq!(pack.stickers.len(), 2);
    assert_eq!(
        pack.stickers[0].image,
        "https://msm.example.test/assets/packs/pack_line_cats/001.png"
    );
    assert_eq!(
        tokio::fs::read(temp.path().join("assets/packs/pack_line_cats/001.png"))
            .await
            .unwrap(),
        b"one"
    );
    assert!(events.iter().any(|event| event.stage == "succeeded"));
}

#[tokio::test]
async fn provider_import_worker_parses_line_product_page_metadata() {
    let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
    let pool = DbPool::connect(&config).await.unwrap();
    pool.run_migrations().await.unwrap();
    let repo = StorageRepository::new(pool);
    repo.create_tenant("tenant_1", "Tenant").await.unwrap();
    repo.create_user("user_1", "leko@example.com", "Leko")
        .await
        .unwrap();
    repo.create_provider_import_job(NewProviderImportJob {
        id: "provider_job_line_page",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        provider_id: "line-stickers",
        remote_id: "12345",
        target_pack_id: Some("pack_line_page"),
        request_json: r#"{
            "providerId": "line-stickers",
            "remoteId": "12345",
            "targetPackId": "pack_line_page",
            "baseUrl": "https://store.line.me"
        }"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let asset_store = LocalAssetStore::new(temp.path());
    let worker = ProviderImportWorker::new(
        repo.clone(),
        asset_store,
        ProviderImportWorkerConfig {
            enabled: false,
            public_asset_base_url: "https://msm.example.test".to_owned(),
            poll_interval: Duration::from_millis(5),
            retry_backoff: Duration::from_millis(5),
        },
        Arc::new(FakeFetcher::new(line_product_page_metadata())),
        Arc::new(FakeDownloader::new(BTreeMap::from([
            (
                "https://stickershop.line-scdn.net/stickershop/v1/sticker/1001/iphone/sticker.png"
                    .to_owned(),
                b"line-page-one".to_vec(),
            ),
            (
                "https://stickershop.line-scdn.net/stickershop/v1/sticker/1002/iPhone/sticker_animation@2x.png"
                    .to_owned(),
                b"line-page-two".to_vec(),
            ),
        ]))),
    );

    let job = worker.run_job("provider_job_line_page").await.unwrap();
    let pack = repo
        .find_sticker_pack("pack_line_page")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(job.status, ExportJobStatus::Succeeded);
    assert_eq!(pack.title, "LINE Cats");
    assert_eq!(
        pack.stickers[1].image,
        "https://msm.example.test/assets/packs/pack_line_page/sticker_animation@2x.png"
    );
    assert_eq!(
        tokio::fs::read(
            temp.path()
                .join("assets/packs/pack_line_page/sticker_animation@2x.png")
        )
        .await
        .unwrap(),
        b"line-page-two"
    );
}

#[tokio::test]
async fn provider_import_worker_requeues_retryable_failures() {
    let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
    let pool = DbPool::connect(&config).await.unwrap();
    pool.run_migrations().await.unwrap();
    let repo = StorageRepository::new(pool);
    repo.create_tenant("tenant_1", "Tenant").await.unwrap();
    repo.create_user("user_1", "leko@example.com", "Leko")
        .await
        .unwrap();
    repo.create_provider_import_job(NewProviderImportJob {
        id: "provider_job_retry",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        provider_id: "telegram",
        remote_id: "cat_pack",
        target_pack_id: Some("pack_cat"),
        request_json: r#"{
            "providerId": "telegram",
            "remoteId": "cat_pack",
            "targetPackId": "pack_cat",
            "baseUrl": "https://api.telegram.org"
        }"#,
        max_attempts: 2,
    })
    .await
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let worker = ProviderImportWorker::new(
        repo.clone(),
        LocalAssetStore::new(temp.path()),
        ProviderImportWorkerConfig {
            enabled: false,
            public_asset_base_url: "https://msm.example.test".to_owned(),
            poll_interval: Duration::from_millis(5),
            retry_backoff: Duration::from_millis(5),
        },
        Arc::new(FakeFetcher::new(br#"{"ok":true}"#.to_vec())),
        Arc::new(FakeDownloader::new(BTreeMap::new())),
    );

    let job = worker.run_job("provider_job_retry").await.unwrap();

    assert_eq!(job.status, ExportJobStatus::Queued);
    assert_eq!(job.attempt_count, 1);
    assert!(job.next_attempt_at.is_some());
    assert!(job
        .error_summary
        .as_deref()
        .unwrap()
        .contains("missing field"));
}

#[tokio::test]
async fn provider_import_worker_resolves_telegram_get_file_assets() {
    let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
    let pool = DbPool::connect(&config).await.unwrap();
    pool.run_migrations().await.unwrap();
    let repo = StorageRepository::new(pool);
    repo.create_tenant("tenant_1", "Tenant").await.unwrap();
    repo.create_user("user_1", "leko@example.com", "Leko")
        .await
        .unwrap();
    repo.create_provider_import_job(NewProviderImportJob {
        id: "provider_job_tg",
        tenant_id: "tenant_1",
        owner_user_id: "user_1",
        provider_id: "telegram",
        remote_id: "cat_pack",
        target_pack_id: Some("pack_cat"),
        request_json: r#"{
            "providerId": "telegram",
            "remoteId": "cat_pack",
            "targetPackId": "pack_cat",
            "baseUrl": "https://api.telegram.org"
        }"#,
        max_attempts: 3,
    })
    .await
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let asset_store = LocalAssetStore::new(temp.path());
    let metadata_url = "https://api.telegram.org/bot<token>/getStickerSet?name=cat_pack";
    let get_file_url = "https://api.telegram.org/bot<token>/getFile?file_id=file_1";
    let worker = ProviderImportWorker::new(
        repo.clone(),
        asset_store,
        ProviderImportWorkerConfig {
            enabled: false,
            public_asset_base_url: "https://msm.example.test".to_owned(),
            poll_interval: Duration::from_millis(5),
            retry_backoff: Duration::from_millis(5),
        },
        Arc::new(FakeFetcher::new_map(BTreeMap::from([
            (metadata_url.to_owned(), telegram_metadata()),
            (
                get_file_url.to_owned(),
                br#"{"ok":true,"result":{"file_path":"stickers/file_1.webp"}}"#.to_vec(),
            ),
        ]))),
        Arc::new(FakeDownloader::new(BTreeMap::from([(
            "https://api.telegram.org/file/bot<token>/stickers/file_1.webp".to_owned(),
            b"telegram-webp".to_vec(),
        )]))),
    );

    let job = worker.run_job("provider_job_tg").await.unwrap();
    let pack = repo.find_sticker_pack("pack_cat").await.unwrap().unwrap();

    assert_eq!(job.status, ExportJobStatus::Succeeded);
    assert_eq!(pack.title, "Cat Pack");
    assert_eq!(
        pack.stickers[0].image,
        "https://msm.example.test/assets/packs/pack_cat/cat_unique_1.webp"
    );
    assert_eq!(
        tokio::fs::read(temp.path().join("assets/packs/pack_cat/cat_unique_1.webp"))
            .await
            .unwrap(),
        b"telegram-webp"
    );
}

struct FakeFetcher {
    metadata: Vec<u8>,
    metadata_by_url: BTreeMap<String, Vec<u8>>,
    seen_urls: Arc<Mutex<Vec<String>>>,
}

impl FakeFetcher {
    fn new(metadata: Vec<u8>) -> Self {
        Self {
            metadata,
            metadata_by_url: BTreeMap::new(),
            seen_urls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn new_map(metadata_by_url: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            metadata: Vec::new(),
            metadata_by_url,
            seen_urls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ProviderMetadataFetcher for FakeFetcher {
    fn fetch_metadata<'a>(
        &'a self,
        plan: &'a ProviderRemoteFetchPlan,
    ) -> Pin<Box<dyn Future<Output = ProviderImportResult<Vec<u8>>> + Send + 'a>> {
        Box::pin(async move {
            self.seen_urls
                .lock()
                .unwrap()
                .push(plan.metadata_request.url.clone());
            Ok(self
                .metadata_by_url
                .get(&plan.metadata_request.url)
                .cloned()
                .unwrap_or_else(|| self.metadata.clone()))
        })
    }
}

struct FakeDownloader {
    bytes: BTreeMap<String, Vec<u8>>,
}

impl FakeDownloader {
    fn new(bytes: BTreeMap<String, Vec<u8>>) -> Self {
        Self { bytes }
    }
}

impl ProviderAssetDownloader for FakeDownloader {
    fn download_asset<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = ProviderImportResult<Vec<u8>>> + Send + 'a>> {
        Box::pin(async move { Ok(self.bytes.get(url).cloned().unwrap_or_default()) })
    }
}

fn line_metadata() -> Vec<u8> {
    br#"{
        "id": "line_cats",
        "title": "LINE Cats",
        "author": { "name": "LINE" },
        "stickers": [
            {
                "id": "001",
                "title": "Wave",
                "staticUrl": "https://cdn.example.test/001.png"
            },
            {
                "id": "002",
                "title": "Dance",
                "staticUrl": "https://cdn.example.test/002.png",
                "animationUrl": "https://cdn.example.test/002.apng"
            }
        ]
    }"#
    .to_vec()
}

fn telegram_metadata() -> Vec<u8> {
    br#"{
        "ok": true,
        "result": {
            "name": "cat_pack",
            "title": "Cat Pack",
            "stickers": [
                {
                    "fileId": "file_1",
                    "fileUniqueId": "cat_unique_1",
                    "emoji": "cat",
                    "isAnimated": false
                }
            ]
        }
    }"#
    .to_vec()
}

fn line_product_page_metadata() -> Vec<u8> {
    br#"
        <!doctype html>
        <script id="__NEXT_DATA__" type="application/json">
          {
            "props": {
              "pageProps": {
                "product": {
                  "id": "12345",
                  "title": "LINE Cats",
                  "author": { "name": "LINE Creators" },
                  "mainImage": "https://stickershop.line-scdn.net/stickershop/v1/product/12345/LINEStorePC/main.png",
                  "stickers": [
                    {
                      "id": "1001",
                      "title": "Wave",
                      "staticUrl": "https://stickershop.line-scdn.net/stickershop/v1/sticker/1001/iphone/sticker.png"
                    },
                    {
                      "id": "1002",
                      "staticUrl": "https://stickershop.line-scdn.net/stickershop/v1/sticker/1002/iphone/sticker.png",
                      "animationUrl": "https://stickershop.line-scdn.net/stickershop/v1/sticker/1002/iPhone/sticker_animation@2x.png"
                    }
                  ]
                }
              }
            }
          }
        </script>
    "#
    .to_vec()
}
