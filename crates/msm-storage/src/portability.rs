use chrono::{DateTime, Utc};
use msm_domain::StickerPack;
use sqlx::{postgres::PgRow, sqlite::SqliteRow, Row};

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
/// Returns an error when SQL fails or stored JSON is invalid.
pub async fn export_user_data(
    repo: &StorageRepository,
    user_id: &str,
) -> StorageResult<PortableUserExport> {
    let user = if let Ok(postgres) = repo.postgres() {
        let row = sqlx::query("SELECT id, email, display_name FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(postgres)
            .await?;
        portable_user_from_pg_row(&row)
    } else {
        let row = sqlx::query("SELECT id, email, display_name FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(repo.sqlite()?)
            .await?;
        portable_user_from_sqlite_row(&row)
    };

    let packs = repo.list_user_sticker_packs(user_id).await?;
    let subscription_groups = export_subscription_groups(repo, user_id).await?;

    Ok(PortableUserExport {
        version: 1,
        exported_at: Utc::now(),
        user,
        packs,
        subscription_groups,
    })
}

/// Imports one user's portable P2 data into an existing tenant.
///
/// # Errors
///
/// Returns an error when SQL fails or embedded pack data is invalid.
pub async fn import_user_data(
    repo: &StorageRepository,
    tenant_id: &str,
    export: &PortableUserExport,
) -> StorageResult<()> {
    let now = Utc::now().to_rfc3339();
    import_user_identity(repo, tenant_id, export, &now).await?;

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
        import_subscription_group(repo, tenant_id, &export.user.id, group, &now).await?;

        for (index, pack_id) in group.pack_ids.iter().enumerate() {
            repo.add_pack_to_subscription_group(
                &group.id,
                pack_id,
                i64::try_from(index).unwrap_or(i64::MAX),
            )
            .await?;
        }
    }

    Ok(())
}

async fn export_subscription_groups(
    repo: &StorageRepository,
    user_id: &str,
) -> StorageResult<Vec<PortableSubscriptionGroup>> {
    if let Ok(postgres) = repo.postgres() {
        let group_rows = sqlx::query(
            "SELECT id, title FROM subscription_groups
            WHERE owner_user_id = $1
            ORDER BY title, id",
        )
        .bind(user_id)
        .fetch_all(postgres)
        .await?;
        portable_subscription_groups_from_rows(repo, group_rows).await
    } else {
        let group_rows = sqlx::query(
            "SELECT id, title FROM subscription_groups
            WHERE owner_user_id = ?
            ORDER BY title, id",
        )
        .bind(user_id)
        .fetch_all(repo.sqlite()?)
        .await?;
        portable_subscription_groups_from_rows(repo, group_rows).await
    }
}

async fn portable_subscription_groups_from_rows<R>(
    repo: &StorageRepository,
    group_rows: Vec<R>,
) -> StorageResult<Vec<PortableSubscriptionGroup>>
where
    R: Row,
    for<'a> &'a str: sqlx::ColumnIndex<R>,
    for<'r> String: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
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
    Ok(subscription_groups)
}

async fn import_user_identity(
    repo: &StorageRepository,
    tenant_id: &str,
    export: &PortableUserExport,
    now: &str,
) -> StorageResult<()> {
    if let Ok(postgres) = repo.postgres() {
        sqlx::query(
            "INSERT INTO users (id, email, display_name, is_disabled, created_at)
            VALUES ($1, $2, $3, FALSE, $4)
            ON CONFLICT(id) DO NOTHING",
        )
        .bind(&export.user.id)
        .bind(&export.user.email)
        .bind(&export.user.display_name)
        .bind(now)
        .execute(postgres)
        .await?;

        sqlx::query(
            "INSERT INTO tenant_members (tenant_id, user_id, role, created_at)
            VALUES ($1, $2, 'member', $3)
            ON CONFLICT(tenant_id, user_id) DO NOTHING",
        )
        .bind(tenant_id)
        .bind(&export.user.id)
        .bind(now)
        .execute(postgres)
        .await?;
    } else {
        let sqlite = repo.sqlite()?;
        sqlx::query(
            "INSERT OR IGNORE INTO users (id, email, display_name, is_disabled, created_at)
            VALUES (?, ?, ?, 0, ?)",
        )
        .bind(&export.user.id)
        .bind(&export.user.email)
        .bind(&export.user.display_name)
        .bind(now)
        .execute(sqlite)
        .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO tenant_members (tenant_id, user_id, role, created_at)
            VALUES (?, ?, 'member', ?)",
        )
        .bind(tenant_id)
        .bind(&export.user.id)
        .bind(now)
        .execute(sqlite)
        .await?;
    }
    Ok(())
}

