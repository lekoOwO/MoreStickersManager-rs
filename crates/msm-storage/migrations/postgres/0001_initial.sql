CREATE TABLE tenants (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    public_asset_url TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    is_disabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL
);

CREATE TABLE tenant_members (
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, user_id)
);

CREATE TABLE roles (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE permissions (
    key TEXT PRIMARY KEY NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE role_permissions (
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_key TEXT NOT NULL REFERENCES permissions(key) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_key)
);

CREATE TABLE sticker_packs (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    owner_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    compatibility_id TEXT NOT NULL,
    title TEXT NOT NULL,
    visibility TEXT NOT NULL CHECK (visibility IN ('public', 'private')),
    source_provider TEXT,
    sticker_pack_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, compatibility_id)
);

CREATE TABLE stickers (
    id TEXT PRIMARY KEY NOT NULL,
    pack_id TEXT NOT NULL REFERENCES sticker_packs(id) ON DELETE CASCADE,
    compatibility_id TEXT NOT NULL,
    title TEXT NOT NULL,
    asset_key TEXT,
    image_url TEXT NOT NULL,
    is_animated BOOLEAN,
    sort_order INTEGER NOT NULL,
    UNIQUE (pack_id, compatibility_id)
);

CREATE TABLE folders (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    owner_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE folder_packs (
    folder_id TEXT NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    pack_id TEXT NOT NULL REFERENCES sticker_packs(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (folder_id, pack_id)
);

CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (tenant_id, name)
);

CREATE TABLE pack_tags (
    pack_id TEXT NOT NULL REFERENCES sticker_packs(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (pack_id, tag_id)
);

CREATE TABLE subscription_groups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    owner_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    visibility TEXT NOT NULL CHECK (visibility IN ('public', 'private')),
    created_at TEXT NOT NULL
);

CREATE TABLE subscription_group_packs (
    subscription_group_id TEXT NOT NULL REFERENCES subscription_groups(id) ON DELETE CASCADE,
    pack_id TEXT NOT NULL REFERENCES sticker_packs(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (subscription_group_id, pack_id)
);

CREATE TABLE system_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE personal_access_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    scopes_json TEXT NOT NULL,
    expires_at TEXT,
    revoked_at TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT REFERENCES tenants(id) ON DELETE SET NULL,
    actor_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT,
    metadata_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);
