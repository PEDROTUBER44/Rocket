use deadpool_postgres::Client;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{file::File, folder::{Folder, FolderWithStats}},
    statement_cache::StatementCache,
};

/// Creates a new folder in the database.
pub async fn create_folder(
    client: &mut Client,
    id: Uuid,
    user_id: Uuid,
    parent_folder_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    stmt_cache: &StatementCache,
) -> Result<Folder> {
    if let Some(parent_id) = parent_folder_id {
        let stmt = stmt_cache
            .get_or_prepare_client(
                client,
                r#"
                SELECT id FROM folders
                WHERE id = $1 AND user_id = $2 AND is_deleted = false
                "#,
            )
            .await?;

        client
            .query_opt(&stmt, &[&parent_id, &user_id])
            .await?
            .ok_or_else(|| AppError::Validation("Parent folder not found".to_string()))?;
    }

    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        INSERT INTO folders (id, user_id, parent_folder_id, name, description)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, parent_folder_id, name, description, is_deleted, deleted_at, created_at, updated_at
        "#,
        )
        .await?;

    let row = client
        .query_one(
            &stmt,
            &[&id, &user_id, &parent_folder_id, &name, &description],
        )
        .await?;

    Ok(Folder::from(&row))
}

/// Lists the contents of a folder.
pub async fn list_folder_contents(
    client: &mut Client,
    folder_id: Option<Uuid>,
    user_id: Uuid,
    stmt_cache: &StatementCache,
) -> Result<(Vec<Folder>, Vec<File>)> {
    let folder_stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        SELECT id, user_id, parent_folder_id, name, description, is_deleted,
               deleted_at, created_at, updated_at
        FROM folders
        WHERE user_id = $1 AND parent_folder_id IS NOT DISTINCT FROM $2 AND is_deleted = false
        ORDER BY name ASC
        "#,
        )
        .await?;

    let folder_rows = client
        .query(&folder_stmt, &[&user_id, &folder_id])
        .await?;
    let folders = folder_rows.iter().map(Folder::from).collect();

    let file_stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        SELECT
            id, user_id, folder_id, original_filename, total_chunks, chunks_metadata,
            encrypted_dek, nonce, dek_version, file_size, mime_type, checksum_sha256,
            upload_status, uploaded_at, is_deleted, deleted_at, access_count
        FROM files
        WHERE user_id = $1 AND folder_id IS NOT DISTINCT FROM $2 AND is_deleted = false
        ORDER BY uploaded_at DESC
        "#,
        )
        .await?;

    let file_rows = client.query(&file_stmt, &[&user_id, &folder_id]).await?;
    let files = file_rows.iter().map(File::from).collect();

    Ok((folders, files))
}

/// Gets a folder with its statistics.
pub async fn get_folder_with_stats(
    client: &mut Client,
    folder_id: Uuid,
    user_id: Uuid,
    stmt_cache: &StatementCache,
) -> Result<Option<FolderWithStats>> {
    let stmt = stmt_cache
        .get_or_prepare_client(
            client,
            r#"
        SELECT id, user_id, parent_folder_id, name, description, is_deleted, deleted_at, created_at, updated_at
        FROM folders
        WHERE id = $1 AND user_id = $2 AND is_deleted = false
        "#,
        )
        .await?;

    let folder_row = client.query_opt(&stmt, &[&folder_id, &user_id]).await?;

    match folder_row {
        Some(row) => {
            let f = Folder::from(&row);

            let file_count_stmt = stmt_cache
                .get_or_prepare_client(
                    client,
                    "SELECT COUNT(*) FROM files WHERE folder_id = $1 AND is_deleted = false",
                )
                .await?;
            let file_count_row = client.query_one(&file_count_stmt, &[&folder_id]).await?;
            let file_count: i64 = file_count_row.get(0);

            let subfolder_count_stmt = stmt_cache
                .get_or_prepare_client(
                    client,
                    "SELECT COUNT(*) FROM folders WHERE parent_folder_id = $1 AND is_deleted = false",
                )
                .await?;
            let subfolder_count_row = client
                .query_one(&subfolder_count_stmt, &[&folder_id])
                .await?;
            let subfolder_count: i64 = subfolder_count_row.get(0);

            let total_size_stmt = stmt_cache
                .get_or_prepare_client(
                    client,
                    "SELECT COALESCE(SUM(file_size), 0) FROM files WHERE folder_id = $1 AND is_deleted = false",
                )
                .await?;
            let total_size_row = client.query_one(&total_size_stmt, &[&folder_id]).await?;
            let total_size: i64 = total_size_row.get(0);

            Ok(Some(FolderWithStats {
                id: f.id,
                name: f.name,
                description: f.description,
                created_at: f.created_at,
                file_count,
                subfolder_count,
                total_size,
            }))
        }
        None => Ok(None),
    }
}


/// Recursively deletes a folder and its contents.
pub async fn delete_folder_recursive(
    client: &mut Client,
    folder_id: Uuid,
    user_id: Uuid,
    stmt_cache: &StatementCache,
) -> Result<()> {
    // Note: Using a transaction to ensure atomicity
    let transaction = client.transaction().await?;

    let update_files_stmt = stmt_cache
        .get_or_prepare_transaction(
            &transaction,
            r#"
        UPDATE files
        SET is_deleted = true, deleted_at = NOW()
        WHERE folder_id IN (
            WITH RECURSIVE folder_tree AS (
                SELECT id FROM folders WHERE id = $1 AND user_id = $2
                UNION ALL
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_folder_id = ft.id
            )
            SELECT id FROM folder_tree
        )
        "#,
        )
        .await?;

    transaction
        .execute(&update_files_stmt, &[&folder_id, &user_id])
        .await?;

    let update_folders_stmt = stmt_cache
        .get_or_prepare_transaction(
            &transaction,
            r#"
        UPDATE folders
        SET is_deleted = true, deleted_at = NOW()
        WHERE id IN (
            WITH RECURSIVE folder_tree AS (
                SELECT id FROM folders WHERE id = $1 AND user_id = $2
                UNION ALL
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_folder_id = ft.id
            )
            SELECT id FROM folder_tree
        )
        "#,
        )
        .await?;

    transaction
        .execute(&update_folders_stmt, &[&folder_id, &user_id])
        .await?;

    transaction.commit().await?;

    Ok(())
}
