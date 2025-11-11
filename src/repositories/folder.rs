use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;
use crate::{
    error::{AppError, Result},
    models::{folder::{Folder, FolderWithStats}, file::File},
};

/// A helper function to map a `tokio_postgres::Row` to a `Folder`.
fn row_to_folder(row: &Row) -> Result<Folder> {
    Ok(Folder {
        id: row.try_get("id").map_err(|_| AppError::MissingData("id".to_string()))?,
        user_id: row.try_get("user_id").map_err(|_| AppError::MissingData("user_id".to_string()))?,
        parent_folder_id: row.try_get("parent_folder_id").map_err(|_| AppError::MissingData("parent_folder_id".to_string()))?,
        name: row.try_get("name").map_err(|_| AppError::MissingData("name".to_string()))?,
        description: row.try_get("description").map_err(|_| AppError::MissingData("description".to_string()))?,
        is_deleted: row.try_get("is_deleted").map_err(|_| AppError::MissingData("is_deleted".to_string()))?,
        deleted_at: row.try_get("deleted_at").map_err(|_| AppError::MissingData("deleted_at".to_string()))?,
        created_at: row.try_get("created_at").map_err(|_| AppError::MissingData("created_at".to_string()))?,
        updated_at: row.try_get("updated_at").map_err(|_| AppError::MissingData("updated_at".to_string()))?,
    })
}

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

/// Creates a new folder in the database.
pub async fn create_folder(
    pool: &Pool,
    id: Uuid,
    user_id: Uuid,
    parent_folder_id: Option<Uuid>,
    name: String,
    description: Option<String>,
) -> Result<Folder> {
    let client = pool.get().await?;
    if let Some(parent_id) = parent_folder_id {
        let parent_exists = client
            .query_opt(
                r#"
                SELECT id FROM folders
                WHERE id = $1 AND user_id = $2 AND is_deleted = false
                "#,
                &[&parent_id, &user_id],
            )
            .await?
            .is_some();
        if !parent_exists {
            return Err(AppError::Validation("Parent folder not found".to_string()));
        }
    }

    let row = client
        .query_one(
            r#"
            INSERT INTO folders (id, user_id, parent_folder_id, name, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            &[&id, &user_id, &parent_folder_id, &name, &description],
        )
        .await?;

    row_to_folder(&row)
}

/// Lists the contents of a folder.
pub async fn list_folder_contents(
    pool: &Pool,
    folder_id: Option<Uuid>,
    user_id: Uuid,
) -> Result<(Vec<Folder>, Vec<File>)> {
    let client = pool.get().await?;
    let folders_rows = client
        .query(
            r#"
            SELECT *
            FROM folders
            WHERE user_id = $1 AND parent_folder_id = $2 AND is_deleted = false
            ORDER BY name ASC
            "#,
            &[&user_id, &folder_id],
        )
        .await?;
    let folders = folders_rows.iter().map(row_to_folder).collect::<Result<Vec<Folder>>>()?;

    let files_rows = client
        .query(
            r#"
            SELECT *
            FROM files
            WHERE user_id = $1 AND folder_id = $2 AND is_deleted = false
            ORDER BY uploaded_at DESC
            "#,
            &[&user_id, &folder_id],
        )
        .await?;
    let files = files_rows.iter().map(row_to_file).collect::<Result<Vec<File>>>()?;

    Ok((folders, files))
}

/// Gets a folder with its statistics.
pub async fn get_folder_with_stats(
    pool: &Pool,
    folder_id: Uuid,
    user_id: Uuid,
) -> Result<Option<FolderWithStats>> {
    let client = pool.get().await?;
    let folder_row = client
        .query_opt(
            r#"
            SELECT *
            FROM folders
            WHERE id = $1 AND user_id = $2 AND is_deleted = false
            "#,
            &[&folder_id, &user_id],
        )
        .await?;

    match folder_row {
        Some(row) => {
            let folder = row_to_folder(&row)?;
            let file_count_row = client
                .query_one(
                    "SELECT COUNT(*) FROM files WHERE folder_id = $1 AND is_deleted = false",
                    &[&folder_id],
                )
                .await?;
            let file_count: i64 = file_count_row.get(0);

            let subfolder_count_row = client
                .query_one(
                    "SELECT COUNT(*) FROM folders WHERE parent_folder_id = $1 AND is_deleted = false",
                    &[&folder_id],
                )
                .await?;
            let subfolder_count: i64 = subfolder_count_row.get(0);

            let total_size_row = client
                .query_one(
                    "SELECT COALESCE(SUM(file_size), 0) FROM files WHERE folder_id = $1 AND is_deleted = false",
                    &[&folder_id],
                )
                .await?;
            let total_size: i64 = total_size_row.get(0);

            Ok(Some(FolderWithStats {
                id: folder.id,
                name: folder.name,
                description: folder.description,
                created_at: folder.created_at,
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
    pool: &Pool,
    folder_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    let client = pool.get().await?;
    client
        .execute(
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
            &[&folder_id, &user_id],
        )
        .await?;

    client
        .execute(
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
            &[&folder_id, &user_id],
        )
        .await?;

    Ok(())
}
