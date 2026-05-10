ALTER TABLE export_jobs ADD COLUMN attempt_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE export_jobs ADD COLUMN max_attempts INTEGER NOT NULL DEFAULT 1;
ALTER TABLE export_jobs ADD COLUMN next_attempt_at TEXT;
