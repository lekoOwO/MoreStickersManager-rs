use std::collections::BTreeSet;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use msm_domain::{Permission, StickerPack};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgRow, sqlite::SqliteRow, PgPool, Row, SqlitePool};
use subtle::ConstantTimeEq;

use crate::{
    models::{
        CreatedOidcLoginState, CreatedPersonalAccessToken, CreatedSubscriptionAccessToken,
        CreatedWebSession, FolderPackRecord, FolderRecord, LocalUserCredentialRecord,
        NewOidcProviderConfig, NewTag, OidcLoginStateRecord, OidcProviderConfigRecord,
        OidcUserLinkRecord, PackTagRecord, PackVisibility, PersonalAccessTokenRecord, RoleRecord,
        StickerPackRecord, SubscriptionAccessResourceType, SubscriptionAccessTokenRecord,
        SubscriptionGroupPackRecord, SubscriptionGroupRecord, TagRecord, TenantMemberRecord,
        TenantRecord, UserRecord, WebSessionRecord,
    },
    DbPool, StorageError, StorageResult,
};

#[derive(Clone)]
pub struct StorageRepository {
    pool: DbPool,
}

impl StorageRepository {
    #[must_use]
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Checks whether the backing database is reachable.
    ///
    /// # Errors
    ///
    /// Returns an error when the database pool cannot execute a trivial query.
    pub async fn check(&self) -> StorageResult<()> {
        self.pool.check().await
    }

