use sqlx::PgPool;
use uuid::Uuid;
use crate::error::Result;
use crate::models::file::File;

/// Creates a new file record in the database.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `id` - The unique identifier for the file.
/// * `user_id` - The ID of the user who owns the file.
/// * `original_filename` - The original filename of the file.
/// * `total_chunks` - The total number of chunks in the file.
/// * `chunks_metadata` - The metadata for the chunks of the file.
/// * `encrypted_dek` - The encrypted data encryption key for the file.
/// * `nonce` - The nonce used to encrypt the data encryption key.
/// * `dek_version` - The version of the key encryption key used to encrypt the data encryption key.
/// * `file_size` - The size of the file in bytes.
/// * `mime_type` - The MIME type of the file.
/// * `checksum_sha256` - The SHA256 checksum of the file.
///
/// # Returns
///
/// A `Result` containing the created `File`.
pub async fn create_file(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    original_filename: String,
    total_chunks: i32,
    chunks_metadata: Vec<u8>,
    encrypted_dek: Vec<u8>,
    nonce: Vec<u8>,
    dek_version: i32,
    file_size: i64,
    mime_type: Option<String>,
    checksum_sha256: Option<String>,
) -> Result<File> {
    let file = sqlx::query_as::<_, File>(
        r#"
        INSERT INTO files (
            id, user_id, original_filename, total_chunks, chunks_metadata,
            encrypted_dek, nonce, dek_version, file_size, mime_type,
            checksum_sha256, upload_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'completed')
        RETURNING
            id, user_id, folder_id, original_filename, total_chunks,
            chunks_metadata, encrypted_dek, nonce, dek_version, file_size,
            mime_type, checksum_sha256, upload_status, uploaded_at,
            is_deleted, deleted_at, access_count
        "#
    )
    .bind(id)
    .bind(user_id)
    .bind(original_filename)
    .bind(total_chunks)
    .bind(chunks_metadata)
    .bind(encrypted_dek)
    .bind(nonce)
    .bind(dek_version)
    .bind(file_size)
    .bind(mime_type)
    .bind(checksum_sha256)
    .fetch_one(pool)
    .await?;

    Ok(file)
}

/// Finds a file by its ID and user ID.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `file_id` - The ID of the file to find.
/// * `user_id` - The ID of the user who owns the file.
///
/// # Returns
///
/// A `Result` containing an `Option<File>`.
pub async fn find_by_id(
    pool: &PgPool,
    file_id: Uuid,
    user_id: Uuid,
) -> Result<Option<File>> {
    let file = sqlx::query_as::<_, File>(
        r#"
        SELECT
            id, user_id, folder_id, original_filename, total_chunks,
            chunks_metadata, encrypted_dek, nonce, dek_version, file_size,
            mime_type, checksum_sha256, upload_status, uploaded_at,
            is_deleted, deleted_at, access_count
        FROM files
        WHERE id = $1 AND user_id = $2 AND is_deleted = false
        "#
    )
    .bind(file_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(file)
}

/// Lists the files for a given user.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The ID of the user.
/// * `limit` - The maximum number of files to return.
/// * `offset` - The number of files to skip.
///
/// # Returns
///
/// A `Result` containing a `Vec<File>`.
pub async fn list_user_files(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<File>> {
    let files = sqlx::query_as::<_, File>(
        r#"
        SELECT
            id, user_id, folder_id, original_filename, total_chunks,
            chunks_metadata, encrypted_dek, nonce, dek_version, file_size,
            mime_type, checksum_sha256, upload_status, uploaded_at,
            is_deleted, deleted_at, access_count
        FROM files
        WHERE user_id = $1 AND is_deleted = false
        ORDER BY uploaded_at DESC
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(files)
}

/// Soft deletes a file.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `file_id` - The ID of the file to delete.
/// * `user_id` - The ID of the user who owns the file.
///
/// # Returns
///
/// A `Result` containing an `Option<i64>` with the size of the deleted file.
pub async fn soft_delete_file(
    pool: &PgPool,
    file_id: Uuid,
    user_id: Uuid,
) -> Result<Option<i64>> {
    let result = sqlx::query!(
        r#"
        UPDATE files
        SET is_deleted = true, deleted_at = NOW()
        WHERE id = $1 AND user_id = $2 AND is_deleted = false
        RETURNING file_size
        "#,
        file_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.file_size))
}

/// Increments the access count for a file.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `file_id` - The ID of the file.
///
/// # Returns
///
/// A `Result<()>`.
pub async fn increment_access_count(pool: &PgPool, file_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE files
        SET access_count = access_count + 1
        WHERE id = $1
        "#,
        file_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
