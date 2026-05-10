use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use msm_domain::StickerPack;
use msm_providers::{
    line::LineStickerProvider, line_sticker_pack_fetch_plan, telegram::TelegramProvider,
    telegram_sticker_set_fetch_plan, ProviderAssetDownloadStrategy, ProviderHttpRequestPlan,
    ProviderRemoteFetchPlan, StickerProvider,
};
use msm_storage::{
    models::{
        ExportJobStatus, NewProviderImportJobEvent, PackVisibility, ProviderConfigRecord,
        ProviderImportJobRecord,
    },
    LocalAssetStore, StorageRepository,
};
use serde::{Deserialize, Serialize};

use crate::{
    internalize_direct_remote_pack_assets, ProviderAssetDownloader, ProviderImportError,
    ProviderImportResult, ProviderMetadataFetcher,
};

/// Result type for provider import worker operations.
pub type ProviderImportWorkerResult<T> = Result<T, ProviderImportWorkerError>;

/// Provider import worker errors.
#[derive(Debug, thiserror::Error)]
pub enum ProviderImportWorkerError {
    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(#[from] msm_storage::StorageError),

    /// Provider runtime helper failed.
    #[error("provider import runtime error: {0}")]
    Runtime(#[from] ProviderImportError),

    /// Provider normalization failed.
    #[error("provider normalization error: {0}")]
    Provider(#[from] msm_providers::ProviderError),

    /// JSON serialization or parsing failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Provider is not implemented by the import worker yet.
    #[error("unsupported provider import worker source: {provider_id}")]
    UnsupportedProvider {
        /// Provider ID.
        provider_id: String,
    },

    /// Asset strategy is not executable by the worker yet.
    #[error("unsupported provider import asset strategy: {strategy:?}")]
    UnsupportedAssetStrategy {
        /// Unsupported strategy.
        strategy: ProviderAssetDownloadStrategy,
    },

    /// Job was not found.
    #[error("provider import job not found: {job_id}")]
    JobNotFound {
        /// Job ID.
        job_id: String,
    },

    /// Retry backoff cannot be represented as a timestamp.
    #[error("provider import retry backoff overflow")]
    RetryBackoffOverflow,
}

/// Provider import worker configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderImportWorkerConfig {
    /// Whether the service should spawn a background import worker loop.
    pub enabled: bool,
    /// Public base URL embedded into self-hosted asset URLs.
    pub public_asset_base_url: String,
    /// Poll interval for the background worker loop.
    pub poll_interval: Duration,
    /// Retry backoff for retryable failures.
    pub retry_backoff: Duration,
}

/// HTTP-backed provider runtime used by the service loop.
#[derive(Clone, Debug)]
pub struct ReqwestProviderImportRuntime {
    client: reqwest::Client,
}

impl ReqwestProviderImportRuntime {
    /// Creates a runtime using a default reqwest client.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for ReqwestProviderImportRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderMetadataFetcher for ReqwestProviderImportRuntime {
    fn fetch_metadata<'a>(
        &'a self,
        plan: &'a ProviderRemoteFetchPlan,
    ) -> Pin<Box<dyn Future<Output = ProviderImportResult<Vec<u8>>> + Send + 'a>> {
        Box::pin(async move {
            if !plan.metadata_request.method.eq_ignore_ascii_case("GET") {
                return Err(ProviderImportError::Fetch(format!(
                    "unsupported metadata request method: {}",
                    plan.metadata_request.method
                )));
            }
            let response = self
                .client
                .get(&plan.metadata_request.url)
                .send()
                .await
                .map_err(|error| ProviderImportError::Fetch(error.to_string()))?
                .error_for_status()
                .map_err(|error| ProviderImportError::Fetch(error.to_string()))?;
            let bytes = response
                .bytes()
                .await
                .map_err(|error| ProviderImportError::Fetch(error.to_string()))?;
            Ok(bytes.to_vec())
        })
    }
}

impl ProviderAssetDownloader for ReqwestProviderImportRuntime {
    fn download_asset<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = ProviderImportResult<Vec<u8>>> + Send + 'a>> {
        Box::pin(async move {
            let response = self
                .client
                .get(url)
                .send()
                .await
                .map_err(|error| ProviderImportError::AssetDownload(error.to_string()))?
                .error_for_status()
                .map_err(|error| ProviderImportError::AssetDownload(error.to_string()))?;
            let bytes = response
                .bytes()
                .await
                .map_err(|error| ProviderImportError::AssetDownload(error.to_string()))?;
            Ok(bytes.to_vec())
        })
    }
}

