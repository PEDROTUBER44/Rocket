use deadpool_postgres::{
    Config as DeadpoolConfig, ManagerConfig, Pool, RecyclingMethod, Runtime, PoolConfig, Timeouts,
};
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_postgres::{Config as PgConfig, NoTls};

use crate::config::Config;
use crate::crypto::kek::KekCache;
use crate::error::{AppError, Result};
use crate::statement_cache::StatementCache;

/// The number of slots in the upload buffer.
pub const UPLOAD_BUFFER_SLOTS: usize = 200; // 200 slots × ~10MB = 2GB max
/// The number of slots in the download buffer.
pub const DOWNLOAD_BUFFER_SLOTS: usize = 200; // 200 slots × ~10MB = 2GB max

/// A rate limiter for uploads.
#[derive(Clone)]
pub struct UploadRateLimiter {
    semaphore: Arc<Semaphore>,
}

impl UploadRateLimiter {
    /// Creates a new `UploadRateLimiter`.
    pub fn new(max_buffer_slots: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_buffer_slots)),
        }
    }

    /// Acquires a permit from the semaphore.
    pub async fn acquire(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.semaphore.acquire().await.unwrap()
    }

    /// Returns the number of available permits.
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// A rate limiter for downloads.
#[derive(Clone)]
pub struct DownloadRateLimiter {
    semaphore: Arc<Semaphore>,
}

impl DownloadRateLimiter {
    /// Creates a new `DownloadRateLimiter`.
    pub fn new(max_buffer_slots: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_buffer_slots)),
        }
    }

    /// Acquires a permit from the semaphore.
    pub async fn acquire(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.semaphore.acquire().await.unwrap()
    }

    /// Returns the number of available permits.
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// The application's state.
#[derive(Clone)]
pub struct AppState {
    /// The database connection pool.
    pub db: Pool,
    /// The Redis connection manager.
    pub redis: ConnectionManager,
    /// The application's configuration.
    pub config: Config,
    /// The KEK cache.
    pub kek_cache: KekCache,
    /// The upload rate limiter.
    pub upload_limiter: UploadRateLimiter,
    /// The download rate limiter.
    pub download_limiter: DownloadRateLimiter,
    // The prepared statement cache.
    pub stmt_cache: StatementCache,
}

impl AppState {
    /// Creates a new `AppState`.
    pub async fn new(config: &Config) -> Result<Self> {
        let pg_config: PgConfig = config.database_url.parse().map_err(AppError::Postgres)?;

        let mut deadpool_cfg = DeadpoolConfig::default();
        deadpool_cfg.user = Some(pg_config.get_user().unwrap().to_string());
        deadpool_cfg.password = pg_config
            .get_password()
            .map(|p| String::from_utf8_lossy(p).to_string());
        deadpool_cfg.host = pg_config.get_hosts().get(0).map(|h| match h {
            tokio_postgres::config::Host::Tcp(s) => s.to_string(),
            _ => "localhost".to_string(),
        });
        deadpool_cfg.port = pg_config.get_ports().get(0).map(|p| *p);
        deadpool_cfg.dbname = Some(pg_config.get_dbname().unwrap().to_string());

        deadpool_cfg.pool = Some(PoolConfig {
            max_size: 48,
            timeouts: Timeouts::new(),
            ..Default::default()
        });
        deadpool_cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let db = deadpool_cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(AppError::PoolBuild)?;

        tracing::info!("✅ Deadpool PostgreSQL Pool initialized: max_size=48 (OPTIMIZED for PgCat)");

        let redis_client = redis::Client::open(config.redis_url.as_str())?;
        let redis = ConnectionManager::new(redis_client).await?;

        tracing::info!("✅ Redis Connection Manager initialized (pooled)");

        let kek_cache = KekCache::new();
        tracing::info!("✅ KEK Cache initialized");

        let stmt_cache = StatementCache::new();
        tracing::info!("✅ Statement Cache initialized");

        let upload_limiter = UploadRateLimiter::new(UPLOAD_BUFFER_SLOTS);
        tracing::info!("✅ Upload RateLimiter initialized (max 2GB)");

        let download_limiter = DownloadRateLimiter::new(DOWNLOAD_BUFFER_SLOTS);
        tracing::info!("✅ Download RateLimiter initialized (max 2GB)");

        Ok(AppState {
            db,
            redis,
            config: config.clone(),
            kek_cache,
            upload_limiter,
            download_limiter,
            stmt_cache,
        })
    }
}
