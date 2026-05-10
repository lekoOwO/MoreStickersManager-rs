use std::sync::Arc;

use msm_storage::{LocalAssetStore, StorageRepository};

use crate::oidc::{
    HttpOidcDiscoveryFetcher, HttpOidcJwksFetcher, HttpOidcTokenExchanger, HttpOidcUserinfoFetcher,
    OidcDiscoveryFetcher, OidcJwksFetcher, OidcTokenExchanger, OidcUserinfoFetcher,
};

#[derive(Clone)]
pub struct ApiState {
    repository: StorageRepository,
    asset_store: LocalAssetStore,
    oidc_token_exchanger: Arc<dyn OidcTokenExchanger>,
    oidc_discovery_fetcher: Arc<dyn OidcDiscoveryFetcher>,
    oidc_jwks_fetcher: Arc<dyn OidcJwksFetcher>,
    oidc_userinfo_fetcher: Arc<dyn OidcUserinfoFetcher>,
    public_asset_url: Option<String>,
}

impl ApiState {
    #[must_use]
    pub fn new(repository: StorageRepository, asset_store: LocalAssetStore) -> Self {
        Self {
            repository,
            asset_store,
            oidc_token_exchanger: Arc::new(HttpOidcTokenExchanger::new()),
            oidc_discovery_fetcher: Arc::new(HttpOidcDiscoveryFetcher::new()),
            oidc_jwks_fetcher: Arc::new(HttpOidcJwksFetcher::new()),
            oidc_userinfo_fetcher: Arc::new(HttpOidcUserinfoFetcher::new()),
            public_asset_url: None,
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
    pub fn with_oidc_jwks_fetcher(mut self, oidc_jwks_fetcher: Arc<dyn OidcJwksFetcher>) -> Self {
        self.oidc_jwks_fetcher = oidc_jwks_fetcher;
        self
    }

    #[must_use]
    pub fn with_oidc_userinfo_fetcher(
        mut self,
        oidc_userinfo_fetcher: Arc<dyn OidcUserinfoFetcher>,
    ) -> Self {
        self.oidc_userinfo_fetcher = oidc_userinfo_fetcher;
        self
    }

    #[must_use]
    pub fn with_public_asset_url(mut self, public_asset_url: Option<String>) -> Self {
        self.public_asset_url = public_asset_url;
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

    #[must_use]
    pub fn oidc_jwks_fetcher(&self) -> &dyn OidcJwksFetcher {
        self.oidc_jwks_fetcher.as_ref()
    }

    #[must_use]
    pub fn oidc_userinfo_fetcher(&self) -> &dyn OidcUserinfoFetcher {
        self.oidc_userinfo_fetcher.as_ref()
    }

    #[must_use]
    pub fn public_asset_url(&self) -> Option<&str> {
        self.public_asset_url.as_deref()
    }
}
