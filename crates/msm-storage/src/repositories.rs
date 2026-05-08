use std::collections::BTreeSet;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use msm_domain::{Permission, StickerPack};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use subtle::ConstantTimeEq;

use crate::{
    models::{
        CreatedPersonalAccessToken, FolderPackRecord, FolderRecord, LocalUserCredentialRecord,
        NewTag, PackTagRecord, PackVisibility, PersonalAccessTokenRecord, StickerPackRecord,
        SubscriptionGroupPackRecord, SubscriptionGroupRecord, TagRecord, UserRecord,
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

    /// Creates a tenant row.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn create_tenant(&self, id: &str, name: &str) -> StorageResult<()> {
        let now = now();
        sqlx::query("INSERT INTO tenants (id, name, created_at) VALUES (?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(now)
            .execute(self.sqlite()?)
            .await?;
        Ok(())
    }

    /// Creates a local user row.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn create_user(
        &self,
        id: &str,
        email: &str,
        display_name: &str,
    ) -> StorageResult<()> {
        let now = now();
        sqlx::query(
            "INSERT INTO users (id, email, display_name, is_disabled, created_at) VALUES (?, ?, ?, 0, ?)",
        )
        .bind(id)
        .bind(email)
        .bind(display_name)
        .bind(now)
        .execute(self.sqlite()?)
        .await?;
        Ok(())
    }

    /// Creates a local user profile and password credential.
    ///
    /// # Errors
    ///
    /// Returns an error when password hashing fails, the repository is not backed by `SQLite`, or
    /// SQL fails.
    pub async fn create_local_user_with_password(
        &self,
        id: &str,
        email: &str,
        display_name: &str,
        password: &str,
    ) -> StorageResult<UserRecord> {
        let password_hash = hash_password(password)?;
        let now = now();
        let sqlite = self.sqlite()?;
        let mut tx = sqlite.begin().await?;

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
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn local_credential_for_user(
        &self,
        user_id: &str,
    ) -> StorageResult<Option<LocalUserCredentialRecord>> {
        let row = sqlx::query(
            "SELECT user_id, password_hash, created_at, updated_at
            FROM local_user_credentials
            WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(self.sqlite()?)
        .await?;

        Ok(row.map(|row| LocalUserCredentialRecord {
            user_id: row.get("user_id"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    /// Verifies a local user password by email.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, or stored user
    /// timestamps are invalid.
    pub async fn verify_local_user_password(
        &self,
        email: &str,
        password: &str,
    ) -> StorageResult<Option<UserRecord>> {
        let row = sqlx::query(
            "SELECT users.id, users.email, users.display_name, users.is_disabled, users.created_at,
                local_user_credentials.password_hash
            FROM users
            JOIN local_user_credentials ON local_user_credentials.user_id = users.id
            WHERE users.email = ?",
        )
        .bind(email)
        .fetch_optional(self.sqlite()?)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
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

    /// Adds a user to a tenant with a coarse role.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn add_tenant_member(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: &str,
    ) -> StorageResult<()> {
        let now = now();
        sqlx::query(
            "INSERT INTO tenant_members (tenant_id, user_id, role, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(role)
        .bind(now)
        .execute(self.sqlite()?)
        .await?;
        Ok(())
    }

    /// Inserts or updates a sticker pack and replaces its sticker rows.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails, the repository is not backed by `SQLite`, or SQL fails.
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
        let sqlite = self.sqlite()?;
        let mut tx = sqlite.begin().await?;

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
        .bind(pack_json)
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
        Ok(())
    }

    /// Creates a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn create_folder(
        &self,
        id: &str,
        tenant_id: &str,
        owner_user_id: &str,
        name: &str,
    ) -> StorageResult<FolderRecord> {
        let now = now();
        sqlx::query(
            "INSERT INTO folders (id, tenant_id, owner_user_id, name, created_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(tenant_id)
        .bind(owner_user_id)
        .bind(name)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

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
    /// Returns an error when the repository is not backed by `SQLite` or SQL/timestamp parsing fails.
    pub async fn list_folders(
        &self,
        tenant_id: &str,
        owner_user_id: &str,
    ) -> StorageResult<Vec<FolderRecord>> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, name, created_at
            FROM folders
            WHERE tenant_id = ? AND owner_user_id = ?
            ORDER BY created_at, id",
        )
        .bind(tenant_id)
        .bind(owner_user_id)
        .fetch_all(self.sqlite()?)
        .await?;

        rows.iter().map(folder_from_row).collect()
    }

    /// Renames a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, the folder does
    /// not exist, or timestamp parsing fails.
    pub async fn rename_folder(&self, id: &str, name: &str) -> StorageResult<FolderRecord> {
        sqlx::query("UPDATE folders SET name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(self.sqlite()?)
            .await?;
        self.find_folder(id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Deletes a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn delete_folder(&self, id: &str) -> StorageResult<bool> {
        let result = sqlx::query("DELETE FROM folders WHERE id = ?")
            .bind(id)
            .execute(self.sqlite()?)
            .await?;
        Ok(result.rows_affected() == 1)
    }

    async fn find_folder(&self, id: &str) -> StorageResult<Option<FolderRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, name, created_at
            FROM folders
            WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.sqlite()?)
        .await?;

        row.as_ref().map(folder_from_row).transpose()
    }

    /// Adds a sticker pack to a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn add_pack_to_folder(
        &self,
        folder_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> StorageResult<FolderPackRecord> {
        sqlx::query(
            "INSERT INTO folder_packs (folder_id, pack_id, sort_order)
            VALUES (?, ?, ?)
            ON CONFLICT(folder_id, pack_id) DO UPDATE SET sort_order = excluded.sort_order",
        )
        .bind(folder_id)
        .bind(pack_id)
        .bind(sort_order)
        .execute(self.sqlite()?)
        .await?;

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
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn list_folder_pack_ids(&self, folder_id: &str) -> StorageResult<Vec<String>> {
        let rows = sqlx::query(
            "SELECT pack_id FROM folder_packs
            WHERE folder_id = ?
            ORDER BY sort_order, pack_id",
        )
        .bind(folder_id)
        .fetch_all(self.sqlite()?)
        .await?;

        Ok(rows.into_iter().map(|row| row.get("pack_id")).collect())
    }

    /// Removes a sticker pack from a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn remove_pack_from_folder(
        &self,
        folder_id: &str,
        pack_id: &str,
    ) -> StorageResult<bool> {
        let result = sqlx::query("DELETE FROM folder_packs WHERE folder_id = ? AND pack_id = ?")
            .bind(folder_id)
            .bind(pack_id)
            .execute(self.sqlite()?)
            .await?;
        Ok(result.rows_affected() == 1)
    }

    /// Creates a tag.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn create_tag(&self, tag: NewTag<'_>) -> StorageResult<TagRecord> {
        let now = now();
        sqlx::query("INSERT INTO tags (id, tenant_id, name, created_at) VALUES (?, ?, ?, ?)")
            .bind(tag.id)
            .bind(tag.tenant_id)
            .bind(tag.name)
            .bind(&now)
            .execute(self.sqlite()?)
            .await?;

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
    /// Returns an error when the repository is not backed by `SQLite` or SQL/timestamp parsing fails.
    pub async fn list_tags(&self, tenant_id: &str) -> StorageResult<Vec<TagRecord>> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, created_at
            FROM tags
            WHERE tenant_id = ?
            ORDER BY name, id",
        )
        .bind(tenant_id)
        .fetch_all(self.sqlite()?)
        .await?;

        rows.iter().map(tag_from_row).collect()
    }

    /// Deletes a tag.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn delete_tag(&self, id: &str) -> StorageResult<bool> {
        let result = sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(id)
            .execute(self.sqlite()?)
            .await?;
        Ok(result.rows_affected() == 1)
    }

    /// Adds a tag to a sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn add_tag_to_pack(
        &self,
        pack_id: &str,
        tag_id: &str,
    ) -> StorageResult<PackTagRecord> {
        sqlx::query(
            "INSERT OR IGNORE INTO pack_tags (pack_id, tag_id)
            VALUES (?, ?)",
        )
        .bind(pack_id)
        .bind(tag_id)
        .execute(self.sqlite()?)
        .await?;

        Ok(PackTagRecord {
            pack_id: pack_id.to_owned(),
            tag_id: tag_id.to_owned(),
        })
    }

    /// Lists tag IDs assigned to a sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn list_pack_tag_ids(&self, pack_id: &str) -> StorageResult<Vec<String>> {
        let rows = sqlx::query(
            "SELECT tag_id FROM pack_tags
            WHERE pack_id = ?
            ORDER BY tag_id",
        )
        .bind(pack_id)
        .fetch_all(self.sqlite()?)
        .await?;

        Ok(rows.into_iter().map(|row| row.get("tag_id")).collect())
    }

    /// Removes a tag from a sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn remove_tag_from_pack(&self, pack_id: &str, tag_id: &str) -> StorageResult<bool> {
        let result = sqlx::query("DELETE FROM pack_tags WHERE pack_id = ? AND tag_id = ?")
            .bind(pack_id)
            .bind(tag_id)
            .execute(self.sqlite()?)
            .await?;
        Ok(result.rows_affected() == 1)
    }

    /// Creates a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn create_subscription_group(
        &self,
        id: &str,
        tenant_id: &str,
        owner_user_id: &str,
        title: &str,
        visibility: PackVisibility,
    ) -> StorageResult<SubscriptionGroupRecord> {
        let now = now();
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
        .execute(self.sqlite()?)
        .await?;
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
    /// Returns an error when the repository is not backed by `SQLite` or SQL/timestamp parsing fails.
    pub async fn list_subscription_groups(
        &self,
        tenant_id: &str,
        owner_user_id: &str,
    ) -> StorageResult<Vec<SubscriptionGroupRecord>> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, title, visibility, created_at
            FROM subscription_groups
            WHERE tenant_id = ? AND owner_user_id = ?
            ORDER BY created_at, id",
        )
        .bind(tenant_id)
        .bind(owner_user_id)
        .fetch_all(self.sqlite()?)
        .await?;

        rows.iter().map(subscription_group_from_row).collect()
    }

    /// Renames a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, the group does
    /// not exist, or timestamp parsing fails.
    pub async fn rename_subscription_group(
        &self,
        id: &str,
        title: &str,
    ) -> StorageResult<SubscriptionGroupRecord> {
        sqlx::query("UPDATE subscription_groups SET title = ? WHERE id = ?")
            .bind(title)
            .bind(id)
            .execute(self.sqlite()?)
            .await?;
        self.find_subscription_group(id)
            .await?
            .ok_or(StorageError::Sqlx(sqlx::Error::RowNotFound))
    }

    /// Deletes a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn delete_subscription_group(&self, id: &str) -> StorageResult<bool> {
        let result = sqlx::query("DELETE FROM subscription_groups WHERE id = ?")
            .bind(id)
            .execute(self.sqlite()?)
            .await?;
        Ok(result.rows_affected() == 1)
    }

    async fn find_subscription_group(
        &self,
        id: &str,
    ) -> StorageResult<Option<SubscriptionGroupRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, title, visibility, created_at
            FROM subscription_groups
            WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.sqlite()?)
        .await?;

        row.as_ref().map(subscription_group_from_row).transpose()
    }

    /// Adds a sticker pack to a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or the insert fails.
    pub async fn add_pack_to_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> StorageResult<SubscriptionGroupPackRecord> {
        sqlx::query(
            "INSERT INTO subscription_group_packs (
                subscription_group_id, pack_id, sort_order
            ) VALUES (?, ?, ?)
            ON CONFLICT(subscription_group_id, pack_id) DO UPDATE SET sort_order = excluded.sort_order",
        )
        .bind(subscription_group_id)
        .bind(pack_id)
        .bind(sort_order)
        .execute(self.sqlite()?)
        .await?;

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
    /// Returns an error when the repository is not backed by `SQLite` or SQL/JSON parsing fails.
    pub async fn find_sticker_pack(&self, id: &str) -> StorageResult<Option<StickerPack>> {
        let row = sqlx::query("SELECT sticker_pack_json FROM sticker_packs WHERE id = ?")
            .bind(id)
            .fetch_optional(self.sqlite()?)
            .await?;

        row.map(|row| {
            let json: String = row.get("sticker_pack_json");
            StickerPack::from_json_str(&json).map_err(Into::into)
        })
        .transpose()
    }

    /// Finds a sticker pack record by internal pack ID, including owner and tenant metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, JSON parsing
    /// fails, or timestamps are invalid.
    pub async fn find_sticker_pack_record(
        &self,
        id: &str,
    ) -> StorageResult<Option<StickerPackRecord>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_user_id, compatibility_id, title, visibility,
                source_provider, sticker_pack_json, created_at, updated_at
            FROM sticker_packs
            WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.sqlite()?)
        .await?;

        row.map(|row| {
            let visibility: String = row.get("visibility");
            let Some(visibility) = PackVisibility::from_storage(&visibility) else {
                return Err(StorageError::InvalidVisibility { visibility });
            };
            let sticker_pack_json: String = row.get("sticker_pack_json");
            let created_at: String = row.get("created_at");
            let updated_at: String = row.get("updated_at");

            Ok(StickerPackRecord {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                owner_user_id: row.get("owner_user_id"),
                compatibility_id: row.get("compatibility_id"),
                title: row.get("title"),
                visibility,
                source_provider: row.get("source_provider"),
                sticker_pack: StickerPack::from_json_str(&sticker_pack_json)?,
                created_at: parse_rfc3339(&created_at)?,
                updated_at: parse_rfc3339(&updated_at)?,
            })
        })
        .transpose()
    }

    /// Updates owned sticker pack metadata without changing sticker contents.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails, the repository is not backed by `SQLite`, or SQL fails.
    pub async fn update_sticker_pack_metadata(
        &self,
        id: &str,
        owner_user_id: &str,
        title: &str,
        visibility: PackVisibility,
    ) -> StorageResult<bool> {
        let sqlite = self.sqlite()?;
        let row = sqlx::query(
            "SELECT sticker_pack_json FROM sticker_packs WHERE id = ? AND owner_user_id = ?",
        )
        .bind(id)
        .bind(owner_user_id)
        .fetch_optional(sqlite)
        .await?;

        let Some(row) = row else {
            return Ok(false);
        };

        let json: String = row.get("sticker_pack_json");
        let mut pack = StickerPack::from_json_str(&json)?;
        pack.title = title.to_owned();
        let pack_json = serde_json::to_string(&pack)?;

        let result = sqlx::query(
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
        .execute(sqlite)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Deletes an owned sticker pack.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn delete_sticker_pack(&self, id: &str, owner_user_id: &str) -> StorageResult<bool> {
        let result = sqlx::query("DELETE FROM sticker_packs WHERE id = ? AND owner_user_id = ?")
            .bind(id)
            .bind(owner_user_id)
            .execute(self.sqlite()?)
            .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Lists sticker packs owned by a user.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL/JSON parsing fails.
    pub async fn list_user_sticker_packs(&self, user_id: &str) -> StorageResult<Vec<StickerPack>> {
        let rows = sqlx::query(
            "SELECT sticker_pack_json FROM sticker_packs WHERE owner_user_id = ? ORDER BY title, id",
        )
        .bind(user_id)
        .fetch_all(self.sqlite()?)
        .await?;

        rows.into_iter()
            .map(|row| {
                let json: String = row.get("sticker_pack_json");
                StickerPack::from_json_str(&json).map_err(Into::into)
            })
            .collect()
    }

    /// Creates a Personal Access Token and returns the raw token once.
    ///
    /// # Errors
    ///
    /// Returns an error when the token ID is invalid, random generation fails, scope serialization
    /// fails, the repository is not backed by `SQLite`, or SQL fails.
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

        sqlx::query(
            "INSERT INTO personal_access_tokens (
                id, user_id, name, token_hash, scopes_json, expires_at, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(&token_hash)
        .bind(scopes_json)
        .bind(expires_at)
        .bind(&now)
        .execute(self.sqlite()?)
        .await?;

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
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, or a stored scope
    /// key is invalid.
    pub async fn list_personal_access_tokens(
        &self,
        user_id: &str,
    ) -> StorageResult<Vec<PersonalAccessTokenRecord>> {
        let rows = sqlx::query(
            "SELECT id, user_id, name, token_hash, scopes_json, expires_at, revoked_at, created_at
            FROM personal_access_tokens
            WHERE user_id = ?
            ORDER BY created_at DESC, id",
        )
        .bind(user_id)
        .fetch_all(self.sqlite()?)
        .await?;

        rows.iter().map(pat_record_from_row).collect()
    }

    /// Verifies a Personal Access Token and returns the active record when valid.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite`, SQL fails, or a stored scope
    /// key is invalid.
    pub async fn verify_personal_access_token(
        &self,
        token: &str,
    ) -> StorageResult<Option<PersonalAccessTokenRecord>> {
        let Some((id, secret)) = parse_pat_token(token) else {
            return Ok(None);
        };

        let row = sqlx::query(
            "SELECT id, user_id, name, token_hash, scopes_json, expires_at, revoked_at, created_at
            FROM personal_access_tokens
            WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.sqlite()?)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let record = pat_record_from_row(&row)?;

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
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn revoke_personal_access_token(&self, id: &str) -> StorageResult<()> {
        sqlx::query("UPDATE personal_access_tokens SET revoked_at = ? WHERE id = ?")
            .bind(now())
            .bind(id)
            .execute(self.sqlite()?)
            .await?;
        Ok(())
    }

    /// Lists pack IDs in a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn list_subscription_pack_ids(
        &self,
        subscription_group_id: &str,
    ) -> StorageResult<Vec<String>> {
        let rows = sqlx::query(
            "SELECT pack_id FROM subscription_group_packs
            WHERE subscription_group_id = ?
            ORDER BY sort_order, pack_id",
        )
        .bind(subscription_group_id)
        .fetch_all(self.sqlite()?)
        .await?;

        Ok(rows.into_iter().map(|row| row.get("pack_id")).collect())
    }

    /// Removes a sticker pack from a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by `SQLite` or SQL fails.
    pub async fn remove_pack_from_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
    ) -> StorageResult<bool> {
        let result = sqlx::query(
            "DELETE FROM subscription_group_packs
            WHERE subscription_group_id = ? AND pack_id = ?",
        )
        .bind(subscription_group_id)
        .bind(pack_id)
        .execute(self.sqlite()?)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub(crate) fn sqlite(&self) -> StorageResult<&SqlitePool> {
        self.pool
            .sqlite()
            .ok_or_else(|| StorageError::UnsupportedDatabaseKind {
                kind: "postgres".to_owned(),
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

fn folder_from_row(row: &SqliteRow) -> StorageResult<FolderRecord> {
    let created_at: String = row.get("created_at");
    Ok(FolderRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_user_id: row.get("owner_user_id"),
        name: row.get("name"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn tag_from_row(row: &SqliteRow) -> StorageResult<TagRecord> {
    let created_at: String = row.get("created_at");
    Ok(TagRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        name: row.get("name"),
        created_at: parse_rfc3339(&created_at)?,
    })
}

fn subscription_group_from_row(row: &SqliteRow) -> StorageResult<SubscriptionGroupRecord> {
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

fn is_expired(expires_at: Option<&str>) -> bool {
    let Some(expires_at) = expires_at else {
        return false;
    };
    DateTime::parse_from_rfc3339(expires_at).map_or(true, |expires_at| {
        expires_at.with_timezone(&Utc) <= Utc::now()
    })
}

fn pat_record_from_row(row: &sqlx::sqlite::SqliteRow) -> StorageResult<PersonalAccessTokenRecord> {
    let scopes_json: String = row.get("scopes_json");
    let scope_keys: Vec<String> = serde_json::from_str(&scopes_json)?;
    let scopes = scope_keys
        .into_iter()
        .map(|scope| {
            Permission::from_key(&scope).ok_or(StorageError::InvalidPersonalAccessToken {
                reason: "unknown scope key",
            })
        })
        .collect::<StorageResult<BTreeSet<_>>>()?;

    Ok(PersonalAccessTokenRecord {
        id: row.get("id"),
        user_id: row.get("user_id"),
        name: row.get("name"),
        token_hash: row.get("token_hash"),
        scopes,
        expires_at: row.get("expires_at"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use chrono::{Duration, Utc};
    use msm_domain::Sticker;

    use crate::{
        db::DbPool, models::PackVisibility, repositories::StorageRepository, DatabaseConfig,
    };

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
