use uuid::Uuid;
use crate::{
    error::Result,
    models::folder::Folder,
    repositories::folder as folder_repo,
    state::AppState,
};

/// Creates a new folder.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `user_id` - The ID of the user who owns the folder.
/// * `parent_folder_id` - The ID of the parent folder, if any.
/// * `name` - The name of the folder.
/// * `description` - The description of the folder.
///
/// # Returns
///
/// A `Result` containing the created `Folder`.
pub async fn create_folder(
    state: &AppState,
    user_id: Uuid,
    parent_folder_id: Option<Uuid>,
    name: String,
    description: Option<String>,
) -> Result<Folder> {
    let folder_id = Uuid::new_v4();
    
    folder_repo::create_folder(
        &state.db,
        folder_id,
        user_id,
        parent_folder_id,
        name,
        description,
    )
    .await
}

/// Lists the contents of a folder.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `user_id` - The ID of the user.
/// * `folder_id` - The ID of the folder to list. If `None`, lists the root folder.
///
/// # Returns
///
/// A `Result` containing a tuple of `(Vec<Folder>, Vec<crate::models::file::File>)`.
pub async fn list_folder_contents(
    state: &AppState,
    user_id: Uuid,
    folder_id: Option<Uuid>,
) -> Result<(Vec<Folder>, Vec<crate::models::file::File>)> {
    folder_repo::list_folder_contents(&state.db, folder_id, user_id).await
}

/// Gets a folder with its statistics.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `user_id` - The ID of the user.
/// * `folder_id` - The ID of the folder.
///
/// # Returns
///
/// A `Result` containing an `Option<crate::models::folder::FolderWithStats>`.
pub async fn get_folder_with_stats(
    state: &AppState,
    user_id: Uuid,
    folder_id: Uuid,
) -> Result<Option<crate::models::folder::FolderWithStats>> {
    folder_repo::get_folder_with_stats(&state.db, folder_id, user_id).await
}

/// Deletes a folder and its contents.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `user_id` - The ID of the user.
/// * `folder_id` - The ID of the folder to delete.
///
/// # Returns
///
/// A `Result<()>`.
pub async fn delete_folder(
    state: &AppState,
    user_id: Uuid,
    folder_id: Uuid,
) -> Result<()> {
    folder_repo::delete_folder_recursive(&state.db, folder_id, user_id).await
}