    /// Creates a tenant row.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn create_tenant(&self, id: &str, name: &str) -> StorageResult<()> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO tenants (id, name, local_registration_enabled, created_at)
                    VALUES (?, ?, ?, ?)",
                )
                .bind(id)
                .bind(name)
                .bind(false)
                .bind(now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO tenants (id, name, local_registration_enabled, created_at)
                    VALUES ($1, $2, $3, $4)",
                )
                .bind(id)
                .bind(name)
                .bind(false)
                .bind(now)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// Counts tenants in the database.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL execution fails.
    pub async fn count_tenants(&self) -> StorageResult<i64> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tenants")
                    .fetch_one(pool)
                    .await?)
            }
            DbPool::Postgres(pool) => {
                Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tenants")
                    .fetch_one(pool)
                    .await?)
            }
        }
    }

    /// Counts users in the database.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL execution fails.
    pub async fn count_users(&self) -> StorageResult<i64> {
        match &self.pool {
            DbPool::Sqlite(pool) => Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
                .fetch_one(pool)
                .await?),
            DbPool::Postgres(pool) => {
                Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
                    .fetch_one(pool)
                    .await?)
            }
        }
    }

    /// Finds a tenant by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn find_tenant(&self, id: &str) -> StorageResult<Option<TenantRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, name, public_asset_url, local_registration_enabled, created_at
                    FROM tenants
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(tenant_from_sqlite_row).transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, name, public_asset_url, local_registration_enabled, created_at
                    FROM tenants
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(tenant_from_pg_row).transpose()
            }
        }
    }

    /// Replaces editable tenant settings.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails, the tenant does not exist, or timestamps are invalid.
    pub async fn update_tenant_settings(
        &self,
        id: &str,
        name: &str,
        public_asset_url: Option<&str>,
        local_registration_enabled: bool,
    ) -> StorageResult<TenantRecord> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => sqlx::query(
                "UPDATE tenants
                SET name = ?, public_asset_url = ?, local_registration_enabled = ?
                WHERE id = ?",
            )
            .bind(name)
            .bind(public_asset_url)
            .bind(local_registration_enabled)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected(),
            DbPool::Postgres(pool) => sqlx::query(
                "UPDATE tenants
                SET name = $1, public_asset_url = $2, local_registration_enabled = $3
                WHERE id = $4",
            )
            .bind(name)
            .bind(public_asset_url)
            .bind(local_registration_enabled)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected(),
        };

        if rows_affected == 0 {
            return Err(StorageError::Sqlx(sqlx::Error::RowNotFound));
        }

        self.find_tenant(id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Creates or replaces an OIDC provider configuration for a tenant.
    ///
    /// # Errors
    ///
    /// Returns an error when JSON serialization fails, the referenced tenant does not exist, SQL
    /// fails, or timestamps are invalid.
    pub async fn upsert_oidc_provider_config(
        &self,
        config: NewOidcProviderConfig<'_>,
    ) -> StorageResult<OidcProviderConfigRecord> {
        let now = now();
        let scopes_json =
            serde_json::to_string(&config.scopes.iter().map(String::as_str).collect::<Vec<_>>())?;
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO oidc_provider_configs (
                        id, tenant_id, display_name, issuer_url, client_id, client_secret, scopes_json,
                        is_enabled, allow_registration, created_at, updated_at
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(id) DO UPDATE SET
                        tenant_id = excluded.tenant_id,
                        display_name = excluded.display_name,
                        issuer_url = excluded.issuer_url,
                        client_id = excluded.client_id,
                        client_secret = excluded.client_secret,
                        scopes_json = excluded.scopes_json,
                        is_enabled = excluded.is_enabled,
                        allow_registration = excluded.allow_registration,
                        updated_at = excluded.updated_at",
                )
                .bind(config.id)
                .bind(config.tenant_id)
                .bind(config.display_name)
                .bind(config.issuer_url)
                .bind(config.client_id)
                .bind(config.client_secret)
                .bind(&scopes_json)
                .bind(config.is_enabled)
                .bind(config.allow_registration)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO oidc_provider_configs (
                        id, tenant_id, display_name, issuer_url, client_id, client_secret, scopes_json,
                        is_enabled, allow_registration, created_at, updated_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                    ON CONFLICT(id) DO UPDATE SET
                        tenant_id = excluded.tenant_id,
                        display_name = excluded.display_name,
                        issuer_url = excluded.issuer_url,
                        client_id = excluded.client_id,
                        client_secret = excluded.client_secret,
                        scopes_json = excluded.scopes_json,
                        is_enabled = excluded.is_enabled,
                        allow_registration = excluded.allow_registration,
                        updated_at = excluded.updated_at",
                )
                .bind(config.id)
                .bind(config.tenant_id)
                .bind(config.display_name)
                .bind(config.issuer_url)
                .bind(config.client_id)
                .bind(config.client_secret)
                .bind(&scopes_json)
                .bind(config.is_enabled)
                .bind(config.allow_registration)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        self.find_oidc_provider_config(config.tenant_id, config.id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Lists OIDC provider configurations for a tenant.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails, JSON is invalid, or timestamps are invalid.
    pub async fn list_oidc_provider_configs(
        &self,
        tenant_id: &str,
    ) -> StorageResult<Vec<OidcProviderConfigRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, display_name, issuer_url, client_id, client_secret, scopes_json,
                        is_enabled, allow_registration, created_at, updated_at
                    FROM oidc_provider_configs
                    WHERE tenant_id = ?
                    ORDER BY display_name, id",
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?;
                rows.iter()
                    .map(oidc_provider_config_from_sqlite_row)
                    .collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, display_name, issuer_url, client_id, client_secret, scopes_json,
                        is_enabled, allow_registration, created_at, updated_at
                    FROM oidc_provider_configs
                    WHERE tenant_id = $1
                    ORDER BY display_name, id",
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?;
                rows.iter().map(oidc_provider_config_from_pg_row).collect()
            }
        }
    }

    /// Finds one OIDC provider configuration by tenant and ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails, JSON is invalid, or timestamps are invalid.
    pub async fn find_oidc_provider_config(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> StorageResult<Option<OidcProviderConfigRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, display_name, issuer_url, client_id, client_secret, scopes_json,
                        is_enabled, allow_registration, created_at, updated_at
                    FROM oidc_provider_configs
                    WHERE tenant_id = ? AND id = ?",
                )
                .bind(tenant_id)
                .bind(id)
                .fetch_optional(pool)
                .await?;
                row.as_ref()
                    .map(oidc_provider_config_from_sqlite_row)
                    .transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, display_name, issuer_url, client_id, client_secret, scopes_json,
                        is_enabled, allow_registration, created_at, updated_at
                    FROM oidc_provider_configs
                    WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tenant_id)
                .bind(id)
                .fetch_optional(pool)
                .await?;
                row.as_ref()
                    .map(oidc_provider_config_from_pg_row)
                    .transpose()
            }
        }
    }

    /// Deletes one OIDC provider configuration by tenant and ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn delete_oidc_provider_config(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> StorageResult<bool> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => {
                let result = sqlx::query(
                    "DELETE FROM oidc_provider_configs
                    WHERE tenant_id = ? AND id = ?",
                )
                .bind(tenant_id)
                .bind(id)
                .execute(pool)
                .await?;
                result.rows_affected()
            }
            DbPool::Postgres(pool) => {
                let result = sqlx::query(
                    "DELETE FROM oidc_provider_configs
                    WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tenant_id)
                .bind(id)
                .execute(pool)
                .await?;
                result.rows_affected()
            }
        };

        Ok(rows_affected > 0)
    }

    /// Creates an OIDC login state token and stores only its hash.
    ///
    /// # Errors
    ///
    /// Returns an error when random generation, timestamp conversion, or SQL fails.
    pub async fn create_oidc_login_state(
        &self,
        tenant_id: &str,
        provider_id: &str,
        redirect_uri: &str,
        expires_at: &str,
    ) -> StorageResult<CreatedOidcLoginState> {
        let id = uuid::Uuid::new_v4().to_string();
        let secret = generate_pat_secret()?;
        let state = format!("msm_oidc_state_{id}_{secret}");
        let state_hash = hash_pat_secret(&secret);
        let nonce_secret = generate_pat_secret()?;
        let nonce = format!("msm_oidc_nonce_{id}_{nonce_secret}");
        let nonce_hash = hash_pat_secret(&nonce_secret);
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO oidc_login_states (
                        id, tenant_id, provider_id, state_hash, nonce_hash, redirect_uri, expires_at,
                        created_at
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&id)
                .bind(tenant_id)
                .bind(provider_id)
                .bind(&state_hash)
                .bind(&nonce_hash)
                .bind(redirect_uri)
                .bind(expires_at)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO oidc_login_states (
                        id, tenant_id, provider_id, state_hash, nonce_hash, redirect_uri, expires_at,
                        created_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(&id)
                .bind(tenant_id)
                .bind(provider_id)
                .bind(&state_hash)
                .bind(&nonce_hash)
                .bind(redirect_uri)
                .bind(expires_at)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        Ok(CreatedOidcLoginState {
            record: OidcLoginStateRecord {
                id,
                tenant_id: tenant_id.to_owned(),
                provider_id: provider_id.to_owned(),
                state_hash,
                nonce_hash,
                redirect_uri: redirect_uri.to_owned(),
                expires_at: expires_at.to_owned(),
                consumed_at: None,
                created_at: now,
            },
            state,
            nonce,
        })
    }

    /// Verifies and consumes an OIDC state token once.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn verify_oidc_login_state(
        &self,
        state: &str,
        nonce: &str,
    ) -> StorageResult<Option<OidcLoginStateRecord>> {
        self.valid_oidc_login_state(state, nonce).await
    }

    /// Verifies and consumes an OIDC state token once.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn consume_oidc_login_state(
        &self,
        state: &str,
        nonce: &str,
    ) -> StorageResult<Option<OidcLoginStateRecord>> {
        let Some(record) = self.valid_oidc_login_state(state, nonce).await? else {
            return Ok(None);
        };
        let consumed_at = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE oidc_login_states
                    SET consumed_at = ?
                    WHERE id = ? AND consumed_at IS NULL",
                )
                .bind(&consumed_at)
                .bind(&record.id)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "UPDATE oidc_login_states
                    SET consumed_at = $1
                    WHERE id = $2 AND consumed_at IS NULL",
                )
                .bind(&consumed_at)
                .bind(&record.id)
                .execute(pool)
                .await?;
            }
        }

        Ok(Some(OidcLoginStateRecord {
            consumed_at: Some(consumed_at),
            ..record
        }))
    }

    async fn valid_oidc_login_state(
        &self,
        state: &str,
        nonce: &str,
    ) -> StorageResult<Option<OidcLoginStateRecord>> {
        let Some((id, secret)) = parse_oidc_login_state(state) else {
            return Ok(None);
        };
        let Some((nonce_id, nonce_secret)) = parse_oidc_nonce(nonce) else {
            return Ok(None);
        };
        if nonce_id != id {
            return Ok(None);
        }
        let Some(record) = self.find_oidc_login_state_by_id(id).await? else {
            return Ok(None);
        };
        if record.consumed_at.is_some() || is_expired(Some(&record.expires_at)) {
            return Ok(None);
        }
        let presented_hash = hash_pat_secret(secret);
        if presented_hash
            .as_bytes()
            .ct_eq(record.state_hash.as_bytes())
            .unwrap_u8()
            == 0
        {
            return Ok(None);
        }
        let presented_nonce_hash = hash_pat_secret(nonce_secret);
        if presented_nonce_hash
            .as_bytes()
            .ct_eq(record.nonce_hash.as_bytes())
            .unwrap_u8()
            == 0
        {
            return Ok(None);
        }
        Ok(Some(record))
    }

    /// Finds an OIDC user link by provider subject.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn find_oidc_user_link(
        &self,
        tenant_id: &str,
        provider_id: &str,
        provider_subject: &str,
    ) -> StorageResult<Option<OidcUserLinkRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT tenant_id, provider_id, provider_subject, user_id, email, display_name,
                        created_at, updated_at
                    FROM oidc_user_links
                    WHERE tenant_id = ? AND provider_id = ? AND provider_subject = ?",
                )
                .bind(tenant_id)
                .bind(provider_id)
                .bind(provider_subject)
                .fetch_optional(pool)
                .await?;
                Ok(row.as_ref().map(oidc_user_link_from_sqlite_row))
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT tenant_id, provider_id, provider_subject, user_id, email, display_name,
                        created_at, updated_at
                    FROM oidc_user_links
                    WHERE tenant_id = $1 AND provider_id = $2 AND provider_subject = $3",
                )
                .bind(tenant_id)
                .bind(provider_id)
                .bind(provider_subject)
                .fetch_optional(pool)
                .await?;
                Ok(row.as_ref().map(oidc_user_link_from_pg_row))
            }
        }
    }

    /// Creates or refreshes an OIDC provider-subject to MSM user link.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn upsert_oidc_user_link(
        &self,
        tenant_id: &str,
        provider_id: &str,
        provider_subject: &str,
        user_id: &str,
        email: &str,
        display_name: &str,
    ) -> StorageResult<OidcUserLinkRecord> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO oidc_user_links (
                        tenant_id, provider_id, provider_subject, user_id, email, display_name,
                        created_at, updated_at
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(tenant_id, provider_id, provider_subject) DO UPDATE SET
                        user_id = excluded.user_id,
                        email = excluded.email,
                        display_name = excluded.display_name,
                        updated_at = excluded.updated_at",
                )
                .bind(tenant_id)
                .bind(provider_id)
                .bind(provider_subject)
                .bind(user_id)
                .bind(email)
                .bind(display_name)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO oidc_user_links (
                        tenant_id, provider_id, provider_subject, user_id, email, display_name,
                        created_at, updated_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT(tenant_id, provider_id, provider_subject) DO UPDATE SET
                        user_id = excluded.user_id,
                        email = excluded.email,
                        display_name = excluded.display_name,
                        updated_at = excluded.updated_at",
                )
                .bind(tenant_id)
                .bind(provider_id)
                .bind(provider_subject)
                .bind(user_id)
                .bind(email)
                .bind(display_name)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        self.find_oidc_user_link(tenant_id, provider_id, provider_subject)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Creates a local user row.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn create_user(
        &self,
        id: &str,
        email: &str,
        display_name: &str,
    ) -> StorageResult<()> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO users (id, email, display_name, is_disabled, created_at) VALUES (?, ?, ?, 0, ?)",
                )
                .bind(id)
                .bind(email)
                .bind(display_name)
                .bind(now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO users (id, email, display_name, is_disabled, created_at) VALUES ($1, $2, $3, FALSE, $4)",
                )
                .bind(id)
                .bind(email)
                .bind(display_name)
                .bind(now)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// Finds a user by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn find_user(&self, id: &str) -> StorageResult<Option<UserRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, email, display_name, is_disabled, created_at
                    FROM users
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(user_from_sqlite_row).transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, email, display_name, is_disabled, created_at
                    FROM users
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(user_from_pg_row).transpose()
            }
        }
    }

    /// Enables or disables a user account.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails, the user does not exist, or timestamps are invalid.
    pub async fn set_user_disabled(
        &self,
        id: &str,
        is_disabled: bool,
    ) -> StorageResult<UserRecord> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => sqlx::query(
                "UPDATE users
                SET is_disabled = ?
                WHERE id = ?",
            )
            .bind(i64::from(is_disabled))
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected(),
            DbPool::Postgres(pool) => sqlx::query(
                "UPDATE users
                SET is_disabled = $1
                WHERE id = $2",
            )
            .bind(is_disabled)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected(),
        };

        if rows_affected == 0 {
            return Err(StorageError::Sqlx(sqlx::Error::RowNotFound));
        }

        self.find_user(id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Creates a local user profile and password credential.
    ///
    /// # Errors
    ///
    /// Returns an error when password hashing fails or SQL fails.
    pub async fn create_local_user_with_password(
        &self,
        id: &str,
        email: &str,
        display_name: &str,
        password: &str,
    ) -> StorageResult<UserRecord> {
        let password_hash = hash_password(password)?;
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let mut tx = pool.begin().await?;

                sqlx::query(
                    "INSERT INTO users (id, email, display_name, is_disabled, created_at) VALUES (?, ?, ?, 0, ?)",
                )
                .bind(id)
                .bind(email)
                .bind(display_name)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO local_user_credentials (user_id, password_hash, created_at, updated_at)
                    VALUES (?, ?, ?, ?)",
                )
                .bind(id)
                .bind(&password_hash)
                .bind(&now)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
            }
            DbPool::Postgres(pool) => {
                let mut tx = pool.begin().await?;

                sqlx::query(
                    "INSERT INTO users (id, email, display_name, is_disabled, created_at) VALUES ($1, $2, $3, FALSE, $4)",
                )
                .bind(id)
                .bind(email)
                .bind(display_name)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO local_user_credentials (user_id, password_hash, created_at, updated_at)
                    VALUES ($1, $2, $3, $4)",
                )
                .bind(id)
                .bind(&password_hash)
                .bind(&now)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
            }
        }

        Ok(UserRecord {
            id: id.to_owned(),
            email: email.to_owned(),
            display_name: display_name.to_owned(),
            is_disabled: false,
            created_at: parse_rfc3339(&now)?,
        })
    }

    /// Finds the local credential for a user.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn local_credential_for_user(
        &self,
        user_id: &str,
    ) -> StorageResult<Option<LocalUserCredentialRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT user_id, password_hash, created_at, updated_at
                    FROM local_user_credentials
                    WHERE user_id = ?",
                )
                .bind(user_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.as_ref().map(local_credential_from_sqlite_row))
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT user_id, password_hash, created_at, updated_at
                    FROM local_user_credentials
                    WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.as_ref().map(local_credential_from_pg_row))
            }
        }
    }

    /// Verifies a local user password by email.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or stored user timestamps are invalid.
    pub async fn verify_local_user_password(
        &self,
        email: &str,
        password: &str,
    ) -> StorageResult<Option<UserRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT users.id, users.email, users.display_name, users.is_disabled, users.created_at,
                        local_user_credentials.password_hash
                    FROM users
                    JOIN local_user_credentials ON local_user_credentials.user_id = users.id
                    WHERE users.email = ?",
                )
                .bind(email)
                .fetch_optional(pool)
                .await?;
                row.as_ref()
                    .map(|row| verified_local_user_from_sqlite_row(row, password))
                    .transpose()
                    .map(Option::flatten)
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT users.id, users.email, users.display_name, users.is_disabled, users.created_at,
                        local_user_credentials.password_hash
                    FROM users
                    JOIN local_user_credentials ON local_user_credentials.user_id = users.id
                    WHERE users.email = $1",
                )
                .bind(email)
                .fetch_optional(pool)
                .await?;
                row.as_ref()
                    .map(|row| verified_local_user_from_pg_row(row, password))
                    .transpose()
                    .map(Option::flatten)
            }
        }
    }

    /// Adds a user to a tenant with a coarse role.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn add_tenant_member(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: &str,
    ) -> StorageResult<()> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO tenant_members (tenant_id, user_id, role, created_at) VALUES (?, ?, ?, ?)",
                )
                .bind(tenant_id)
                .bind(user_id)
                .bind(role)
                .bind(now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO tenant_members (tenant_id, user_id, role, created_at) VALUES ($1, $2, $3, $4)",
                )
                .bind(tenant_id)
                .bind(user_id)
                .bind(role)
                .bind(now)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// Adds or updates a tenant member role.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn upsert_tenant_member(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: &str,
    ) -> StorageResult<TenantMemberRecord> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO tenant_members (tenant_id, user_id, role, created_at)
                    VALUES (?, ?, ?, ?)
                    ON CONFLICT(tenant_id, user_id) DO UPDATE SET role = excluded.role",
                )
                .bind(tenant_id)
                .bind(user_id)
                .bind(role)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO tenant_members (tenant_id, user_id, role, created_at)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT(tenant_id, user_id) DO UPDATE SET role = excluded.role",
                )
                .bind(tenant_id)
                .bind(user_id)
                .bind(role)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        self.find_tenant_member(tenant_id, user_id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Lists members in a tenant.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn list_tenant_members(
        &self,
        tenant_id: &str,
    ) -> StorageResult<Vec<TenantMemberRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT tenant_id, user_id, role, created_at
                    FROM tenant_members
                    WHERE tenant_id = ?
                    ORDER BY created_at, user_id",
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(tenant_member_from_sqlite_row).collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT tenant_id, user_id, role, created_at
                    FROM tenant_members
                    WHERE tenant_id = $1
                    ORDER BY created_at, user_id",
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(tenant_member_from_pg_row).collect()
            }
        }
    }

    /// Lists all tenant memberships for one user.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn list_user_tenant_members(
        &self,
        user_id: &str,
    ) -> StorageResult<Vec<TenantMemberRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT tenant_id, user_id, role, created_at
                    FROM tenant_members
                    WHERE user_id = ?
                    ORDER BY tenant_id",
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(tenant_member_from_sqlite_row).collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT tenant_id, user_id, role, created_at
                    FROM tenant_members
                    WHERE user_id = $1
                    ORDER BY tenant_id",
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(tenant_member_from_pg_row).collect()
            }
        }
    }

    /// Finds one tenant member.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn find_tenant_member(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> StorageResult<Option<TenantMemberRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT tenant_id, user_id, role, created_at
                    FROM tenant_members
                    WHERE tenant_id = ? AND user_id = ?",
                )
                .bind(tenant_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(tenant_member_from_sqlite_row).transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT tenant_id, user_id, role, created_at
                    FROM tenant_members
                    WHERE tenant_id = $1 AND user_id = $2",
                )
                .bind(tenant_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(tenant_member_from_pg_row).transpose()
            }
        }
    }

    /// Adds or updates a tenant-scoped role template and replaces its permissions.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn upsert_role_template(
        &self,
        id: &str,
        tenant_id: &str,
        name: &str,
        permissions: &BTreeSet<Permission>,
    ) -> StorageResult<RoleRecord> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let mut tx = pool.begin().await?;

                sqlx::query(
                    "INSERT INTO roles (id, tenant_id, name, created_at)
                    VALUES (?, ?, ?, ?)
                    ON CONFLICT(id) DO UPDATE SET
                        tenant_id = excluded.tenant_id,
                        name = excluded.name",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(name)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                sqlx::query("DELETE FROM role_permissions WHERE role_id = ?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;

                for permission in permissions {
                    sqlx::query(
                        "INSERT OR IGNORE INTO permissions (key, description)
                        VALUES (?, ?)",
                    )
                    .bind(permission.as_key())
                    .bind(permission.as_key())
                    .execute(&mut *tx)
                    .await?;
                    sqlx::query(
                        "INSERT INTO role_permissions (role_id, permission_key)
                        VALUES (?, ?)",
                    )
                    .bind(id)
                    .bind(permission.as_key())
                    .execute(&mut *tx)
                    .await?;
                }

                tx.commit().await?;
            }
            DbPool::Postgres(pool) => {
                let mut tx = pool.begin().await?;

                sqlx::query(
                    "INSERT INTO roles (id, tenant_id, name, created_at)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT(id) DO UPDATE SET
                        tenant_id = excluded.tenant_id,
                        name = excluded.name",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(name)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;

                for permission in permissions {
                    sqlx::query(
                        "INSERT INTO permissions (key, description)
                        VALUES ($1, $2)
                        ON CONFLICT(key) DO NOTHING",
                    )
                    .bind(permission.as_key())
                    .bind(permission.as_key())
                    .execute(&mut *tx)
                    .await?;
                    sqlx::query(
                        "INSERT INTO role_permissions (role_id, permission_key)
                        VALUES ($1, $2)",
                    )
                    .bind(id)
                    .bind(permission.as_key())
                    .execute(&mut *tx)
                    .await?;
                }

                tx.commit().await?;
            }
        }

        self.find_role_template(tenant_id, id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Lists role templates for one tenant.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn list_role_templates(&self, tenant_id: &str) -> StorageResult<Vec<RoleRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, name, created_at
                    FROM roles
                    WHERE tenant_id = ?
                    ORDER BY name, id",
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?;

                let mut roles = Vec::with_capacity(rows.len());
                for row in rows {
                    roles.push(self.role_from_sqlite_row_with_permissions(&row).await?);
                }
                Ok(roles)
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, name, created_at
                    FROM roles
                    WHERE tenant_id = $1
                    ORDER BY name, id",
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?;

                let mut roles = Vec::with_capacity(rows.len());
                for row in rows {
                    roles.push(self.role_from_pg_row_with_permissions(&row).await?);
                }
                Ok(roles)
            }
        }
    }

    /// Finds one tenant role template.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamps are invalid.
    pub async fn find_role_template(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> StorageResult<Option<RoleRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, name, created_at
                    FROM roles
                    WHERE tenant_id = ? AND id = ?",
                )
                .bind(tenant_id)
                .bind(id)
                .fetch_optional(pool)
                .await?;

                match row {
                    Some(row) => Ok(Some(
                        self.role_from_sqlite_row_with_permissions(&row).await?,
                    )),
                    None => Ok(None),
                }
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, name, created_at
                    FROM roles
                    WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tenant_id)
                .bind(id)
                .fetch_optional(pool)
                .await?;

                match row {
                    Some(row) => Ok(Some(self.role_from_pg_row_with_permissions(&row).await?)),
                    None => Ok(None),
                }
            }
        }
    }

    /// Inserts or updates a sticker pack and replaces its sticker rows.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails or SQL fails.
    #[allow(clippy::too_many_lines)]
    pub async fn upsert_sticker_pack(
        &self,
        id: &str,
        tenant_id: &str,
        owner_user_id: &str,
        visibility: PackVisibility,
        source_provider: Option<&str>,
        pack: &StickerPack,
    ) -> StorageResult<()> {
        let now = now();
        let pack_json = serde_json::to_string(pack)?;
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let mut tx = pool.begin().await?;

                sqlx::query(
                    "INSERT INTO sticker_packs (
                        id, tenant_id, owner_user_id, compatibility_id, title, visibility,
                        source_provider, sticker_pack_json, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(id) DO UPDATE SET
                        compatibility_id = excluded.compatibility_id,
                        title = excluded.title,
                        visibility = excluded.visibility,
                        source_provider = excluded.source_provider,
                        sticker_pack_json = excluded.sticker_pack_json,
                        updated_at = excluded.updated_at",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(owner_user_id)
                .bind(&pack.id)
                .bind(&pack.title)
                .bind(visibility.as_str())
                .bind(source_provider)
                .bind(&pack_json)
                .bind(&now)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                sqlx::query("DELETE FROM stickers WHERE pack_id = ?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;

                for (index, sticker) in pack.stickers.iter().enumerate() {
                    sqlx::query(
                        "INSERT INTO stickers (
                            id, pack_id, compatibility_id, title, asset_key, image_url, is_animated, sort_order
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(format!("{id}:{}", sticker.id))
                    .bind(id)
                    .bind(&sticker.id)
                    .bind(&sticker.title)
                    .bind(sticker.filename.as_deref())
                    .bind(&sticker.image)
                    .bind(sticker.is_animated.map(i64::from))
                    .bind(i64::try_from(index).unwrap_or(i64::MAX))
                    .execute(&mut *tx)
                    .await?;
                }

                tx.commit().await?;
            }
            DbPool::Postgres(pool) => {
                let mut tx = pool.begin().await?;

                sqlx::query(
                    "INSERT INTO sticker_packs (
                        id, tenant_id, owner_user_id, compatibility_id, title, visibility,
                        source_provider, sticker_pack_json, created_at, updated_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    ON CONFLICT(id) DO UPDATE SET
                        compatibility_id = excluded.compatibility_id,
                        title = excluded.title,
                        visibility = excluded.visibility,
                        source_provider = excluded.source_provider,
                        sticker_pack_json = excluded.sticker_pack_json,
                        updated_at = excluded.updated_at",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(owner_user_id)
                .bind(&pack.id)
                .bind(&pack.title)
                .bind(visibility.as_str())
                .bind(source_provider)
                .bind(&pack_json)
                .bind(&now)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                sqlx::query("DELETE FROM stickers WHERE pack_id = $1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;

                for (index, sticker) in pack.stickers.iter().enumerate() {
                    sqlx::query(
                        "INSERT INTO stickers (
                            id, pack_id, compatibility_id, title, asset_key, image_url, is_animated, sort_order
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    )
                    .bind(format!("{id}:{}", sticker.id))
                    .bind(id)
                    .bind(&sticker.id)
                    .bind(&sticker.title)
                    .bind(sticker.filename.as_deref())
                    .bind(&sticker.image)
                    .bind(sticker.is_animated)
                    .bind(i64::try_from(index).unwrap_or(i64::MAX))
                    .execute(&mut *tx)
                    .await?;
                }

                tx.commit().await?;
            }
        }
        Ok(())
    }

    /// Creates a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn create_folder(
        &self,
        id: &str,
        tenant_id: &str,
        owner_user_id: &str,
        name: &str,
    ) -> StorageResult<FolderRecord> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO folders (id, tenant_id, owner_user_id, name, created_at)
                    VALUES (?, ?, ?, ?, ?)",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(owner_user_id)
                .bind(name)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO folders (id, tenant_id, owner_user_id, name, created_at)
                    VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(owner_user_id)
                .bind(name)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        Ok(FolderRecord {
            id: id.to_owned(),
            tenant_id: tenant_id.to_owned(),
            owner_user_id: owner_user_id.to_owned(),
            name: name.to_owned(),
            created_at: parse_rfc3339(&now)?,
        })
    }

    /// Lists folders for one owner in one tenant.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL/timestamp parsing fails.
    pub async fn list_folders(
        &self,
        tenant_id: &str,
        owner_user_id: &str,
    ) -> StorageResult<Vec<FolderRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, name, created_at
                    FROM folders
                    WHERE tenant_id = ? AND owner_user_id = ?
                    ORDER BY created_at, id",
                )
                .bind(tenant_id)
                .bind(owner_user_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(folder_from_sqlite_row).collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, name, created_at
                    FROM folders
                    WHERE tenant_id = $1 AND owner_user_id = $2
                    ORDER BY created_at, id",
                )
                .bind(tenant_id)
                .bind(owner_user_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(folder_from_pg_row).collect()
            }
        }
    }

    /// Renames a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails, the folder does not exist, or timestamp parsing fails.
    pub async fn rename_folder(&self, id: &str, name: &str) -> StorageResult<FolderRecord> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query("UPDATE folders SET name = ? WHERE id = ?")
                    .bind(name)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query("UPDATE folders SET name = $1 WHERE id = $2")
                    .bind(name)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
        }
        self.find_folder(id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Deletes a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn delete_folder(&self, id: &str) -> StorageResult<bool> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => sqlx::query("DELETE FROM folders WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected(),
            DbPool::Postgres(pool) => sqlx::query("DELETE FROM folders WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected(),
        };
        Ok(rows_affected == 1)
    }

    /// Finds a folder by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamp parsing fails.
    pub async fn find_folder_record(&self, id: &str) -> StorageResult<Option<FolderRecord>> {
        self.find_folder(id).await
    }

    async fn find_folder(&self, id: &str) -> StorageResult<Option<FolderRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, name, created_at
                    FROM folders
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(folder_from_sqlite_row).transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, name, created_at
                    FROM folders
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(folder_from_pg_row).transpose()
            }
        }
    }

    /// Adds a sticker pack to a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn add_pack_to_folder(
        &self,
        folder_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> StorageResult<FolderPackRecord> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO folder_packs (folder_id, pack_id, sort_order)
                    VALUES (?, ?, ?)
                    ON CONFLICT(folder_id, pack_id) DO UPDATE SET sort_order = excluded.sort_order",
                )
                .bind(folder_id)
                .bind(pack_id)
                .bind(sort_order)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO folder_packs (folder_id, pack_id, sort_order)
                    VALUES ($1, $2, $3)
                    ON CONFLICT(folder_id, pack_id) DO UPDATE SET sort_order = excluded.sort_order",
                )
                .bind(folder_id)
                .bind(pack_id)
                .bind(sort_order)
                .execute(pool)
                .await?;
            }
        }

        Ok(FolderPackRecord {
            folder_id: folder_id.to_owned(),
            pack_id: pack_id.to_owned(),
            sort_order,
        })
    }

    /// Lists pack IDs in a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn list_folder_pack_ids(&self, folder_id: &str) -> StorageResult<Vec<String>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT pack_id FROM folder_packs
                    WHERE folder_id = ?
                    ORDER BY sort_order, pack_id",
                )
                .bind(folder_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get("pack_id")).collect())
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT pack_id FROM folder_packs
                    WHERE folder_id = $1
                    ORDER BY sort_order, pack_id",
                )
                .bind(folder_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get("pack_id")).collect())
            }
        }
    }

    /// Removes a sticker pack from a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn remove_pack_from_folder(
        &self,
        folder_id: &str,
        pack_id: &str,
    ) -> StorageResult<bool> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query("DELETE FROM folder_packs WHERE folder_id = ? AND pack_id = ?")
                    .bind(folder_id)
                    .bind(pack_id)
                    .execute(pool)
                    .await?
                    .rows_affected()
            }
            DbPool::Postgres(pool) => {
                sqlx::query("DELETE FROM folder_packs WHERE folder_id = $1 AND pack_id = $2")
                    .bind(folder_id)
                    .bind(pack_id)
                    .execute(pool)
                    .await?
                    .rows_affected()
            }
        };
        Ok(rows_affected == 1)
    }

    /// Creates a tag.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn create_tag(&self, tag: NewTag<'_>) -> StorageResult<TagRecord> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO tags (id, tenant_id, name, created_at) VALUES (?, ?, ?, ?)",
                )
                .bind(tag.id)
                .bind(tag.tenant_id)
                .bind(tag.name)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO tags (id, tenant_id, name, created_at) VALUES ($1, $2, $3, $4)",
                )
                .bind(tag.id)
                .bind(tag.tenant_id)
                .bind(tag.name)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        Ok(TagRecord {
            id: tag.id.to_owned(),
            tenant_id: tag.tenant_id.to_owned(),
            name: tag.name.to_owned(),
            created_at: parse_rfc3339(&now)?,
        })
    }

    /// Lists tags for one tenant.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL/timestamp parsing fails.
    pub async fn list_tags(&self, tenant_id: &str) -> StorageResult<Vec<TagRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, name, created_at
                    FROM tags
                    WHERE tenant_id = ?
                    ORDER BY name, id",
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(tag_from_sqlite_row).collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, name, created_at
                    FROM tags
                    WHERE tenant_id = $1
                    ORDER BY name, id",
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(tag_from_pg_row).collect()
            }
        }
    }

    /// Finds a tag by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or timestamp parsing fails.
    pub async fn find_tag_record(&self, id: &str) -> StorageResult<Option<TagRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, name, created_at
                    FROM tags
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(tag_from_sqlite_row).transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, name, created_at
                    FROM tags
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(tag_from_pg_row).transpose()
            }
        }
    }

    /// Deletes a tag.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn delete_tag(&self, id: &str) -> StorageResult<bool> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => sqlx::query("DELETE FROM tags WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected(),
            DbPool::Postgres(pool) => sqlx::query("DELETE FROM tags WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected(),
        };
        Ok(rows_affected == 1)
    }

    /// Adds a tag to a sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn add_tag_to_pack(
        &self,
        pack_id: &str,
        tag_id: &str,
    ) -> StorageResult<PackTagRecord> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT OR IGNORE INTO pack_tags (pack_id, tag_id)
                    VALUES (?, ?)",
                )
                .bind(pack_id)
                .bind(tag_id)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO pack_tags (pack_id, tag_id)
                    VALUES ($1, $2)
                    ON CONFLICT(pack_id, tag_id) DO NOTHING",
                )
                .bind(pack_id)
                .bind(tag_id)
                .execute(pool)
                .await?;
            }
        }

        Ok(PackTagRecord {
            pack_id: pack_id.to_owned(),
            tag_id: tag_id.to_owned(),
        })
    }

    /// Lists tag IDs assigned to a sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn list_pack_tag_ids(&self, pack_id: &str) -> StorageResult<Vec<String>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT tag_id FROM pack_tags
                    WHERE pack_id = ?
                    ORDER BY tag_id",
                )
                .bind(pack_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get("tag_id")).collect())
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT tag_id FROM pack_tags
                    WHERE pack_id = $1
                    ORDER BY tag_id",
                )
                .bind(pack_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get("tag_id")).collect())
            }
        }
    }

    /// Removes a tag from a sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn remove_tag_from_pack(&self, pack_id: &str, tag_id: &str) -> StorageResult<bool> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query("DELETE FROM pack_tags WHERE pack_id = ? AND tag_id = ?")
                    .bind(pack_id)
                    .bind(tag_id)
                    .execute(pool)
                    .await?
                    .rows_affected()
            }
            DbPool::Postgres(pool) => {
                sqlx::query("DELETE FROM pack_tags WHERE pack_id = $1 AND tag_id = $2")
                    .bind(pack_id)
                    .bind(tag_id)
                    .execute(pool)
                    .await?
                    .rows_affected()
            }
        };
        Ok(rows_affected == 1)
    }

    /// Creates a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn create_subscription_group(
        &self,
        id: &str,
        tenant_id: &str,
        owner_user_id: &str,
        title: &str,
        visibility: PackVisibility,
    ) -> StorageResult<SubscriptionGroupRecord> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO subscription_groups (
                        id, tenant_id, owner_user_id, title, visibility, created_at
                    ) VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(owner_user_id)
                .bind(title)
                .bind(visibility.as_str())
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO subscription_groups (
                        id, tenant_id, owner_user_id, title, visibility, created_at
                    ) VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(owner_user_id)
                .bind(title)
                .bind(visibility.as_str())
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }
        Ok(SubscriptionGroupRecord {
            id: id.to_owned(),
            tenant_id: tenant_id.to_owned(),
            owner_user_id: owner_user_id.to_owned(),
            title: title.to_owned(),
            visibility,
            created_at: parse_rfc3339(&now)?,
        })
    }

    /// Lists subscription groups for one owner in one tenant.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL/timestamp parsing fails.
    pub async fn list_subscription_groups(
        &self,
        tenant_id: &str,
        owner_user_id: &str,
    ) -> StorageResult<Vec<SubscriptionGroupRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, title, visibility, created_at
                    FROM subscription_groups
                    WHERE tenant_id = ? AND owner_user_id = ?
                    ORDER BY created_at, id",
                )
                .bind(tenant_id)
                .bind(owner_user_id)
                .fetch_all(pool)
                .await?;

                rows.iter()
                    .map(subscription_group_from_sqlite_row)
                    .collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, title, visibility, created_at
                    FROM subscription_groups
                    WHERE tenant_id = $1 AND owner_user_id = $2
                    ORDER BY created_at, id",
                )
                .bind(tenant_id)
                .bind(owner_user_id)
                .fetch_all(pool)
                .await?;

                rows.iter().map(subscription_group_from_pg_row).collect()
            }
        }
    }

    /// Renames a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails, the group does not exist, or timestamp parsing fails.
    pub async fn rename_subscription_group(
        &self,
        id: &str,
        title: &str,
    ) -> StorageResult<SubscriptionGroupRecord> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query("UPDATE subscription_groups SET title = ? WHERE id = ?")
                    .bind(title)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query("UPDATE subscription_groups SET title = $1 WHERE id = $2")
                    .bind(title)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
        }
        self.find_subscription_group(id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Deletes a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn delete_subscription_group(&self, id: &str) -> StorageResult<bool> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => sqlx::query("DELETE FROM subscription_groups WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected(),
            DbPool::Postgres(pool) => sqlx::query("DELETE FROM subscription_groups WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected(),
        };
        Ok(rows_affected == 1)
    }

    /// Finds a subscription group by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails, visibility is invalid, or timestamp parsing fails.
    pub async fn find_subscription_group_record(
        &self,
        id: &str,
    ) -> StorageResult<Option<SubscriptionGroupRecord>> {
        self.find_subscription_group(id).await
    }

    async fn find_subscription_group(
        &self,
        id: &str,
    ) -> StorageResult<Option<SubscriptionGroupRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, title, visibility, created_at
                    FROM subscription_groups
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref()
                    .map(subscription_group_from_sqlite_row)
                    .transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, title, visibility, created_at
                    FROM subscription_groups
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref().map(subscription_group_from_pg_row).transpose()
            }
        }
    }

    /// Adds a sticker pack to a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub async fn add_pack_to_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> StorageResult<SubscriptionGroupPackRecord> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO subscription_group_packs (
                        subscription_group_id, pack_id, sort_order
                    ) VALUES (?, ?, ?)
                    ON CONFLICT(subscription_group_id, pack_id) DO UPDATE SET sort_order = excluded.sort_order",
                )
                .bind(subscription_group_id)
                .bind(pack_id)
                .bind(sort_order)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO subscription_group_packs (
                        subscription_group_id, pack_id, sort_order
                    ) VALUES ($1, $2, $3)
                    ON CONFLICT(subscription_group_id, pack_id) DO UPDATE SET sort_order = excluded.sort_order",
                )
                .bind(subscription_group_id)
                .bind(pack_id)
                .bind(sort_order)
                .execute(pool)
                .await?;
            }
        }

        Ok(SubscriptionGroupPackRecord {
            subscription_group_id: subscription_group_id.to_owned(),
            pack_id: pack_id.to_owned(),
            sort_order,
        })
    }

    /// Finds a sticker pack by internal pack ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL/JSON parsing fails.
    pub async fn find_sticker_pack(&self, id: &str) -> StorageResult<Option<StickerPack>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query("SELECT sticker_pack_json FROM sticker_packs WHERE id = ?")
                    .bind(id)
                    .fetch_optional(pool)
                    .await?;

                row.map(|row| {
                    let json: String = row.get("sticker_pack_json");
                    StickerPack::from_json_str(&json).map_err(Into::into)
                })
                .transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query("SELECT sticker_pack_json FROM sticker_packs WHERE id = $1")
                    .bind(id)
                    .fetch_optional(pool)
                    .await?;

                row.map(|row| {
                    let json: String = row.get("sticker_pack_json");
                    StickerPack::from_json_str(&json).map_err(Into::into)
                })
                .transpose()
            }
        }
    }

    /// Finds a sticker pack record by internal pack ID, including owner and tenant metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails, JSON parsing fails, or timestamps are invalid.
    pub async fn find_sticker_pack_record(
        &self,
        id: &str,
    ) -> StorageResult<Option<StickerPackRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, compatibility_id, title, visibility,
                        source_provider, sticker_pack_json, created_at, updated_at
                    FROM sticker_packs
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref()
                    .map(sticker_pack_record_from_sqlite_row)
                    .transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, compatibility_id, title, visibility,
                        source_provider, sticker_pack_json, created_at, updated_at
                    FROM sticker_packs
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                row.as_ref()
                    .map(sticker_pack_record_from_pg_row)
                    .transpose()
            }
        }
    }

    /// Updates owned sticker pack metadata without changing sticker contents.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails or SQL fails.
    pub async fn update_sticker_pack_metadata(
        &self,
        id: &str,
        owner_user_id: &str,
        title: &str,
        visibility: PackVisibility,
    ) -> StorageResult<bool> {
        let stored_pack_json = match &self.pool {
            DbPool::Sqlite(pool) => sqlx::query(
                "SELECT sticker_pack_json FROM sticker_packs WHERE id = ? AND owner_user_id = ?",
            )
            .bind(id)
            .bind(owner_user_id)
            .fetch_optional(pool)
            .await?
            .map(|row| row.get::<String, _>("sticker_pack_json")),
            DbPool::Postgres(pool) => sqlx::query(
                "SELECT sticker_pack_json FROM sticker_packs WHERE id = $1 AND owner_user_id = $2",
            )
            .bind(id)
            .bind(owner_user_id)
            .fetch_optional(pool)
            .await?
            .map(|row| row.get::<String, _>("sticker_pack_json")),
        };

        let Some(json) = stored_pack_json else {
            return Ok(false);
        };

        let mut pack = StickerPack::from_json_str(&json)?;
        pack.title = title.to_owned();
        let pack_json = serde_json::to_string(&pack)?;

        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => sqlx::query(
                "UPDATE sticker_packs
                SET title = ?, visibility = ?, sticker_pack_json = ?, updated_at = ?
                WHERE id = ? AND owner_user_id = ?",
            )
            .bind(title)
            .bind(visibility.as_str())
            .bind(pack_json)
            .bind(now())
            .bind(id)
            .bind(owner_user_id)
            .execute(pool)
            .await?
            .rows_affected(),
            DbPool::Postgres(pool) => sqlx::query(
                "UPDATE sticker_packs
                SET title = $1, visibility = $2, sticker_pack_json = $3, updated_at = $4
                WHERE id = $5 AND owner_user_id = $6",
            )
            .bind(title)
            .bind(visibility.as_str())
            .bind(pack_json)
            .bind(now())
            .bind(id)
            .bind(owner_user_id)
            .execute(pool)
            .await?
            .rows_affected(),
        };

        Ok(rows_affected == 1)
    }

    /// Deletes an owned sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn delete_sticker_pack(&self, id: &str, owner_user_id: &str) -> StorageResult<bool> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query("DELETE FROM sticker_packs WHERE id = ? AND owner_user_id = ?")
                    .bind(id)
                    .bind(owner_user_id)
                    .execute(pool)
                    .await?
                    .rows_affected()
            }
            DbPool::Postgres(pool) => {
                sqlx::query("DELETE FROM sticker_packs WHERE id = $1 AND owner_user_id = $2")
                    .bind(id)
                    .bind(owner_user_id)
                    .execute(pool)
                    .await?
                    .rows_affected()
            }
        };

        Ok(rows_affected == 1)
    }

    /// Lists sticker packs owned by a user.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL/JSON parsing fails.
    pub async fn list_user_sticker_packs(&self, user_id: &str) -> StorageResult<Vec<StickerPack>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT sticker_pack_json FROM sticker_packs WHERE owner_user_id = ? ORDER BY title, id",
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?;

                rows.into_iter()
                    .map(|row| {
                        let json: String = row.get("sticker_pack_json");
                        StickerPack::from_json_str(&json).map_err(Into::into)
                    })
                    .collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT sticker_pack_json FROM sticker_packs WHERE owner_user_id = $1 ORDER BY title, id",
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?;

                rows.into_iter()
                    .map(|row| {
                        let json: String = row.get("sticker_pack_json");
                        StickerPack::from_json_str(&json).map_err(Into::into)
                    })
                    .collect()
            }
        }
    }

    /// Lists sticker packs owned by a user in tenants where the user is still a member.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL/JSON parsing fails.
    pub async fn list_user_accessible_sticker_packs(
        &self,
        user_id: &str,
    ) -> StorageResult<Vec<StickerPack>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT sticker_packs.sticker_pack_json
                FROM sticker_packs
                INNER JOIN tenant_members
                    ON tenant_members.tenant_id = sticker_packs.tenant_id
                    AND tenant_members.user_id = sticker_packs.owner_user_id
                WHERE sticker_packs.owner_user_id = ?
                ORDER BY sticker_packs.title, sticker_packs.id",
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?;
                rows.into_iter()
                    .map(|row| {
                        let json: String = row.get("sticker_pack_json");
                        StickerPack::from_json_str(&json).map_err(Into::into)
                    })
                    .collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT sticker_packs.sticker_pack_json
                FROM sticker_packs
                INNER JOIN tenant_members
                    ON tenant_members.tenant_id = sticker_packs.tenant_id
                    AND tenant_members.user_id = sticker_packs.owner_user_id
                WHERE sticker_packs.owner_user_id = $1
                ORDER BY sticker_packs.title, sticker_packs.id",
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?;
                rows.into_iter()
                    .map(|row| {
                        let json: String = row.get("sticker_pack_json");
                        StickerPack::from_json_str(&json).map_err(Into::into)
                    })
                    .collect()
            }
        }
    }

    /// Creates a Personal Access Token and returns the raw token once.
    ///
    /// # Errors
    ///
    /// Returns an error when the token ID is invalid, random generation fails, scope serialization
    /// fails, or SQL fails.
    pub async fn create_personal_access_token(
        &self,
        id: &str,
        user_id: &str,
        name: &str,
        scopes: &BTreeSet<Permission>,
        expires_at: Option<&str>,
    ) -> StorageResult<CreatedPersonalAccessToken> {
        validate_pat_id(id)?;
        let secret = generate_pat_secret()?;
        let token = format!("msm_pat_{id}_{secret}");
        let token_hash = hash_pat_secret(&secret);
        let scopes_json = serde_json::to_string(
            &scopes
                .iter()
                .map(|permission| permission.as_key())
                .collect::<Vec<_>>(),
        )?;
        let now = now();

        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO personal_access_tokens (
                        id, user_id, name, token_hash, scopes_json, expires_at, created_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(id)
                .bind(user_id)
                .bind(name)
                .bind(&token_hash)
                .bind(&scopes_json)
                .bind(expires_at)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO personal_access_tokens (
                        id, user_id, name, token_hash, scopes_json, expires_at, created_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(id)
                .bind(user_id)
                .bind(name)
                .bind(&token_hash)
                .bind(&scopes_json)
                .bind(expires_at)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        let record = PersonalAccessTokenRecord {
            id: id.to_owned(),
            user_id: user_id.to_owned(),
            name: name.to_owned(),
            token_hash,
            scopes: scopes.clone(),
            expires_at: expires_at.map(ToOwned::to_owned),
            revoked_at: None,
            created_at: now,
        };

        Ok(CreatedPersonalAccessToken { record, token })
    }

    /// Lists Personal Access Tokens for a user without exposing raw token secrets.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or a stored scope key is invalid.
    pub async fn list_personal_access_tokens(
        &self,
        user_id: &str,
    ) -> StorageResult<Vec<PersonalAccessTokenRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, user_id, name, token_hash, scopes_json, expires_at, revoked_at, created_at
                    FROM personal_access_tokens
                    WHERE user_id = ?
                    ORDER BY created_at DESC, id",
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?;
                rows.iter().map(pat_record_from_sqlite_row).collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT id, user_id, name, token_hash, scopes_json, expires_at, revoked_at, created_at
                    FROM personal_access_tokens
                    WHERE user_id = $1
                    ORDER BY created_at DESC, id",
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?;
                rows.iter().map(pat_record_from_pg_row).collect()
            }
        }
    }

    /// Finds one Personal Access Token by ID without exposing its raw token.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or a stored scope key is invalid.
    pub async fn find_personal_access_token(
        &self,
        id: &str,
    ) -> StorageResult<Option<PersonalAccessTokenRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, user_id, name, token_hash, scopes_json, expires_at, revoked_at, created_at
                    FROM personal_access_tokens
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                row.as_ref().map(pat_record_from_sqlite_row).transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, user_id, name, token_hash, scopes_json, expires_at, revoked_at, created_at
                    FROM personal_access_tokens
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                row.as_ref().map(pat_record_from_pg_row).transpose()
            }
        }
    }

    /// Verifies a Personal Access Token and returns the active record when valid.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or a stored scope key is invalid.
    pub async fn verify_personal_access_token(
        &self,
        token: &str,
    ) -> StorageResult<Option<PersonalAccessTokenRecord>> {
        let Some((id, secret)) = parse_pat_token(token) else {
            return Ok(None);
        };

        let Some(record) = self.find_personal_access_token(id).await? else {
            return Ok(None);
        };

        if record.revoked_at.is_some() || is_expired(record.expires_at.as_deref()) {
            return Ok(None);
        }

        let presented_hash = hash_pat_secret(secret);
        if presented_hash
            .as_bytes()
            .ct_eq(record.token_hash.as_bytes())
            .into()
        {
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Revokes a Personal Access Token by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn revoke_personal_access_token(&self, id: &str) -> StorageResult<()> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query("UPDATE personal_access_tokens SET revoked_at = ? WHERE id = ?")
                    .bind(now())
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query("UPDATE personal_access_tokens SET revoked_at = $1 WHERE id = $2")
                    .bind(now())
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
        }
        Ok(())
    }

    /// Creates a Web session token and returns the raw cookie token once.
    ///
    /// # Errors
    ///
    /// Returns an error when the session ID is invalid, random generation fails, the repository is
    /// SQL fails.
    pub async fn create_web_session(
        &self,
        id: &str,
        user_id: &str,
        expires_at: Option<&str>,
    ) -> StorageResult<CreatedWebSession> {
        validate_web_session_id(id)?;
        let secret = generate_pat_secret()?;
        let token = format!("msm_session_{id}_{secret}");
        let session_hash = hash_pat_secret(&secret);
        let now = now();

        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO web_sessions (
                        id, user_id, session_hash, expires_at, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(id)
                .bind(user_id)
                .bind(&session_hash)
                .bind(expires_at)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO web_sessions (
                        id, user_id, session_hash, expires_at, created_at, updated_at
                    ) VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(id)
                .bind(user_id)
                .bind(&session_hash)
                .bind(expires_at)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        Ok(CreatedWebSession {
            record: WebSessionRecord {
                id: id.to_owned(),
                user_id: user_id.to_owned(),
                session_hash,
                expires_at: expires_at.map(ToOwned::to_owned),
                revoked_at: None,
                created_at: now.clone(),
                updated_at: now,
            },
            token,
        })
    }

    /// Verifies a Web session token and returns the active record when valid.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn verify_web_session(&self, token: &str) -> StorageResult<Option<WebSessionRecord>> {
        let Some((id, secret)) = parse_web_session_token(token) else {
            return Ok(None);
        };

        let record = match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, user_id, session_hash, expires_at, revoked_at, created_at, updated_at
                    FROM web_sessions
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                row.as_ref().map(web_session_from_sqlite_row)
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, user_id, session_hash, expires_at, revoked_at, created_at, updated_at
                    FROM web_sessions
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                row.as_ref().map(web_session_from_pg_row)
            }
        };
        let Some(record) = record else {
            return Ok(None);
        };
        if record.revoked_at.is_some() || is_expired(record.expires_at.as_deref()) {
            return Ok(None);
        }

        let presented_hash = hash_pat_secret(secret);
        if presented_hash
            .as_bytes()
            .ct_eq(record.session_hash.as_bytes())
            .into()
        {
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Revokes a Web session by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn revoke_web_session(&self, id: &str) -> StorageResult<()> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE web_sessions
                    SET revoked_at = ?, updated_at = ?
                    WHERE id = ?",
                )
                .bind(&now)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "UPDATE web_sessions
                    SET revoked_at = $1, updated_at = $2
                    WHERE id = $3",
                )
                .bind(&now)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// Creates a subscription access token and returns the raw token once.
    ///
    /// # Errors
    ///
    /// Returns an error when the token ID is invalid, random generation fails, or SQL fails.
    pub async fn create_subscription_access_token(
        &self,
        id: &str,
        tenant_id: &str,
        owner_user_id: &str,
        resource_type: SubscriptionAccessResourceType,
        resource_id: &str,
    ) -> StorageResult<CreatedSubscriptionAccessToken> {
        validate_subscription_access_token_id(id)?;
        let secret = generate_pat_secret()?;
        let token = format!("msm_sub_{id}_{secret}");
        let token_hash = hash_pat_secret(&secret);
        let now = now();

        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO subscription_access_tokens (
                        id, tenant_id, owner_user_id, resource_type, resource_id, token_hash,
                        created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(owner_user_id)
                .bind(resource_type.as_str())
                .bind(resource_id)
                .bind(&token_hash)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO subscription_access_tokens (
                        id, tenant_id, owner_user_id, resource_type, resource_id, token_hash,
                        created_at, updated_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(owner_user_id)
                .bind(resource_type.as_str())
                .bind(resource_id)
                .bind(&token_hash)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }

        Ok(CreatedSubscriptionAccessToken {
            record: SubscriptionAccessTokenRecord {
                id: id.to_owned(),
                tenant_id: tenant_id.to_owned(),
                owner_user_id: owner_user_id.to_owned(),
                resource_type,
                resource_id: resource_id.to_owned(),
                token_hash,
                revoked_at: None,
                created_at: now.clone(),
                updated_at: now,
            },
            token,
        })
    }

    /// Lists subscription access tokens owned by one user without exposing raw token secrets.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or a stored resource type is invalid.
    pub async fn list_subscription_access_tokens(
        &self,
        owner_user_id: &str,
    ) -> StorageResult<Vec<SubscriptionAccessTokenRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, resource_type, resource_id, token_hash,
                        revoked_at, created_at, updated_at
                    FROM subscription_access_tokens
                    WHERE owner_user_id = ?
                    ORDER BY created_at DESC, id",
                )
                .bind(owner_user_id)
                .fetch_all(pool)
                .await?;
                rows.iter()
                    .map(subscription_access_token_from_sqlite_row)
                    .collect()
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, resource_type, resource_id, token_hash,
                        revoked_at, created_at, updated_at
                    FROM subscription_access_tokens
                    WHERE owner_user_id = $1
                    ORDER BY created_at DESC, id",
                )
                .bind(owner_user_id)
                .fetch_all(pool)
                .await?;
                rows.iter()
                    .map(subscription_access_token_from_pg_row)
                    .collect()
            }
        }
    }

    /// Verifies a subscription access token and returns the active record when valid.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or a stored resource type is invalid.
    pub async fn verify_subscription_access_token(
        &self,
        token: &str,
    ) -> StorageResult<Option<SubscriptionAccessTokenRecord>> {
        let Some((id, secret)) = parse_subscription_access_token(token) else {
            return Ok(None);
        };
        let Some(record) = self.find_subscription_access_token(id).await? else {
            return Ok(None);
        };
        if record.revoked_at.is_some() {
            return Ok(None);
        }

        let presented_hash = hash_pat_secret(secret);
        if presented_hash
            .as_bytes()
            .ct_eq(record.token_hash.as_bytes())
            .into()
        {
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Rotates a subscription access token secret by ID and returns the new raw token once.
    ///
    /// # Errors
    ///
    /// Returns an error when random generation fails, SQL fails, the token does not exist, or a
    /// stored resource type is invalid.
    pub async fn rotate_subscription_access_token(
        &self,
        id: &str,
    ) -> StorageResult<CreatedSubscriptionAccessToken> {
        let secret = generate_pat_secret()?;
        let token = format!("msm_sub_{id}_{secret}");
        let token_hash = hash_pat_secret(&secret);
        let now = now();
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => {
                let result = sqlx::query(
                    "UPDATE subscription_access_tokens
                    SET token_hash = ?, revoked_at = NULL, updated_at = ?
                    WHERE id = ?",
                )
                .bind(&token_hash)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
                result.rows_affected()
            }
            DbPool::Postgres(pool) => {
                let result = sqlx::query(
                    "UPDATE subscription_access_tokens
                    SET token_hash = $1, revoked_at = NULL, updated_at = $2
                    WHERE id = $3",
                )
                .bind(&token_hash)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
                result.rows_affected()
            }
        };
        if rows_affected != 1 {
            return Err(StorageError::Sqlx(sqlx::Error::RowNotFound));
        }
        let record = self
            .find_subscription_access_token(id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))?;
        Ok(CreatedSubscriptionAccessToken { record, token })
    }

    /// Revokes a subscription access token by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn revoke_subscription_access_token(&self, id: &str) -> StorageResult<()> {
        let now = now();
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE subscription_access_tokens
                    SET revoked_at = ?, updated_at = ?
                    WHERE id = ?",
                )
                .bind(&now)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "UPDATE subscription_access_tokens
                    SET revoked_at = $1, updated_at = $2
                    WHERE id = $3",
                )
                .bind(&now)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// Finds a subscription access token by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails or a stored resource type is invalid.
    pub async fn find_subscription_access_token(
        &self,
        id: &str,
    ) -> StorageResult<Option<SubscriptionAccessTokenRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, resource_type, resource_id, token_hash,
                        revoked_at, created_at, updated_at
                    FROM subscription_access_tokens
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                row.as_ref()
                    .map(subscription_access_token_from_sqlite_row)
                    .transpose()
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, owner_user_id, resource_type, resource_id, token_hash,
                        revoked_at, created_at, updated_at
                    FROM subscription_access_tokens
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                row.as_ref()
                    .map(subscription_access_token_from_pg_row)
                    .transpose()
            }
        }
    }

    /// Lists pack IDs in a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn list_subscription_pack_ids(
        &self,
        subscription_group_id: &str,
    ) -> StorageResult<Vec<String>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT pack_id FROM subscription_group_packs
                    WHERE subscription_group_id = ?
                    ORDER BY sort_order, pack_id",
                )
                .bind(subscription_group_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get("pack_id")).collect())
            }
            DbPool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT pack_id FROM subscription_group_packs
                    WHERE subscription_group_id = $1
                    ORDER BY sort_order, pack_id",
                )
                .bind(subscription_group_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get("pack_id")).collect())
            }
        }
    }

    /// Removes a sticker pack from a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL fails.
    pub async fn remove_pack_from_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
    ) -> StorageResult<bool> {
        let rows_affected = match &self.pool {
            DbPool::Sqlite(pool) => sqlx::query(
                "DELETE FROM subscription_group_packs
                WHERE subscription_group_id = ? AND pack_id = ?",
            )
            .bind(subscription_group_id)
            .bind(pack_id)
            .execute(pool)
            .await?
            .rows_affected(),
            DbPool::Postgres(pool) => sqlx::query(
                "DELETE FROM subscription_group_packs
                WHERE subscription_group_id = $1 AND pack_id = $2",
            )
            .bind(subscription_group_id)
            .bind(pack_id)
            .execute(pool)
            .await?
            .rows_affected(),
        };
        Ok(rows_affected == 1)
    }

    async fn role_from_sqlite_row_with_permissions(
        &self,
        row: &SqliteRow,
    ) -> StorageResult<RoleRecord> {
        let id: String = row.get("id");
        let permission_rows = sqlx::query(
            "SELECT permission_key
            FROM role_permissions
            WHERE role_id = ?
            ORDER BY permission_key",
        )
        .bind(&id)
        .fetch_all(self.sqlite()?)
        .await?;
        let mut permissions = BTreeSet::new();
        for row in permission_rows {
            let key: String = row.get("permission_key");
            let permission =
                Permission::from_key(&key).ok_or(StorageError::InvalidPersonalAccessToken {
                    reason: "unknown role permission",
                })?;
            permissions.insert(permission);
        }
        role_from_row(row, permissions)
    }

    async fn role_from_pg_row_with_permissions(&self, row: &PgRow) -> StorageResult<RoleRecord> {
        let id: String = row.get("id");
        let permission_rows = sqlx::query(
            "SELECT permission_key
            FROM role_permissions
            WHERE role_id = $1
            ORDER BY permission_key",
        )
        .bind(&id)
        .fetch_all(self.postgres()?)
        .await?;
        let mut permissions = BTreeSet::new();
        for row in permission_rows {
            let key: String = row.get("permission_key");
            let permission =
                Permission::from_key(&key).ok_or(StorageError::InvalidPersonalAccessToken {
                    reason: "unknown role permission",
                })?;
            permissions.insert(permission);
        }
        role_from_pg_row(row, permissions)
    }

    pub(crate) fn sqlite(&self) -> StorageResult<&SqlitePool> {
        self.pool
            .sqlite()
            .ok_or_else(|| StorageError::UnsupportedDatabaseKind {
                kind: "postgres".to_owned(),
            })
    }

    #[allow(dead_code)]
    pub(crate) fn postgres(&self) -> StorageResult<&PgPool> {
        self.pool
            .postgres()
            .ok_or_else(|| StorageError::UnsupportedDatabaseKind {
                kind: "sqlite".to_owned(),
            })
    }
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn parse_rfc3339(value: &str) -> StorageResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| StorageError::InvalidTimestamp {
            value: value.to_owned(),
            message: error.to_string(),
        })
}

