use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;
use crate::error::{AppError, Result};
use crate::models::file::File;

/// A helper function to map a `tokio_postgres::Row` to a `File`.
fn row_to_file(row: &Row) -> Result<File> {
    Ok(File {
        id: row.try_get("id").map_err(|_| AppError::MissingData("id".to_string()))?,
        user_id: row.try_get("user_id").map_err(|_| AppError::MissingData("user_id".to_string()))?,
        folder_id: row.try_get("folder_id").map_err(|_| AppError::MissingData("folder_id".to_string()))?,
        original_filename: row.try_get("original_filename").map_err(|_| AppError::MissingData("original_filename".to_string()))?,
        total_chunks: row.try_get("total_chunks").map_err(|_| AppError::MissingData("total_chunks".to_string()))?,
        chunks_metadata: row.try_get("chunks_metadata").map_err(|_| AppError::MissingData("chunks_metadata".to_string()))?,
        encrypted_dek: row.try_get("encrypted_dek").map_err(|_| AppError::MissingData("encrypted_dek".to_string()))?,
        nonce: row.try_get("nonce").map_err(|_| AppError::MissingData("nonce".to_string()))?,
        dek_version: row.try_get("dek_version").map_err(|_| AppError::MissingData("dek_version".to_string()))?,
        file_size: row.try_get("file_size").map_err(|_| AppError::MissingData("file_size".to_string()))?,
        mime_type: row.try_get("mime_type").map_err(|_| AppError::MissingData("mime_type".to_string()))?,
        checksum_sha256: row.try_get("checksum_sha256").map_err(|_| AppError::MissingData("checksum_sha256".to_string()))?,
        upload_status: row.try_get("upload_status").map_err(|_| AppError::MissingData("upload_status".to_string()))?,
        uploaded_at: row.try_get("uploaded_at").map_err(|_| AppError::MissingData("uploaded_at".to_string()))?,
        is_deleted: row.try_get("is_deleted").map_err(|_| AppError::MissingData("is_deleted".to_string()))?,
        deleted_at: row.try_get("deleted_at").map_err(|_| AppError::MissingData("deleted_at".to_string()))?,
        access_count: row.try_get("access_count").map_err(|_| AppError::MissingData("access_count".to_string()))?,
    })
}

/// Creates a new file record in the database.
pub async fn create_file(
    pool: &Pool,
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
    let client = pool.get().await?;
    let row = client
        .query_one(
            r#"
            INSERT INTO files (
                id, user_id, original_filename, total_chunks, chunks_metadata,
                encrypted_dek, nonce, dek_version, file_size, mime_type,
                checksum_sha256, upload_status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'completed')
            RETURNING *
            "#,
            &[
                &id,
                &user_id,
                &original_filename,
                &total_chunks,
                &chunks_metadata,
                &encrypted_dek,
                &nonce,
                &dek_version,
                &file_size,
                &mime_type,
                &checksum_sha256,
            ],
        )
        .await?;
    row_to_file(&row)
}

/// Finds a file by its ID and user ID.
pub async fn find_by_id(
    pool: &Pool,
    file_id: Uuid,
    user_id: Uuid,
) -> Result<Option<File>> {
    let client = pool.get().await?;
    let row = client
        .query_opt(
            r#"
            SELECT *
            FROM files
            WHERE id = $1 AND user_id = $2 AND is_deleted = false
            "#,
            &[&file_id, &user_id],
        )
        .await?;
    row.map(|r| row_to_file(&r)).transpose()
}

/// Lists the files for a given user.
pub async fn list_user_files(
    pool: &Pool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<File>> {
    let client = pool.get().await?;
    let rows = client
        .query(
            r#"
            SELECT *
            FROM files
            WHERE user_id = $1 AND is_deleted = false
            ORDER BY uploaded_at DESC
            LIMIT $2 OFFSET $3
            "#,
            &[&user_id, &limit, &offset],
        )
        .await?;
    rows.iter().map(row_to_file).collect()
}

/// Soft deletes a file.
pub async fn soft_delete_file(
    pool: &Pool,
    file_id: Uuid,
    user_id: Uuid,
) -> Result<Option<i64>> {
    let client = pool.get().await?;
    let row = client
        .query_opt(
            r#"
            UPDATE files
            SET is_deleted = true, deleted_at = NOW()
            WHERE id = $1 AND user_id = $2 AND is_deleted = false
            RETURNING file_size
            "#,
            &[&file_id, &user_id],
        )
        .await?;
    Ok(row.map(|r| r.get("file_size")))
}

/// Increments the access count for a file.
pub async fn increment_access_count(pool: &Pool, file_id: Uuid) -> Result<()> {
    let client = pool.get().await?;
    client
        .execute(
            r#"
            UPDATE files
            SET access_count = access_count + 1
            WHERE id = $1
            "#,
            &[&file_id],
        )
        .await?;
    Ok(())
}
