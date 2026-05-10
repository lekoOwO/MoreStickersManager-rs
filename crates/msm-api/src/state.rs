use std::sync::Arc;

use msm_storage::{LocalAssetStore, StorageRepository};

use crate::oidc::{
    HttpOidcDiscoveryFetcher, HttpOidcTokenExchanger, OidcDiscoveryFetcher, OidcTokenExchanger,
};

#[derive(Clone)]
pub struct ApiState {
    repository: StorageRepository,
    asset_store: LocalAssetStore,
    oidc_token_exchanger: Arc<dyn OidcTokenExchanger>,
    oidc_discovery_fetcher: Arc<dyn OidcDiscoveryFetcher>,
}

impl ApiState {
    #[must_use]
    pub fn new(repository: StorageRepository, asset_store: LocalAssetStore) -> Self {
        Self {
            repository,
            asset_store,
            oidc_token_exchanger: Arc::new(HttpOidcTokenExchanger::new()),
            oidc_discovery_fetcher: Arc::new(HttpOidcDiscoveryFetcher::new()),
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
    pub fn with_oidc_discovery_fetcher(
        mut self,
        oidc_discovery_fetcher: Arc<dyn OidcDiscoveryFetcher>,
    ) -> Self {
        self.oidc_discovery_fetcher = oidc_discovery_fetcher;
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

    #[must_use]
    pub fn oidc_discovery_fetcher(&self) -> &dyn OidcDiscoveryFetcher {
        self.oidc_discovery_fetcher.as_ref()
    }
}
