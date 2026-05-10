CREATE TABLE subscription_access_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    owner_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    resource_type TEXT NOT NULL CHECK (resource_type IN ('pack', 'subscription_group')),
    resource_id TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    revoked_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_subscription_access_tokens_owner
    ON subscription_access_tokens(owner_user_id, created_at);

CREATE INDEX idx_subscription_access_tokens_resource
    ON subscription_access_tokens(resource_type, resource_id);
