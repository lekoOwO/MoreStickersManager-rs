use std::path::PathBuf;

use msm_exporters::{
    MoreStickersExportTarget, TelegramExportOptions, TelegramExportPlanner, TelegramStickerSetType,
    TelegramTargetConfig, TelegramTargetError,
};
use msm_storage::{
    models::{ExportJobRecord, ExportJobStatus, ExportTargetRecord, NewExportJobEvent},
    StorageRepository,
};

/// Export worker runtime configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWorkerConfig {
    /// `ffmpeg` executable path used by later conversion execution.
    pub ffmpeg_path: PathBuf,
    /// `ffprobe` executable path used by later probing execution.
    pub ffprobe_path: PathBuf,
    /// Maximum jobs a future background loop may run concurrently.
    pub max_concurrent_jobs: usize,
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

    /// Export job has too many events to assign an `i64` sequence.
    #[error("export job event sequence overflow")]
    EventSequenceOverflow,
}

/// Single-job export worker.
#[derive(Clone)]
pub struct ExportWorker {
    repository: StorageRepository,
    config: ExportWorkerConfig,
}

impl ExportWorker {
    /// Creates a worker from storage and runtime configuration.
    #[must_use]
    pub fn new(repository: StorageRepository, config: ExportWorkerConfig) -> Self {
        Self { repository, config }
    }

    /// Runs the oldest queued job if one exists.
    ///
    /// # Errors
    ///
    /// Returns an error when storage, planning, or serialization fails.
    pub async fn run_next_queued(&self) -> ExportWorkerResult<Option<ExportJobRecord>> {
        let Some(job) = self
            .repository
            .find_next_export_job_by_status(ExportJobStatus::Queued)
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
                self.repository
                    .update_export_job_status(
                        &job.id,
                        ExportJobStatus::Failed,
                        Some(&message),
                        None,
                    )
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
            "telegram" => self.plan_telegram_job(job, &target, &pack.sticker_pack),
            kind => Err(ExportWorkerError::UnsupportedTargetKind {
                kind: kind.to_owned(),
            }),
        }
    }

    fn plan_telegram_job(
        &self,
        job: &ExportJobRecord,
        target: &ExportTargetRecord,
        pack: &msm_domain::StickerPack,
    ) -> ExportWorkerResult<WorkerJobResult> {
        let target_config: TelegramTargetConfigJson = serde_json::from_str(&target.config_json)?;
        let request: WorkerExportJobRequest = serde_json::from_str(&job.request_json)?;
        let options = request.options;
        let set_type = match options.set_type.as_deref() {
            Some("customEmoji" | "custom_emoji") => TelegramStickerSetType::CustomEmoji,
            _ => TelegramStickerSetType::Regular,
        };
        let plan = TelegramExportPlanner::plan_pack(
            pack,
            TelegramExportOptions {
                target: TelegramTargetConfig {
                    bot_username: target_config.bot_username,
                    owner_user_id: target_config.owner_user_id,
                },
                set_name_slug: options.set_name_slug.unwrap_or_else(|| pack.title.clone()),
                set_title: options.set_title.unwrap_or_else(|| pack.title.clone()),
                set_type,
                default_emoji: options.default_emoji.unwrap_or_else(|| "😀".to_owned()),
                existing_sticker_set_names: options.existing_sticker_set_names,
            },
        )?;

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
            dry_run: true,
        })
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
    #[serde(default)]
    existing_sticker_set_names: Vec<String>,
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
        dry_run: bool,
    },
}
