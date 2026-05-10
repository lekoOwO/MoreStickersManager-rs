use chrono::Utc;
use sqlx::Row;

use crate::{
    models::{
        ExportJobEventRecord, ExportJobRecord, ExportJobStatus, ExportTargetRecord, NewExportJob,
        NewExportJobEvent, NewExportTarget, NewPreparedMediaAsset, NewProviderImportJob,
        NewProviderImportJobEvent, NewTelegramPublication, NewTelegramStickerMapping,
        PreparedMediaAssetRecord, ProviderImportJobEventRecord, ProviderImportJobRecord,
        TelegramPublicationRecord, TelegramStickerMappingRecord,
    },
    StorageError, StorageRepository, StorageResult,
};

impl StorageRepository {
    /// Creates an export target.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn create_export_target(
        &self,
        target: NewExportTarget<'_>,
    ) -> StorageResult<ExportTargetRecord> {
        let now = now();
        sqlx::query(
            "INSERT INTO export_targets (
                id, tenant_id, kind, name, config_json, is_enabled, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(target.id)
        .bind(target.tenant_id)
        .bind(target.kind)
        .bind(target.name)
        .bind(target.config_json)
        .bind(i64::from(target.is_enabled))
        .bind(&now)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

        Ok(ExportTargetRecord {
            id: target.id.to_owned(),
            tenant_id: target.tenant_id.to_owned(),
            kind: target.kind.to_owned(),
            name: target.name.to_owned(),
            config_json: target.config_json.to_owned(),
            is_enabled: target.is_enabled,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Lists export targets for one tenant.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn list_export_targets(
        &self,
        tenant_id: &str,
    ) -> StorageResult<Vec<ExportTargetRecord>> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, kind, name, config_json, is_enabled, created_at, updated_at
            FROM export_targets
            WHERE tenant_id = ?
            ORDER BY name, id",
        )
        .bind(tenant_id)
        .fetch_all(self.sqlite()?)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| export_target_from_row(&row))
            .collect())
    }

    /// Finds an export target by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn find_export_target(&self, id: &str) -> StorageResult<Option<ExportTargetRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, kind, name, config_json, is_enabled, created_at, updated_at
            FROM export_targets
            WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.sqlite()?)
        .await?;

        Ok(row.map(|row| export_target_from_row(&row)))
    }

    /// Updates an export target.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn update_export_target(
        &self,
        id: &str,
        name: &str,
        config_json: &str,
        is_enabled: bool,
    ) -> StorageResult<Option<ExportTargetRecord>> {
        let result = sqlx::query(
            "UPDATE export_targets
            SET name = ?, config_json = ?, is_enabled = ?, updated_at = ?
            WHERE id = ?",
        )
        .bind(name)
        .bind(config_json)
        .bind(i64::from(is_enabled))
        .bind(now())
        .bind(id)
        .execute(self.sqlite()?)
        .await?;

        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.find_export_target(id).await
        }
    }

    /// Deletes an export target.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn delete_export_target(&self, id: &str) -> StorageResult<bool> {
        let result = sqlx::query("DELETE FROM export_targets WHERE id = ?")
            .bind(id)
            .execute(self.sqlite()?)
            .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Creates a queued export job.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn create_export_job(&self, job: NewExportJob<'_>) -> StorageResult<ExportJobRecord> {
        let now = now();
        let status = ExportJobStatus::Queued;
        sqlx::query(
            "INSERT INTO export_jobs (
                id, tenant_id, owner_user_id, source_pack_id, target_id, status,
                request_json, result_json, error_summary, attempt_count, max_attempts,
                next_attempt_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL, 0, ?, NULL, ?, ?)",
        )
        .bind(job.id)
        .bind(job.tenant_id)
        .bind(job.owner_user_id)
        .bind(job.source_pack_id)
        .bind(job.target_id)
        .bind(status.as_str())
        .bind(job.request_json)
        .bind(job.max_attempts)
        .bind(&now)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

        Ok(ExportJobRecord {
            id: job.id.to_owned(),
            tenant_id: job.tenant_id.to_owned(),
            owner_user_id: job.owner_user_id.to_owned(),
            source_pack_id: job.source_pack_id.to_owned(),
            target_id: job.target_id.to_owned(),
            status,
            request_json: job.request_json.to_owned(),
            result_json: None,
            error_summary: None,
            attempt_count: 0,
            max_attempts: job.max_attempts,
            next_attempt_at: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Finds an export job by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, or stored status
    /// is invalid.
    pub async fn find_export_job(&self, id: &str) -> StorageResult<Option<ExportJobRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, source_pack_id, target_id, status,
                request_json, result_json, error_summary, attempt_count, max_attempts,
                next_attempt_at, created_at, updated_at
            FROM export_jobs
            WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.sqlite()?)
        .await?;

        row.map(|row| export_job_from_row(&row)).transpose()
    }

    /// Finds the oldest export job with the requested status.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, or stored status
    /// is invalid.
    pub async fn find_next_export_job_by_status(
        &self,
        status: ExportJobStatus,
    ) -> StorageResult<Option<ExportJobRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, source_pack_id, target_id, status,
                request_json, result_json, error_summary, attempt_count, max_attempts,
                next_attempt_at, created_at, updated_at
            FROM export_jobs
            WHERE status = ?
            ORDER BY created_at, id
            LIMIT 1",
        )
        .bind(status.as_str())
        .fetch_optional(self.sqlite()?)
        .await?;

        row.map(|row| export_job_from_row(&row)).transpose()
    }

    /// Finds the oldest queued export job whose retry backoff has elapsed.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, or stored status
    /// is invalid.
    pub async fn find_next_due_export_job(
        &self,
        now: &str,
    ) -> StorageResult<Option<ExportJobRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, source_pack_id, target_id, status,
                request_json, result_json, error_summary, attempt_count, max_attempts,
                next_attempt_at, created_at, updated_at
            FROM export_jobs
            WHERE status = ? AND (next_attempt_at IS NULL OR next_attempt_at <= ?)
            ORDER BY created_at, id
            LIMIT 1",
        )
        .bind(ExportJobStatus::Queued.as_str())
        .bind(now)
        .fetch_optional(self.sqlite()?)
        .await?;

        row.map(|row| export_job_from_row(&row)).transpose()
    }

    /// Updates an export job status and optional payload fields.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn update_export_job_status(
        &self,
        id: &str,
        status: ExportJobStatus,
        error_summary: Option<&str>,
        result_json: Option<&str>,
    ) -> StorageResult<bool> {
        let result = sqlx::query(
            "UPDATE export_jobs
            SET status = ?, error_summary = ?, result_json = ?, next_attempt_at = NULL, updated_at = ?
            WHERE id = ?",
        )
        .bind(status.as_str())
        .bind(error_summary)
        .bind(result_json)
        .bind(now())
        .bind(id)
        .execute(self.sqlite()?)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Records a failed attempt and requeues the export job for a later retry.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn record_export_job_retry(
        &self,
        id: &str,
        error_summary: &str,
        next_attempt_at: &str,
    ) -> StorageResult<Option<ExportJobRecord>> {
        let result = sqlx::query(
            "UPDATE export_jobs
            SET status = ?, error_summary = ?, result_json = NULL,
                attempt_count = attempt_count + 1, next_attempt_at = ?, updated_at = ?
            WHERE id = ?",
        )
        .bind(ExportJobStatus::Queued.as_str())
        .bind(error_summary)
        .bind(next_attempt_at)
        .bind(now())
        .bind(id)
        .execute(self.sqlite()?)
        .await?;

        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.find_export_job(id).await
        }
    }

    /// Records a terminal failed attempt.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn record_export_job_failure(
        &self,
        id: &str,
        error_summary: &str,
    ) -> StorageResult<Option<ExportJobRecord>> {
        let result = sqlx::query(
            "UPDATE export_jobs
            SET status = ?, error_summary = ?, result_json = NULL,
                attempt_count = attempt_count + 1, next_attempt_at = NULL, updated_at = ?
            WHERE id = ?",
        )
        .bind(ExportJobStatus::Failed.as_str())
        .bind(error_summary)
        .bind(now())
        .bind(id)
        .execute(self.sqlite()?)
        .await?;

        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.find_export_job(id).await
        }
    }

    /// Appends an ordered export job event.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn append_export_job_event(
        &self,
        event: NewExportJobEvent<'_>,
    ) -> StorageResult<ExportJobEventRecord> {
        let now = now();
        sqlx::query(
            "INSERT INTO export_job_events (
                job_id, sequence, level, stage, message, metadata_json, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(event.job_id)
        .bind(event.sequence)
        .bind(event.level)
        .bind(event.stage)
        .bind(event.message)
        .bind(event.metadata_json)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

        Ok(ExportJobEventRecord {
            job_id: event.job_id.to_owned(),
            sequence: event.sequence,
            level: event.level.to_owned(),
            stage: event.stage.to_owned(),
            message: event.message.to_owned(),
            metadata_json: event.metadata_json.to_owned(),
            created_at: now,
        })
    }

    /// Lists export job events by ascending sequence.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn list_export_job_events(
        &self,
        job_id: &str,
    ) -> StorageResult<Vec<ExportJobEventRecord>> {
        let rows = sqlx::query(
            "SELECT job_id, sequence, level, stage, message, metadata_json, created_at
            FROM export_job_events
            WHERE job_id = ?
            ORDER BY sequence",
        )
        .bind(job_id)
        .fetch_all(self.sqlite()?)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ExportJobEventRecord {
                job_id: row.get("job_id"),
                sequence: row.get("sequence"),
                level: row.get("level"),
                stage: row.get("stage"),
                message: row.get("message"),
                metadata_json: row.get("metadata_json"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    /// Creates a queued provider import job.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn create_provider_import_job(
        &self,
        job: NewProviderImportJob<'_>,
    ) -> StorageResult<ProviderImportJobRecord> {
        let now = now();
        let status = ExportJobStatus::Queued;
        sqlx::query(
            "INSERT INTO provider_import_jobs (
                id, tenant_id, owner_user_id, provider_id, remote_id, target_pack_id, status,
                request_json, result_json, error_summary, attempt_count, max_attempts,
                next_attempt_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, 0, ?, NULL, ?, ?)",
        )
        .bind(job.id)
        .bind(job.tenant_id)
        .bind(job.owner_user_id)
        .bind(job.provider_id)
        .bind(job.remote_id)
        .bind(job.target_pack_id)
        .bind(status.as_str())
        .bind(job.request_json)
        .bind(job.max_attempts)
        .bind(&now)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

        Ok(ProviderImportJobRecord {
            id: job.id.to_owned(),
            tenant_id: job.tenant_id.to_owned(),
            owner_user_id: job.owner_user_id.to_owned(),
            provider_id: job.provider_id.to_owned(),
            remote_id: job.remote_id.to_owned(),
            target_pack_id: job.target_pack_id.map(str::to_owned),
            status,
            request_json: job.request_json.to_owned(),
            result_json: None,
            error_summary: None,
            attempt_count: 0,
            max_attempts: job.max_attempts,
            next_attempt_at: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Finds a provider import job by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, or stored status
    /// is invalid.
    pub async fn find_provider_import_job(
        &self,
        id: &str,
    ) -> StorageResult<Option<ProviderImportJobRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, provider_id, remote_id, target_pack_id, status,
                request_json, result_json, error_summary, attempt_count, max_attempts,
                next_attempt_at, created_at, updated_at
            FROM provider_import_jobs
            WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.sqlite()?)
        .await?;

        row.map(|row| provider_import_job_from_row(&row))
            .transpose()
    }

    /// Finds the oldest queued provider import job whose retry backoff has elapsed.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, or stored status
    /// is invalid.
    pub async fn find_next_due_provider_import_job(
        &self,
        now: &str,
    ) -> StorageResult<Option<ProviderImportJobRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, provider_id, remote_id, target_pack_id, status,
                request_json, result_json, error_summary, attempt_count, max_attempts,
                next_attempt_at, created_at, updated_at
            FROM provider_import_jobs
            WHERE status = ? AND (next_attempt_at IS NULL OR next_attempt_at <= ?)
            ORDER BY created_at, id
            LIMIT 1",
        )
        .bind(ExportJobStatus::Queued.as_str())
        .bind(now)
        .fetch_optional(self.sqlite()?)
        .await?;

        row.map(|row| provider_import_job_from_row(&row))
            .transpose()
    }

    /// Updates a provider import job status and optional payload fields.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn update_provider_import_job_status(
        &self,
        id: &str,
        status: ExportJobStatus,
        error_summary: Option<&str>,
        result_json: Option<&str>,
    ) -> StorageResult<bool> {
        let result = sqlx::query(
            "UPDATE provider_import_jobs
            SET status = ?, error_summary = ?, result_json = ?, next_attempt_at = NULL, updated_at = ?
            WHERE id = ?",
        )
        .bind(status.as_str())
        .bind(error_summary)
        .bind(result_json)
        .bind(now())
        .bind(id)
        .execute(self.sqlite()?)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Records a failed provider import attempt and requeues the job.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn record_provider_import_job_retry(
        &self,
        id: &str,
        error_summary: &str,
        next_attempt_at: &str,
    ) -> StorageResult<Option<ProviderImportJobRecord>> {
        let result = sqlx::query(
            "UPDATE provider_import_jobs
            SET status = ?, error_summary = ?, result_json = NULL,
                attempt_count = attempt_count + 1, next_attempt_at = ?, updated_at = ?
            WHERE id = ?",
        )
        .bind(ExportJobStatus::Queued.as_str())
        .bind(error_summary)
        .bind(next_attempt_at)
        .bind(now())
        .bind(id)
        .execute(self.sqlite()?)
        .await?;

        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.find_provider_import_job(id).await
        }
    }

    /// Records a terminal provider import failure.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn record_provider_import_job_failure(
        &self,
        id: &str,
        error_summary: &str,
    ) -> StorageResult<Option<ProviderImportJobRecord>> {
        let result = sqlx::query(
            "UPDATE provider_import_jobs
            SET status = ?, error_summary = ?, result_json = NULL,
                attempt_count = attempt_count + 1, next_attempt_at = NULL, updated_at = ?
            WHERE id = ?",
        )
        .bind(ExportJobStatus::Failed.as_str())
        .bind(error_summary)
        .bind(now())
        .bind(id)
        .execute(self.sqlite()?)
        .await?;

        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.find_provider_import_job(id).await
        }
    }

    /// Appends an ordered provider import job event.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn append_provider_import_job_event(
        &self,
        event: NewProviderImportJobEvent<'_>,
    ) -> StorageResult<ProviderImportJobEventRecord> {
        let now = now();
        sqlx::query(
            "INSERT INTO provider_import_job_events (
                job_id, sequence, level, stage, message, metadata_json, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(event.job_id)
        .bind(event.sequence)
        .bind(event.level)
        .bind(event.stage)
        .bind(event.message)
        .bind(event.metadata_json)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

        Ok(ProviderImportJobEventRecord {
            job_id: event.job_id.to_owned(),
            sequence: event.sequence,
            level: event.level.to_owned(),
            stage: event.stage.to_owned(),
            message: event.message.to_owned(),
            metadata_json: event.metadata_json.to_owned(),
            created_at: now,
        })
    }

    /// Lists provider import job events by ascending sequence.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn list_provider_import_job_events(
        &self,
        job_id: &str,
    ) -> StorageResult<Vec<ProviderImportJobEventRecord>> {
        let rows = sqlx::query(
            "SELECT job_id, sequence, level, stage, message, metadata_json, created_at
            FROM provider_import_job_events
            WHERE job_id = ?
            ORDER BY sequence",
        )
        .bind(job_id)
        .fetch_all(self.sqlite()?)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ProviderImportJobEventRecord {
                job_id: row.get("job_id"),
                sequence: row.get("sequence"),
                level: row.get("level"),
                stage: row.get("stage"),
                message: row.get("message"),
                metadata_json: row.get("metadata_json"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    /// Inserts or replaces a prepared media cache record.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn upsert_prepared_media_asset(
        &self,
        asset: NewPreparedMediaAsset<'_>,
    ) -> StorageResult<PreparedMediaAssetRecord> {
        let now = now();
        sqlx::query(
            "INSERT INTO prepared_media_assets (
                source_asset_hash, profile_key, output_asset_key, mime_type,
                width_px, height_px, duration_ms, file_size_bytes, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(source_asset_hash, profile_key) DO UPDATE SET
                output_asset_key = excluded.output_asset_key,
                mime_type = excluded.mime_type,
                width_px = excluded.width_px,
                height_px = excluded.height_px,
                duration_ms = excluded.duration_ms,
                file_size_bytes = excluded.file_size_bytes,
                updated_at = excluded.updated_at",
        )
        .bind(asset.source_asset_hash)
        .bind(asset.profile_key)
        .bind(asset.output_asset_key)
        .bind(asset.mime_type)
        .bind(asset.width_px)
        .bind(asset.height_px)
        .bind(asset.duration_ms)
        .bind(asset.file_size_bytes)
        .bind(&now)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

        self.find_prepared_media_asset(asset.source_asset_hash, asset.profile_key)
            .await?
            .ok_or_else(|| StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Finds a prepared media cache record by source hash and profile key.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn find_prepared_media_asset(
        &self,
        source_asset_hash: &str,
        profile_key: &str,
    ) -> StorageResult<Option<PreparedMediaAssetRecord>> {
        let row = sqlx::query(
            "SELECT source_asset_hash, profile_key, output_asset_key, mime_type,
                width_px, height_px, duration_ms, file_size_bytes, created_at, updated_at
            FROM prepared_media_assets
            WHERE source_asset_hash = ? AND profile_key = ?",
        )
        .bind(source_asset_hash)
        .bind(profile_key)
        .fetch_optional(self.sqlite()?)
        .await?;

        Ok(row.map(|row| PreparedMediaAssetRecord {
            source_asset_hash: row.get("source_asset_hash"),
            profile_key: row.get("profile_key"),
            output_asset_key: row.get("output_asset_key"),
            mime_type: row.get("mime_type"),
            width_px: row.get("width_px"),
            height_px: row.get("height_px"),
            duration_ms: row.get("duration_ms"),
            file_size_bytes: row.get("file_size_bytes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    /// Inserts or updates a Telegram publication by `(target_id, sticker_set_name)`.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn upsert_telegram_publication(
        &self,
        publication: NewTelegramPublication<'_>,
    ) -> StorageResult<TelegramPublicationRecord> {
        let now = now();
        sqlx::query(
            "INSERT INTO telegram_publications (
                id, pack_id, target_id, job_id, sticker_set_name, sticker_set_url,
                sticker_count, sticker_type, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(target_id, sticker_set_name) DO UPDATE SET
                pack_id = excluded.pack_id,
                job_id = excluded.job_id,
                sticker_set_url = excluded.sticker_set_url,
                sticker_count = excluded.sticker_count,
                sticker_type = excluded.sticker_type,
                updated_at = excluded.updated_at",
        )
        .bind(publication.id)
        .bind(publication.pack_id)
        .bind(publication.target_id)
        .bind(publication.job_id)
        .bind(publication.sticker_set_name)
        .bind(publication.sticker_set_url)
        .bind(publication.sticker_count)
        .bind(publication.sticker_type)
        .bind(&now)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

        self.find_telegram_publication_by_target_set(
            publication.target_id,
            publication.sticker_set_name,
        )
        .await?
        .ok_or_else(|| StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Finds a Telegram publication by its stable ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn find_telegram_publication(
        &self,
        id: &str,
    ) -> StorageResult<Option<TelegramPublicationRecord>> {
        let row = sqlx::query(
            "SELECT id, pack_id, target_id, job_id, sticker_set_name, sticker_set_url,
                sticker_count, sticker_type, created_at, updated_at
            FROM telegram_publications
            WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.sqlite()?)
        .await?;

        Ok(row.map(|row| telegram_publication_from_row(&row)))
    }

    /// Finds a Telegram publication by target and sticker set name.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn find_telegram_publication_by_target_set(
        &self,
        target_id: &str,
        sticker_set_name: &str,
    ) -> StorageResult<Option<TelegramPublicationRecord>> {
        let row = sqlx::query(
            "SELECT id, pack_id, target_id, job_id, sticker_set_name, sticker_set_url,
                sticker_count, sticker_type, created_at, updated_at
            FROM telegram_publications
            WHERE target_id = ? AND sticker_set_name = ?",
        )
        .bind(target_id)
        .bind(sticker_set_name)
        .fetch_optional(self.sqlite()?)
        .await?;

        Ok(row.map(|row| telegram_publication_from_row(&row)))
    }

    /// Lists Telegram publications for one source pack.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn list_telegram_publications_for_pack(
        &self,
        pack_id: &str,
    ) -> StorageResult<Vec<TelegramPublicationRecord>> {
        let rows = sqlx::query(
            "SELECT id, pack_id, target_id, job_id, sticker_set_name, sticker_set_url,
                sticker_count, sticker_type, created_at, updated_at
            FROM telegram_publications
            WHERE pack_id = ?
            ORDER BY updated_at DESC, sticker_set_name, id",
        )
        .bind(pack_id)
        .fetch_all(self.sqlite()?)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| telegram_publication_from_row(&row))
            .collect())
    }

    /// Inserts or updates a Telegram sticker mapping by target, set, and source sticker.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn upsert_telegram_sticker_mapping(
        &self,
        mapping: NewTelegramStickerMapping<'_>,
    ) -> StorageResult<TelegramStickerMappingRecord> {
        let now = now();
        let id = telegram_sticker_mapping_id(
            mapping.target_id,
            mapping.sticker_set_name,
            mapping.source_sticker_id,
        );
        sqlx::query(
            "INSERT INTO telegram_sticker_mappings (
                id, publication_id, target_id, sticker_set_name, source_sticker_id,
                telegram_file_id, telegram_file_unique_id, position, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(target_id, sticker_set_name, source_sticker_id) DO UPDATE SET
                publication_id = excluded.publication_id,
                telegram_file_id = excluded.telegram_file_id,
                telegram_file_unique_id = excluded.telegram_file_unique_id,
                position = excluded.position,
                updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(mapping.publication_id)
        .bind(mapping.target_id)
        .bind(mapping.sticker_set_name)
        .bind(mapping.source_sticker_id)
        .bind(mapping.telegram_file_id)
        .bind(mapping.telegram_file_unique_id)
        .bind(mapping.position)
        .bind(&now)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

        self.find_telegram_sticker_mapping_by_source(
            mapping.target_id,
            mapping.sticker_set_name,
            mapping.source_sticker_id,
        )
        .await?
        .ok_or_else(|| StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Finds a Telegram sticker mapping by target, set, and source sticker ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn find_telegram_sticker_mapping_by_source(
        &self,
        target_id: &str,
        sticker_set_name: &str,
        source_sticker_id: &str,
    ) -> StorageResult<Option<TelegramStickerMappingRecord>> {
        let row = sqlx::query(
            "SELECT id, publication_id, target_id, sticker_set_name, source_sticker_id,
                telegram_file_id, telegram_file_unique_id, position, created_at, updated_at
            FROM telegram_sticker_mappings
            WHERE target_id = ? AND sticker_set_name = ? AND source_sticker_id = ?",
        )
        .bind(target_id)
        .bind(sticker_set_name)
        .bind(source_sticker_id)
        .fetch_optional(self.sqlite()?)
        .await?;

        Ok(row.map(|row| telegram_sticker_mapping_from_row(&row)))
    }

    /// Lists Telegram sticker mappings for one publication.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn list_telegram_sticker_mappings_for_publication(
        &self,
        publication_id: &str,
    ) -> StorageResult<Vec<TelegramStickerMappingRecord>> {
        let rows = sqlx::query(
            "SELECT id, publication_id, target_id, sticker_set_name, source_sticker_id,
                telegram_file_id, telegram_file_unique_id, position, created_at, updated_at
            FROM telegram_sticker_mappings
            WHERE publication_id = ?
            ORDER BY position, source_sticker_id",
        )
        .bind(publication_id)
        .fetch_all(self.sqlite()?)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| telegram_sticker_mapping_from_row(&row))
            .collect())
    }
}

fn export_job_from_row(row: &sqlx::sqlite::SqliteRow) -> StorageResult<ExportJobRecord> {
    let status_value: String = row.get("status");
    let status = ExportJobStatus::from_storage(&status_value).ok_or(
        StorageError::InvalidExportJobStatus {
            status: status_value,
        },
    )?;

    Ok(ExportJobRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        source_pack_id: row.get("source_pack_id"),
        target_id: row.get("target_id"),
        status,
        request_json: row.get("request_json"),
        result_json: row.get("result_json"),
        error_summary: row.get("error_summary"),
        attempt_count: row.get("attempt_count"),
        max_attempts: row.get("max_attempts"),
        next_attempt_at: row.get("next_attempt_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn provider_import_job_from_row(
    row: &sqlx::sqlite::SqliteRow,
) -> StorageResult<ProviderImportJobRecord> {
    let status_value: String = row.get("status");
    let status = ExportJobStatus::from_storage(&status_value).ok_or(
        StorageError::InvalidExportJobStatus {
            status: status_value,
        },
    )?;

    Ok(ProviderImportJobRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        provider_id: row.get("provider_id"),
        remote_id: row.get("remote_id"),
        target_pack_id: row.get("target_pack_id"),
        status,
        request_json: row.get("request_json"),
        result_json: row.get("result_json"),
        error_summary: row.get("error_summary"),
        attempt_count: row.get("attempt_count"),
        max_attempts: row.get("max_attempts"),
        next_attempt_at: row.get("next_attempt_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn export_target_from_row(row: &sqlx::sqlite::SqliteRow) -> ExportTargetRecord {
    let is_enabled: i64 = row.get("is_enabled");
    ExportTargetRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        kind: row.get("kind"),
        name: row.get("name"),
        config_json: row.get("config_json"),
        is_enabled: is_enabled != 0,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn telegram_publication_from_row(row: &sqlx::sqlite::SqliteRow) -> TelegramPublicationRecord {
    TelegramPublicationRecord {
        id: row.get("id"),
        pack_id: row.get("pack_id"),
        target_id: row.get("target_id"),
        job_id: row.get("job_id"),
        sticker_set_name: row.get("sticker_set_name"),
        sticker_set_url: row.get("sticker_set_url"),
        sticker_count: row.get("sticker_count"),
        sticker_type: row.get("sticker_type"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn telegram_sticker_mapping_from_row(
    row: &sqlx::sqlite::SqliteRow,
) -> TelegramStickerMappingRecord {
    TelegramStickerMappingRecord {
        id: row.get("id"),
        publication_id: row.get("publication_id"),
        target_id: row.get("target_id"),
        sticker_set_name: row.get("sticker_set_name"),
        source_sticker_id: row.get("source_sticker_id"),
        telegram_file_id: row.get("telegram_file_id"),
        telegram_file_unique_id: row.get("telegram_file_unique_id"),
        position: row.get("position"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn telegram_sticker_mapping_id(
    target_id: &str,
    sticker_set_name: &str,
    source_sticker_id: &str,
) -> String {
    format!("telegram-sticker:{target_id}:{sticker_set_name}:{source_sticker_id}")
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use crate::{
        models::{ExportJobStatus, NewProviderImportJob, NewProviderImportJobEvent},
        DatabaseConfig, DbPool, StorageRepository,
    };

    #[tokio::test]
    async fn provider_import_jobs_can_be_created_and_read_with_events() {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        let repo = StorageRepository::new(pool);

        repo.create_tenant("tenant_1", "Tenant").await.unwrap();
        repo.create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();

        let job = repo
            .create_provider_import_job(NewProviderImportJob {
                id: "provider_import_1",
                tenant_id: "tenant_1",
                owner_user_id: "user_1",
                provider_id: "line-stickers",
                remote_id: "12345",
                target_pack_id: Some("pack_line_12345"),
                request_json: r#"{"providerId":"line-stickers"}"#,
                max_attempts: 3,
            })
            .await
            .unwrap();
        repo.append_provider_import_job_event(NewProviderImportJobEvent {
            job_id: "provider_import_1",
            sequence: 1,
            level: "info",
            stage: "queued",
            message: "Provider import queued.",
            metadata_json: "{}",
        })
        .await
        .unwrap();

        let found = repo
            .find_provider_import_job("provider_import_1")
            .await
            .unwrap()
            .unwrap();
        let events = repo
            .list_provider_import_job_events("provider_import_1")
            .await
            .unwrap();
        let updated = repo
            .update_provider_import_job_status(
                "provider_import_1",
                ExportJobStatus::Succeeded,
                None,
                Some(r#"{"packId":"pack_line_12345"}"#),
            )
            .await
            .unwrap();
        let due = repo
            .find_next_due_provider_import_job(&chrono::Utc::now().to_rfc3339())
            .await
            .unwrap();

        assert_eq!(job.status, ExportJobStatus::Queued);
        assert_eq!(found.provider_id, "line-stickers");
        assert_eq!(found.target_pack_id.as_deref(), Some("pack_line_12345"));
        assert_eq!(events[0].stage, "queued");
        assert!(updated);
        assert!(due.is_none());
    }
}
