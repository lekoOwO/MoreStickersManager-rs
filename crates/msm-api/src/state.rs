use std::sync::Arc;

use msm_storage::{LocalAssetStore, StorageRepository};

use crate::oidc::{HttpOidcTokenExchanger, OidcTokenExchanger};

#[derive(Clone)]
pub struct ApiState {
    repository: StorageRepository,
    asset_store: LocalAssetStore,
    oidc_token_exchanger: Arc<dyn OidcTokenExchanger>,
}

impl ApiState {
    #[must_use]
    pub fn new(repository: StorageRepository, asset_store: LocalAssetStore) -> Self {
        Self {
            repository,
            asset_store,
            oidc_token_exchanger: Arc::new(HttpOidcTokenExchanger::new()),
        }
    }

    #[must_use]
    pub fn with_oidc_token_exchanger(
        mut self,
        oidc_token_exchanger: Arc<dyn OidcTokenExchanger>,
    ) -> Self {
        self.oidc_token_exchanger = oidc_token_exchanger;
        self
    }

    #[must_use]
    pub fn repository(&self) -> &StorageRepository {
        &self.repository
    }

    #[must_use]
    pub fn asset_store(&self) -> &LocalAssetStore {
        &self.asset_store
    }

    #[must_use]
    pub fn oidc_token_exchanger(&self) -> &dyn OidcTokenExchanger {
        self.oidc_token_exchanger.as_ref()
    }
}
