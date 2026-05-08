use std::{
    future::Future, path::PathBuf, pin::Pin, process::ExitStatus, sync::Arc, time::Duration,
};

use msm_exporters::{
    MoreStickersExportTarget, PlannedTelegramSticker, TelegramExportOptions, TelegramExportPlan,
    TelegramExportPlanner, TelegramStickerSetType, TelegramTargetConfig, TelegramTargetError,
};
use msm_media::{ConversionCommand, ConverterToolchain, PreparedMediaSpec, StickerTargetProfile};
use msm_storage::{
    models::{
        ExportJobRecord, ExportJobStatus, ExportTargetRecord, NewExportJobEvent,
        NewPreparedMediaAsset, NewTelegramPublication,
    },
    StorageRepository,
};
use msm_telegram::{
    publish_sticker_set, TelegramBotConfig, TelegramBotError, TelegramPublishError,
    TelegramPublishRequest, TelegramPublishSticker, TelegramPublishedSet,
    TeloxideTelegramStickerSetApi,
};
use sha2::{Digest, Sha256};
use teloxide::types::{InputFile, StickerType};

/// Export worker runtime configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWorkerConfig {
    /// Whether the service should spawn a background worker loop.
    pub enabled: bool,
    /// `ffmpeg` executable path used by later conversion execution.
    pub ffmpeg_path: PathBuf,
    /// `ffprobe` executable path used by later probing execution.
    pub ffprobe_path: PathBuf,
    /// Directory for prepared media outputs.
    pub prepared_media_dir: PathBuf,
    /// Maximum jobs a future background loop may run concurrently.
    pub max_concurrent_jobs: usize,
    /// Poll interval for the background worker loop.
    pub poll_interval: Duration,
    /// Delay before a failed retryable export job becomes due again.
    pub retry_backoff: Duration,
}

/// Export worker result type.
pub type ExportWorkerResult<T> = Result<T, ExportWorkerError>;

