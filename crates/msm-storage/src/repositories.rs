use chrono::Utc;
use msm_domain::StickerPack;
use sqlx::{Row, SqlitePool};

use crate::{DbPool, StorageError, StorageResult, models::PackVisibility};

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
    /// Returns an error when the repository is not backed by SQLite or the insert fails.
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
    /// Returns an error when the repository is not backed by SQLite or the insert fails.
    pub async fn create_user(&self, id: &str, email: &str, display_name: &str) -> StorageResult<()> {
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

    /// Adds a user to a tenant with a coarse role.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by SQLite or the insert fails.
    pub async fn add_tenant_member(&self, tenant_id: &str, user_id: &str, role: &str) -> StorageResult<()> {
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
    /// Returns an error when serialization fails, the repository is not backed by SQLite, or SQL fails.
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

    /// Creates a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by SQLite or the insert fails.
    pub async fn create_subscription_group(
        &self,
        id: &str,
        tenant_id: &str,
        owner_user_id: &str,
        title: &str,
        visibility: PackVisibility,
    ) -> StorageResult<()> {
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
        .bind(now)
        .execute(self.sqlite()?)
        .await?;
        Ok(())
    }

    /// Adds a sticker pack to a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by SQLite or the insert fails.
    pub async fn add_pack_to_subscription_group(
        &self,
        subscription_group_id: &str,
        pack_id: &str,
        sort_order: i64,
    ) -> StorageResult<()> {
        sqlx::query(
            "INSERT INTO subscription_group_packs (
                subscription_group_id, pack_id, sort_order
            ) VALUES (?, ?, ?)",
        )
        .bind(subscription_group_id)
        .bind(pack_id)
        .bind(sort_order)
        .execute(self.sqlite()?)
        .await?;
        Ok(())
    }

    /// Finds a sticker pack by internal pack ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by SQLite or SQL/JSON parsing fails.
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

    /// Lists sticker packs owned by a user.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by SQLite or SQL/JSON parsing fails.
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

    /// Lists pack IDs in a subscription group.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository is not backed by SQLite or SQL fails.
    pub async fn list_subscription_pack_ids(&self, subscription_group_id: &str) -> StorageResult<Vec<String>> {
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

#[cfg(test)]
mod tests {
    use msm_domain::Sticker;

    use crate::{
        DatabaseConfig,
        db::DbPool,
        models::PackVisibility,
        repositories::StorageRepository,
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
