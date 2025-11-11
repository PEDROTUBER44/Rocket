use deadpool_postgres::{Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use crate::error::{AppError, Result};
use std::time::Duration;

/// Creates a new database connection pool.
///
/// # Arguments
///
/// * `database_url` - The URL of the PostgreSQL database.
///
/// # Returns
///
/// A `Result` containing the `Pool`.
pub fn create_pool(database_url: &str) -> Result<Pool> {
    let mut cfg = Config::new();
    let pg_config: tokio_postgres::Config = database_url.parse()?;

    if let Some(host) = pg_config.get_hosts().first() {
        if let Some(hostname) = host.host() {
            cfg.host = Some(hostname.to_string());
        }
        if let Some(port) = host.port() {
            cfg.port = Some(port);
        }
    }

    if let Some(dbname) = pg_config.get_dbname() {
        cfg.dbname = Some(dbname.to_string());
    }

    if let Some(user) = pg_config.get_user() {
        cfg.user = Some(user.to_string());
    }

    if let Some(password) = pg_config.get_password() {
        cfg.password = Some(String::from_utf8_lossy(password).to_string());
    }

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    cfg.pool = Some(PoolConfig {
        max_size: 100,
        timeouts: deadpool_postgres::Timeouts {
            wait: Some(Duration::from_secs(5)),
            create: Some(Duration::from_secs(2)),
            recycle: Some(Duration::from_secs(1)),
        },
    });

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(AppError::from)
}