/// Spawns the provider import worker loop when enabled by configuration.
#[must_use]
pub fn spawn_provider_import_worker_if_enabled(
    repository: StorageRepository,
    asset_store: LocalAssetStore,
    config: ProviderImportWorkerConfig,
) -> Option<tokio::task::JoinHandle<()>> {
    if !config.enabled {
        return None;
    }

    Some(tokio::spawn(async move {
        let poll_interval = config.poll_interval;
        let runtime = Arc::new(ReqwestProviderImportRuntime::new());
        let worker =
            ProviderImportWorker::new(repository, asset_store, config, runtime.clone(), runtime);
        let mut interval = tokio::time::interval(poll_interval);
        loop {
            interval.tick().await;
            if worker.run_next_queued().await.is_err() {
                interval.tick().await;
            }
        }
    }))
}

/// Single-job provider import worker.
#[derive(Clone)]
pub struct ProviderImportWorker {
    repository: StorageRepository,
    asset_store: LocalAssetStore,
    config: ProviderImportWorkerConfig,
    metadata_fetcher: Arc<dyn ProviderMetadataFetcher + Send + Sync>,
    asset_downloader: Arc<dyn ProviderAssetDownloader + Send + Sync>,
}

impl ProviderImportWorker {
    /// Creates a worker with injected provider runtimes.
    #[must_use]
    pub fn new(
        repository: StorageRepository,
        asset_store: LocalAssetStore,
        config: ProviderImportWorkerConfig,
        metadata_fetcher: Arc<dyn ProviderMetadataFetcher + Send + Sync>,
        asset_downloader: Arc<dyn ProviderAssetDownloader + Send + Sync>,
    ) -> Self {
        Self {
            repository,
            asset_store,
            config,
            metadata_fetcher,
            asset_downloader,
        }
    }

    /// Runs the oldest queued provider import job if one is due.
    ///
    /// # Errors
    ///
    /// Returns an error when storage, provider fetch, normalization, or asset
    /// internalization fails.
    pub async fn run_next_queued(
        &self,
    ) -> ProviderImportWorkerResult<Option<ProviderImportJobRecord>> {
        let Some(job) = self
            .repository
            .find_next_due_provider_import_job(&chrono::Utc::now().to_rfc3339())
            .await?
        else {
            return Ok(None);
        };

        self.run_job(&job.id).await.map(Some)
    }

    /// Runs one provider import job by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the job is missing or execution fails.
    pub async fn run_job(
        &self,
        job_id: &str,
    ) -> ProviderImportWorkerResult<ProviderImportJobRecord> {
        let Some(job) = self.repository.find_provider_import_job(job_id).await? else {
            return Err(ProviderImportWorkerError::JobNotFound {
                job_id: job_id.to_owned(),
            });
        };
        self.repository
            .update_provider_import_job_status(&job.id, ExportJobStatus::Running, None, None)
            .await?;
        self.append_event(&job.id, "info", "running", "provider import job started")
            .await?;

        match self.execute_job(&job).await {
            Ok(result) => {
                let result_json = serde_json::to_string(&result)?;
                self.repository
                    .update_provider_import_job_status(
                        &job.id,
                        ExportJobStatus::Succeeded,
                        None,
                        Some(&result_json),
                    )
                    .await?;
                self.append_event(
                    &job.id,
                    "info",
                    "succeeded",
                    "provider import job succeeded",
                )
                .await?;
            }
            Err(error) => {
                let message = error.to_string();
                let failed_attempt_count = job.attempt_count + 1;
                if failed_attempt_count < job.max_attempts {
                    let next_attempt_at = next_attempt_at(self.config.retry_backoff)?;
                    self.repository
                        .record_provider_import_job_retry(&job.id, &message, &next_attempt_at)
                        .await?;
                    self.append_event(&job.id, "warn", "retry_scheduled", &message)
                        .await?;
                    return self
                        .repository
                        .find_provider_import_job(&job.id)
                        .await?
                        .ok_or_else(|| ProviderImportWorkerError::JobNotFound {
                            job_id: job.id.clone(),
                        });
                }

                self.repository
                    .record_provider_import_job_failure(&job.id, &message)
                    .await?;
                self.append_event(&job.id, "error", "failed", &message)
                    .await?;
                return Err(error);
            }
        }

        self.repository
            .find_provider_import_job(&job.id)
            .await?
            .ok_or_else(|| ProviderImportWorkerError::JobNotFound {
                job_id: job.id.clone(),
            })
    }