/// Export worker errors.
#[derive(Debug, thiserror::Error)]
pub enum ExportWorkerError {
    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(#[from] msm_storage::StorageError),

    /// Exporter planning or serialization failed.
    #[error("exporter error: {0}")]
    Exporter(#[from] msm_exporters::ExportError),

    /// JSON serialization or deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Source pack does not exist.
    #[error("source pack not found: {pack_id}")]
    PackNotFound {
        /// Missing pack ID.
        pack_id: String,
    },

    /// Configured target does not exist.
    #[error("export target not found: {target_id}")]
    TargetNotFound {
        /// Missing target ID.
        target_id: String,
    },

    /// Export target kind is not executable yet.
    #[error("unsupported export target kind: {kind}")]
    UnsupportedTargetKind {
        /// Unsupported target kind.
        kind: String,
    },

    /// Telegram planning failed.
    #[error("Telegram export planning failed: {0}")]
    TelegramPlan(#[from] TelegramTargetError),

    /// Telegram bot configuration failed.
    #[error("Telegram bot configuration failed: {0}")]
    TelegramBot(#[from] TelegramBotError),

    /// Telegram publication failed.
    #[error("Telegram publication failed: {0}")]
    TelegramPublish(#[from] TelegramPublishError),

    /// Telegram remote publication requires a bot token.
    #[error("Telegram publication requires target config field `botToken`")]
    MissingTelegramBotToken,

    /// Planned sticker has no prepared media output.
    #[error("prepared media output missing for sticker {sticker_id} profile {profile_key}")]
    MissingPreparedMedia {
        /// Source sticker compatibility ID.
        sticker_id: String,
        /// Target media profile key.
        profile_key: String,
    },

    /// Export job has too many events to assign an `i64` sequence.
    #[error("export job event sequence overflow")]
    EventSequenceOverflow,

    /// Retry backoff cannot be represented as a timestamp duration.
    #[error("retry backoff overflow")]
    RetryBackoffOverflow,

    /// Published sticker count does not fit storage.
    #[error("published sticker count overflow")]
    StickerCountOverflow,

    /// Converter process did not complete before its timeout.
    #[error("converter timed out")]
    ConverterTimeout,

    /// Converter process exited unsuccessfully.
    #[error("converter exited unsuccessfully: {status}")]
    ConverterFailed {
        /// Failed exit status.
        status: ExitStatus,
    },

    /// Planned sticker references a source sticker that is missing from the source pack.
    #[error("planned sticker source not found: {sticker_id}")]
    StickerNotFound {
        /// Missing sticker compatibility ID.
        sticker_id: String,
    },

    /// Planned media profile is unknown to the worker.
    #[error("unknown media profile key: {profile_key}")]
    UnknownMediaProfile {
        /// Unknown media profile key.
        profile_key: String,
    },

    /// Prepared media output path does not have a parent directory.
    #[error("prepared media output path has no parent: {path}")]
    InvalidPreparedMediaPath {
        /// Invalid output path.
        path: PathBuf,
    },

    /// I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Request passed from the worker to a prepared media executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedMediaRequest {
    /// Source sticker compatibility ID.
    pub sticker_id: String,
    /// Source image URL or future local asset URI.
    pub source_uri: String,
    /// Stable hash of the source asset identity.
    pub source_asset_hash: String,
    /// Target media profile key.
    pub profile_key: String,
    /// Expected output MIME type.
    pub mime_type: String,
    /// Expected output file extension.
    pub extension: String,
    /// Expected output width.
    pub width_px: i64,
    /// Expected output height.
    pub height_px: i64,
    /// Expected output duration for animated/video outputs.
    pub duration_ms: Option<i64>,
}

/// Prepared media executor output cached by the worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedMediaOutput {
    /// Stable hash of the source asset identity.
    pub source_asset_hash: String,
    /// Target media profile key.
    pub profile_key: String,
    /// Prepared output asset key.
    pub output_asset_key: String,
    /// Output MIME type.
    pub mime_type: String,
    /// Output width.
    pub width_px: i64,
    /// Output height.
    pub height_px: i64,
    /// Output duration for animated/video outputs.
    pub duration_ms: Option<i64>,
    /// Prepared output size.
    pub file_size_bytes: i64,
}

/// Boundary for converting or preparing target-specific media.
pub trait PreparedMediaExecutor: std::fmt::Debug + Send + Sync {
    /// Prepares one media output.
    fn prepare(
        &self,
        request: PreparedMediaRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<Option<PreparedMediaOutput>>> + Send + '_>>;
}

/// Runs one planned converter command.
pub trait ConversionCommandRunner: std::fmt::Debug + Send + Sync {
    /// Executes one shell-free conversion command.
    fn run(
        &self,
        command: ConversionCommand,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<()>> + Send + '_>>;
}

/// Request passed from the worker to a Telegram publication executor.
#[derive(Debug)]
pub struct TelegramPublicationRequest {
    /// Telegram bot token used only for the current publication call.
    pub bot_token: String,
    /// Fully prepared Telegram sticker set publication request.
    pub publish_request: TelegramPublishRequest,
}

/// Boundary for publishing prepared Telegram sticker sets.
pub trait TelegramPublicationExecutor: std::fmt::Debug + Send + Sync {
    /// Publishes one Telegram sticker set.
    fn publish(
        &self,
        request: TelegramPublicationRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<TelegramPublishedSet>> + Send + '_>>;
}

/// Telegram publication executor backed by `teloxide`.
#[derive(Debug, Default)]
pub struct TeloxideTelegramPublicationExecutor;

impl TelegramPublicationExecutor for TeloxideTelegramPublicationExecutor {
    fn publish(
        &self,
        request: TelegramPublicationRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<TelegramPublishedSet>> + Send + '_>> {
        Box::pin(async move {
            let bot = TelegramBotConfig::new(request.bot_token)?.build_bot();
            let api = TeloxideTelegramStickerSetApi::new(bot);
            publish_sticker_set(&api, request.publish_request)
                .await
                .map_err(Into::into)
        })
    }
}

/// `tokio::process` command runner for real converter execution.
#[derive(Debug)]
pub struct TokioConversionCommandRunner;

impl ConversionCommandRunner for TokioConversionCommandRunner {
    fn run(
        &self,
        command: ConversionCommand,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<()>> + Send + '_>> {
        Box::pin(async move {
            let mut child = tokio::process::Command::new(command.executable())
                .args(command.args())
                .kill_on_drop(true)
                .spawn()?;
            let status = tokio::time::timeout(command.timeout(), child.wait())
                .await
                .map_err(|_| ExportWorkerError::ConverterTimeout)??;
            if status.success() {
                Ok(())
            } else {
                Err(ExportWorkerError::ConverterFailed { status })
            }
        })
    }
}

/// Prepared media executor backed by a converter process.
#[derive(Debug)]
pub struct ProcessPreparedMediaExecutor {
    ffmpeg_path: PathBuf,
    output_dir: PathBuf,
    runner: Arc<dyn ConversionCommandRunner>,
}

impl ProcessPreparedMediaExecutor {
    /// Creates a process-backed media executor using `tokio::process`.
    #[must_use]
    pub fn new(ffmpeg_path: PathBuf, output_dir: PathBuf) -> Self {
        Self::with_runner(
            ffmpeg_path,
            output_dir,
            Arc::new(TokioConversionCommandRunner),
        )
    }

    /// Creates a process-backed media executor with an injected command runner.
    #[must_use]
    pub fn with_runner(
        ffmpeg_path: PathBuf,
        output_dir: PathBuf,
        runner: Arc<dyn ConversionCommandRunner>,
    ) -> Self {
        Self {
            ffmpeg_path,
            output_dir,
            runner,
        }
    }
}

impl PreparedMediaExecutor for ProcessPreparedMediaExecutor {
    fn prepare(
        &self,
        request: PreparedMediaRequest,
    ) -> Pin<Box<dyn Future<Output = ExportWorkerResult<Option<PreparedMediaOutput>>> + Send + '_>>
    {
        Box::pin(async move {
            let prepared_media = prepared_media_spec_for_key(&request.profile_key)?;
            let output_asset_key = prepared_output_asset_key(&request);
            let output_path = self.output_dir.join(&output_asset_key);
            let Some(parent) = output_path.parent() else {
                return Err(ExportWorkerError::InvalidPreparedMediaPath { path: output_path });
            };
            tokio::fs::create_dir_all(parent).await?;

            let toolchain = ConverterToolchain::new(self.ffmpeg_path.clone());
            let command = ConversionCommand::for_prepared_media(
                &toolchain,
                &prepared_media,
                &PathBuf::from(&request.source_uri),
                &output_path,
            );
            self.runner.run(command).await?;
            let metadata = tokio::fs::metadata(&output_path).await?;

            Ok(Some(PreparedMediaOutput {
                source_asset_hash: request.source_asset_hash,
                profile_key: request.profile_key,
                output_asset_key,
                mime_type: request.mime_type,
                width_px: request.width_px,
                height_px: request.height_px,
                duration_ms: request.duration_ms,
                file_size_bytes: i64::try_from(metadata.len()).unwrap_or(i64::MAX),
            }))
        })
    }
}

/// Spawns the export worker loop when enabled by configuration.
#[must_use]
pub fn spawn_export_worker_if_enabled(
    repository: StorageRepository,
    config: ExportWorkerConfig,
) -> Option<tokio::task::JoinHandle<()>> {
    if !config.enabled {
        return None;
    }

    Some(tokio::spawn(async move {
        let poll_interval = config.poll_interval;
        let worker = ExportWorker::new(repository, config);
        let mut interval = tokio::time::interval(poll_interval);
        loop {
            interval.tick().await;
            if worker.run_next_queued().await.is_err() {
                interval.tick().await;
            }
        }
    }))
}

/// Single-job export worker.
#[derive(Clone)]
pub struct ExportWorker {
    repository: StorageRepository,
    config: ExportWorkerConfig,
    media_executor: Arc<dyn PreparedMediaExecutor>,
    telegram_publication_executor: Arc<dyn TelegramPublicationExecutor>,
}

impl ExportWorker {
    /// Creates a worker from storage and runtime configuration.
    #[must_use]
    pub fn new(repository: StorageRepository, config: ExportWorkerConfig) -> Self {
        let media_executor = Arc::new(ProcessPreparedMediaExecutor::new(
            config.ffmpeg_path.clone(),
            config.prepared_media_dir.clone(),
        ));
        Self::with_media_executor(repository, config, media_executor)
    }

    /// Creates a worker with an explicit prepared media executor.
    #[must_use]
    pub fn with_media_executor(
        repository: StorageRepository,
        config: ExportWorkerConfig,
        media_executor: Arc<dyn PreparedMediaExecutor>,
    ) -> Self {
        Self::with_media_and_telegram_executors(
            repository,
            config,
            media_executor,
            Arc::new(TeloxideTelegramPublicationExecutor),
        )
    }

    /// Creates a worker with explicit prepared media and Telegram publication executors.
    #[must_use]
    pub fn with_media_and_telegram_executors(
        repository: StorageRepository,
        config: ExportWorkerConfig,
        media_executor: Arc<dyn PreparedMediaExecutor>,
        telegram_publication_executor: Arc<dyn TelegramPublicationExecutor>,
    ) -> Self {
        Self {
            repository,
            config,
            media_executor,
            telegram_publication_executor,
        }
    }

    /// Runs the oldest queued job if one exists.
    ///
    /// # Errors
    ///
    /// Returns an error when storage, planning, or serialization fails.
    pub async fn run_next_queued(&self) -> ExportWorkerResult<Option<ExportJobRecord>> {
        let Some(job) = self
            .repository
            .find_next_due_export_job(&chrono::Utc::now().to_rfc3339())
            .await?
        else {
            return Ok(None);
        };

        self.run_job(&job.id).await.map(Some)
    }

    /// Runs one export job by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when storage, planning, or serialization fails.
    pub async fn run_job(&self, job_id: &str) -> ExportWorkerResult<ExportJobRecord> {
        let Some(job) = self.repository.find_export_job(job_id).await? else {
            return Err(ExportWorkerError::PackNotFound {
                pack_id: job_id.to_owned(),
            });
        };
        self.repository
            .update_export_job_status(&job.id, ExportJobStatus::Running, None, None)
            .await?;
        self.append_event(&job.id, "info", "running", "export job started")
            .await?;

        match self.execute_job(&job).await {
            Ok(result) => {
                let result_json = serde_json::to_string(&result)?;
                self.repository
                    .update_export_job_status(
                        &job.id,
                        ExportJobStatus::Succeeded,
                        None,
                        Some(&result_json),
                    )
                    .await?;
                self.append_event(&job.id, "info", "succeeded", "export job succeeded")
                    .await?;
            }
            Err(error) => {
                let message = error.to_string();
                let failed_attempt_count = job.attempt_count + 1;
                if failed_attempt_count < job.max_attempts {
                    let next_attempt_at = next_attempt_at(self.config.retry_backoff)?;
                    self.repository
                        .record_export_job_retry(&job.id, &message, &next_attempt_at)
                        .await?;
                    self.append_event(&job.id, "warn", "retry_scheduled", &message)
                        .await?;
                    return self
                        .repository
                        .find_export_job(&job.id)
                        .await?
                        .ok_or_else(|| ExportWorkerError::PackNotFound {
                            pack_id: job.id.clone(),
                        });
                }

                self.repository
                    .record_export_job_failure(&job.id, &message)
                    .await?;
                self.append_event(&job.id, "error", "failed", &message)
                    .await?;
                return Err(error);
            }
        }

        self.repository
            .find_export_job(&job.id)
            .await?
            .ok_or_else(|| ExportWorkerError::PackNotFound {
                pack_id: job.id.clone(),
            })
    }

    async fn execute_job(&self, job: &ExportJobRecord) -> ExportWorkerResult<WorkerJobResult> {
        let pack = self
            .repository
            .find_sticker_pack_record(&job.source_pack_id)
            .await?
            .ok_or_else(|| ExportWorkerError::PackNotFound {
                pack_id: job.source_pack_id.clone(),
            })?;
        let target = self
            .repository
            .find_export_target(&job.target_id)
            .await?
            .ok_or_else(|| ExportWorkerError::TargetNotFound {
                target_id: job.target_id.clone(),
            })?;

        match target.kind.as_str() {
            "morestickers" => {
                let artifact = MoreStickersExportTarget.export_pack(&pack.sticker_pack)?;
                Ok(WorkerJobResult::MoreStickers {
                    target_kind: target.kind,
                    file_name: artifact.file_name,
                    mime_type: artifact.mime_type.to_owned(),
                    byte_len: artifact.contents.len(),
                })
            }
            "telegram" => {
                self.plan_telegram_job(job, &target, &pack.sticker_pack)
                    .await
            }
            kind => Err(ExportWorkerError::UnsupportedTargetKind {
                kind: kind.to_owned(),
            }),
        }
    }

    async fn plan_telegram_job(
        &self,
        job: &ExportJobRecord,
        target: &ExportTargetRecord,
        pack: &msm_domain::StickerPack,
    ) -> ExportWorkerResult<WorkerJobResult> {
        let target_config: TelegramTargetConfigJson = serde_json::from_str(&target.config_json)?;
        let TelegramTargetConfigJson {
            bot_username,
            owner_user_id,
            bot_token,
        } = target_config;
        let request: WorkerExportJobRequest = serde_json::from_str(&job.request_json)?;
        let options = request.options;
        let dry_run = options.dry_run.unwrap_or(true);
        let set_type = match options.set_type.as_deref() {
            Some("customEmoji" | "custom_emoji") => TelegramStickerSetType::CustomEmoji,
            _ => TelegramStickerSetType::Regular,
        };
        let plan = TelegramExportPlanner::plan_pack(
            pack,
            TelegramExportOptions {
                target: TelegramTargetConfig {
                    bot_username,
                    owner_user_id,
                },
                set_name_slug: options.set_name_slug.unwrap_or_else(|| pack.title.clone()),
                set_title: options.set_title.unwrap_or_else(|| pack.title.clone()),
                set_type,
                default_emoji: options.default_emoji.unwrap_or_else(|| "😀".to_owned()),
                existing_sticker_set_names: options.existing_sticker_set_names,
            },
        )?;

        self.append_event(
            &job.id,
            "info",
            "telegram.prepare",
            "preparing Telegram media",
        )
        .await?;
        let prepared_media = self
            .prepare_telegram_media(pack, &plan.initial_stickers, &plan.append_stickers)
            .await?;

        if !dry_run {
            return self
                .publish_telegram_job(job, target, bot_token, set_type, plan, prepared_media)
                .await;
        }

        Ok(WorkerJobResult::TelegramDryRun {
            target_kind: target.kind.clone(),
            sticker_set_name: plan.sticker_set_name,
            initial_sticker_count: plan.initial_stickers.len(),
            append_sticker_count: plan.append_stickers.len(),
            media_profile_keys: plan
                .initial_stickers
                .iter()
                .chain(plan.append_stickers.iter())
                .map(|sticker| sticker.target_profile_key.clone())
                .collect(),
            ffmpeg_path: self.config.ffmpeg_path.display().to_string(),
            ffprobe_path: self.config.ffprobe_path.display().to_string(),
            prepared_media,
            dry_run: true,
        })
    }

    async fn publish_telegram_job(
        &self,
        job: &ExportJobRecord,
        target: &ExportTargetRecord,
        bot_token: Option<String>,
        set_type: TelegramStickerSetType,
        plan: TelegramExportPlan,
        prepared_media: Vec<CachedPreparedMediaSummary>,
    ) -> ExportWorkerResult<WorkerJobResult> {
        let bot_token = bot_token.ok_or(ExportWorkerError::MissingTelegramBotToken)?;
        let sticker_type = sticker_type_for_plan(set_type);
        let initial_stickers =
            self.telegram_publish_stickers(&plan.initial_stickers, &prepared_media)?;
        let append_stickers =
            self.telegram_publish_stickers(&plan.append_stickers, &prepared_media)?;
        self.append_event(
            &job.id,
            "info",
            "telegram.publish.create",
            "creating Telegram sticker set",
        )
        .await?;
        self.append_event(
            &job.id,
            "info",
            "telegram.publish.append",
            "appending Telegram stickers",
        )
        .await?;
        let published = self
            .telegram_publication_executor
            .publish(TelegramPublicationRequest {
                bot_token,
                publish_request: TelegramPublishRequest {
                    owner_user_id: plan.owner_user_id,
                    sticker_set_name: plan.sticker_set_name,
                    title: plan.title,
                    sticker_type,
                    initial_stickers,
                    append_stickers,
                },
            })
            .await?;

        let publication_id = telegram_publication_id(&target.id, &published.sticker_set_name);
        let sticker_count = i64::try_from(published.sticker_count)
            .map_err(|_| ExportWorkerError::StickerCountOverflow)?;
        let sticker_type = sticker_type_label(set_type).to_owned();
        self.repository
            .upsert_telegram_publication(NewTelegramPublication {
                id: &publication_id,
                pack_id: &job.source_pack_id,
                target_id: &target.id,
                job_id: &job.id,
                sticker_set_name: &published.sticker_set_name,
                sticker_set_url: &published.url,
                sticker_count,
                sticker_type: &sticker_type,
            })
            .await?;

        Ok(WorkerJobResult::TelegramPublished {
            target_kind: target.kind.clone(),
            sticker_set_name: published.sticker_set_name,
            sticker_set_url: published.url,
            sticker_count: published.sticker_count,
            sticker_type,
            prepared_media,
            dry_run: false,
        })
    }

    fn telegram_publish_stickers(
        &self,
        planned_stickers: &[PlannedTelegramSticker],
        prepared_media: &[CachedPreparedMediaSummary],
    ) -> ExportWorkerResult<Vec<TelegramPublishSticker>> {
        planned_stickers
            .iter()
            .map(|planned| {
                let prepared = prepared_media
                    .iter()
                    .find(|prepared| {
                        prepared.sticker_id == planned.sticker_id
                            && prepared.profile_key == planned.target_profile_key
                    })
                    .ok_or_else(|| ExportWorkerError::MissingPreparedMedia {
                        sticker_id: planned.sticker_id.clone(),
                        profile_key: planned.target_profile_key.clone(),
                    })?;
                let file = InputFile::file(
                    self.config
                        .prepared_media_dir
                        .join(&prepared.output_asset_key),
                );
                Ok(TelegramPublishSticker {
                    source_sticker_id: planned.sticker_id.clone(),
                    input: planned.to_input_sticker(file),
                })
            })
            .collect()
    }

    async fn prepare_telegram_media(
        &self,
        pack: &msm_domain::StickerPack,
        initial_stickers: &[PlannedTelegramSticker],
        append_stickers: &[PlannedTelegramSticker],
    ) -> ExportWorkerResult<Vec<CachedPreparedMediaSummary>> {
        let mut prepared = Vec::new();
        for planned in initial_stickers.iter().chain(append_stickers.iter()) {
            let source = pack
                .stickers
                .iter()
                .find(|sticker| sticker.id == planned.sticker_id)
                .ok_or_else(|| ExportWorkerError::StickerNotFound {
                    sticker_id: planned.sticker_id.clone(),
                })?;
            let profile = media_profile_for_key(&planned.target_profile_key)?;
            let source_hash = source_asset_hash(&source.image);
            let output = self
                .media_executor
                .prepare(PreparedMediaRequest {
                    sticker_id: planned.sticker_id.clone(),
                    source_uri: source.image.clone(),
                    source_asset_hash: source_hash,
                    profile_key: planned.target_profile_key.clone(),
                    mime_type: profile.mime_type.to_owned(),
                    extension: profile.extension.to_owned(),
                    width_px: profile.width_px,
                    height_px: profile.height_px,
                    duration_ms: profile.duration_ms,
                })
                .await?;

            if let Some(output) = output {
                self.repository
                    .upsert_prepared_media_asset(NewPreparedMediaAsset {
                        source_asset_hash: &output.source_asset_hash,
                        profile_key: &output.profile_key,
                        output_asset_key: &output.output_asset_key,
                        mime_type: &output.mime_type,
                        width_px: output.width_px,
                        height_px: output.height_px,
                        duration_ms: output.duration_ms,
                        file_size_bytes: output.file_size_bytes,
                    })
                    .await?;
                prepared.push(CachedPreparedMediaSummary {
                    sticker_id: planned.sticker_id.clone(),
                    source_asset_hash: output.source_asset_hash,
                    profile_key: output.profile_key,
                    output_asset_key: output.output_asset_key,
                    mime_type: output.mime_type,
                    file_size_bytes: output.file_size_bytes,
                });
            }
        }

        Ok(prepared)
    }

    async fn append_event(
        &self,
        job_id: &str,
        level: &str,
        stage: &str,
        message: &str,
    ) -> ExportWorkerResult<()> {
        let event_count = self.repository.list_export_job_events(job_id).await?.len();
        let sequence =
            i64::try_from(event_count).map_err(|_| ExportWorkerError::EventSequenceOverflow)? + 1;
        self.repository
            .append_export_job_event(NewExportJobEvent {
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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelegramTargetConfigJson {
    bot_username: String,
    owner_user_id: i64,
    bot_token: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerExportJobRequest {
    options: WorkerTelegramOptions,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerTelegramOptions {
    set_name_slug: Option<String>,
    set_title: Option<String>,
    set_type: Option<String>,
    default_emoji: Option<String>,
    dry_run: Option<bool>,
    #[serde(default)]
    existing_sticker_set_names: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CachedPreparedMediaSummary {
    sticker_id: String,
    source_asset_hash: String,
    profile_key: String,
    output_asset_key: String,
    mime_type: String,
    file_size_bytes: i64,
}

#[derive(Debug, serde::Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
enum WorkerJobResult {
    MoreStickers {
        target_kind: String,
        file_name: String,
        mime_type: String,
        byte_len: usize,
    },
    TelegramDryRun {
        target_kind: String,
        sticker_set_name: String,
        initial_sticker_count: usize,
        append_sticker_count: usize,
        media_profile_keys: Vec<String>,
        ffmpeg_path: String,
        ffprobe_path: String,
        prepared_media: Vec<CachedPreparedMediaSummary>,
        dry_run: bool,
    },
    TelegramPublished {
        target_kind: String,
        sticker_set_name: String,
        sticker_set_url: String,
        sticker_count: usize,
        sticker_type: String,
        prepared_media: Vec<CachedPreparedMediaSummary>,
        dry_run: bool,
    },
}

#[derive(Clone, Copy, Debug)]
struct MediaProfileSummary {
    mime_type: &'static str,
    extension: &'static str,
    width_px: i64,
    height_px: i64,
    duration_ms: Option<i64>,
}

fn media_profile_for_key(profile_key: &str) -> ExportWorkerResult<MediaProfileSummary> {
    match profile_key {
        "telegram.sticker.static.v1" => Ok(MediaProfileSummary {
            mime_type: "image/png",
            extension: "png",
            width_px: 512,
            height_px: 512,
            duration_ms: None,
        }),
        "telegram.sticker.video.v1" => Ok(MediaProfileSummary {
            mime_type: "video/webm",
            extension: "webm",
            width_px: 512,
            height_px: 512,
            duration_ms: Some(3_000),
        }),
        _ => Err(ExportWorkerError::UnknownMediaProfile {
            profile_key: profile_key.to_owned(),
        }),
    }
}

fn prepared_media_spec_for_key(profile_key: &str) -> ExportWorkerResult<PreparedMediaSpec> {
    match profile_key {
        "telegram.sticker.static.v1" => Ok(PreparedMediaSpec::new(
            StickerTargetProfile::telegram_static_sticker(),
            "image/png",
            "png",
        )),
        "telegram.sticker.video.v1" => Ok(PreparedMediaSpec::new(
            StickerTargetProfile::telegram_video_sticker(),
            "video/webm",
            "webm",
        )),
        _ => Err(ExportWorkerError::UnknownMediaProfile {
            profile_key: profile_key.to_owned(),
        }),
    }
}

fn source_asset_hash(source_uri: &str) -> String {
    let digest = Sha256::digest(source_uri.as_bytes());
    format!("sha256:{digest:x}")
}

fn prepared_output_asset_key(request: &PreparedMediaRequest) -> String {
    let safe_hash = request.source_asset_hash.replace([':', '/'], "_");
    format!("{}/{safe_hash}.{}", request.profile_key, request.extension)
}

fn telegram_publication_id(target_id: &str, sticker_set_name: &str) -> String {
    format!("telegram:{target_id}:{sticker_set_name}")
}

fn next_attempt_at(backoff: Duration) -> ExportWorkerResult<String> {
    let backoff =
        chrono::Duration::from_std(backoff).map_err(|_| ExportWorkerError::RetryBackoffOverflow)?;
    Ok((chrono::Utc::now() + backoff).to_rfc3339())
}

const fn sticker_type_for_plan(set_type: TelegramStickerSetType) -> StickerType {
    match set_type {
        TelegramStickerSetType::Regular => StickerType::Regular,
        TelegramStickerSetType::CustomEmoji => StickerType::CustomEmoji,
    }
}

const fn sticker_type_label(set_type: TelegramStickerSetType) -> &'static str {
    match set_type {
        TelegramStickerSetType::Regular => "regular",
        TelegramStickerSetType::CustomEmoji => "customEmoji",
    }
}
