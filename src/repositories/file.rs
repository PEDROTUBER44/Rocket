use deadpool_postgres::Client;
use uuid::Uuid;

use crate::{
    error::Result,
    models::file::File,
    statement_cache::StatementCache,
};

/// Creates a new file record in the database.
pub async fn create_file(
    client: &Client,
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
    stmt_cache: &StatementCache,
) -> Result<File> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
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
        "#,
        )
        .await?;

    let row = client
        .query_one(
            &stmt,
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

    Ok(File::from(&row))
}

/// Finds a file by its ID and user ID.
pub async fn find_by_id(
    client: &Client,
    file_id: Uuid,
    user_id: Uuid,
    stmt_cache: &StatementCache,
) -> Result<Option<File>> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        SELECT
            id, user_id, folder_id, original_filename, total_chunks,
            chunks_metadata, encrypted_dek, nonce, dek_version, file_size,
            mime_type, checksum_sha256, upload_status, uploaded_at,
            is_deleted, deleted_at, access_count
        FROM files
        WHERE id = $1 AND user_id = $2 AND is_deleted = false
        "#,
        )
        .await?;

    let row = client.query_opt(&stmt, &[&file_id, &user_id]).await?;

    Ok(row.map(|r| File::from(&r)))
}

/// Lists the files for a given user.
pub async fn list_user_files(
    client: &Client,
    user_id: Uuid,
    limit: i64,
    offset: i64,
    stmt_cache: &StatementCache,
) -> Result<Vec<File>> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
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
        "#,
        )
        .await?;

    let rows = client
        .query(&stmt, &[&user_id, &limit, &offset])
        .await?;

    Ok(rows.iter().map(File::from).collect())
}

/// Soft deletes a file.
pub async fn soft_delete_file(
    client: &Client,
    file_id: Uuid,
    user_id: Uuid,
    stmt_cache: &StatementCache,
) -> Result<Option<i64>> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        UPDATE files
        SET is_deleted = true, deleted_at = NOW()
        WHERE id = $1 AND user_id = $2 AND is_deleted = false
        RETURNING file_size
        "#,
        )
        .await?;

    let row = client.query_opt(&stmt, &[&file_id, &user_id]).await?;

    Ok(row.map(|r| r.get("file_size")))
}

/// Increments the access count for a file.
pub async fn increment_access_count(
    client: &Client,
    file_id: Uuid,
    stmt_cache: &StatementCache,
) -> Result<()> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        UPDATE files
        SET access_count = access_count + 1
        WHERE id = $1
        "#,
        )
        .await?;

    client.execute(&stmt, &[&file_id]).await?;

    Ok(())
}
