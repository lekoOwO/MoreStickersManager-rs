ALTER TABLE oidc_login_states
    ADD COLUMN nonce_hash TEXT NOT NULL DEFAULT '';
