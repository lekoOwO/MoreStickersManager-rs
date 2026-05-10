CREATE TABLE oidc_provider_configs (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    issuer_url TEXT NOT NULL,
    client_id TEXT NOT NULL,
    client_secret TEXT NOT NULL,
    scopes_json TEXT NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    allow_registration BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_oidc_provider_configs_tenant_id
    ON oidc_provider_configs(tenant_id);