async fn import_subscription_group(
    repo: &StorageRepository,
    tenant_id: &str,
    user_id: &str,
    group: &PortableSubscriptionGroup,
    now: &str,
) -> StorageResult<()> {
    if let Ok(postgres) = repo.postgres() {
        sqlx::query(
            "INSERT INTO subscription_groups (
                id, tenant_id, owner_user_id, title, visibility, created_at
            ) VALUES ($1, $2, $3, $4, 'private', $5)
            ON CONFLICT(id) DO NOTHING",
        )
        .bind(&group.id)
        .bind(tenant_id)
        .bind(user_id)
        .bind(&group.title)
        .bind(now)
        .execute(postgres)
        .await?;
    } else {
        sqlx::query(
            "INSERT OR IGNORE INTO subscription_groups (
                id, tenant_id, owner_user_id, title, visibility, created_at
            ) VALUES (?, ?, ?, ?, 'private', ?)",
        )
        .bind(&group.id)
        .bind(tenant_id)
        .bind(user_id)
        .bind(&group.title)
        .bind(now)
        .execute(repo.sqlite()?)
        .await?;
    }
    Ok(())
}

fn portable_user_from_sqlite_row(row: &SqliteRow) -> PortableUser {
    PortableUser {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
    }
}

fn portable_user_from_pg_row(row: &PgRow) -> PortableUser {
    PortableUser {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use msm_domain::{Sticker, StickerPack};

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

    #[tokio::test]
    async fn exports_and_imports_user_data_with_postgres_when_configured() {
        let Some(repo) = postgres_repo().await else {
            return;
        };
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let source_tenant_id = format!("tenant_source_{suffix}");
        let target_tenant_id = format!("tenant_target_{suffix}");
        let user_id = format!("user_{suffix}");
        let group_id = format!("sub_{suffix}");
        let pack_id = format!("MoreStickers:Telegram:Pack:{suffix}");

        repo.create_tenant(&source_tenant_id, "Source")
            .await
            .unwrap();
        repo.create_tenant(&target_tenant_id, "Target")
            .await
            .unwrap();
        repo.create_user(&user_id, &format!("{suffix}@example.com"), "Leko")
            .await
            .unwrap();
        repo.add_tenant_member(&source_tenant_id, &user_id, "admin")
            .await
            .unwrap();

        let pack = sample_pack_with_ids(&pack_id, &suffix);
        repo.upsert_sticker_pack(
            &pack.id,
            &source_tenant_id,
            &user_id,
            PackVisibility::Private,
            Some("telegram"),
            &pack,
        )
        .await
        .unwrap();
        repo.create_subscription_group(
            &group_id,
            &source_tenant_id,
            &user_id,
            "Favorites",
            PackVisibility::Private,
        )
        .await
        .unwrap();
        repo.add_pack_to_subscription_group(&group_id, &pack.id, 0)
            .await
            .unwrap();

        let export = export_user_data(&repo, &user_id).await.unwrap();
        assert_eq!(export.user.id, user_id);
        assert_eq!(export.packs, vec![pack.clone()]);
        assert_eq!(
            export.subscription_groups,
            vec![crate::portability::PortableSubscriptionGroup {
                id: group_id.clone(),
                title: "Favorites".to_owned(),
                pack_ids: vec![pack_id.clone()],
            }]
        );

        import_user_data(&repo, &target_tenant_id, &export)
            .await
            .unwrap();

        assert_eq!(
            repo.list_user_sticker_packs(&user_id).await.unwrap(),
            vec![pack]
        );
        assert_eq!(
            repo.list_subscription_pack_ids(&group_id).await.unwrap(),
            vec![pack_id]
        );
    }

    async fn migrated_repo() -> StorageRepository {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        StorageRepository::new(pool)
    }

    async fn postgres_repo() -> Option<StorageRepository> {
        let database_url = env::var("MSM_TEST_POSTGRES_URL").ok()?;
        let config = DatabaseConfig::parse(&database_url).unwrap();
        let pool = DbPool::connect(&config).await.unwrap();
        pool.run_migrations().await.unwrap();
        Some(StorageRepository::new(pool))
    }

    fn sample_pack() -> msm_domain::StickerPack {
        sample_pack_with_ids("MoreStickers:Telegram:Pack:sample", "sample")
    }

    fn sample_pack_with_ids(pack_id: &str, suffix: &str) -> StickerPack {
        let sticker = Sticker {
            id: format!("MoreStickers:Telegram:Sticker:{suffix}:file"),
            image: format!("https://msm.example/assets/packs/{suffix}/file.webp"),
            title: "file".to_owned(),
            sticker_pack_id: pack_id.to_owned(),
            filename: Some("file.webp".to_owned()),
            is_animated: Some(false),
        };

        StickerPack {
            id: pack_id.to_owned(),
            title: "Sample".to_owned(),
            author: None,
            logo: sticker.clone(),
            stickers: vec![sticker],
        }
    }
}
