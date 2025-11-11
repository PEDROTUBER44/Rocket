use std::env;
use anyhow::{Context, Result};
use zeroize::{Zeroize, Zeroizing};

/// The application's configuration.
#[derive(Clone)]
pub struct Config {
    /// The URL of the PostgreSQL database.
    pub database_url: String,
    /// The URL of the Redis server.
    pub redis_url: String,
    /// The duration of a session in days.
    pub session_duration_days: i64,
    /// The master key used for encryption.
    pub master_key: Zeroizing<Vec<u8>>,
}

impl Config {
    /// Creates a new `Config` from environment variables.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Config`.
    pub fn from_env() -> Result<Self> {
        let mut master_key_hex = env::var("MASTER_KEY")
            .context("MASTER_KEY must be set (generate with: openssl rand -hex 32)")?;
        
        let master_key_bytes = hex::decode(&master_key_hex)
            .context("MASTER_KEY must be valid hexadecimal")?;
        
        master_key_hex.zeroize();
        
        if master_key_bytes.len() != 32 {
            anyhow::bail!("MASTER_KEY must be exactly 32 bytes (64 hex characters)");
        }
        
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            session_duration_days: env::var("SESSION_DURATION_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .context("Invalid SESSION_DURATION_DAYS")?,
            master_key: Zeroizing::new(master_key_bytes),
        })
    }
}
