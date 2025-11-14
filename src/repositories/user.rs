use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    error::{AppError, Result},
    models::user::User,
};

/// Creates a new user in the database.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `id` - The unique identifier for the user.
/// * `email` - The user's email address.
/// * `password_hash` - The user's hashed password.
/// * `encrypted_dek` - The user's encrypted data encryption key.
/// * `dek_salt` - The salt used to derive the key that encrypts the data encryption key.
///
/// # Returns
///
/// A `Result` containing the created `User`.
pub async fn create_user(
    pool: &PgPool,
    id: Uuid,
    email: Option<String>,
    password_hash: String,
    encrypted_dek: Vec<u8>,
    dek_salt: Vec<u8>,
) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, email, password, encrypted_dek, dek_salt)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING 
            id,
            name,
            username,
            email,
            password,
            roles,
            encrypted_dek,
            dek_salt,
            dek_kek_version,
            storage_quota_bytes,
            storage_used_bytes,
            created_at,
            updated_at,
            last_password_change,
            is_active
        "#,
        id,
        email,
        password_hash,
        encrypted_dek,
        dek_salt
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Finds a user by their email address.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `email` - The email address to search for.
///
/// # Returns
///
/// A `Result` containing an `Option<User>`.
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
            id,
            name,
            username,
            email,
            password,
            roles,
            encrypted_dek,
            dek_salt,
            dek_kek_version,
            storage_quota_bytes,
            storage_used_bytes,
            created_at,
            updated_at,
            last_password_change,
            is_active
        FROM users 
        WHERE email = $1 AND is_active = true
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Finds a user by their ID.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The ID of the user to find.
///
/// # Returns
///
/// A `Result` containing an `Option<User>`.
pub async fn find_by_id(pool: &PgPool, user_id: &Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
            id,
            name,
            username,
            email,
            password,
            roles,
            encrypted_dek,
            dek_salt,
            dek_kek_version,
            storage_quota_bytes,
            storage_used_bytes,
            created_at,
            updated_at,
            last_password_change,
            is_active
        FROM users 
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Updates a user's password.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The ID of the user.
/// * `new_password` - The new hashed password.
/// * `encrypted_dek` - The new encrypted data encryption key.
/// * `dek_salt` - The new salt.
///
/// # Returns
///
/// A `Result<()>`.
pub async fn update_password(
    pool: &PgPool,
    user_id: &Uuid,
    new_password: String,
    encrypted_dek: Vec<u8>,
    dek_salt: Vec<u8>,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET 
            password = $1,
            encrypted_dek = $2,
            dek_salt = $3,
            last_password_change = NOW()
        WHERE id = $4
        "#,
        new_password,
        encrypted_dek,
        dek_salt,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Updates a user's storage usage with a quota check.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The ID of the user.
/// * `file_size` - The size of the file being added.
///
/// # Returns
///
/// A `Result` containing a `StorageCheckResult`.
pub async fn update_storage_with_quota_check(
    pool: &PgPool,
    user_id: &Uuid,
    file_size: i64,
) -> Result<StorageCheckResult> {
    let result = sqlx::query!(
        r#"
        SELECT 
            success,
            available_bytes,
            new_storage_used
        FROM update_storage_with_quota_check($1, $2)
        "#,
        user_id,
        file_size
    )
    .fetch_one(pool)
    .await?;

    Ok(StorageCheckResult {
        success: result.success.unwrap_or(false),
        available_bytes: result.available_bytes.unwrap_or(0),
        new_storage_used: result.new_storage_used.unwrap_or(0),
    })
}

/// Rolls back a user's storage usage.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The ID of the user.
/// * `file_size` - The size of the file to roll back.
///
/// # Returns
///
/// A `Result<()>`.
pub async fn rollback_storage_usage(
    pool: &PgPool,
    user_id: &Uuid,
    file_size: i64,
) -> Result<()> {
    sqlx::query!(
        r#"
        SELECT rollback_storage_usage($1, $2) as success
        "#,
        user_id,
        file_size
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}

/// Gets a user's storage information.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The ID of the user.
///
/// # Returns
///
/// A `Result` containing a tuple of `(storage_quota_bytes, storage_used_bytes)`.
pub async fn get_user_storage_info(pool: &PgPool, user_id: &Uuid) -> Result<(i64, i64)> {
    let result = sqlx::query!(
        r#"
        SELECT storage_quota_bytes, storage_used_bytes
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok((result.storage_quota_bytes, result.storage_used_bytes))
}

/// The result of a storage check.
pub struct StorageCheckResult {
    /// Whether the storage check was successful.
    pub success: bool,
    /// The number of available bytes.
    pub available_bytes: i64,
    /// The new storage usage in bytes.
    pub new_storage_used: i64,
}
