use msm_storage::{LocalAssetStore, StorageRepository};

#[derive(Clone)]
pub struct ApiState {
    repository: StorageRepository,
    asset_store: LocalAssetStore,
}

impl ApiState {
    #[must_use]
    pub fn new(repository: StorageRepository, asset_store: LocalAssetStore) -> Self {
        Self {
            repository,
            asset_store,
        }
    }

    #[must_use]
    pub fn repository(&self) -> &StorageRepository {
        &self.repository
    }

    #[must_use]
    pub fn asset_store(&self) -> &LocalAssetStore {
        &self.asset_store
    }
}
