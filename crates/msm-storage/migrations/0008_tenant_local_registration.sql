ALTER TABLE tenants
    ADD COLUMN local_registration_enabled INTEGER NOT NULL DEFAULT 1;
