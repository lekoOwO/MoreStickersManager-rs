use chrono::{DateTime, Utc};
use msm_domain::StickerPack;
use sqlx::Row;

use crate::{models::PackVisibility, StorageRepository, StorageResult};

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableUserExport {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub user: PortableUser,
    pub packs: Vec<StickerPack>,
    pub subscription_groups: Vec<PortableSubscriptionGroup>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableSubscriptionGroup {
    pub id: String,
    pub title: String,
    pub pack_ids: Vec<String>,
}

/// Exports one user's portable P2 data.
///
/// # Errors
///
/// Returns an error when the repository is not backed by `SQLite`, SQL fails, or stored JSON is invalid.
pub async fn export_user_data(
    repo: &StorageRepository,
    user_id: &str,
) -> StorageResult<PortableUserExport> {
    let sqlite = repo.sqlite()?;
    let user_row = sqlx::query("SELECT id, email, display_name FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(sqlite)
        .await?;

    let packs = repo.list_user_sticker_packs(user_id).await?;
    let group_rows = sqlx::query(
        "SELECT id, title FROM subscription_groups
        WHERE owner_user_id = ?
        ORDER BY title, id",
    )
    .bind(user_id)
    .fetch_all(sqlite)
    .await?;

    let mut subscription_groups = Vec::with_capacity(group_rows.len());
    for row in group_rows {
        let id: String = row.get("id");
        let title: String = row.get("title");
        let pack_ids = repo.list_subscription_pack_ids(&id).await?;
        subscription_groups.push(PortableSubscriptionGroup {
            id,
            title,
            pack_ids,
        });
    }

    Ok(PortableUserExport {
        version: 1,
        exported_at: Utc::now(),
        user: PortableUser {
            id: user_row.get("id"),
            email: user_row.get("email"),
            display_name: user_row.get("display_name"),
        },
        packs,
        subscription_groups,
    })
}

/// Imports one user's portable P2 data into an existing tenant.
///
/// # Errors
///
/// Returns an error when the repository is not backed by `SQLite`, SQL fails, or embedded pack data is invalid.
pub async fn import_user_data(
    repo: &StorageRepository,
    tenant_id: &str,
    export: &PortableUserExport,
) -> StorageResult<()> {
    let sqlite = repo.sqlite()?;
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT OR IGNORE INTO users (id, email, display_name, is_disabled, created_at)
        VALUES (?, ?, ?, 0, ?)",
    )
    .bind(&export.user.id)
    .bind(&export.user.email)
    .bind(&export.user.display_name)
    .bind(&now)
    .execute(sqlite)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO tenant_members (tenant_id, user_id, role, created_at)
        VALUES (?, ?, 'member', ?)",
    )
    .bind(tenant_id)
    .bind(&export.user.id)
    .bind(&now)
    .execute(sqlite)
    .await?;

    for pack in &export.packs {
        repo.upsert_sticker_pack(
            &pack.id,
            tenant_id,
            &export.user.id,
            PackVisibility::Private,
            None,
            pack,
        )
        .await?;
    }

    for group in &export.subscription_groups {
        sqlx::query(
            "INSERT OR IGNORE INTO subscription_groups (
                id, tenant_id, owner_user_id, title, visibility, created_at
            ) VALUES (?, ?, ?, ?, 'private', ?)",
        )
        .bind(&group.id)
        .bind(tenant_id)
        .bind(&export.user.id)
        .bind(&group.title)
        .bind(&now)
        .execute(sqlite)
        .await?;

        for (index, pack_id) in group.pack_ids.iter().enumerate() {
            sqlx::query(
                "INSERT OR IGNORE INTO subscription_group_packs (
                    subscription_group_id, pack_id, sort_order
                ) VALUES (?, ?, ?)",
            )
            .bind(&group.id)
            .bind(pack_id)
            .bind(i64::try_from(index).unwrap_or(i64::MAX))
            .execute(sqlite)
            .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use msm_domain::Sticker;

    use crate::{
        db::DbPool,
        models::PackVisibility,
        portability::{export_user_data, import_user_data},
        repositories::StorageRepository,
        DatabaseConfig,
    };

    #[tokio::test]
    async fn exports_and_imports_user_data_between_sqlite_databases() {
        let source = migrated_repo().await;
        source.create_tenant("tenant_1", "Source").await.unwrap();
        source
            .create_user("user_1", "leko@example.com", "Leko")
            .await
            .unwrap();
        source
            .add_tenant_member("tenant_1", "user_1", "admin")
            .await
            .unwrap();
        let pack = sample_pack();
        source
            .upsert_sticker_pack(
                &pack.id,
                "tenant_1",
                "user_1",
                PackVisibility::Private,
                Some("telegram"),
                &pack,
            )
            .await
            .unwrap();
        source
            .create_subscription_group(
                "sub_1",
                "tenant_1",
                "user_1",
                "Favorites",
                PackVisibility::Private,
            )
            .await
            .unwrap();
        source
            .add_pack_to_subscription_group("sub_1", &pack.id, 0)
            .await
            .unwrap();

        let export = export_user_data(&source, "user_1").await.unwrap();
        assert_eq!(export.version, 1);
        assert_eq!(export.packs.len(), 1);
        assert_eq!(export.subscription_groups.len(), 1);

        let target = migrated_repo().await;
        target.create_tenant("tenant_2", "Target").await.unwrap();
        import_user_data(&target, "tenant_2", &export)
            .await
            .unwrap();

        let pack_id = pack.id.clone();
        let imported_packs = target.list_user_sticker_packs("user_1").await.unwrap();
        assert_eq!(imported_packs, vec![pack]);
        assert_eq!(
            target.list_subscription_pack_ids("sub_1").await.unwrap(),
            vec![pack_id]
        );
    }

    async fn migrated_repo() -> StorageRepository {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        StorageRepository::new(pool)
    }

    fn sample_pack() -> msm_domain::StickerPack {
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
