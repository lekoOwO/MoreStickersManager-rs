CREATE TABLE oidc_login_states (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL REFERENCES oidc_provider_configs(id) ON DELETE CASCADE,
    state_hash TEXT NOT NULL,
    redirect_uri TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    consumed_at TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_oidc_login_states_tenant_provider
    ON oidc_login_states(tenant_id, provider_id);

CREATE TABLE oidc_user_links (
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL REFERENCES oidc_provider_configs(id) ON DELETE CASCADE,
    provider_subject TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, provider_id, provider_subject)
);

CREATE INDEX idx_oidc_user_links_user_id
    ON oidc_user_links(user_id);
