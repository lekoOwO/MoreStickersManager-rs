ALTER TABLE tenants
    ADD COLUMN local_registration_enabled BOOLEAN NOT NULL DEFAULT TRUE;
