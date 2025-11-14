use uuid::Uuid;
use crate::{
    crypto::{aes, kek},
    error::{AppError, Result},
    models::file::File,
    repositories::{file as file_repo, user as user_repo},
    state::AppState,
};

/// Saves a file's metadata to the database.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `file_id` - The unique identifier for the file.
/// * `user_id` - The ID of the user who owns the file.
/// * `original_filename` - The original filename of the file.
/// * `total_chunks` - The total number of chunks in the file.
/// * `chunks_metadata` - The metadata for the chunks of the file.
/// * `file_size` - The size of the file in bytes.
/// * `mime_type` - The MIME type of the file.
/// * `user_dek` - The user's data encryption key.
/// * `checksum_sha256` - The SHA256 checksum of the file.
///
/// # Returns
///
/// A `Result` containing the created `File`.
pub async fn save_file_metadata(
    state: &AppState,
    file_id: Uuid,
    user_id: Uuid,
    original_filename: String,
    total_chunks: i32,
    chunks_metadata: Vec<u8>,
    file_size: i64,
    mime_type: Option<String>,
    user_dek: &[u8],
    checksum_sha256: Option<String>,
) -> Result<File> {
    let mut tx = state.db.begin().await?;

    let user_quota = sqlx::query!(
        r#"
        SELECT storage_quota_bytes, storage_used_bytes
        FROM users
        WHERE id = $1
        FOR UPDATE
        "#,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| AppError::Unauthorized)?;

    let available_space = user_quota.storage_quota_bytes - user_quota.storage_used_bytes;

    if file_size > available_space {
        tx.rollback().await?;
        return Err(AppError::Validation(format!(
            "Insufficient storage quota. Required: {} bytes, Available: {} bytes",
            file_size, available_space
        )));
    }

    sqlx::query!(
        r#"
        UPDATE users
        SET storage_used_bytes = storage_used_bytes + $1
        WHERE id = $2
        "#,
        file_size,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let (kek_version, kek_bytes) =
        kek::get_active_kek(&state.db, state.config.master_key.as_ref(), &state.kek_cache).await?;

    let kek_array: [u8; 32] = kek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid KEK size".to_string()))?;

    let (encrypted_dek, dek_nonce) = aes::encrypt(&kek_array, user_dek)?;

    let file_record = file_repo::create_file(
        &state.db,
        file_id,
        user_id,
        original_filename,
        total_chunks,
        chunks_metadata,
        encrypted_dek,
        dek_nonce.to_vec(),
        kek_version,
        file_size,
        mime_type,
        checksum_sha256,
    )
    .await?;


    Ok(file_record)
}

/// Deletes a file.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `user_id` - The ID of the user who owns the file.
/// * `file_id` - The ID of the file to delete.
///
/// # Returns
///
/// A `Result<()>`.
pub async fn delete_file(state: &AppState, user_id: Uuid, file_id: Uuid) -> Result<()> {
    let file_size = file_repo::soft_delete_file(&state.db, file_id, user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut tx = state.db.begin().await?;

    sqlx::query!(
        r#"
        UPDATE users
        SET storage_used_bytes = GREATEST(0, storage_used_bytes - $1)
        WHERE id = $2
        "#,
        file_size,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Lists the files for a given user.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `user_id` - The ID of the user.
/// * `limit` - The maximum number of files to return.
/// * `offset` - The number of files to skip.
///
/// # Returns
///
/// A `Result` containing a `Vec<File>`.
pub async fn list_files(
    state: &AppState,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<File>> {
    file_repo::list_user_files(&state.db, user_id, limit, offset).await
}

/// Gets a user's storage information.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `user_id` - The ID of the user.
///
/// # Returns
///
/// A `Result` containing a tuple of `(storage_quota_bytes, storage_used_bytes, available_bytes)`.
pub async fn get_user_storage_info(state: &AppState, user_id: Uuid) -> Result<(i64, i64, i64)> {
    let user = sqlx::query!(
        r#"
        SELECT storage_quota_bytes, storage_used_bytes
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await?;

    let available = user.storage_quota_bytes - user.storage_used_bytes;
    Ok((user.storage_quota_bytes, user.storage_used_bytes, available))
}
