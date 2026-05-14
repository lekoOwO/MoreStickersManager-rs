use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use msm_storage::{LocalAssetStore, StorageRepository};

use crate::oidc::{
    HttpOidcDiscoveryFetcher, HttpOidcJwksFetcher, HttpOidcTokenExchanger, HttpOidcUserinfoFetcher,
    OidcDiscoveryFetcher, OidcJwksFetcher, OidcTokenExchanger, OidcUserinfoFetcher,
};

pub const DEFAULT_IMPORT_RATE_LIMIT_REQUESTS: usize = 60;
pub const DEFAULT_IMPORT_RATE_LIMIT_WINDOW_SECS: u64 = 60;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitConfig {
    pub import_requests: usize,
    pub import_window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            import_requests: DEFAULT_IMPORT_RATE_LIMIT_REQUESTS,
            import_window: Duration::from_secs(DEFAULT_IMPORT_RATE_LIMIT_WINDOW_SECS),
        }
    }
}

#[derive(Default)]
struct ImportRateLimiter {
    buckets: HashMap<String, VecDeque<Instant>>,
}

#[derive(Clone)]
pub struct ApiState {
    repository: StorageRepository,
    asset_store: LocalAssetStore,
    oidc_token_exchanger: Arc<dyn OidcTokenExchanger>,
    oidc_discovery_fetcher: Arc<dyn OidcDiscoveryFetcher>,
    oidc_jwks_fetcher: Arc<dyn OidcJwksFetcher>,
    oidc_userinfo_fetcher: Arc<dyn OidcUserinfoFetcher>,
    public_asset_url: Option<String>,
    cors_allowed_origins: Vec<String>,
    rate_limit_config: RateLimitConfig,
    import_rate_limiter: Arc<Mutex<ImportRateLimiter>>,
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
            cors_allowed_origins: Vec::new(),
            rate_limit_config: RateLimitConfig::default(),
            import_rate_limiter: Arc::new(Mutex::new(ImportRateLimiter::default())),
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
    pub fn with_cors_allowed_origins(mut self, cors_allowed_origins: Vec<String>) -> Self {
        self.cors_allowed_origins = cors_allowed_origins;
        self
    }

    #[must_use]
    pub fn with_rate_limit_config(mut self, rate_limit_config: RateLimitConfig) -> Self {
        self.rate_limit_config = rate_limit_config;
        self.import_rate_limiter = Arc::new(Mutex::new(ImportRateLimiter::default()));
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

    #[must_use]
    pub fn cors_allowed_origins(&self) -> &[String] {
        &self.cors_allowed_origins
    }

    #[must_use]
    pub fn rate_limit_config(&self) -> &RateLimitConfig {
        &self.rate_limit_config
    }

    #[must_use]
    pub fn check_import_rate_limit(&self, key: &str) -> bool {
        let now = Instant::now();
        let Ok(mut limiter) = self.import_rate_limiter.lock() else {
            return false;
        };
        let bucket = limiter.buckets.entry(key.to_owned()).or_default();
        while bucket.front().is_some_and(|instant| {
            now.duration_since(*instant) >= self.rate_limit_config.import_window
        }) {
            bucket.pop_front();
        }
        if bucket.len() >= self.rate_limit_config.import_requests {
            false
        } else {
            bucket.push_back(now);
            true
        }
    }
}
