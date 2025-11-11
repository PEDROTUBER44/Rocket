use uuid::Uuid;
use crate::{
    crypto::{aes, kek},
    error::{AppError, Result},
    models::file::File,
    repositories::{file as file_repo, user as user_repo},
    state::AppState,
};

/// Saves a file's metadata to the database.
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
    let client = state.db.get().await?;
    let transaction = client.transaction().await?;

    let user_storage_info = user_repo::get_user_storage_info(&state.db, &user_id).await?;
    let available_space = user_storage_info.0 - user_storage_info.1;

    if file_size > available_space {
        return Err(AppError::Validation(format!(
            "Insufficient storage quota. Required: {} bytes, Available: {} bytes",
            file_size, available_space
        )));
    }

    user_repo::update_storage_with_quota_check(&state.db, &user_id, file_size).await?;

    transaction.commit().await?;

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
pub async fn delete_file(state: &AppState, user_id: Uuid, file_id: Uuid) -> Result<()> {
    let file = file_repo::find_by_id(&state.db, file_id, user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    file_repo::soft_delete_file(&state.db, file_id, user_id).await?;

    user_repo::rollback_storage_usage(&state.db, &user_id, file.file_size).await?;

    Ok(())
}

/// Lists the files for a given user.
pub async fn list_files(
    state: &AppState,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<File>> {
    file_repo::list_user_files(&state.db, user_id, limit, offset).await
}

/// Gets a user's storage information.
pub async fn get_user_storage_info(state: &AppState, user_id: Uuid) -> Result<(i64, i64, i64)> {
    let (storage_quota_bytes, storage_used_bytes) = user_repo::get_user_storage_info(&state.db, &user_id).await?;
    let available = storage_quota_bytes - storage_used_bytes;
    Ok((storage_quota_bytes, storage_used_bytes, available))
}
