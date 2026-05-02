use crate::{StorageError, StorageResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseKind {
    Sqlite,
    Postgres,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabaseConfig {
    kind: DatabaseKind,
    url: String,
}

impl DatabaseConfig {
    /// Parses a database URL into an MSM storage database config.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is empty or does not use a supported database scheme.
    pub fn parse(url: impl Into<String>) -> StorageResult<Self> {
        let url = url.into();
        if url.is_empty() {
            return Err(StorageError::InvalidDatabaseUrl {
                url,
                reason: "database URL must not be empty",
            });
        }

        let kind = if url.starts_with("sqlite:") {
            DatabaseKind::Sqlite
        } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
            DatabaseKind::Postgres
        } else {
            return Err(StorageError::InvalidDatabaseUrl {
                url,
                reason: "supported schemes are sqlite, postgres, and postgresql",
            });
        };

        Ok(Self { kind, url })
    }

    #[must_use]
    pub fn kind(&self) -> &DatabaseKind {
        &self.kind
    }

    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
mod tests {
    use super::{DatabaseConfig, DatabaseKind};

    #[test]
    fn parses_sqlite_urls() {
        let config = DatabaseConfig::parse("sqlite:data/msm.sqlite3").unwrap();

        assert_eq!(config.kind(), &DatabaseKind::Sqlite);
        assert_eq!(config.url(), "sqlite:data/msm.sqlite3");
    }

    #[test]
    fn parses_postgres_urls() {
        let config = DatabaseConfig::parse("postgres://localhost/msm").unwrap();

        assert_eq!(config.kind(), &DatabaseKind::Postgres);
        assert_eq!(config.url(), "postgres://localhost/msm");
    }

    #[test]
    fn rejects_unsupported_urls() {
        assert!(DatabaseConfig::parse("mysql://localhost/msm").is_err());
        assert!(DatabaseConfig::parse("").is_err());
    }
}
