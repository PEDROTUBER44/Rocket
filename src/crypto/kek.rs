use sqlx::PgPool;
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
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `master_key` - The master key used to encrypt the KEK.
/// * `kek_cache` - The KEK cache.
///
/// # Returns
///
/// The version of the KEK.
pub async fn ensure_kek_exists(
    pool: &PgPool,
    master_key: &[u8],
    kek_cache: &KekCache,
) -> Result<i32> {
    let existing = sqlx::query_scalar::<_, i32>(
        "SELECT version FROM keks WHERE version = 1 AND is_active = true AND is_deprecated = false"
    )
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        tracing::info!("✅ KEK version 1 already exists and is active");
        return Ok(1);
    }

    tracing::warn!("⚠️  KEK version 1 not found, creating...");

    let version = 1i32;
    let kek = aes::generate_key();
    let keydata = kek.as_bytes().to_vec();

    let master_key_array: [u8; 32] = master_key.try_into()
        .map_err(|_| AppError::Encryption("Invalid master key size".to_string()))?;

    let (encrypted_keydata, nonce) = aes::encrypt(&master_key_array, &keydata)?;

    sqlx::query!(
        r#"
        INSERT INTO keks (version, encrypted_keydata, nonce, is_active, is_deprecated, created_at)
        VALUES ($1, $2, $3, $4, false, NOW())
        ON CONFLICT (version) DO NOTHING
        "#,
        version,
        &encrypted_keydata,
        &nonce.to_vec(),
        true
    )
    .execute(pool)
    .await?;

    // Cachear a chave
    kek_cache.insert(version, keydata).await;

    tracing::info!("✅ KEK version 1 created successfully and cached");
    Ok(version)
}

/// Gets the active Key Encryption Key (KEK).
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `master_key` - The master key used to decrypt the KEK.
/// * `kek_cache` - The KEK cache.
///
/// # Returns
///
/// A tuple containing the version and key data of the active KEK.
pub async fn get_active_kek(
    pool: &PgPool,
    master_key: &[u8],
    kek_cache: &KekCache,
) -> Result<(i32, Vec<u8>)> {
    let record = sqlx::query!(
        r#"
        SELECT version, encrypted_keydata, nonce
        FROM keks
        WHERE is_active = true AND is_deprecated = false
        ORDER BY version DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;

    match record {
        Some(r) => {
            if let Some(cached_keydata) = kek_cache.get(r.version).await {
                tracing::debug!("✅ KEK v{} retrieved from cache", r.version);
                return Ok((r.version, cached_keydata));
            }

            let master_key_array: [u8; 32] = master_key.try_into()
                .map_err(|_| AppError::Encryption("Invalid master key size".to_string()))?;

            let nonce: [u8; 12] = r.nonce.try_into()
                .map_err(|_| AppError::Encryption("Invalid nonce size".to_string()))?;

            let keydata = aes::decrypt(&master_key_array, &r.encrypted_keydata, &nonce)?;

            kek_cache.insert(r.version, keydata.clone()).await;

            tracing::debug!("✅ KEK v{} retrieved from database and cached", r.version);
            Ok((r.version, keydata))
        }
        None => {
            tracing::warn!("⚠️  No active KEK found, creating version 1...");

            let version = 1i32;
            let kek = aes::generate_key();
            let keydata = kek.as_bytes().to_vec();

            let master_key_array: [u8; 32] = master_key.try_into()
                .map_err(|_| AppError::Encryption("Invalid master key size".to_string()))?;

            let (encrypted_keydata, nonce) = aes::encrypt(&master_key_array, &keydata)?;

            sqlx::query!(
                r#"
                INSERT INTO keks (version, encrypted_keydata, nonce, is_active, is_deprecated, created_at)
                VALUES ($1, $2, $3, true, false, NOW())
                ON CONFLICT (version) DO NOTHING
                "#,
                version,
                &encrypted_keydata,
                &nonce.to_vec()
            )
            .execute(pool)
            .await?;

            kek_cache.insert(version, keydata.clone()).await;

            tracing::info!("✅ KEK version 1 created successfully");
            Ok((version, keydata))
        }
    }
}

/// Gets a Key Encryption Key (KEK) by its version.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `version` - The version of the KEK to get.
/// * `master_key` - The master key used to decrypt the KEK.
/// * `kek_cache` - The KEK cache.
///
/// # Returns
///
/// The key data of the specified KEK.
pub async fn get_kek_by_version(
    pool: &PgPool,
    version: i32,
    master_key: &[u8],
    kek_cache: &KekCache,
) -> Result<Vec<u8>> {
    if let Some(cached_keydata) = kek_cache.get(version).await {
        tracing::debug!("✅ KEK v{} retrieved from cache", version);
        return Ok(cached_keydata);
    }

    let record = sqlx::query!(
        r#"
        SELECT encrypted_keydata, nonce
        FROM keks
        WHERE version = $1
        "#,
        version
    )
    .fetch_optional(pool)
    .await?;

    match record {
        Some(r) => {
            let master_key_array: [u8; 32] = master_key.try_into()
                .map_err(|_| AppError::Encryption("Invalid master key size".to_string()))?;

            let nonce: [u8; 12] = r.nonce.try_into()
                .map_err(|_| AppError::Encryption("Invalid nonce size".to_string()))?;

            let keydata = aes::decrypt(&master_key_array, &r.encrypted_keydata, &nonce)?;

            kek_cache.insert(version, keydata.clone()).await;

            tracing::debug!("✅ KEK v{} retrieved from database and cached", version);
            Ok(keydata)
        }
        None => {
            tracing::warn!("⚠️  KEK version {} not found, creating...", version);

            let kek = aes::generate_key();
            let keydata = kek.as_bytes().to_vec();

            let master_key_array: [u8; 32] = master_key.try_into()
                .map_err(|_| AppError::Encryption("Invalid master key size".to_string()))?;

            let (encrypted_keydata, nonce) = aes::encrypt(&master_key_array, &keydata)?;

            let is_active = version == 1;

            sqlx::query!(
                r#"
                INSERT INTO keks (version, encrypted_keydata, nonce, is_active, is_deprecated, created_at)
                VALUES ($1, $2, $3, $4, false, NOW())
                ON CONFLICT (version) DO NOTHING
                "#,
                version,
                &encrypted_keydata,
                &nonce.to_vec(),
                is_active
            )
            .execute(pool)
            .await?;

            kek_cache.insert(version, keydata.clone()).await;

            tracing::info!("✅ KEK version {} created successfully", version);
            Ok(keydata)
        }
    }
}
