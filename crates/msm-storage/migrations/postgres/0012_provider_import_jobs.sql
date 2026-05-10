CREATE TABLE provider_import_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    owner_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL,
    remote_id TEXT NOT NULL,
    target_pack_id TEXT,
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    request_json TEXT NOT NULL,
    result_json TEXT,
    error_summary TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_attempt_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE provider_import_job_events (
    job_id TEXT NOT NULL REFERENCES provider_import_jobs(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL,
    level TEXT NOT NULL,
    stage TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (job_id, sequence)
);

CREATE INDEX provider_import_jobs_tenant_owner_idx
    ON provider_import_jobs (tenant_id, owner_user_id, created_at);

CREATE INDEX provider_import_jobs_status_idx
    ON provider_import_jobs (status, next_attempt_at, created_at);
