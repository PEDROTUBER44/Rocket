use deadpool_postgres::Client;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::user::User,
    statement_cache::StatementCache,
};

/// Creates a new user in the database.
pub async fn create_user(
    client: &Client,
    id: Uuid,
    email: Option<String>,
    password_hash: String,
    encrypted_dek: Vec<u8>,
    dek_salt: Vec<u8>,
    stmt_cache: &StatementCache,
) -> Result<User> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
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
        )
        .await?;

    let row = client
        .query_one(
            &stmt,
            &[
                &id,
                &email,
                &password_hash,
                &encrypted_dek,
                &dek_salt,
            ],
        )
        .await?;

    Ok(User::from(&row))
}

/// Finds a user by their email address.
pub async fn find_by_email(
    client: &Client,
    email: &str,
    stmt_cache: &StatementCache,
) -> Result<Option<User>> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
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
        )
        .await?;

    let row = client.query_opt(&stmt, &[&email]).await?;

    Ok(row.map(|r| User::from(&r)))
}

/// Finds a user by their ID.
pub async fn find_by_id(
    client: &Client,
    user_id: &Uuid,
    stmt_cache: &StatementCache,
) -> Result<Option<User>> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
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
        )
        .await?;

    let row = client.query_opt(&stmt, &[&user_id]).await?;

    Ok(row.map(|r| User::from(&r)))
}

/// Updates a user's password.
pub async fn update_password(
    client: &Client,
    user_id: &Uuid,
    new_password: String,
    encrypted_dek: Vec<u8>,
    dek_salt: Vec<u8>,
    stmt_cache: &StatementCache,
) -> Result<()> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        UPDATE users
        SET 
            password = $1,
            encrypted_dek = $2,
            dek_salt = $3,
            last_password_change = NOW()
        WHERE id = $4
        "#,
        )
        .await?;

    client
        .execute(
            &stmt,
            &[&new_password, &encrypted_dek, &dek_salt, &user_id],
        )
        .await?;

    Ok(())
}

/// The result of a storage check.
#[derive(Debug)]
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
    client: &Client,
    user_id: &Uuid,
    file_size: i64,
    stmt_cache: &StatementCache,
) -> Result<StorageCheckResult> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        SELECT 
            success,
            available_bytes,
            new_storage_used
        FROM update_storage_with_quota_check($1, $2)
        "#,
        )
        .await?;

    let row = client.query_one(&stmt, &[&user_id, &file_size]).await?;

    Ok(StorageCheckResult {
        success: row.try_get("success")?,
        available_bytes: row.try_get("available_bytes")?,
        new_storage_used: row.try_get("new_storage_used")?,
    })
}

/// Rolls back a user's storage usage.
pub async fn rollback_storage_usage(
    client: &Client,
    user_id: &Uuid,
    file_size: i64,
    stmt_cache: &StatementCache,
) -> Result<()> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        SELECT rollback_storage_usage($1, $2) as success
        "#,
        )
        .await?;

    client.query_one(&stmt, &[&user_id, &file_size]).await?;

    Ok(())
}

/// Gets a user's storage information.
pub async fn get_user_storage_info(
    client: &Client,
    user_id: &Uuid,
    stmt_cache: &StatementCache,
) -> Result<(i64, i64)> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        SELECT storage_quota_bytes, storage_used_bytes
        FROM users
        WHERE id = $1
        "#,
        )
        .await?;

    let row = client
        .query_opt(&stmt, &[&user_id])
        .await?
        .ok_or(AppError::NotFound)?;

    let storage_quota_bytes: i64 = row.try_get("storage_quota_bytes")?;
    let storage_used_bytes: i64 = row.try_get("storage_used_bytes")?;

    Ok((storage_quota_bytes, storage_used_bytes))
}