fn folder_from_sqlite_row(row: &SqliteRow) -> StorageResult<FolderRecord> {
    let created_at: String = row.get("created_at");
    Ok(FolderRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        name: row.get("name"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn folder_from_pg_row(row: &PgRow) -> StorageResult<FolderRecord> {
    let created_at: String = row.get("created_at");
    Ok(FolderRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        name: row.get("name"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn tag_from_sqlite_row(row: &SqliteRow) -> StorageResult<TagRecord> {
    let created_at: String = row.get("created_at");
    Ok(TagRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        name: row.get("name"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn tag_from_pg_row(row: &PgRow) -> StorageResult<TagRecord> {
    let created_at: String = row.get("created_at");
    Ok(TagRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        name: row.get("name"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn tenant_member_from_sqlite_row(row: &SqliteRow) -> StorageResult<TenantMemberRecord> {
    let created_at: String = row.get("created_at");
    Ok(TenantMemberRecord {
        tenant_id: row.get("tenant_id"),
        user_id: row.get("user_id"),
        role: row.get("role"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn tenant_member_from_pg_row(row: &PgRow) -> StorageResult<TenantMemberRecord> {
    let created_at: String = row.get("created_at");
    Ok(TenantMemberRecord {
        tenant_id: row.get("tenant_id"),
        user_id: row.get("user_id"),
        role: row.get("role"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn local_credential_from_sqlite_row(row: &SqliteRow) -> LocalUserCredentialRecord {
    LocalUserCredentialRecord {
        user_id: row.get("user_id"),
        password_hash: row.get("password_hash"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn local_credential_from_pg_row(row: &PgRow) -> LocalUserCredentialRecord {
    LocalUserCredentialRecord {
        user_id: row.get("user_id"),
        password_hash: row.get("password_hash"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn verified_local_user_from_sqlite_row(
    row: &SqliteRow,
    password: &str,
) -> StorageResult<Option<UserRecord>> {
    let password_hash: String = row.get("password_hash");
    if !verify_password(password, &password_hash) {
        return Ok(None);
    }
    let is_disabled: i64 = row.get("is_disabled");
    if is_disabled != 0 {
        return Ok(None);
    }

    let created_at: String = row.get("created_at");
    Ok(Some(UserRecord {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        is_disabled: false,
        created_at: parse_rfc3339(&created_at)?,
    }))
}

fn verified_local_user_from_pg_row(
    row: &PgRow,
    password: &str,
) -> StorageResult<Option<UserRecord>> {
    let password_hash: String = row.get("password_hash");
    if !verify_password(password, &password_hash) {
        return Ok(None);
    }
    let is_disabled: bool = row.get("is_disabled");
    if is_disabled {
        return Ok(None);
    }

    let created_at: String = row.get("created_at");
    Ok(Some(UserRecord {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        is_disabled: false,
        created_at: parse_rfc3339(&created_at)?,
    }))
}

fn tenant_from_sqlite_row(row: &SqliteRow) -> StorageResult<TenantRecord> {
    let created_at: String = row.get("created_at");
    let local_registration_enabled: i64 = row.get("local_registration_enabled");
    Ok(TenantRecord {
        id: row.get("id"),
        name: row.get("name"),
        public_asset_url: row.get("public_asset_url"),
        local_registration_enabled: local_registration_enabled != 0,
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn tenant_from_pg_row(row: &PgRow) -> StorageResult<TenantRecord> {
    let created_at: String = row.get("created_at");
    Ok(TenantRecord {
        id: row.get("id"),
        name: row.get("name"),
        public_asset_url: row.get("public_asset_url"),
        local_registration_enabled: row.get("local_registration_enabled"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn user_from_sqlite_row(row: &SqliteRow) -> StorageResult<UserRecord> {
    let created_at: String = row.get("created_at");
    let is_disabled: i64 = row.get("is_disabled");
    Ok(UserRecord {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        is_disabled: is_disabled != 0,
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn user_from_pg_row(row: &PgRow) -> StorageResult<UserRecord> {
    let created_at: String = row.get("created_at");
    Ok(UserRecord {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        is_disabled: row.get("is_disabled"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn sticker_pack_record_from_sqlite_row(row: &SqliteRow) -> StorageResult<StickerPackRecord> {
    let sticker_pack_json: String = row.get("sticker_pack_json");
    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");
    sticker_pack_record_from_values(
        row.get("id"),
        row.get("tenant_id"),
        row.get("owner_user_id"),
        row.get("compatibility_id"),
        row.get("title"),
        row.get("visibility"),
        row.get("source_provider"),
        &sticker_pack_json,
        &created_at,
        &updated_at,
    )
}

fn sticker_pack_record_from_pg_row(row: &PgRow) -> StorageResult<StickerPackRecord> {
    let sticker_pack_json: String = row.get("sticker_pack_json");
    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");
    sticker_pack_record_from_values(
        row.get("id"),
        row.get("tenant_id"),
        row.get("owner_user_id"),
        row.get("compatibility_id"),
        row.get("title"),
        row.get("visibility"),
        row.get("source_provider"),
        &sticker_pack_json,
        &created_at,
        &updated_at,
    )
}

#[allow(clippy::too_many_arguments)]
fn sticker_pack_record_from_values(
    id: String,
    tenant_id: String,
    owner_user_id: String,
    compatibility_id: String,
    title: String,
    visibility: String,
    source_provider: Option<String>,
    sticker_pack_json: &str,
    created_at: &str,
    updated_at: &str,
) -> StorageResult<StickerPackRecord> {
    let Some(visibility) = PackVisibility::from_storage(&visibility) else {
        return Err(StorageError::InvalidVisibility { visibility });
    };

    Ok(StickerPackRecord {
        id,
        tenant_id,
        owner_user_id,
        compatibility_id,
        title,
        visibility,
        source_provider,
        sticker_pack: StickerPack::from_json_str(sticker_pack_json)?,
        created_at: parse_rfc3339(created_at)?,
        updated_at: parse_rfc3339(updated_at)?,
    })
}

fn role_from_row(row: &SqliteRow, permissions: BTreeSet<Permission>) -> StorageResult<RoleRecord> {
    let created_at: String = row.get("created_at");
    Ok(RoleRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        name: row.get("name"),
        permissions,
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn role_from_pg_row(row: &PgRow, permissions: BTreeSet<Permission>) -> StorageResult<RoleRecord> {
    let created_at: String = row.get("created_at");
    Ok(RoleRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        name: row.get("name"),
        permissions,
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn oidc_provider_config_from_sqlite_row(
    row: &SqliteRow,
) -> StorageResult<OidcProviderConfigRecord> {
    let is_enabled: i64 = row.get("is_enabled");
    let allow_registration: i64 = row.get("allow_registration");
    oidc_provider_config_from_values(OidcProviderConfigValues {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        display_name: row.get("display_name"),
        issuer_url: row.get("issuer_url"),
        client_id: row.get("client_id"),
        client_secret: row.get("client_secret"),
        scopes_json: row.get("scopes_json"),
        is_enabled: is_enabled != 0,
        allow_registration: allow_registration != 0,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn oidc_provider_config_from_pg_row(row: &PgRow) -> StorageResult<OidcProviderConfigRecord> {
    oidc_provider_config_from_values(OidcProviderConfigValues {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        display_name: row.get("display_name"),
        issuer_url: row.get("issuer_url"),
        client_id: row.get("client_id"),
        client_secret: row.get("client_secret"),
        scopes_json: row.get("scopes_json"),
        is_enabled: row.get("is_enabled"),
        allow_registration: row.get("allow_registration"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

struct OidcProviderConfigValues {
    id: String,
    tenant_id: String,
    display_name: String,
    issuer_url: String,
    client_id: String,
    client_secret: String,
    scopes_json: String,
    is_enabled: bool,
    allow_registration: bool,
    created_at: String,
    updated_at: String,
}

fn oidc_provider_config_from_values(
    values: OidcProviderConfigValues,
) -> StorageResult<OidcProviderConfigRecord> {
    let scope_keys: Vec<String> = serde_json::from_str(&values.scopes_json)?;
    Ok(OidcProviderConfigRecord {
        id: values.id,
        tenant_id: values.tenant_id,
        display_name: values.display_name,
        issuer_url: values.issuer_url,
        client_id: values.client_id,
        client_secret: values.client_secret,
        scopes: scope_keys.into_iter().collect(),
        is_enabled: values.is_enabled,
        allow_registration: values.allow_registration,
        created_at: parse_rfc3339(&values.created_at)?,
        updated_at: parse_rfc3339(&values.updated_at)?,
    })
}

fn oidc_login_state_from_sqlite_row(row: &SqliteRow) -> OidcLoginStateRecord {
    OidcLoginStateRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        provider_id: row.get("provider_id"),
        state_hash: row.get("state_hash"),
        nonce_hash: row.get("nonce_hash"),
        redirect_uri: row.get("redirect_uri"),
        expires_at: row.get("expires_at"),
        consumed_at: row.get("consumed_at"),
        created_at: row.get("created_at"),
    }
}

fn oidc_login_state_from_pg_row(row: &PgRow) -> OidcLoginStateRecord {
    OidcLoginStateRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        provider_id: row.get("provider_id"),
        state_hash: row.get("state_hash"),
        nonce_hash: row.get("nonce_hash"),
        redirect_uri: row.get("redirect_uri"),
        expires_at: row.get("expires_at"),
        consumed_at: row.get("consumed_at"),
        created_at: row.get("created_at"),
    }
}

fn oidc_user_link_from_sqlite_row(row: &SqliteRow) -> OidcUserLinkRecord {
    OidcUserLinkRecord {
        tenant_id: row.get("tenant_id"),
        provider_id: row.get("provider_id"),
        provider_subject: row.get("provider_subject"),
        user_id: row.get("user_id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn oidc_user_link_from_pg_row(row: &PgRow) -> OidcUserLinkRecord {
    OidcUserLinkRecord {
        tenant_id: row.get("tenant_id"),
        provider_id: row.get("provider_id"),
        provider_subject: row.get("provider_subject"),
        user_id: row.get("user_id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn subscription_group_from_sqlite_row(row: &SqliteRow) -> StorageResult<SubscriptionGroupRecord> {
    let visibility: String = row.get("visibility");
    let Some(visibility) = PackVisibility::from_storage(&visibility) else {
        return Err(StorageError::InvalidVisibility { visibility });
    };
    let created_at: String = row.get("created_at");
    Ok(SubscriptionGroupRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        title: row.get("title"),
        visibility,
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn subscription_group_from_pg_row(row: &PgRow) -> StorageResult<SubscriptionGroupRecord> {
    let visibility: String = row.get("visibility");
    let Some(visibility) = PackVisibility::from_storage(&visibility) else {
        return Err(StorageError::InvalidVisibility { visibility });
    };
    let created_at: String = row.get("created_at");
    Ok(SubscriptionGroupRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        title: row.get("title"),
        visibility,
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn hash_password(password: &str) -> StorageResult<String> {
    let mut salt_bytes = [0_u8; 16];
    getrandom::fill(&mut salt_bytes).map_err(|error| StorageError::Random {
        message: error.to_string(),
    })?;
    let salt = SaltString::encode_b64(&salt_bytes).map_err(|error| StorageError::PasswordHash {
        message: error.to_string(),
    })?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| StorageError::PasswordHash {
            message: error.to_string(),
        })
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn validate_pat_id(id: &str) -> StorageResult<()> {
    if id.is_empty() || id.contains('_') {
        return Err(StorageError::InvalidPersonalAccessToken {
            reason: "token id must not be empty or contain '_'",
        });
    }
    Ok(())
}

fn validate_subscription_access_token_id(id: &str) -> StorageResult<()> {
    if id.is_empty() || id.contains('_') {
        return Err(StorageError::InvalidPersonalAccessToken {
            reason: "subscription access token id must not be empty or contain '_'",
        });
    }
    Ok(())
}

fn validate_web_session_id(id: &str) -> StorageResult<()> {
    if id.is_empty() || id.contains('_') {
        return Err(StorageError::InvalidPersonalAccessToken {
            reason: "web session id must not be empty or contain '_'",
        });
    }
    Ok(())
}

fn generate_pat_secret() -> StorageResult<String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| StorageError::Random {
        message: error.to_string(),
    })?;
    Ok(hex::encode(bytes))
}

fn hash_pat_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

fn parse_pat_token(token: &str) -> Option<(&str, &str)> {
    token.strip_prefix("msm_pat_")?.split_once('_')
}

fn parse_subscription_access_token(token: &str) -> Option<(&str, &str)> {
    token.strip_prefix("msm_sub_")?.split_once('_')
}

fn parse_web_session_token(token: &str) -> Option<(&str, &str)> {
    token.strip_prefix("msm_session_")?.split_once('_')
}

fn is_expired(expires_at: Option<&str>) -> bool {
    let Some(expires_at) = expires_at else {
        return false;
    };
    DateTime::parse_from_rfc3339(expires_at).map_or(true, |expires_at| {
        expires_at.with_timezone(&Utc) <= Utc::now()
    })
}

fn pat_record_from_sqlite_row(row: &SqliteRow) -> StorageResult<PersonalAccessTokenRecord> {
    pat_record_from_values(PatRecordValues {
        id: row.get("id"),
        user_id: row.get("user_id"),
        name: row.get("name"),
        token_hash: row.get("token_hash"),
        scopes_json: row.get("scopes_json"),
        expires_at: row.get("expires_at"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
    })
}

fn pat_record_from_pg_row(row: &PgRow) -> StorageResult<PersonalAccessTokenRecord> {
    pat_record_from_values(PatRecordValues {
        id: row.get("id"),
        user_id: row.get("user_id"),
        name: row.get("name"),
        token_hash: row.get("token_hash"),
        scopes_json: row.get("scopes_json"),
        expires_at: row.get("expires_at"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
    })
}

struct PatRecordValues {
    id: String,
    user_id: String,
    name: String,
    token_hash: String,
    scopes_json: String,
    expires_at: Option<String>,
    revoked_at: Option<String>,
    created_at: String,
}

fn pat_record_from_values(values: PatRecordValues) -> StorageResult<PersonalAccessTokenRecord> {
    let scope_keys: Vec<String> = serde_json::from_str(&values.scopes_json)?;
    let scopes = scope_keys
        .into_iter()
        .map(|scope| {
            Permission::from_key(&scope).ok_or(StorageError::InvalidPersonalAccessToken {
                reason: "unknown scope key",
            })
        })
        .collect::<StorageResult<BTreeSet<_>>>()?;

    Ok(PersonalAccessTokenRecord {
        id: values.id,
        user_id: values.user_id,
        name: values.name,
        token_hash: values.token_hash,
        scopes,
        expires_at: values.expires_at,
        revoked_at: values.revoked_at,
        created_at: values.created_at,
    })
}

fn web_session_from_sqlite_row(row: &SqliteRow) -> WebSessionRecord {
    web_session_from_values(WebSessionValues {
        id: row.get("id"),
        user_id: row.get("user_id"),
        session_hash: row.get("session_hash"),
        expires_at: row.get("expires_at"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn web_session_from_pg_row(row: &PgRow) -> WebSessionRecord {
    web_session_from_values(WebSessionValues {
        id: row.get("id"),
        user_id: row.get("user_id"),
        session_hash: row.get("session_hash"),
        expires_at: row.get("expires_at"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

struct WebSessionValues {
    id: String,
    user_id: String,
    session_hash: String,
    expires_at: Option<String>,
    revoked_at: Option<String>,
    created_at: String,
    updated_at: String,
}

fn web_session_from_values(values: WebSessionValues) -> WebSessionRecord {
    WebSessionRecord {
        id: values.id,
        user_id: values.user_id,
        session_hash: values.session_hash,
        expires_at: values.expires_at,
        revoked_at: values.revoked_at,
        created_at: values.created_at,
        updated_at: values.updated_at,
    }
}

fn subscription_access_token_from_sqlite_row(
    row: &SqliteRow,
) -> StorageResult<SubscriptionAccessTokenRecord> {
    subscription_access_token_from_values(SubscriptionAccessTokenValues {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        resource_type: row.get("resource_type"),
        resource_id: row.get("resource_id"),
        token_hash: row.get("token_hash"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn subscription_access_token_from_pg_row(
    row: &PgRow,
) -> StorageResult<SubscriptionAccessTokenRecord> {
    subscription_access_token_from_values(SubscriptionAccessTokenValues {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        resource_type: row.get("resource_type"),
        resource_id: row.get("resource_id"),
        token_hash: row.get("token_hash"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

struct SubscriptionAccessTokenValues {
    id: String,
    tenant_id: String,
    owner_user_id: String,
    resource_type: String,
    resource_id: String,
    token_hash: String,
    revoked_at: Option<String>,
    created_at: String,
    updated_at: String,
}

fn subscription_access_token_from_values(
    values: SubscriptionAccessTokenValues,
) -> StorageResult<SubscriptionAccessTokenRecord> {
    let resource_type = SubscriptionAccessResourceType::from_storage(&values.resource_type).ok_or(
        StorageError::InvalidPersonalAccessToken {
            reason: "unknown subscription access token resource type",
        },
    )?;

    Ok(SubscriptionAccessTokenRecord {
        id: values.id,
        tenant_id: values.tenant_id,
        owner_user_id: values.owner_user_id,
        resource_type,
        resource_id: values.resource_id,
        token_hash: values.token_hash,
        revoked_at: values.revoked_at,
        created_at: values.created_at,
        updated_at: values.updated_at,
    })
}

fn parse_oidc_login_state(token: &str) -> Option<(&str, &str)> {
    let rest = token.strip_prefix("msm_oidc_state_")?;
    rest.rsplit_once('_')
}

fn parse_oidc_nonce(token: &str) -> Option<(&str, &str)> {
    let rest = token.strip_prefix("msm_oidc_nonce_")?;
    rest.rsplit_once('_')
}

impl StorageRepository {
    async fn find_oidc_login_state_by_id(
        &self,
        id: &str,
    ) -> StorageResult<Option<OidcLoginStateRecord>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, provider_id, state_hash, nonce_hash, redirect_uri, expires_at,
                        consumed_at, created_at
                    FROM oidc_login_states
                    WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.as_ref().map(oidc_login_state_from_sqlite_row))
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, tenant_id, provider_id, state_hash, nonce_hash, redirect_uri, expires_at,
                        consumed_at, created_at
                    FROM oidc_login_states
                    WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.as_ref().map(oidc_login_state_from_pg_row))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, env};

    use chrono::{Duration, Utc};
    use msm_domain::{Permission, Sticker};

    use crate::{
        db::DbPool,
        models::{NewOidcProviderConfig, NewTag, PackVisibility, SubscriptionAccessResourceType},
        repositories::StorageRepository,
        DatabaseConfig,
    };

    #[tokio::test]
    async fn core_identity_records_work_on_sqlite() {
        let repo = test_repo().await;
        assert_core_identity_contract(&repo, "sqlite_core").await;
    }

    #[tokio::test]
    async fn core_identity_records_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_core_identity_contract(&repo, "postgres_core").await;
    }

    #[tokio::test]
    async fn pack_records_work_on_sqlite() {
        let repo = test_repo().await;
        assert_pack_contract(&repo, "sqlite_pack").await;
    }

    #[tokio::test]
    async fn pack_records_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_pack_contract(&repo, "postgres_pack").await;
    }

    #[tokio::test]
    async fn folder_records_work_on_sqlite() {
        let repo = test_repo().await;
        assert_folder_contract(&repo, "sqlite_folder").await;
    }

    #[tokio::test]
    async fn folder_records_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_folder_contract(&repo, "postgres_folder").await;
    }

    #[tokio::test]
    async fn tag_records_work_on_sqlite() {
        let repo = test_repo().await;
        assert_tag_contract(&repo, "sqlite_tag").await;
    }

    #[tokio::test]
    async fn tag_records_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_tag_contract(&repo, "postgres_tag").await;
    }

    #[tokio::test]
    async fn subscription_group_records_work_on_sqlite() {
        let repo = test_repo().await;
        assert_subscription_group_contract(&repo, "sqlite_subscription").await;
    }

    #[tokio::test]
    async fn subscription_group_records_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_subscription_group_contract(&repo, "postgres_subscription").await;
    }

    #[tokio::test]
    async fn metadata_memberships_work_on_sqlite() {
        let repo = test_repo().await;
        assert_metadata_membership_contract(&repo, "sqlite_membership").await;
    }

    #[tokio::test]
    async fn metadata_memberships_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_metadata_membership_contract(&repo, "postgres_membership").await;
    }

    #[tokio::test]
    async fn personal_access_token_records_work_on_sqlite() {
        let repo = test_repo().await;
        assert_personal_access_token_contract(&repo, "sqlite_pat").await;
    }

    #[tokio::test]
    async fn personal_access_token_records_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_personal_access_token_contract(&repo, "postgres_pat").await;
    }

    #[tokio::test]
    async fn web_session_records_work_on_sqlite() {
        let repo = test_repo().await;
        assert_web_session_contract(&repo, "sqlite_session").await;
    }

    #[tokio::test]
    async fn web_session_records_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_web_session_contract(&repo, "postgres_session").await;
    }

    #[tokio::test]
    async fn local_credentials_work_on_sqlite() {
        let repo = test_repo().await;
        assert_local_credential_contract(&repo, "sqlite_local").await;
    }

    #[tokio::test]
    async fn local_credentials_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_local_credential_contract(&repo, "postgres_local").await;
    }

    #[tokio::test]
    async fn oidc_records_work_on_sqlite() {
        let repo = test_repo().await;
        assert_oidc_contract(&repo, "sqlite_oidc").await;
    }

    #[tokio::test]
    async fn oidc_records_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_oidc_contract(&repo, "postgres_oidc").await;
    }

    #[tokio::test]
    async fn subscription_access_tokens_work_on_sqlite() {
        let repo = test_repo().await;
        assert_subscription_access_token_contract(&repo, "sqlite_subtoken").await;
    }

    #[tokio::test]
    async fn subscription_access_tokens_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_subscription_access_token_contract(&repo, "postgres_subtoken").await;
    }

    #[tokio::test]
    async fn tenant_admin_helpers_work_on_sqlite() {
        let repo = test_repo().await;
        assert_tenant_admin_helper_contract(&repo, "sqlite_admin").await;
    }

    #[tokio::test]
    async fn tenant_admin_helpers_work_on_postgres_when_configured() {
        let Some(repo) = optional_postgres_repo().await else {
            return;
        };

        assert_tenant_admin_helper_contract(&repo, "postgres_admin").await;
    }

    async fn assert_core_identity_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let user_id = format!("{prefix}_user_{suffix}");
        let email = format!("{prefix}_{suffix}@example.com");

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
        repo.create_user(&user_id, &email, "User").await.unwrap();
        repo.add_tenant_member(&tenant_id, &user_id, "admin")
            .await
            .unwrap();

        let tenant = repo.find_tenant(&tenant_id).await.unwrap().unwrap();
        assert_eq!(tenant.id, tenant_id);
        assert_eq!(tenant.name, "Tenant");
        assert!(!tenant.local_registration_enabled);

        let user = repo.find_user(&user_id).await.unwrap().unwrap();
        assert_eq!(user.email, email);
        assert!(!user.is_disabled);

        let member = repo
            .find_tenant_member(&tenant.id, &user.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(member.role, "admin");
        assert_eq!(
            repo.list_tenant_members(&tenant.id).await.unwrap(),
            vec![member.clone()]
        );
        assert_eq!(
            repo.list_user_tenant_members(&user.id).await.unwrap(),
            vec![member]
        );
    }

    async fn assert_tenant_admin_helper_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let user_id = format!("{prefix}_user_{suffix}");
        let email = format!("{prefix}_{suffix}@example.com");
        let role_id = format!("{prefix}_role_{suffix}");

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
        repo.create_user(&user_id, &email, "User").await.unwrap();
        repo.add_tenant_member(&tenant_id, &user_id, "user")
            .await
            .unwrap();

        let updated_tenant = repo
            .update_tenant_settings(
                &tenant_id,
                "Production Tenant",
                Some("https://cdn.example.test/msm"),
                false,
            )
            .await
            .unwrap();
        assert_eq!(updated_tenant.name, "Production Tenant");
        assert_eq!(
            updated_tenant.public_asset_url.as_deref(),
            Some("https://cdn.example.test/msm")
        );
        assert!(!updated_tenant.local_registration_enabled);
        let cleared_tenant = repo
            .update_tenant_settings(&tenant_id, "Production Tenant", None, true)
            .await
            .unwrap();
        assert_eq!(cleared_tenant.public_asset_url, None);
        assert!(cleared_tenant.local_registration_enabled);

        let disabled_user = repo.set_user_disabled(&user_id, true).await.unwrap();
        assert!(disabled_user.is_disabled);
        let enabled_user = repo.set_user_disabled(&user_id, false).await.unwrap();
        assert!(!enabled_user.is_disabled);

        let member = repo
            .upsert_tenant_member(&tenant_id, &user_id, "admin")
            .await
            .unwrap();
        assert_eq!(member.role, "admin");
        assert_eq!(
            repo.find_tenant_member(&tenant_id, &user_id)
                .await
                .unwrap()
                .unwrap(),
            member
        );

        let permissions = BTreeSet::from([Permission::PackRead, Permission::PackUpdate]);
        let created = repo
            .upsert_role_template(&role_id, &tenant_id, "Editors", &permissions)
            .await
            .unwrap();
        assert_eq!(created.id, role_id);
        assert_eq!(created.tenant_id.as_deref(), Some(tenant_id.as_str()));
        assert_eq!(created.permissions, permissions);

        let updated_permissions = BTreeSet::from([Permission::PackRead]);
        let updated = repo
            .upsert_role_template(&role_id, &tenant_id, "Readers", &updated_permissions)
            .await
            .unwrap();
        assert_eq!(updated.name, "Readers");
        assert_eq!(updated.permissions, updated_permissions);
        assert_eq!(
            repo.find_role_template(&tenant_id, &role_id)
                .await
                .unwrap()
                .unwrap(),
            updated
        );
        assert_eq!(
            repo.list_role_templates(&tenant_id).await.unwrap(),
            vec![updated]
        );
    }

    async fn assert_pack_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let user_id = format!("{prefix}_user_{suffix}");
        let email = format!("{prefix}_{suffix}@example.com");
        let pack_id = format!("{prefix}_pack_{suffix}");

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
        repo.create_user(&user_id, &email, "User").await.unwrap();
        repo.add_tenant_member(&tenant_id, &user_id, "admin")
            .await
            .unwrap();

        let pack = sample_pack();
        repo.upsert_sticker_pack(
            &pack_id,
            &tenant_id,
            &user_id,
            PackVisibility::Private,
            Some("telegram"),
            &pack,
        )
        .await
        .unwrap();

        assert_eq!(
            repo.find_sticker_pack(&pack_id).await.unwrap(),
            Some(pack.clone())
        );
        assert_eq!(
            repo.list_user_sticker_packs(&user_id).await.unwrap(),
            vec![pack.clone()]
        );

        let record = repo
            .find_sticker_pack_record(&pack_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(record.id, pack_id);
        assert_eq!(record.tenant_id, tenant_id);
        assert_eq!(record.owner_user_id, user_id);
        assert_eq!(record.visibility, PackVisibility::Private);
        assert_eq!(record.sticker_pack, pack);

        assert_eq!(
            repo.list_user_accessible_sticker_packs(&user_id)
                .await
                .unwrap(),
            vec![pack.clone()]
        );

        assert!(repo
            .update_sticker_pack_metadata(&pack_id, &user_id, "Renamed", PackVisibility::Public)
            .await
            .unwrap());
        let updated = repo
            .find_sticker_pack_record(&pack_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.title, "Renamed");
        assert_eq!(updated.visibility, PackVisibility::Public);
        assert_eq!(updated.sticker_pack.title, "Renamed");

        assert!(repo.delete_sticker_pack(&pack_id, &user_id).await.unwrap());
        assert!(!repo.delete_sticker_pack(&pack_id, &user_id).await.unwrap());
        assert!(repo.find_sticker_pack(&pack_id).await.unwrap().is_none());
    }

    async fn assert_folder_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let user_id = format!("{prefix}_user_{suffix}");
        let email = format!("{prefix}_{suffix}@example.com");
        let folder_id = format!("{prefix}_folder_{suffix}");

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
        repo.create_user(&user_id, &email, "User").await.unwrap();
        repo.add_tenant_member(&tenant_id, &user_id, "admin")
            .await
            .unwrap();

        let created = repo
            .create_folder(&folder_id, &tenant_id, &user_id, "Favorites")
            .await
            .unwrap();
        assert_eq!(created.id, folder_id);
        assert_eq!(created.tenant_id, tenant_id);
        assert_eq!(created.owner_user_id, user_id);
        assert_eq!(created.name, "Favorites");

        assert_eq!(
            repo.find_folder_record(&created.id).await.unwrap(),
            Some(created.clone())
        );
        assert_eq!(
            repo.list_folders(&created.tenant_id, &created.owner_user_id)
                .await
                .unwrap(),
            vec![created.clone()]
        );
        let renamed = repo.rename_folder(&created.id, "Renamed").await.unwrap();
        assert_eq!(renamed.name, "Renamed");
        assert!(repo.delete_folder(&created.id).await.unwrap());
        assert!(!repo.delete_folder(&created.id).await.unwrap());
        assert!(repo
            .find_folder_record(&created.id)
            .await
            .unwrap()
            .is_none());
    }

    async fn assert_tag_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let tag_id = format!("{prefix}_tag_{suffix}");

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();

        let created = repo
            .create_tag(NewTag {
                id: &tag_id,
                tenant_id: &tenant_id,
                name: "Favorites",
            })
            .await
            .unwrap();
        assert_eq!(created.id, tag_id);
        assert_eq!(created.tenant_id, tenant_id);
        assert_eq!(created.name, "Favorites");

        assert_eq!(
            repo.find_tag_record(&created.id).await.unwrap(),
            Some(created.clone())
        );
        assert_eq!(
            repo.list_tags(&created.tenant_id).await.unwrap(),
            vec![created.clone()]
        );
        assert!(repo.delete_tag(&created.id).await.unwrap());
        assert!(!repo.delete_tag(&created.id).await.unwrap());
        assert!(repo.find_tag_record(&created.id).await.unwrap().is_none());
    }

    async fn assert_subscription_group_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let user_id = format!("{prefix}_user_{suffix}");
        let email = format!("{prefix}_{suffix}@example.com");
        let group_id = format!("{prefix}_group_{suffix}");

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
        repo.create_user(&user_id, &email, "User").await.unwrap();
        repo.add_tenant_member(&tenant_id, &user_id, "admin")
            .await
            .unwrap();

        let created = repo
            .create_subscription_group(
                &group_id,
                &tenant_id,
                &user_id,
                "Favorites",
                PackVisibility::Private,
            )
            .await
            .unwrap();
        assert_eq!(created.id, group_id);
        assert_eq!(created.tenant_id, tenant_id);
        assert_eq!(created.owner_user_id, user_id);
        assert_eq!(created.title, "Favorites");
        assert_eq!(created.visibility, PackVisibility::Private);

        assert_eq!(
            repo.find_subscription_group_record(&created.id)
                .await
                .unwrap(),
            Some(created.clone())
        );
        assert_eq!(
            repo.list_subscription_groups(&created.tenant_id, &created.owner_user_id)
                .await
                .unwrap(),
            vec![created.clone()]
        );
        let renamed = repo
            .rename_subscription_group(&created.id, "Renamed")
            .await
            .unwrap();
        assert_eq!(renamed.title, "Renamed");
        assert!(repo.delete_subscription_group(&created.id).await.unwrap());
        assert!(!repo.delete_subscription_group(&created.id).await.unwrap());
        assert!(repo
            .find_subscription_group_record(&created.id)
            .await
            .unwrap()
            .is_none());
    }

    async fn assert_metadata_membership_contract(repo: &StorageRepository, prefix: &str) {
        let fixture = seed_metadata_membership_fixture(repo, prefix).await;

        assert_folder_membership_contract(repo, &fixture).await;
        assert_pack_tag_membership_contract(repo, &fixture).await;
        assert_subscription_membership_contract(repo, &fixture).await;
    }

    struct MetadataMembershipFixture {
        pack: String,
        second_pack: String,
        folder: String,
        tag: String,
        group: String,
    }

    async fn seed_metadata_membership_fixture(
        repo: &StorageRepository,
        prefix: &str,
    ) -> MetadataMembershipFixture {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let user_id = format!("{prefix}_user_{suffix}");
        let email = format!("{prefix}_{suffix}@example.com");
        let pack_id = format!("{prefix}_pack_{suffix}");
        let second_pack_id = format!("{prefix}_pack_2_{suffix}");
        let folder_id = format!("{prefix}_folder_{suffix}");
        let tag_id = format!("{prefix}_tag_{suffix}");
        let group_id = format!("{prefix}_group_{suffix}");

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
        repo.create_user(&user_id, &email, "User").await.unwrap();
        repo.add_tenant_member(&tenant_id, &user_id, "admin")
            .await
            .unwrap();

        let pack = sample_pack();
        let mut second_pack = sample_pack();
        second_pack.id = format!("MoreStickers:Telegram:Pack:{second_pack_id}");
        second_pack.title = "Sample 2".to_owned();
        second_pack.logo.id = format!("MoreStickers:Telegram:Sticker:{second_pack_id}:logo");
        second_pack.logo.sticker_pack_id = second_pack.id.clone();
        for (index, sticker) in second_pack.stickers.iter_mut().enumerate() {
            sticker.id = format!("MoreStickers:Telegram:Sticker:{second_pack_id}:{index}");
            sticker.sticker_pack_id = second_pack.id.clone();
        }
        repo.upsert_sticker_pack(
            &pack_id,
            &tenant_id,
            &user_id,
            PackVisibility::Private,
            Some("telegram"),
            &pack,
        )
        .await
        .unwrap();
        repo.upsert_sticker_pack(
            &second_pack_id,
            &tenant_id,
            &user_id,
            PackVisibility::Private,
            Some("telegram"),
            &second_pack,
        )
        .await
        .unwrap();

        repo.create_folder(&folder_id, &tenant_id, &user_id, "Favorites")
            .await
            .unwrap();
        repo.create_tag(NewTag {
            id: &tag_id,
            tenant_id: &tenant_id,
            name: "Animated",
        })
        .await
        .unwrap();
        repo.create_subscription_group(
            &group_id,
            &tenant_id,
            &user_id,
            "Daily",
            PackVisibility::Private,
        )
        .await
        .unwrap();

        MetadataMembershipFixture {
            pack: pack_id,
            second_pack: second_pack_id,
            folder: folder_id,
            tag: tag_id,
            group: group_id,
        }
    }

    async fn assert_folder_membership_contract(
        repo: &StorageRepository,
        fixture: &MetadataMembershipFixture,
    ) {
        let folder_pack = repo
            .add_pack_to_folder(&fixture.folder, &fixture.pack, 20)
            .await
            .unwrap();
        assert_eq!(folder_pack.folder_id, fixture.folder);
        assert_eq!(folder_pack.pack_id, fixture.pack);
        assert_eq!(folder_pack.sort_order, 20);
        repo.add_pack_to_folder(&folder_pack.folder_id, &fixture.second_pack, 10)
            .await
            .unwrap();
        repo.add_pack_to_folder(&folder_pack.folder_id, &fixture.pack, 5)
            .await
            .unwrap();
        assert_eq!(
            repo.list_folder_pack_ids(&folder_pack.folder_id)
                .await
                .unwrap(),
            vec![fixture.pack.clone(), fixture.second_pack.clone()]
        );
        assert!(repo
            .remove_pack_from_folder(&folder_pack.folder_id, &fixture.second_pack)
            .await
            .unwrap());
        assert!(!repo
            .remove_pack_from_folder(&folder_pack.folder_id, &fixture.second_pack)
            .await
            .unwrap());
        assert_eq!(
            repo.list_folder_pack_ids(&folder_pack.folder_id)
                .await
                .unwrap(),
            vec![fixture.pack.clone()]
        );
    }

    async fn assert_pack_tag_membership_contract(
        repo: &StorageRepository,
        fixture: &MetadataMembershipFixture,
    ) {
        let pack_tag = repo
            .add_tag_to_pack(&fixture.pack, &fixture.tag)
            .await
            .unwrap();
        assert_eq!(pack_tag.pack_id, fixture.pack);
        assert_eq!(pack_tag.tag_id, fixture.tag);
        repo.add_tag_to_pack(&pack_tag.pack_id, &pack_tag.tag_id)
            .await
            .unwrap();
        assert_eq!(
            repo.list_pack_tag_ids(&pack_tag.pack_id).await.unwrap(),
            vec![fixture.tag.clone()]
        );
        assert!(repo
            .remove_tag_from_pack(&pack_tag.pack_id, &pack_tag.tag_id)
            .await
            .unwrap());
        assert!(!repo
            .remove_tag_from_pack(&pack_tag.pack_id, &pack_tag.tag_id)
            .await
            .unwrap());
        assert!(repo
            .list_pack_tag_ids(&pack_tag.pack_id)
            .await
            .unwrap()
            .is_empty());
    }

    async fn assert_subscription_membership_contract(
        repo: &StorageRepository,
        fixture: &MetadataMembershipFixture,
    ) {
        let subscription_pack = repo
            .add_pack_to_subscription_group(&fixture.group, &fixture.pack, 20)
            .await
            .unwrap();
        assert_eq!(subscription_pack.subscription_group_id, fixture.group);
        assert_eq!(subscription_pack.pack_id, fixture.pack);
        assert_eq!(subscription_pack.sort_order, 20);
        repo.add_pack_to_subscription_group(
            &subscription_pack.subscription_group_id,
            &fixture.second_pack,
            10,
        )
        .await
        .unwrap();
        repo.add_pack_to_subscription_group(
            &subscription_pack.subscription_group_id,
            &fixture.pack,
            5,
        )
        .await
        .unwrap();
        assert_eq!(
            repo.list_subscription_pack_ids(&subscription_pack.subscription_group_id)
                .await
                .unwrap(),
            vec![fixture.pack.clone(), fixture.second_pack.clone()]
        );
        assert!(repo
            .remove_pack_from_subscription_group(
                &subscription_pack.subscription_group_id,
                &fixture.second_pack,
            )
            .await
            .unwrap());
        assert!(!repo
            .remove_pack_from_subscription_group(
                &subscription_pack.subscription_group_id,
                &fixture.second_pack,
            )
            .await
            .unwrap());
        assert_eq!(
            repo.list_subscription_pack_ids(&subscription_pack.subscription_group_id)
                .await
                .unwrap(),
            vec![fixture.pack.clone()]
        );
    }

    async fn assert_personal_access_token_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let user_id = format!("{prefix}_user_{suffix}");
        let token_id = format!("{}pat{}", prefix.replace('_', ""), suffix);
        let email = format!("{prefix}_{suffix}@example.com");
        let scopes = BTreeSet::from([
            msm_domain::Permission::PackRead,
            msm_domain::Permission::PatManage,
        ]);

        repo.create_user(&user_id, &email, "User").await.unwrap();
        let created = repo
            .create_personal_access_token(&token_id, &user_id, "CLI", &scopes, None)
            .await
            .unwrap();

        assert!(created.token.starts_with(&format!("msm_pat_{token_id}_")));
        assert_ne!(created.record.token_hash, created.token);
        assert_eq!(created.record.scopes, scopes);
        assert_eq!(
            repo.list_personal_access_tokens(&user_id).await.unwrap(),
            vec![created.record.clone()]
        );
        assert_eq!(
            repo.find_personal_access_token(&token_id).await.unwrap(),
            Some(created.record.clone())
        );
        assert_eq!(
            repo.verify_personal_access_token(&created.token)
                .await
                .unwrap(),
            Some(created.record.clone())
        );
        assert!(repo
            .verify_personal_access_token(&created.token.replace('a', "b"))
            .await
            .unwrap()
            .is_none());

        repo.revoke_personal_access_token(&token_id).await.unwrap();
        let revoked = repo
            .find_personal_access_token(&token_id)
            .await
            .unwrap()
            .unwrap();
        assert!(revoked.revoked_at.is_some());
        assert!(repo
            .verify_personal_access_token(&created.token)
            .await
            .unwrap()
            .is_none());
    }

    async fn assert_web_session_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let user_id = format!("{prefix}_user_{suffix}");
        let session_id = format!("{}session{}", prefix.replace('_', ""), suffix);
        let email = format!("{prefix}_{suffix}@example.com");

        repo.create_user(&user_id, &email, "User").await.unwrap();
        let created = repo
            .create_web_session(&session_id, &user_id, None)
            .await
            .unwrap();

        assert!(created
            .token
            .starts_with(&format!("msm_session_{session_id}_")));
        assert_ne!(created.record.session_hash, created.token);
        assert_eq!(
            repo.verify_web_session(&created.token).await.unwrap(),
            Some(created.record.clone())
        );
        assert!(repo
            .verify_web_session(&created.token.replace('a', "b"))
            .await
            .unwrap()
            .is_none());

        repo.revoke_web_session(&session_id).await.unwrap();
        assert!(repo
            .verify_web_session(&created.token)
            .await
            .unwrap()
            .is_none());
    }

    async fn assert_local_credential_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let user_id = format!("{prefix}_user_{suffix}");
        let email = format!("{prefix}_{suffix}@example.com");
        let password = "correct horse battery staple";

        let user = repo
            .create_local_user_with_password(&user_id, &email, "User", password)
            .await
            .unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.email, email);
        assert!(!user.is_disabled);

        let credential = repo
            .local_credential_for_user(&user_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(credential.user_id, user_id);
        assert!(credential.password_hash.starts_with("$argon2"));
        assert!(!credential.password_hash.contains(password));

        assert_eq!(
            repo.verify_local_user_password(&email, password)
                .await
                .unwrap(),
            Some(user)
        );
        assert!(repo
            .verify_local_user_password(&email, "wrong password")
            .await
            .unwrap()
            .is_none());
    }

    async fn assert_oidc_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let user_id = format!("{prefix}_user_{suffix}");
        let email = format!("{prefix}_{suffix}@example.com");
        let provider_id = format!("{prefix}_provider_{suffix}");

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
        repo.create_user(&user_id, &email, "User").await.unwrap();

        assert_oidc_provider_contract(repo, &tenant_id, &provider_id).await;
        assert_oidc_login_state_contract(repo, &tenant_id, &provider_id).await;
        assert_oidc_user_link_contract(repo, &tenant_id, &provider_id, &user_id, &email).await;
        assert!(repo
            .delete_oidc_provider_config(&tenant_id, &provider_id)
            .await
            .unwrap());
        assert!(repo
            .find_oidc_provider_config(&tenant_id, &provider_id)
            .await
            .unwrap()
            .is_none());
    }

    async fn assert_oidc_provider_contract(
        repo: &StorageRepository,
        tenant_id: &str,
        provider_id: &str,
    ) {
        let scopes = BTreeSet::from(["openid".to_owned(), "email".to_owned()]);
        let created_provider = repo
            .upsert_oidc_provider_config(NewOidcProviderConfig {
                id: provider_id,
                tenant_id,
                display_name: "Example",
                issuer_url: "https://accounts.example.com",
                client_id: "client-id",
                client_secret: "client-secret",
                scopes: &scopes,
                is_enabled: true,
                allow_registration: true,
            })
            .await
            .unwrap();
        assert_eq!(created_provider.scopes, scopes);

        let updated_scopes = BTreeSet::from(["openid".to_owned(), "profile".to_owned()]);
        let updated_provider = repo
            .upsert_oidc_provider_config(NewOidcProviderConfig {
                id: provider_id,
                tenant_id,
                display_name: "Example Workspace",
                issuer_url: "https://accounts.example.com",
                client_id: "client-id-2",
                client_secret: "client-secret-2",
                scopes: &updated_scopes,
                is_enabled: false,
                allow_registration: false,
            })
            .await
            .unwrap();
        assert_eq!(updated_provider.scopes, updated_scopes);
        assert!(!updated_provider.is_enabled);
        assert!(!updated_provider.allow_registration);
        assert_eq!(
            repo.list_oidc_provider_configs(tenant_id).await.unwrap(),
            vec![updated_provider.clone()]
        );
        assert_eq!(
            repo.find_oidc_provider_config(tenant_id, provider_id)
                .await
                .unwrap(),
            Some(updated_provider)
        );
    }

    async fn assert_oidc_login_state_contract(
        repo: &StorageRepository,
        tenant_id: &str,
        provider_id: &str,
    ) {
        let expires_at = (Utc::now() + Duration::minutes(5)).to_rfc3339();
        let state = repo
            .create_oidc_login_state(
                tenant_id,
                provider_id,
                "https://msm.example/auth/callback",
                &expires_at,
            )
            .await
            .unwrap();
        assert!(state.state.starts_with("msm_oidc_state_"));
        assert!(state.nonce.starts_with("msm_oidc_nonce_"));
        assert_ne!(state.record.state_hash, state.state);
        assert_ne!(state.record.nonce_hash, state.nonce);
        assert_eq!(
            repo.verify_oidc_login_state(&state.state, &state.nonce)
                .await
                .unwrap()
                .map(|record| record.id),
            Some(state.record.id.clone())
        );
        let consumed = repo
            .consume_oidc_login_state(&state.state, &state.nonce)
            .await
            .unwrap()
            .unwrap();
        assert!(consumed.consumed_at.is_some());
        assert!(repo
            .consume_oidc_login_state(&state.state, &state.nonce)
            .await
            .unwrap()
            .is_none());
    }

    async fn assert_oidc_user_link_contract(
        repo: &StorageRepository,
        tenant_id: &str,
        provider_id: &str,
        user_id: &str,
        email: &str,
    ) {
        let link = repo
            .upsert_oidc_user_link(tenant_id, provider_id, "subject-1", user_id, email, "User")
            .await
            .unwrap();
        assert_eq!(link.user_id, user_id);
        let updated_link = repo
            .upsert_oidc_user_link(
                tenant_id,
                provider_id,
                "subject-1",
                user_id,
                "user@example.org",
                "Updated User",
            )
            .await
            .unwrap();
        assert_eq!(updated_link.email, "user@example.org");
        assert_eq!(
            repo.find_oidc_user_link(tenant_id, provider_id, "subject-1")
                .await
                .unwrap(),
            Some(updated_link)
        );
    }

    async fn assert_subscription_access_token_contract(repo: &StorageRepository, prefix: &str) {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let tenant_id = format!("{prefix}_tenant_{suffix}");
        let user_id = format!("{prefix}_user_{suffix}");
        let token_id = format!("{}token{}", prefix.replace('_', ""), suffix);
        let group_token_id = format!("{}group{}", prefix.replace('_', ""), suffix);

        repo.create_tenant(&tenant_id, "Tenant").await.unwrap();
        repo.create_user(&user_id, &format!("{prefix}_{suffix}@example.com"), "User")
            .await
            .unwrap();

        let created = repo
            .create_subscription_access_token(
                &token_id,
                &tenant_id,
                &user_id,
                SubscriptionAccessResourceType::Pack,
                "pack_1",
            )
            .await
            .unwrap();
        assert!(created.token.starts_with(&format!("msm_sub_{token_id}_")));
        assert_eq!(
            created.record.resource_type,
            SubscriptionAccessResourceType::Pack
        );
        assert_eq!(
            repo.verify_subscription_access_token(&created.token)
                .await
                .unwrap(),
            Some(created.record.clone())
        );

        let group_token = repo
            .create_subscription_access_token(
                &group_token_id,
                &tenant_id,
                &user_id,
                SubscriptionAccessResourceType::SubscriptionGroup,
                "sub_1",
            )
            .await
            .unwrap();
        assert_eq!(
            group_token.record.resource_type,
            SubscriptionAccessResourceType::SubscriptionGroup
        );
        assert_eq!(
            repo.list_subscription_access_tokens(&user_id)
                .await
                .unwrap()
                .len(),
            2
        );

        let rotated = repo
            .rotate_subscription_access_token(&token_id)
            .await
            .unwrap();
        assert_ne!(rotated.token, created.token);
        assert!(repo
            .verify_subscription_access_token(&created.token)
            .await
            .unwrap()
            .is_none());
        assert_eq!(
            repo.verify_subscription_access_token(&rotated.token)
                .await
                .unwrap()
                .map(|record| record.id),
            Some(token_id.clone())
        );

        repo.revoke_subscription_access_token(&token_id)
            .await
            .unwrap();
        assert!(repo
            .verify_subscription_access_token(&rotated.token)
            .await
            .unwrap()
            .is_none());
        assert!(repo
            .find_subscription_access_token(&token_id)
            .await
            .unwrap()
            .unwrap()
            .revoked_at
            .is_some());
    }

    #[tokio::test]
    async fn repository_inserts_core_records() {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        let repo = StorageRepository::new(pool);

        repo.create_tenant("tenant_1", "Tenant").await.unwrap();
        repo.create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();
        repo.add_tenant_member("tenant_1", "user_1", "admin")
            .await
            .unwrap();

        let pack = sample_pack();
        repo.upsert_sticker_pack(
            "pack_1",
            "tenant_1",
            "user_1",
            PackVisibility::Private,
            Some("telegram"),
            &pack,
        )
        .await
        .unwrap();

        repo.create_subscription_group(
            "sub_1",
            "tenant_1",
            "user_1",
            "Favorites",
            PackVisibility::Private,
        )
        .await
        .unwrap();
        repo.add_pack_to_subscription_group("sub_1", "pack_1", 0)
            .await
            .unwrap();

        assert_eq!(repo.find_sticker_pack("pack_1").await.unwrap(), Some(pack));
        assert_eq!(
            repo.list_subscription_pack_ids("sub_1").await.unwrap(),
            vec!["pack_1".to_owned()]
        );
    }

    #[tokio::test]
    async fn tenant_members_can_be_listed_and_roles_updated() {
        let repo = test_repo().await;
        repo.create_tenant("tenant_1", "Tenant").await.unwrap();
        repo.create_user("admin_1", "admin@example.com", "Admin")
            .await
            .unwrap();
        repo.create_user("user_1", "user@example.com", "User")
            .await
            .unwrap();
        repo.add_tenant_member("tenant_1", "admin_1", "admin")
            .await
            .unwrap();

        let created = repo
            .upsert_tenant_member("tenant_1", "user_1", "user")
            .await
            .unwrap();
        assert_eq!(created.role, "user");

        let updated = repo
            .upsert_tenant_member("tenant_1", "user_1", "admin")
            .await
            .unwrap();
        assert_eq!(updated.role, "admin");

        let members = repo.list_tenant_members("tenant_1").await.unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].user_id, "admin_1");
        assert_eq!(members[1].user_id, "user_1");
        assert_eq!(
            repo.find_tenant_member("tenant_1", "user_1")
                .await
                .unwrap()
                .unwrap()
                .role,
            "admin"
        );
    }

    #[tokio::test]
    async fn tenant_settings_can_be_read_and_updated() {
        let repo = test_repo().await;
        repo.create_tenant("tenant_1", "Tenant").await.unwrap();

        let initial = repo.find_tenant("tenant_1").await.unwrap().unwrap();
        assert_eq!(initial.name, "Tenant");
        assert_eq!(initial.public_asset_url, None);
        assert!(!initial.local_registration_enabled);

        let updated = repo
            .update_tenant_settings(
                "tenant_1",
                "Production Tenant",
                Some("https://cdn.example.test/msm"),
                false,
            )
            .await
            .unwrap();
        assert_eq!(updated.name, "Production Tenant");
        assert_eq!(
            updated.public_asset_url.as_deref(),
            Some("https://cdn.example.test/msm")
        );
        assert!(!updated.local_registration_enabled);

        let cleared = repo
            .update_tenant_settings("tenant_1", "Production Tenant", None, true)
            .await
            .unwrap();
        assert_eq!(cleared.public_asset_url, None);
        assert!(cleared.local_registration_enabled);
    }

    #[tokio::test]
    async fn user_disabled_status_can_be_updated() {
        let repo = test_repo().await;
        repo.create_user("user_1", "user@example.com", "User")
            .await
            .unwrap();

        let disabled = repo.set_user_disabled("user_1", true).await.unwrap();
        assert!(disabled.is_disabled);

        let enabled = repo.set_user_disabled("user_1", false).await.unwrap();
        assert!(!enabled.is_disabled);
        assert!(!repo.find_user("user_1").await.unwrap().unwrap().is_disabled);
    }

    #[tokio::test]
    async fn role_templates_can_be_upserted_and_listed() {
        let repo = test_repo().await;
        repo.create_tenant("tenant_1", "Tenant").await.unwrap();
        let permissions = BTreeSet::from([
            msm_domain::Permission::PackRead,
            msm_domain::Permission::PackUpdate,
        ]);

        let created = repo
            .upsert_role_template("role_editor", "tenant_1", "Editors", &permissions)
            .await
            .unwrap();
        assert_eq!(created.id, "role_editor");
        assert_eq!(created.tenant_id.as_deref(), Some("tenant_1"));
        assert_eq!(created.permissions, permissions);

        let updated_permissions = BTreeSet::from([msm_domain::Permission::PackRead]);
        let updated = repo
            .upsert_role_template("role_editor", "tenant_1", "Readers", &updated_permissions)
            .await
            .unwrap();
        assert_eq!(updated.name, "Readers");
        assert_eq!(updated.permissions, updated_permissions);

        let roles = repo.list_role_templates("tenant_1").await.unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "Readers");
    }

    #[tokio::test]
    async fn oidc_provider_configs_can_be_upserted_listed_and_deleted() {
        let repo = test_repo().await;
        repo.create_tenant("tenant_1", "Tenant").await.unwrap();
        let scopes = BTreeSet::from(["email".to_owned(), "openid".to_owned()]);

        let created = repo
            .upsert_oidc_provider_config(NewOidcProviderConfig {
                id: "oidc_google",
                tenant_id: "tenant_1",
                display_name: "Google",
                issuer_url: "https://accounts.google.com",
                client_id: "client-id",
                client_secret: "client-secret",
                scopes: &scopes,
                is_enabled: true,
                allow_registration: false,
            })
            .await
            .unwrap();
        assert_eq!(created.display_name, "Google");
        assert_eq!(created.scopes, scopes);
        assert!(created.is_enabled);
        assert!(!created.allow_registration);

        let updated_scopes = BTreeSet::from([
            "email".to_owned(),
            "openid".to_owned(),
            "profile".to_owned(),
        ]);
        let updated = repo
            .upsert_oidc_provider_config(NewOidcProviderConfig {
                id: "oidc_google",
                tenant_id: "tenant_1",
                display_name: "Google Workspace",
                issuer_url: "https://accounts.google.com",
                client_id: "new-client-id",
                client_secret: "new-client-secret",
                scopes: &updated_scopes,
                is_enabled: false,
                allow_registration: true,
            })
            .await
            .unwrap();
        assert_eq!(updated.display_name, "Google Workspace");
        assert_eq!(updated.client_id, "new-client-id");
        assert_eq!(updated.client_secret, "new-client-secret");
        assert_eq!(updated.scopes, updated_scopes);
        assert!(!updated.is_enabled);
        assert!(updated.allow_registration);
        assert!(updated.updated_at >= updated.created_at);

        let listed = repo.list_oidc_provider_configs("tenant_1").await.unwrap();
        assert_eq!(listed, vec![updated.clone()]);
        let found = repo
            .find_oidc_provider_config("tenant_1", "oidc_google")
            .await
            .unwrap();
        assert_eq!(found, Some(updated));
        assert!(repo
            .delete_oidc_provider_config("tenant_1", "oidc_google")
            .await
            .unwrap());
        assert!(repo
            .find_oidc_provider_config("tenant_1", "oidc_google")
            .await
            .unwrap()
            .is_none());
        assert!(!repo
            .delete_oidc_provider_config("tenant_1", "oidc_google")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn oidc_login_states_are_hashed_and_consumed_once() {
        let repo = test_repo().await;
        repo.create_tenant("tenant_1", "Tenant").await.unwrap();
        let scopes = BTreeSet::from(["openid".to_owned()]);
        repo.upsert_oidc_provider_config(NewOidcProviderConfig {
            id: "oidc_google",
            tenant_id: "tenant_1",
            display_name: "Google",
            issuer_url: "https://accounts.google.com",
            client_id: "client-id",
            client_secret: "client-secret",
            scopes: &scopes,
            is_enabled: true,
            allow_registration: true,
        })
        .await
        .unwrap();
        let expires_at = (Utc::now() + Duration::minutes(10)).to_rfc3339();

        let created = repo
            .create_oidc_login_state(
                "tenant_1",
                "oidc_google",
                "https://msm.example/auth/callback",
                &expires_at,
            )
            .await
            .unwrap();

        assert!(created.state.starts_with("msm_oidc_state_"));
        assert!(created.nonce.starts_with("msm_oidc_nonce_"));
        assert_ne!(created.record.state_hash, created.state);
        assert_ne!(created.record.nonce_hash, created.nonce);
        let consumed = repo
            .consume_oidc_login_state(&created.state, &created.nonce)
            .await
            .unwrap()
            .expect("state should verify");
        assert_eq!(consumed.tenant_id, "tenant_1");
        assert_eq!(consumed.provider_id, "oidc_google");
        assert!(consumed.consumed_at.is_some());
        assert!(repo
            .consume_oidc_login_state(&created.state, &created.nonce)
            .await
            .unwrap()
            .is_none());
        assert!(repo
            .consume_oidc_login_state(&created.state.replace('a', "b"), &created.nonce)
            .await
            .unwrap()
            .is_none());
        assert!(repo
            .consume_oidc_login_state(&created.state, &created.nonce.replace('a', "b"))
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn oidc_user_links_can_be_upserted_and_found() {
        let repo = test_repo().await;
        repo.create_tenant("tenant_1", "Tenant").await.unwrap();
        repo.create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();
        let scopes = BTreeSet::from(["openid".to_owned()]);
        repo.upsert_oidc_provider_config(NewOidcProviderConfig {
            id: "oidc_google",
            tenant_id: "tenant_1",
            display_name: "Google",
            issuer_url: "https://accounts.google.com",
            client_id: "client-id",
            client_secret: "client-secret",
            scopes: &scopes,
            is_enabled: true,
            allow_registration: true,
        })
        .await
        .unwrap();

        let created = repo
            .upsert_oidc_user_link(
                "tenant_1",
                "oidc_google",
                "google-subject",
                "user_1",
                "leko@example.com",
                "Leko",
            )
            .await
            .unwrap();
        assert_eq!(created.user_id, "user_1");

        let updated = repo
            .upsert_oidc_user_link(
                "tenant_1",
                "oidc_google",
                "google-subject",
                "user_1",
                "leko@leko.moe",
                "Leko OwO",
            )
            .await
            .unwrap();
        assert_eq!(updated.email, "leko@leko.moe");
        assert_eq!(updated.display_name, "Leko OwO");
        assert!(updated.updated_at >= updated.created_at);
        assert_eq!(
            repo.find_oidc_user_link("tenant_1", "oidc_google", "google-subject")
                .await
                .unwrap(),
            Some(updated)
        );
    }

    #[tokio::test]
    async fn personal_access_tokens_verify_and_revoke() {
        let repo = test_repo().await;
        repo.create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();
        let scopes = BTreeSet::from([msm_domain::Permission::PackRead]);

        let created = repo
            .create_personal_access_token("pat1", "user_1", "CLI", &scopes, None)
            .await
            .unwrap();

        assert!(created.token.starts_with("msm_pat_pat1_"));
        assert_ne!(created.record.token_hash, created.token);
        let listed = repo.list_personal_access_tokens("user_1").await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].scopes, scopes);

        let verified = repo
            .verify_personal_access_token(&created.token)
            .await
            .unwrap()
            .expect("token should verify");
        assert_eq!(verified.id, "pat1");

        let invalid = created.token.replace('a', "b");
        assert!(repo
            .verify_personal_access_token(&invalid)
            .await
            .unwrap()
            .is_none());

        repo.revoke_personal_access_token("pat1").await.unwrap();
        assert!(repo
            .verify_personal_access_token(&created.token)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn personal_access_tokens_reject_expired_tokens() {
        let repo = test_repo().await;
        repo.create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();
        let scopes = BTreeSet::from([msm_domain::Permission::PackRead]);
        let expires_at = (Utc::now() - Duration::days(1)).to_rfc3339();

        let created = repo
            .create_personal_access_token("pat1", "user_1", "Expired", &scopes, Some(&expires_at))
            .await
            .unwrap();

        assert!(repo
            .verify_personal_access_token(&created.token)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn personal_access_tokens_reject_invalid_token_ids() {
        let repo = test_repo().await;
        let scopes = BTreeSet::from([msm_domain::Permission::PackRead]);

        let error = repo
            .create_personal_access_token("pat_1", "user_1", "Invalid", &scopes, None)
            .await
            .expect_err("token IDs with underscores should be rejected");

        assert!(error.to_string().contains("token id"));
    }

    #[tokio::test]
    async fn web_sessions_verify_and_revoke() {
        let repo = test_repo().await;
        repo.create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();

        let created = repo
            .create_web_session("session1", "user_1", None)
            .await
            .unwrap();

        assert!(created.token.starts_with("msm_session_session1_"));
        assert_ne!(created.record.session_hash, created.token);
        let verified = repo
            .verify_web_session(&created.token)
            .await
            .unwrap()
            .expect("session should verify");
        assert_eq!(verified.id, "session1");
        assert_eq!(verified.user_id, "user_1");

        repo.revoke_web_session("session1").await.unwrap();
        assert!(repo
            .verify_web_session(&created.token)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn local_credentials_register_and_verify_passwords() {
        let repo = test_repo().await;

        let user = repo
            .create_local_user_with_password(
                "user_1",
                "leko@example.com",
                "Leko",
                "correct horse battery staple",
            )
            .await
            .unwrap();

        assert_eq!(user.id, "user_1");
        let credential = repo
            .local_credential_for_user("user_1")
            .await
            .unwrap()
            .unwrap();
        assert!(credential.password_hash.starts_with("$argon2"));
        assert!(!credential
            .password_hash
            .contains("correct horse battery staple"));

        let verified = repo
            .verify_local_user_password("leko@example.com", "correct horse battery staple")
            .await
            .unwrap()
            .expect("password should verify");
        assert_eq!(verified.id, "user_1");

        assert!(repo
            .verify_local_user_password("leko@example.com", "wrong password")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn local_credentials_reject_duplicate_user_credentials() {
        let repo = test_repo().await;
        repo.create_local_user_with_password("user_1", "leko@example.com", "Leko", "password")
            .await
            .unwrap();

        let duplicate = repo
            .create_local_user_with_password("user_1", "other@example.com", "Other", "password")
            .await;

        assert!(duplicate.is_err());
    }

    #[tokio::test]
    async fn updates_owned_sticker_pack_metadata() {
        let repo = seeded_pack_repo().await;

        let updated = repo
            .update_sticker_pack_metadata(
                "pack_1",
                "user_1",
                "Renamed Pack",
                PackVisibility::Public,
            )
            .await
            .unwrap();

        assert!(updated);
        let pack = repo
            .find_sticker_pack("pack_1")
            .await
            .unwrap()
            .expect("pack should exist");
        assert_eq!(pack.title, "Renamed Pack");
        let listed = repo.list_user_sticker_packs("user_1").await.unwrap();
        assert_eq!(listed[0].title, "Renamed Pack");
    }

    #[tokio::test]
    async fn update_sticker_pack_metadata_rejects_non_owner() {
        let repo = seeded_pack_repo().await;

        let updated = repo
            .update_sticker_pack_metadata("pack_1", "user_2", "Nope", PackVisibility::Public)
            .await
            .unwrap();

        assert!(!updated);
        assert_eq!(
            repo.find_sticker_pack("pack_1")
                .await
                .unwrap()
                .unwrap()
                .title,
            "Sample"
        );
    }

    #[tokio::test]
    async fn deletes_owned_sticker_pack() {
        let repo = seeded_pack_repo().await;

        let deleted = repo.delete_sticker_pack("pack_1", "user_1").await.unwrap();

        assert!(deleted);
        assert!(repo.find_sticker_pack("pack_1").await.unwrap().is_none());
        assert!(repo
            .list_user_sticker_packs("user_1")
            .await
            .unwrap()
            .is_empty());
    }

    async fn test_repo() -> StorageRepository {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        StorageRepository::new(pool)
    }

    async fn optional_postgres_repo() -> Option<StorageRepository> {
        let url = env::var("MSM_TEST_POSTGRES_URL").ok()?;
        let config = DatabaseConfig::parse(url).unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        Some(StorageRepository::new(pool))
    }

    async fn seeded_pack_repo() -> StorageRepository {
        let repo = test_repo().await;
        repo.create_tenant("tenant_1", "Tenant").await.unwrap();
        repo.create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();
        repo.upsert_sticker_pack(
            "pack_1",
            "tenant_1",
            "user_1",
            PackVisibility::Private,
            Some("telegram"),
            &sample_pack(),
        )
        .await
        .unwrap();
        repo
    }

    pub(crate) fn sample_pack() -> msm_domain::StickerPack {
        let sticker = Sticker {
            id: "MoreStickers:Telegram:Sticker:sample:file".to_owned(),
            image: "https://msm.example/assets/packs/sample/file.webp".to_owned(),
            title: "file".to_owned(),
            sticker_pack_id: "MoreStickers:Telegram:Pack:sample".to_owned(),
            filename: Some("file.webp".to_owned()),
            is_animated: Some(false),
        };

        msm_domain::StickerPack {
            id: "MoreStickers:Telegram:Pack:sample".to_owned(),
            title: "Sample".to_owned(),
            author: None,
            logo: sticker.clone(),
            stickers: vec![sticker],
        }
    }
}
