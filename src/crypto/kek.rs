use deadpool_postgres::Pool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::crypto::aes;
use crate::error::{AppError, Result};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A cached Key Encryption Key (KEK).
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CachedKek {
    /// The version of the KEK.
    #[zeroize(skip)]
    pub version: i32,
    /// The key data.
    pub keydata: Vec<u8>,
}

/// A cache for Key Encryption Keys (KEKs).
#[derive(Clone)]
pub struct KekCache {
    cache: Arc<RwLock<HashMap<i32, CachedKek>>>,
}

impl KekCache {
    /// Creates a new `KekCache`.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gets a KEK from the cache by version.
    ///
    /// # Arguments
    ///
    /// * `version` - The version of the KEK to get.
    ///
    /// # Returns
    ///
    /// An `Option` containing the key data if the KEK is found in the cache.
    pub async fn get(&self, version: i32) -> Option<Vec<u8>> {
        let cache = self.cache.read().await;
        cache.get(&version).map(|kek| kek.keydata.clone())
    }

    /// Inserts a KEK into the cache.
    ///
    /// # Arguments
    ///
    /// * `version` - The version of the KEK.
    /// * `keydata` - The key data to insert.
    pub async fn insert(&self, version: i32, keydata: Vec<u8>) {
        let mut cache = self.cache.write().await;
        cache.insert(version, CachedKek { version, keydata });
    }

    /// Clears the KEK cache.
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

/// Ensures that a KEK with version 1 exists in the database.
pub async fn ensure_kek_exists(
    pool: &Pool,
    master_key: &[u8],
    kek_cache: &KekCache,
) -> Result<i32> {
    let client = pool.get().await?;
    let stmt = client
        .prepare(
            "SELECT version FROM keks WHERE version = 1 AND is_active = true AND is_deprecated = false",
        )
        .await?;

    let existing = client.query_opt(&stmt, &[]).await?;

    if existing.is_some() {
        tracing::info!("✅ KEK version 1 already exists and is active");
        return Ok(1);
    }

    tracing::warn!("⚠️  KEK version 1 not found, creating...");

    let version = 1i32;
    let kek = aes::generate_key();
    let keydata = kek.as_bytes().to_vec();

    let master_key_array: [u8; 32] = master_key
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid master key size".to_string()))?;

    let (encrypted_keydata, nonce) = aes::encrypt(&master_key_array, &keydata)?;

    let insert_stmt = client
        .prepare(
            r#"
        INSERT INTO keks (version, encrypted_keydata, nonce, is_active, is_deprecated, created_at)
        VALUES ($1, $2, $3, $4, false, NOW())
        ON CONFLICT (version) DO NOTHING
        "#,
        )
        .await?;

    client
        .execute(
            &insert_stmt,
            &[&version, &encrypted_keydata, &nonce.to_vec(), &true],
        )
        .await?;

    // Cachear a chave
    kek_cache.insert(version, keydata).await;

    tracing::info!("✅ KEK version 1 created successfully and cached");
    Ok(version)
}
