use std::{fs, path::Path, str::FromStr};

use sqlx::{
    ConnectOptions, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use crate::{config::AppConfig, error::AppResult};

pub async fn init_pool(config: &AppConfig) -> AppResult<SqlitePool> {
    ensure_sqlite_parent_dir(&config.database_url)?;

    let options = SqliteConnectOptions::from_str(&config.database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .disable_statement_logging();

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

fn ensure_sqlite_parent_dir(database_url: &str) -> AppResult<()> {
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
    }

    Ok(())
}
