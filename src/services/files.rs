use uuid::Uuid;
use crate::{
    error::Result,
    models::file::File,
    repositories::file as file_repo,
    state::AppState,
};

/// Lists the files for a given user.
pub async fn list_files(
    state: &AppState,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<File>> {
    let client = state.db.get().await?;
    file_repo::list_user_files(&client, user_id, limit, offset, &state.stmt_cache).await
}

/// Gets a user's storage information.
pub async fn get_user_storage_info(state: &AppState, user_id: Uuid) -> Result<(i64, i64, i64)> {
    let client = state.db.get().await?;
    let (storage_quota_bytes, storage_used_bytes) =
        crate::repositories::user::get_user_storage_info(&client, &user_id, &state.stmt_cache).await?;

    let available = storage_quota_bytes - storage_used_bytes;
    Ok((storage_quota_bytes, storage_used_bytes, available))
}
