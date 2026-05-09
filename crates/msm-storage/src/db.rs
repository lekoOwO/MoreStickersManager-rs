use sqlx::{
    postgres::PgPoolOptions,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    PgPool, SqlitePool,
};

use crate::{DatabaseConfig, DatabaseKind, StorageResult};

#[derive(Clone)]
pub enum DbPool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}

impl DbPool {
    /// Connects to the database described by `config`.
    ///
    /// # Errors
    ///
    /// Returns an error when `SQLx` cannot establish the database connection.
    pub async fn connect(config: &DatabaseConfig) -> StorageResult<Self> {
        match config.kind() {
            DatabaseKind::Sqlite => connect_sqlite(config.url()).await.map(Self::Sqlite),
            DatabaseKind::Postgres => PgPoolOptions::new()
                .max_connections(5)
                .connect(config.url())
                .await
                .map(Self::Postgres)
                .map_err(Into::into),
        }
    }

    /// Runs embedded `SQLx` migrations against the database pool.
    ///
    /// # Errors
    ///
    /// Returns an error when a migration fails.
    pub async fn run_migrations(&self) -> StorageResult<()> {
        match self {
            Self::Sqlite(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
            }
            Self::Postgres(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn sqlite(&self) -> Option<&SqlitePool> {
        match self {
            Self::Sqlite(pool) => Some(pool),
            Self::Postgres(_) => None,
        }
    }
}

/// Connects to a `SQLite` database URL.
///
/// # Errors
///
/// Returns an error when `SQLx` cannot open the `SQLite` database.
pub async fn connect_sqlite(url: &str) -> StorageResult<SqlitePool> {
    let options = url
        .parse::<SqliteConnectOptions>()?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?)
}

#[cfg(test)]
mod tests {
    use sqlx::Row;

    use crate::{db::DbPool, DatabaseConfig};

    #[tokio::test]
    async fn runs_sqlite_migrations() {
        let config = DatabaseConfig::parse("sqlite::memory:").unwrap();
        let pool = DbPool::connect(&config).await.unwrap();

        pool.run_migrations().await.unwrap();

        let sqlite = pool.sqlite().unwrap();
        let row = sqlx::query("SELECT COUNT(*) AS count FROM sqlite_master WHERE type = 'table'")
            .fetch_one(sqlite)
            .await
            .unwrap();
        let count: i64 = row.get("count");

        assert!(count >= 20);
    }
}
