ALTER TABLE tenants
    ALTER COLUMN local_registration_enabled SET DEFAULT FALSE;

UPDATE tenants
SET local_registration_enabled = FALSE;
