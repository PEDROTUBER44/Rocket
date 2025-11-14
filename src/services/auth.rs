use crate::crypto::dek;
use crate::error::{AppError, Result};
use crate::models::user::User;
use crate::repositories::user as user_repo;
use crate::state::AppState;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, ParamsBuilder,
};
use rand::{
    rngs::OsRng,
    TryRngCore
};
use sqlx::PgPool;
use zeroize::Zeroize;

/// The memory cost for Argon2 in MB.
const ARGON2_MEMORY_MB: u32 = 19;
/// The number of iterations for Argon2.
const ARGON2_ITERATIONS: u32 = 3;
/// The parallelism factor for Argon2.
const ARGON2_PARALLELISM: u32 = 6;

/// Hashes a password using Argon2id.
///
/// # Arguments
///
/// * `password` - The password to hash.
///
/// # Returns
///
/// A `Result` containing the hashed password.
fn hash_password(password: &str) -> Result<String> {
    let mut password_bytes = password.as_bytes().to_vec();

    let mut salt_bytes = [0u8; 16];
    OsRng.try_fill_bytes(&mut salt_bytes)
        .map_err(|e| AppError::Internal(format!("Failed to generate salt: {}", e)))?;
    
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| AppError::Encryption(format!("Salt encoding error: {}", e)))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        ParamsBuilder::new()
            .m_cost(ARGON2_MEMORY_MB * 1024)
            .t_cost(ARGON2_ITERATIONS)
            .p_cost(ARGON2_PARALLELISM)
            .build()
            .map_err(|e| AppError::Encryption(format!("Argon2 params: {}", e)))?,
    );

    let password_hash = argon2
        .hash_password(&password_bytes, &salt)
        .map_err(|e| AppError::Encryption(format!("Argon2 hash error: {}", e)))?
        .to_string();

    password_bytes.zeroize();
    tracing::debug!("Password hashed successfully with Argon2");
    Ok(password_hash)
}

/// Verifies a password against a hash.
///
/// # Arguments
///
/// * `password` - The password to verify.
/// * `hash` - The hash to verify against.
///
/// # Returns
///
/// A `Result` containing `true` if the password is valid, `false` otherwise.
fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let mut password_bytes = password.as_bytes().to_vec();
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Encryption(format!("Hash parse error: {}", e)))?;
    let argon2 = Argon2::default();
    let result = argon2
        .verify_password(&password_bytes, &parsed_hash)
        .is_ok();

    password_bytes.zeroize();
    tracing::debug!("Password verification completed");
    Ok(result)
}

/// Creates a new user.
///
/// # Arguments
///
/// * `db` - The database connection pool.
/// * `name` - The user's name.
/// * `username` - The user's username.
/// * `password` - The user's password.
/// * `_master_key` - The master key (unused).
///
/// # Returns
///
/// A `Result` containing the created `User`.
pub async fn create_user(
    db: &PgPool,
    name: String,
    username: String,
    password: String,
    _master_key: &[u8],
) -> Result<User> {
    tracing::debug!("🔐 Creating user: {}", username);
    let hashed_password = hash_password(&password)?;
    let (encrypted_dek, dek_salt) = dek::create_user_dek(&password)?;
    
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (name, username, password, encrypted_dek, dek_salt)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, username, email, password, roles, encrypted_dek, dek_salt,
        dek_kek_version, storage_quota_bytes, storage_used_bytes, created_at,
        updated_at, last_password_change, is_active
        "#,
    )
    .bind(name)
    .bind(username)
    .bind(hashed_password)
    .bind(encrypted_dek)
    .bind(dek_salt)
    .fetch_one(db)
    .await?;

    tracing::info!("✅ User created with ID: {}", user.id);
    Ok(user)
}

/// Authenticates a user.
///
/// # Arguments
///
/// * `db` - The database connection pool.
/// * `username` - The user's username.
/// * `password` - The user's password.
/// * `_master_key` - The master key (unused).
///
/// # Returns
///
/// A `Result` containing the authenticated `User`.
pub async fn authenticate_user(
    db: &PgPool,
    username: String,
    password: String,
    _master_key: &[u8],
) -> Result<User> {
    tracing::debug!("🔐 Authenticating user: {}", username);

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, username, email, password, roles, encrypted_dek, dek_salt,
        dek_kek_version, storage_quota_bytes, storage_used_bytes, created_at,
        updated_at, last_password_change, is_active
        FROM users
        WHERE username = $1 AND is_active = true
        "#,
    )
    .bind(&username)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Authentication("Invalid username or password".to_string()))?;

    if !verify_password(&password, &user.password)? {
        return Err(AppError::Authentication(
            "Invalid username or password".to_string(),
        ));
    }

    tracing::info!("✅ User authenticated: {}", user.id);

    Ok(user)
}

/// Changes a user's password.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `user_id` - The ID of the user.
/// * `old_password` - The user's old password.
/// * `new_password` - The user's new password.
///
/// # Returns
///
/// A `Result<()>`.
pub async fn change_password(
    state: &AppState,
    user_id: uuid::Uuid,
    old_password: String,
    new_password: String,
) -> Result<()> {
    tracing::info!("🔑 Changing password for user: {}", user_id);

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, username, email, password, roles, encrypted_dek, dek_salt,
        dek_kek_version, storage_quota_bytes, storage_used_bytes, created_at,
        updated_at, last_password_change, is_active
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if !verify_password(&old_password, &user.password)? {
        return Err(AppError::Authentication(
            "Invalid current password".to_string(),
        ));
    }

    let new_hashed_password = hash_password(&new_password)?;

    let enc_dek = user
        .encrypted_dek
        .clone()
        .ok_or_else(|| AppError::Encryption("Missing encrypted DEK".to_string()))?;
    let dek_salt = user
        .dek_salt
        .clone()
        .ok_or_else(|| AppError::Encryption("Missing DEK salt".to_string()))?;

    let (new_encrypted_dek, new_dek_salt) =
        dek::change_user_password_dek(&enc_dek, &dek_salt, &old_password, &new_password)?;

    sqlx::query(
        r#"
        UPDATE users
        SET password = $1, encrypted_dek = $2, dek_salt = $3, last_password_change = NOW()
        WHERE id = $4
        "#,
    )
    .bind(&new_hashed_password)
    .bind(&new_encrypted_dek)
    .bind(&new_dek_salt)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    tracing::info!("✅ Password changed for user: {}", user_id);

    Ok(())
}
