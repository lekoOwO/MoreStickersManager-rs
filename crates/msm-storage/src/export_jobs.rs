use chrono::Utc;
use sqlx::Row;

use crate::{
    models::{
        ExportJobEventRecord, ExportJobRecord, ExportJobStatus, ExportTargetRecord, NewExportJob,
        NewExportJobEvent, NewExportTarget, NewPreparedMediaAsset, PreparedMediaAssetRecord,
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
                request_json, result_json, error_summary, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL, ?, ?)",
        )
        .bind(job.id)
        .bind(job.tenant_id)
        .bind(job.owner_user_id)
        .bind(job.source_pack_id)
        .bind(job.target_id)
        .bind(status.as_str())
        .bind(job.request_json)
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
                request_json, result_json, error_summary, created_at, updated_at
            FROM export_jobs
            WHERE id = ?",
        )
        .bind(id)
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
            SET status = ?, error_summary = ?, result_json = ?, updated_at = ?
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
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn now() -> String {
    Utc::now().to_rfc3339()
}
