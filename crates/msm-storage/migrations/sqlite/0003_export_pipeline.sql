CREATE TABLE export_targets (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    config_json TEXT NOT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE export_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    owner_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    source_pack_id TEXT NOT NULL REFERENCES sticker_packs(id) ON DELETE CASCADE,
    target_id TEXT NOT NULL REFERENCES export_targets(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    request_json TEXT NOT NULL,
    result_json TEXT,
    error_summary TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE export_job_events (
    job_id TEXT NOT NULL REFERENCES export_jobs(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL,
    level TEXT NOT NULL,
    stage TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (job_id, sequence)
);

CREATE TABLE prepared_media_assets (
    source_asset_hash TEXT NOT NULL,
    profile_key TEXT NOT NULL,
    output_asset_key TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    width_px INTEGER NOT NULL,
    height_px INTEGER NOT NULL,
    duration_ms INTEGER,
    file_size_bytes INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (source_asset_hash, profile_key)
);

CREATE TABLE telegram_publications (
    id TEXT PRIMARY KEY NOT NULL,
    pack_id TEXT NOT NULL REFERENCES sticker_packs(id) ON DELETE CASCADE,
    target_id TEXT NOT NULL REFERENCES export_targets(id) ON DELETE CASCADE,
    job_id TEXT NOT NULL REFERENCES export_jobs(id) ON DELETE CASCADE,
    sticker_set_name TEXT NOT NULL,
    sticker_set_url TEXT NOT NULL,
    sticker_count INTEGER NOT NULL,
    sticker_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (target_id, sticker_set_name)
);
