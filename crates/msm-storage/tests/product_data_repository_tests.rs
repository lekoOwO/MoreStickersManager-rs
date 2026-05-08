use msm_storage::{
    models::{NewTag, PackVisibility},
    DatabaseConfig, DbPool, StorageRepository,
};

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
