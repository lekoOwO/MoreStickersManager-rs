use std::{sync::Arc, time::Duration};

use msm_domain::StickerPack;
use msm_providers::{
    line::LineStickerProvider, line_sticker_pack_fetch_plan, telegram_sticker_set_fetch_plan,
    ProviderAssetDownloadStrategy, ProviderRemoteFetchPlan, StickerProvider,
};
use msm_storage::{
    models::{ExportJobStatus, NewProviderImportJobEvent, PackVisibility, ProviderImportJobRecord},
    LocalAssetStore, StorageRepository,
};
use serde::{Deserialize, Serialize};

use crate::{
    internalize_direct_remote_pack_assets, ProviderAssetDownloader, ProviderImportError,
    ProviderMetadataFetcher,
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
    /// Public base URL embedded into self-hosted asset URLs.
    pub public_asset_base_url: String,
    /// Retry backoff for retryable failures.
    pub retry_backoff: Duration,
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
        let plan = provider_import_plan(
            &request.provider_id,
            &request.remote_id,
            request.base_url.as_deref(),
        )?;
        let metadata_bytes = self.metadata_fetcher.fetch_metadata(&plan).await?;
        let metadata = String::from_utf8_lossy(&metadata_bytes);
        let pack = normalize_pack(
            &request.provider_id,
            &metadata,
            &self.config.public_asset_base_url,
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
) -> ProviderImportWorkerResult<StickerPack> {
    match provider_id {
        "line-stickers" => {
            Ok(LineStickerProvider.normalize_pack_json(metadata, public_asset_base_url)?)
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
        strategy @ ProviderAssetDownloadStrategy::TelegramBotFileApi => {
            Err(ProviderImportWorkerError::UnsupportedAssetStrategy { strategy })
        }
    }
}

fn next_attempt_at(backoff: Duration) -> ProviderImportWorkerResult<String> {
    let duration = chrono::Duration::from_std(backoff)
        .map_err(|_| ProviderImportWorkerError::RetryBackoffOverflow)?;
    Ok((chrono::Utc::now() + duration).to_rfc3339())
}
