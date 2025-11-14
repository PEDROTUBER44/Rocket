use uuid::Uuid;
use crate::{
    error::Result,
    models::folder::{Folder, FolderWithStats},
    repositories::folder as folder_repo,
    state::AppState,
};

/// Creates a new folder.
pub async fn create_folder(
    state: &AppState,
    user_id: Uuid,
    parent_folder_id: Option<Uuid>,
    name: String,
    description: Option<String>,
) -> Result<Folder> {
    let folder_id = Uuid::new_v4();
    
    let mut client = state.db.get().await?;
    folder_repo::create_folder(
        &mut client,
        folder_id,
        user_id,
        parent_folder_id,
        name,
        description,
        &state.stmt_cache,
    )
    .await
}

/// Lists the contents of a folder.
pub async fn list_folder_contents(
    state: &AppState,
    user_id: Uuid,
    folder_id: Option<Uuid>,
) -> Result<(Vec<Folder>, Vec<crate::models::file::File>)> {
    let mut client = state.db.get().await?;
    folder_repo::list_folder_contents(&mut client, folder_id, user_id, &state.stmt_cache).await
}

/// Gets a folder with its statistics.
pub async fn get_folder_with_stats(
    state: &AppState,
    user_id: Uuid,
    folder_id: Uuid,
) -> Result<Option<FolderWithStats>> {
    let mut client = state.db.get().await?;
    folder_repo::get_folder_with_stats(&mut client, folder_id, user_id, &state.stmt_cache).await
}

/// Deletes a folder and its contents.
pub async fn delete_folder(
    state: &AppState,
    user_id: Uuid,
    folder_id: Uuid,
) -> Result<()> {
    let mut client = state.db.get().await?;
    folder_repo::delete_folder_recursive(&mut client, folder_id, user_id, &state.stmt_cache).await
}
