use msm_storage::{
    models::{NewTag, PackVisibility},
    DatabaseConfig, DbPool, StorageRepository,
};

use msm_domain::{Sticker, StickerPack};

#[tokio::test]
async fn folders_tags_and_subscription_groups_can_be_managed() {
    let repo = seeded_repo().await;

    let folder = repo
        .create_folder("folder_1", "tenant_1", "user_1", "Favorites")
        .await
        .unwrap();
    assert_eq!(folder.name, "Favorites");

    let renamed_folder = repo.rename_folder("folder_1", "Pinned").await.unwrap();
    assert_eq!(renamed_folder.name, "Pinned");
    assert_eq!(
        repo.list_folders("tenant_1", "user_1").await.unwrap(),
        vec![renamed_folder]
    );

    let tag = repo
        .create_tag(NewTag {
            id: "tag_1",
            tenant_id: "tenant_1",
            name: "cute",
        })
        .await
        .unwrap();
    assert_eq!(tag.name, "cute");
    assert_eq!(repo.list_tags("tenant_1").await.unwrap(), vec![tag]);

    let subscription = repo
        .create_subscription_group(
            "sub_1",
            "tenant_1",
            "user_1",
            "Weekly",
            PackVisibility::Private,
        )
        .await
        .unwrap();
    assert_eq!(subscription.title, "Weekly");

    let renamed_subscription = repo
        .rename_subscription_group("sub_1", "Favorites Feed")
        .await
        .unwrap();
    assert_eq!(renamed_subscription.title, "Favorites Feed");
    assert_eq!(
        repo.list_subscription_groups("tenant_1", "user_1")
            .await
            .unwrap(),
        vec![renamed_subscription]
    );

    assert!(repo.delete_folder("folder_1").await.unwrap());
    assert!(repo.delete_tag("tag_1").await.unwrap());
    assert!(repo.delete_subscription_group("sub_1").await.unwrap());
    assert!(repo
        .list_folders("tenant_1", "user_1")
        .await
        .unwrap()
        .is_empty());
    assert!(repo.list_tags("tenant_1").await.unwrap().is_empty());
    assert!(repo
        .list_subscription_groups("tenant_1", "user_1")
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn pack_memberships_can_be_managed() {
    let repo = seeded_repo().await;
    seed_pack(&repo).await;
    repo.create_folder("folder_1", "tenant_1", "user_1", "Favorites")
        .await
        .unwrap();
    repo.create_tag(NewTag {
        id: "tag_1",
        tenant_id: "tenant_1",
        name: "cute",
    })
    .await
    .unwrap();
    repo.create_subscription_group(
        "sub_1",
        "tenant_1",
        "user_1",
        "Weekly",
        PackVisibility::Private,
    )
    .await
    .unwrap();

    let folder_link = repo
        .add_pack_to_folder("folder_1", "pack_1", 10)
        .await
        .unwrap();
    assert_eq!(folder_link.pack_id, "pack_1");
    assert_eq!(
        repo.list_folder_pack_ids("folder_1").await.unwrap(),
        vec!["pack_1"]
    );

    let tag_link = repo.add_tag_to_pack("pack_1", "tag_1").await.unwrap();
    assert_eq!(tag_link.tag_id, "tag_1");
    assert_eq!(
        repo.list_pack_tag_ids("pack_1").await.unwrap(),
        vec!["tag_1"]
    );

    let subscription_link = repo
        .add_pack_to_subscription_group("sub_1", "pack_1", 20)
        .await
        .unwrap();
    assert_eq!(subscription_link.subscription_group_id, "sub_1");
    assert_eq!(
        repo.list_subscription_pack_ids("sub_1").await.unwrap(),
        vec!["pack_1"]
    );

    assert!(repo
        .remove_pack_from_folder("folder_1", "pack_1")
        .await
        .unwrap());
    assert!(repo.remove_tag_from_pack("pack_1", "tag_1").await.unwrap());
    assert!(repo
        .remove_pack_from_subscription_group("sub_1", "pack_1")
        .await
        .unwrap());
    assert!(repo
        .list_folder_pack_ids("folder_1")
        .await
        .unwrap()
        .is_empty());
    assert!(repo.list_pack_tag_ids("pack_1").await.unwrap().is_empty());
    assert!(repo
        .list_subscription_pack_ids("sub_1")
        .await
        .unwrap()
        .is_empty());
}

async fn seeded_repo() -> StorageRepository {
    let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
    let pool = DbPool::connect(&config).await.unwrap();
    pool.run_migrations().await.unwrap();
    let repo = StorageRepository::new(pool);
    repo.create_tenant("tenant_1", "Tenant").await.unwrap();
    repo.create_user("user_1", "leko@example.com", "Leko")
        .await
        .unwrap();
    repo
}

async fn seed_pack(repo: &StorageRepository) {
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
}

fn sample_pack() -> StickerPack {
    let sticker = Sticker {
        id: "MoreStickers:Telegram:Sticker:sample:file".to_owned(),
        image: "https://msm.example/assets/packs/sample/file.webp".to_owned(),
        title: "file".to_owned(),
        sticker_pack_id: "MoreStickers:Telegram:Pack:sample".to_owned(),
        filename: Some("file.webp".to_owned()),
        is_animated: Some(false),
    };

    StickerPack {
        id: "MoreStickers:Telegram:Pack:sample".to_owned(),
        title: "Sample".to_owned(),
        author: None,
        logo: sticker.clone(),
        stickers: vec![sticker],
    }
}
