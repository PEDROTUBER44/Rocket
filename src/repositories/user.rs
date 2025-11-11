use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;
use crate::{
    error::{AppError, Result},
    models::user::User,
};
use chrono::{DateTime, Utc};
use std::convert::TryFrom;

/// A helper function to map a `tokio_postgres::Row` to a `User`.
fn row_to_user(row: &Row) -> Result<User> {
    Ok(User {
        id: row.try_get("id").map_err(|_| AppError::MissingData("id".to_string()))?,
        name: row.try_get("name").map_err(|_| AppError::MissingData("name".to_string()))?,
        username: row.try_get("username").map_err(|_| AppError::MissingData("username".to_string()))?,
        email: row.try_get("email").map_err(|_| AppError::MissingData("email".to_string()))?,
        password: row.try_get("password").map_err(|_| AppError::MissingData("password".to_string()))?,
        roles: row.try_get("roles").map_err(|_| AppError::MissingData("roles".to_string()))?,
        encrypted_dek: row.try_get("encrypted_dek").map_err(|_| AppError::MissingData("encrypted_dek".to_string()))?,
        dek_salt: row.try_get("dek_salt").map_err(|_| AppError::MissingData("dek_salt".to_string()))?,
        dek_kek_version: row.try_get("dek_kek_version").map_err(|_| AppError::MissingData("dek_kek_version".to_string()))?,
        storage_quota_bytes: row.try_get("storage_quota_bytes").map_err(|_| AppError::MissingData("storage_quota_bytes".to_string()))?,
        storage_used_bytes: row.try_get("storage_used_bytes").map_err(|_| AppError::MissingData("storage_used_bytes".to_string()))?,
        created_at: row.try_get("created_at").map_err(|_| AppError::MissingData("created_at".to_string()))?,
        updated_at: row.try_get("updated_at").map_err(|_| AppError::MissingData("updated_at".to_string()))?,
        last_password_change: row.try_get("last_password_change").map_err(|_| AppError::MissingData("last_password_change".to_string()))?,
        is_active: row.try_get("is_active").map_err(|_| AppError::MissingData("is_active".to_string()))?,
    })
}

/// Creates a new user in the database.
pub async fn create_user(
    pool: &Pool,
    id: Uuid,
    email: Option<String>,
    password_hash: String,
    encrypted_dek: Vec<u8>,
    dek_salt: Vec<u8>,
) -> Result<User> {
    let client = pool.get().await?;
    let row = client
        .query_one(
            r#"
            INSERT INTO users (id, email, password, encrypted_dek, dek_salt)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            &[&id, &email, &password_hash, &encrypted_dek, &dek_salt],
        )
        .await?;
    row_to_user(&row)
}

/// Finds a user by their email address.
pub async fn find_by_email(pool: &Pool, email: &str) -> Result<Option<User>> {
    let client = pool.get().await?;
    let row = client
        .query_opt(
            r#"
            SELECT *
            FROM users
            WHERE email = $1 AND is_active = true
            "#,
            &[&email],
        )
        .await?;
    row.map(|r| row_to_user(&r)).transpose()
}

/// Finds a user by their ID.
pub async fn find_by_id(pool: &Pool, user_id: &Uuid) -> Result<Option<User>> {
    let client = pool.get().await?;
    let row = client
        .query_opt(
            r#"
            SELECT *
            FROM users
            WHERE id = $1
            "#,
            &[user_id],
        )
        .await?;
    row.map(|r| row_to_user(&r)).transpose()
}

/// Updates a user's password.
pub async fn update_password(
    pool: &Pool,
    user_id: &Uuid,
    new_password: String,
    encrypted_dek: Vec<u8>,
    dek_salt: Vec<u8>,
) -> Result<()> {
    let client = pool.get().await?;
    client
        .execute(
            r#"
            UPDATE users
            SET
                password = $1,
                encrypted_dek = $2,
                dek_salt = $3,
                last_password_change = NOW()
            WHERE id = $4
            "#,
            &[&new_password, &encrypted_dek, &dek_salt, user_id],
        )
        .await?;
    Ok(())
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

/// Updates a user's storage usage with a quota check.
pub async fn update_storage_with_quota_check(
    pool: &Pool,
    user_id: &Uuid,
    file_size: i64,
) -> Result<StorageCheckResult> {
    let client = pool.get().await?;
    let row = client
        .query_one(
            r#"
            SELECT
                success,
                available_bytes,
                new_storage_used
            FROM update_storage_with_quota_check($1, $2)
            "#,
            &[user_id, &file_size],
        )
        .await?;
    Ok(StorageCheckResult {
        success: row.try_get("success").unwrap_or(false),
        available_bytes: row.try_get("available_bytes").unwrap_or(0),
        new_storage_used: row.try_get("new_storage_used").unwrap_or(0),
    })
}

/// Rolls back a user's storage usage.
pub async fn rollback_storage_usage(
    pool: &Pool,
    user_id: &Uuid,
    file_size: i64,
) -> Result<()> {
    let client = pool.get().await?;
    client
        .execute(
            r#"
            SELECT rollback_storage_usage($1, $2) as success
            "#,
            &[user_id, &file_size],
        )
        .await?;
    Ok(())
}

/// Gets a user's storage information.
pub async fn get_user_storage_info(pool: &Pool, user_id: &Uuid) -> Result<(i64, i64)> {
    let client = pool.get().await?;
    let row = client
        .query_opt(
            r#"
            SELECT storage_quota_bytes, storage_used_bytes
            FROM users
            WHERE id = $1
            "#,
            &[user_id],
        )
        .await?
        .ok_or(AppError::NotFound)?;
    let storage_quota_bytes: i64 = row.try_get("storage_quota_bytes").map_err(|_| AppError::MissingData("storage_quota_bytes".to_string()))?;
    let storage_used_bytes: i64 = row.try_get("storage_used_bytes").map_err(|_| AppError::MissingData("storage_used_bytes".to_string()))?;
    Ok((storage_quota_bytes, storage_used_bytes))
}