    async fn execute_job(
        &self,
        job: &ProviderImportJobRecord,
    ) -> ProviderImportWorkerResult<ProviderImportWorkerJobResult> {
        let request: WorkerProviderImportJobRequest = serde_json::from_str(&job.request_json)?;
        let provider_settings = self
            .enabled_provider_settings(&job.tenant_id, &request.provider_id)
            .await?;
        let base_url = request
            .base_url
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned)
            .or_else(|| provider_settings.api_base_url.clone());
        let mut plan = provider_import_plan(
            &request.provider_id,
            &request.remote_id,
            base_url.as_deref(),
        )?;
        apply_provider_settings(&mut plan, &provider_settings);
        let metadata_bytes = self.metadata_fetcher.fetch_metadata(&plan).await?;
        let metadata = String::from_utf8_lossy(&metadata_bytes);
        let telegram_files = if request.provider_id == "telegram" {
            resolve_telegram_files(&plan, &metadata, self.metadata_fetcher.as_ref()).await?
        } else {
            Vec::new()
        };
        let pack = normalize_pack(
            &request.provider_id,
            &metadata,
            &self.config.public_asset_base_url,
            &telegram_files,
        )?;
        let target_pack_id = request
            .target_pack_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&job.id);
        let internalized = internalize_assets(
            &pack,
            target_pack_id,
            &self.config.public_asset_base_url,
            plan,
            &telegram_files,
            self.asset_downloader.as_ref(),
            &self.asset_store,
        )
        .await?;

        self.repository
            .upsert_sticker_pack(
                target_pack_id,
                &job.tenant_id,
                &job.owner_user_id,
                PackVisibility::Private,
                Some(&request.provider_id),
                &internalized,
            )
            .await?;

        Ok(ProviderImportWorkerJobResult {
            pack_id: target_pack_id.to_owned(),
            provider_id: request.provider_id,
            remote_id: request.remote_id,
            sticker_count: internalized.stickers.len(),
        })
    }

    async fn enabled_provider_settings(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> ProviderImportWorkerResult<ProviderCredentialSettings> {
        let configs = self.repository.list_provider_configs(tenant_id).await?;
        let Some(config) = configs
            .iter()
            .find(|config| config.provider_id == provider_id && config.is_enabled)
        else {
            return Ok(ProviderCredentialSettings::default());
        };
        provider_credential_settings(config)
    }

    async fn append_event(
        &self,
        job_id: &str,
        level: &str,
        stage: &str,
        message: &str,
    ) -> ProviderImportWorkerResult<()> {
        let event_count = self
            .repository
            .list_provider_import_job_events(job_id)
            .await?
            .len();
        let sequence = i64::try_from(event_count).unwrap_or(i64::MAX - 1) + 1;
        self.repository
            .append_provider_import_job_event(NewProviderImportJobEvent {
                job_id,
                sequence,
                level,
                stage,
                message,
                metadata_json: "{}",
            })
            .await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerProviderImportJobRequest {
    provider_id: String,
    remote_id: String,
    target_pack_id: Option<String>,
    base_url: Option<String>,
}

#[derive(Debug, Default)]
struct ProviderCredentialSettings {
    api_base_url: Option<String>,
    bot_token: Option<String>,
}

fn provider_credential_settings(
    config: &ProviderConfigRecord,
) -> ProviderImportWorkerResult<ProviderCredentialSettings> {
    let value: serde_json::Value = serde_json::from_str(&config.config_json)?;
    Ok(ProviderCredentialSettings {
        api_base_url: config_string(
            &value,
            &["apiBaseUrl", "api_base_url", "baseUrl", "base_url"],
        ),
        bot_token: config_string(&value, &["botToken", "bot_token", "token"]),
    })
}

fn config_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| value.get(*key))
        .filter_map(serde_json::Value::as_str)
        .map(str::trim)
        .find(|value| !value.is_empty())
        .map(str::to_owned)
}

