use sqlx::PgPool;
use uuid::Uuid;
use crate::{error::Result, models::folder::Folder};

/// Creates a new folder in the database.
///
/// # Arguments
///
/// * `db` - The database connection pool.
/// * `id` - The unique identifier for the folder.
/// * `user_id` - The ID of the user who owns the folder.
/// * `parent_folder_id` - The ID of the parent folder, if any.
/// * `name` - The name of the folder.
/// * `description` - The description of the folder.
///
/// # Returns
///
/// A `Result` containing the created `Folder`.
pub async fn create_folder(
    db: &PgPool,
    id: Uuid,
    user_id: Uuid,
    parent_folder_id: Option<Uuid>,
    name: String,
    description: Option<String>,
) -> Result<Folder> {
    if let Some(parent_id) = parent_folder_id {
        sqlx::query!(
            r#"
            SELECT id FROM folders
            WHERE id = $1 AND user_id = $2 AND is_deleted = false
            "#,
            parent_id,
            user_id
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| crate::error::AppError::Validation(
            "Parent folder not found".to_string()
        ))?;
    }

    let folder = sqlx::query_as!(
        Folder,
        r#"
        INSERT INTO folders (id, user_id, parent_folder_id, name, description)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, parent_folder_id, name, description, is_deleted, deleted_at, created_at, updated_at
        "#,
        id,
        user_id,
        parent_folder_id,
        name,
        description
    )
    .fetch_one(db)
    .await?;

    Ok(folder)
}

/// Lists the contents of a folder.
///
/// # Arguments
///
/// * `db` - The database connection pool.
/// * `folder_id` - The ID of the folder to list. If `None`, lists the root folder.
/// * `user_id` - The ID of the user.
///
/// # Returns
///
/// A `Result` containing a tuple of `(Vec<Folder>, Vec<crate::models::file::File>)`.
pub async fn list_folder_contents(
    db: &PgPool,
    folder_id: Option<Uuid>,
    user_id: Uuid,
) -> Result<(Vec<Folder>, Vec<crate::models::file::File>)> {
    let folders = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, user_id, parent_folder_id, name, description, is_deleted,
               deleted_at, created_at, updated_at
        FROM folders
        WHERE user_id = $1 AND parent_folder_id = $2 AND is_deleted = false
        ORDER BY name ASC
        "#
    )
    .bind(user_id)
    .bind(folder_id)
    .fetch_all(db)
    .await?;

    let files = sqlx::query_as::<_, crate::models::file::File>(
        r#"
        SELECT
            id, user_id, folder_id, original_filename, total_chunks, chunks_metadata,
            encrypted_dek, nonce, dek_version, file_size, mime_type, checksum_sha256,
            upload_status, uploaded_at, is_deleted, deleted_at, access_count
        FROM files
        WHERE user_id = $1 AND folder_id = $2 AND is_deleted = false
        ORDER BY uploaded_at DESC
        "#
    )
    .bind(user_id)
    .bind(folder_id)
    .fetch_all(db)
    .await?;

    Ok((folders, files))
}

/// Gets a folder with its statistics.
///
/// # Arguments
///
/// * `db` - The database connection pool.
/// * `folder_id` - The ID of the folder.
/// * `user_id` - The ID of the user.
///
/// # Returns
///
/// A `Result` containing an `Option<crate::models::folder::FolderWithStats>`.
pub async fn get_folder_with_stats(
    db: &PgPool,
    folder_id: Uuid,
    user_id: Uuid,
) -> Result<Option<crate::models::folder::FolderWithStats>> {
    let folder = sqlx::query_as!(
        Folder,
        r#"
        SELECT id, user_id, parent_folder_id, name, description, is_deleted, deleted_at, created_at, updated_at
        FROM folders
        WHERE id = $1 AND user_id = $2 AND is_deleted = false
        "#,
        folder_id,
        user_id
    )
    .fetch_optional(db)
    .await?;

    match folder {
        Some(f) => {
            let file_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM files WHERE folder_id = $1 AND is_deleted = false"
            )
            .bind(folder_id)
            .fetch_one(db)
            .await?;

            let subfolder_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM folders WHERE parent_folder_id = $1 AND is_deleted = false"
            )
            .bind(folder_id)
            .fetch_one(db)
            .await?;

            let total_size = sqlx::query_scalar::<_, Option<i64>>(
                "SELECT COALESCE(SUM(file_size), 0) FROM files WHERE folder_id = $1 AND is_deleted = false"
            )
            .bind(folder_id)
            .fetch_one(db)
            .await?
            .unwrap_or(0);

            Ok(Some(crate::models::folder::FolderWithStats {
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
///
/// # Arguments
///
/// * `db` - The database connection pool.
/// * `folder_id` - The ID of the folder to delete.
/// * `user_id` - The ID of the user.
///
/// # Returns
///
/// A `Result<()>`.
pub async fn delete_folder_recursive(
    db: &PgPool,
    folder_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query!(
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
        folder_id,
        user_id
    )
    .execute(db)
    .await?;

    sqlx::query!(
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
        folder_id,
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
}