fn apply_provider_settings(
    plan: &mut ProviderRemoteFetchPlan,
    settings: &ProviderCredentialSettings,
) {
    if plan.provider_id == "telegram" {
        if let Some(bot_token) = settings.bot_token.as_deref() {
            plan.metadata_request.url = plan.metadata_request.url.replace("<token>", bot_token);
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderImportWorkerJobResult {
    pack_id: String,
    provider_id: String,
    remote_id: String,
    sticker_count: usize,
}

fn provider_import_plan(
    provider_id: &str,
    remote_id: &str,
    base_url: Option<&str>,
) -> ProviderImportWorkerResult<ProviderRemoteFetchPlan> {
    match provider_id {
        "telegram" => Ok(telegram_sticker_set_fetch_plan(
            base_url.unwrap_or("https://api.telegram.org"),
            remote_id,
        )?),
        "line-stickers" => Ok(line_sticker_pack_fetch_plan(
            base_url.unwrap_or("https://store.line.me"),
            remote_id,
        )?),
        other => Err(ProviderImportWorkerError::UnsupportedProvider {
            provider_id: other.to_owned(),
        }),
    }
}

fn normalize_pack(
    provider_id: &str,
    metadata: &str,
    public_asset_base_url: &str,
    telegram_files: &[TelegramResolvedFile],
) -> ProviderImportWorkerResult<StickerPack> {
    match provider_id {
        "line-stickers" => {
            Ok(LineStickerProvider.normalize_pack_json(metadata, public_asset_base_url)?)
        }
        "telegram" => {
            let fixture = telegram_fixture_json(metadata, telegram_files)?;
            Ok(TelegramProvider.normalize_pack_json(&fixture, public_asset_base_url)?)
        }
        other => Err(ProviderImportWorkerError::UnsupportedProvider {
            provider_id: other.to_owned(),
        }),
    }
}

async fn internalize_assets(
    pack: &StickerPack,
    pack_public_id: &str,
    public_asset_base_url: &str,
    plan: ProviderRemoteFetchPlan,
    telegram_files: &[TelegramResolvedFile],
    downloader: &(dyn ProviderAssetDownloader + Send + Sync),
    asset_store: &LocalAssetStore,
) -> ProviderImportWorkerResult<StickerPack> {
    match plan.asset_strategy {
        ProviderAssetDownloadStrategy::DirectRemoteUrls => {
            Ok(internalize_direct_remote_pack_assets(
                pack,
                pack_public_id,
                public_asset_base_url,
                ProviderAssetDownloadStrategy::DirectRemoteUrls,
                downloader,
                asset_store,
            )
            .await?)
        }
        ProviderAssetDownloadStrategy::TelegramBotFileApi => {
            internalize_telegram_pack_assets(
                pack,
                pack_public_id,
                public_asset_base_url,
                &plan,
                telegram_files,
                downloader,
                asset_store,
            )
            .await
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelegramGetStickerSetResponse {
    result: TelegramStickerSetResult,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelegramStickerSetResult {
    name: String,
    title: String,
    stickers: Vec<TelegramStickerResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelegramStickerResult {
    file_id: String,
    file_unique_id: String,
    #[serde(default)]
    emoji: Option<String>,
    #[serde(default)]
    is_animated: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct TelegramGetFileResponse {
    result: TelegramGetFileResult,
}

#[derive(Debug, Clone, Deserialize)]
struct TelegramGetFileResult {
    file_path: String,
}

#[derive(Debug, Clone)]
struct TelegramResolvedFile {
    id: String,
    unique_id: String,
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TelegramFixtureSticker<'a> {
    file_unique_id: &'a str,
    emoji: Option<&'a str>,
    extension: String,
    is_animated: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TelegramFixture<'a> {
    name: &'a str,
    title: &'a str,
    stickers: Vec<TelegramFixtureSticker<'a>>,
}

async fn resolve_telegram_files(
    plan: &ProviderRemoteFetchPlan,
    metadata: &str,
    fetcher: &(dyn ProviderMetadataFetcher + Send + Sync),
) -> ProviderImportWorkerResult<Vec<TelegramResolvedFile>> {
    let sticker_set: TelegramGetStickerSetResponse = serde_json::from_str(metadata)?;
    let method_base = telegram_method_base(&plan.metadata_request.url);
    let mut files = Vec::with_capacity(sticker_set.result.stickers.len());
    for sticker in sticker_set.result.stickers {
        let file_plan = ProviderRemoteFetchPlan {
            provider_id: "telegram".to_owned(),
            remote_id: sticker.file_id.clone(),
            metadata_request: ProviderHttpRequestPlan {
                method: "GET".to_owned(),
                url: format!(
                    "{method_base}/getFile?file_id={}",
                    percent_encode(&sticker.file_id)
                ),
                redacted_headers: plan.metadata_request.redacted_headers.clone(),
            },
            asset_strategy: ProviderAssetDownloadStrategy::TelegramBotFileApi,
        };
        let bytes = fetcher.fetch_metadata(&file_plan).await?;
        let file: TelegramGetFileResponse = serde_json::from_slice(&bytes)?;
        files.push(TelegramResolvedFile {
            id: sticker.file_id,
            unique_id: sticker.file_unique_id,
            path: file.result.file_path,
        });
    }
    Ok(files)
}

fn telegram_fixture_json(
    metadata: &str,
    files: &[TelegramResolvedFile],
) -> ProviderImportWorkerResult<String> {
    let sticker_set: TelegramGetStickerSetResponse = serde_json::from_str(metadata)?;

    let stickers = sticker_set
        .result
        .stickers
        .iter()
        .map(|sticker| {
            let extension = files
                .iter()
                .find(|file| file.id == sticker.file_id || file.unique_id == sticker.file_unique_id)
                .and_then(|file| file.path.rsplit('.').next())
                .filter(|extension| !extension.contains('/'))
                .unwrap_or("webp")
                .to_owned();
            TelegramFixtureSticker {
                file_unique_id: &sticker.file_unique_id,
                emoji: sticker.emoji.as_deref(),
                extension,
                is_animated: sticker.is_animated,
            }
        })
        .collect();
    Ok(serde_json::to_string(&TelegramFixture {
        name: &sticker_set.result.name,
        title: &sticker_set.result.title,
        stickers,
    })?)
}

async fn internalize_telegram_pack_assets(
    pack: &StickerPack,
    pack_public_id: &str,
    public_asset_base_url: &str,
    plan: &ProviderRemoteFetchPlan,
    files: &[TelegramResolvedFile],
    downloader: &(dyn ProviderAssetDownloader + Send + Sync),
    asset_store: &LocalAssetStore,
) -> ProviderImportWorkerResult<StickerPack> {
    let base_url = public_asset_base_url.trim().trim_end_matches('/');
    let file_base = telegram_file_base(&plan.metadata_request.url);
    let mut rewritten = pack.clone();
    for sticker in &mut rewritten.stickers {
        let Some(filename) = sticker.filename.clone() else {
            continue;
        };
        let Some(file) = files
            .iter()
            .find(|file| sticker.id.ends_with(&file.unique_id))
        else {
            return Err(ProviderImportWorkerError::UnsupportedProvider {
                provider_id: format!("missing telegram file for {}", sticker.id),
            });
        };
        let key = msm_storage::AssetKey::new(pack_public_id, filename.clone())?;
        let download_url = format!("{file_base}/{}", file.path.trim_start_matches('/'));
        let bytes = downloader.download_asset(&download_url).await?;
        asset_store.write(&key, &bytes).await?;
        sticker.image = format!("{base_url}/assets/packs/{pack_public_id}/{filename}");
    }
    if let Some(first) = rewritten.stickers.first() {
        rewritten.logo = first.clone();
    }
    Ok(rewritten)
}

fn telegram_method_base(url: &str) -> String {
    url.split("/getStickerSet")
        .next()
        .unwrap_or(url)
        .trim_end_matches('/')
        .to_owned()
}

fn telegram_file_base(url: &str) -> String {
    let method_base = telegram_method_base(url);
    if let Some((prefix, token)) = method_base.rsplit_once("/bot") {
        format!("{prefix}/file/bot{token}")
    } else {
        format!("{method_base}/file")
    }
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn next_attempt_at(backoff: Duration) -> ProviderImportWorkerResult<String> {
    let duration = chrono::Duration::from_std(backoff)
        .map_err(|_| ProviderImportWorkerError::RetryBackoffOverflow)?;
    Ok((chrono::Utc::now() + duration).to_rfc3339())
}
