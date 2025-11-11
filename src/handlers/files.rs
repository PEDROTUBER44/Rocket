use axum::{
    extract::{Multipart, Path, Query, State},
    body::{Body, Bytes},
    http::StatusCode,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Extension,
};
use futures::{
    stream::{self, StreamExt},
    TryStreamExt
};
use bincode::{Encode, Decode};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    time::{timeout, Duration}
};
use std::path::PathBuf;
use chrono::Utc;
use crate::{
    error::{AppError, Result},
    models::session::Session,
    state::AppState,
    state::{UPLOAD_BUFFER_SLOTS, DOWNLOAD_BUFFER_SLOTS},
};
use redis::AsyncCommands;

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024 * 1024;
const CHUNK_SIZE: usize = 6 * 1024 * 1024;
const UPLOAD_TIMEOUT: u64 = 300;
const UPLOAD_EXPIRATION_SECS: u64 = 86400;
const DOWNLOAD_EXPIRATION_SECS: u64 = 3600;
const CLEANUP_BATCH_SIZE: usize = 50;

#[derive(Debug, Clone, Encode, Decode)]
struct ChunkInfo {
    index: usize,
    nonce: [u8; 12],
    filename: Vec<u8>,
    size_encrypted: i64,
}

impl ChunkInfo {
    fn new(index: usize, nonce: [u8; 12], filename: String, size_encrypted: i64) -> Self {
        Self {
            index,
            nonce,
            filename: filename.into_bytes(),
            size_encrypted,
        }
    }

    fn get_filename(&self) -> Result<String> {
        String::from_utf8(self.filename.clone())
            .map_err(|_| AppError::Internal("Invalid filename encoding".to_string()))
    }
}

#[derive(Deserialize)]
pub struct ListFilesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode)]
struct UploadMetadata {
    pub upload_session_id: String,
    #[bincode(with_serde)]
    pub user_id: Uuid,
    pub filename: String,
    pub total_size: i64,
    pub total_chunks: usize,
    pub chunks_received_count: usize,
    pub expected_hash: Option<String>,
    pub created_at: i64,
    pub chunks_written_bytes: i64,
    pub chunk_nonces: Vec<[u8; 12]>,
}

#[derive(Deserialize)]
pub struct InitUploadRequest {
    pub filename: String,
    pub file_size: i64,
    pub total_chunks: usize,
    pub expected_hash: Option<String>,
}

#[derive(Deserialize)]
pub struct FinalizeUploadRequest {
    pub upload_session_id: String,
    pub folder_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CancelUploadRequest {
    pub upload_session_id: String,
}

#[derive(Serialize)]
pub struct StorageInfoResponse {
    pub storage_quota_bytes: i64,
    pub storage_used_bytes: i64,
    pub available_bytes: i64,
    pub usage_percentage: f64,
}

async fn cleanup_failed_upload(
    state: &AppState,
    user_id: Uuid,
    upload_session_id: &str,
    metadata: &UploadMetadata,
) -> Result<()> {
    tracing::warn!(
        "🧹 Cleaning up failed upload: {} for user {}",
        upload_session_id,
        user_id
    );

    let upload_dir = PathBuf::from("uploads/files");
    let mut deleted_count = 0;

    // ✅ REMOVER APENAS CHUNKS PARCIALMENTE ENVIADOS
    for chunk_batch_start in (0..metadata.chunks_received_count).step_by(CLEANUP_BATCH_SIZE) {
        let batch_end = (chunk_batch_start + CLEANUP_BATCH_SIZE).min(metadata.chunks_received_count);
        for chunk_idx in chunk_batch_start..batch_end {
            let chunk_filename = format!("{}_{}.encrypted_chunk", upload_session_id, chunk_idx);
            let chunk_path = upload_dir.join(&chunk_filename);
            if tokio::fs::remove_file(&chunk_path).await.is_ok() {
                deleted_count += 1;
            }
        }
    }

    tracing::debug!("✅ Removed {} chunk files from disk", deleted_count);

    // ✅ NÃO REVERTER QUOTA - NUNCA FOI DEBITADA
    let mut redis = state.redis.clone();
    let redis_key = format!("upload:{}:{}", user_id, upload_session_id);
    let _ = redis.del::<_, ()>(&redis_key).await.ok();

    let lock_key = format!("user_uploading:{}", user_id);
    let _ = redis.del::<_, ()>(&lock_key).await.ok();

    tracing::info!(
        "✅ Upload cleanup completed for session: {}",
        upload_session_id
    );

    Ok(())
}

pub async fn init_upload(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    axum::Json(req): axum::Json<InitUploadRequest>,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    tracing::info!(
        "🔑 Init upload - user: {}, file: {}, size: {} bytes, chunks: {}",
        user_id,
        req.filename,
        req.file_size,
        req.total_chunks
    );

    let mut redis = state.redis.clone();

    let lock_key = format!("user_uploading:{}", user_id);
    match redis.get::<_, Option<String>>(&lock_key).await {
        Ok(Some(_)) => {
            return Err(AppError::Validation(
                "Já há um upload ativo para este usuário. Aguarde a conclusão.".to_string(),
            ));
        }
        Err(e) => return Err(AppError::Redis(e)),
        _ => {}
    }

    if req.file_size <= 0 {
        return Err(AppError::Validation("File size must be positive".into()));
    }

    if req.file_size > MAX_FILE_SIZE as i64 {
        return Err(AppError::Validation(format!(
            "File size exceeds maximum allowed ({}GB)",
            MAX_FILE_SIZE / (1024 * 1024 * 1024)
        )));
    }

    if req.total_chunks == 0 {
        return Err(AppError::Validation(
            "total_chunks must be greater than 0".into(),
        ));
    }

    // ✅ APENAS VALIDAR ESPAÇO, NÃO DEBITAR AINDA
    let mut tx = state.db.begin().await?;
    let user_quota = sqlx::query!(
        r#"
        SELECT storage_quota_bytes, storage_used_bytes
        FROM users
        WHERE id = $1
        FOR UPDATE
        "#,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| AppError::Unauthorized)?;

    let available_space = user_quota.storage_quota_bytes - user_quota.storage_used_bytes;
    if req.file_size > available_space {
        tx.rollback().await?;
        return Err(AppError::Validation(format!(
            "Insufficient storage quota. Required: {} bytes, Available: {} bytes",
            req.file_size, available_space
        )));
    }

    tx.commit().await?;

    tracing::info!(
        "✅ Quota check passed: {} bytes available for user {}",
        available_space,
        user_id
    );

    let upload_session_id = Uuid::new_v4();
    let metadata = UploadMetadata {
        upload_session_id: upload_session_id.to_string(),
        user_id,
        filename: req.filename.clone(),
        total_size: req.file_size,
        total_chunks: req.total_chunks,
        chunks_received_count: 0,
        expected_hash: req.expected_hash.clone(),
        created_at: Utc::now().timestamp(),
        chunks_written_bytes: 0,
        chunk_nonces: vec![[0u8; 12]; req.total_chunks],
    };

    let redis_key = format!("upload:{}:{}", user_id, upload_session_id);
    let config = bincode::config::standard();
    let metadata_bytes = bincode::encode_to_vec(&metadata, config)
        .map_err(|e| AppError::Internal(format!("Bincode encode failed: {}", e)))?;

    let _: () = redis
        .set_ex(&redis_key, &metadata_bytes, UPLOAD_EXPIRATION_SECS)
        .await
        .map_err(|e| AppError::Redis(e))?;

    let _: () = redis
        .set_ex(&lock_key, "locked", UPLOAD_EXPIRATION_SECS)
        .await
        .map_err(|e| AppError::Redis(e))?;

    tracing::info!(
        "✅ Upload session created: {} (expires in 24h)",
        upload_session_id
    );

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "upload_session_id": upload_session_id.to_string(),
        "message": "Upload session initialized. Ready to receive chunks.",
        "quota_reserved": 0,
        "available_space_before": available_space,
        "chunks_to_send": req.total_chunks,
        "chunk_size_bytes": CHUNK_SIZE,
        "upload_timeout_seconds": UPLOAD_TIMEOUT
    }))
    .unwrap();

    Ok((StatusCode::OK, response).into_response())
}

pub async fn upload_chunk(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    tracing::info!("📤 Upload chunk (ENCRYPTED) from user: {}", user_id);

    let mut redis = state.redis.clone();

    let _permit = state.upload_limiter.acquire().await;

    let available = state.upload_limiter.available_permits();
    let total_slots = UPLOAD_BUFFER_SLOTS;
    let concurrent_uploads = total_slots.saturating_sub(available);
    let buffer_mb = std::cmp::max(2usize, 2048 / (concurrent_uploads.max(1) + 1));
    let dynamic_buffer = buffer_mb * 1024 * 1024;

    tracing::debug!(
        "📊 Dynamic upload buffer: {} MB (concurrent: {}, available: {})",
        buffer_mb,
        concurrent_uploads,
        available
    );

    let mut upload_session_id: Option<String> = None;
    let mut chunk_index: Option<usize> = None;
    let mut chunk_data: Option<Vec<u8>> = None;

    let timeout_duration = Duration::from_secs(UPLOAD_TIMEOUT);

    loop {
        match timeout(timeout_duration, multipart.next_field()).await {
            Ok(Ok(Some(field))) => {
                let field_name = field.name().unwrap_or("").to_string();
                match field_name.as_str() {
                    "upload_session_id" => {
                        upload_session_id = Some(
                            field
                                .text()
                                .await
                                .map_err(|e| AppError::Multipart(format!("upload_session_id: {}", e)))?,
                        );
                    }
                    "chunk_index" => {
                        let text = field
                            .text()
                            .await
                            .map_err(|e| AppError::Multipart(format!("chunk_index: {}", e)))?;
                        chunk_index = Some(text.parse().map_err(|_| {
                            AppError::Validation("Invalid chunk_index".into())
                        })?);
                    }
                    "chunk" => {
                        chunk_data = Some(
                            field
                                .bytes()
                                .await
                                .map_err(|e| AppError::Multipart(format!("chunk data: {}", e)))?
                                .to_vec(),
                        );
                    }
                    _ => {}
                }
            }
            Ok(Ok(None)) => break,
            Ok(Err(e)) => {
                return Err(AppError::Multipart(format!("Parse error: {}", e)));
            }
            Err(_) => return Err(AppError::Multipart("Upload timeout exceeded".into())),
        }
    }

    let session_id = upload_session_id
        .ok_or(AppError::Validation("Missing upload_session_id".into()))?;
    let chunk_idx = chunk_index
        .ok_or(AppError::Validation("Missing chunk_index".into()))?;
    let data = chunk_data.ok_or(AppError::Validation("Missing chunk data".into()))?;
    let _session_uuid = Uuid::parse_str(&session_id)
        .map_err(|_| AppError::Validation("Invalid session ID format".into()))?;

    tracing::debug!(
        "📋 Parsed multipart - session: {}, chunk_idx: {}, data_size: {} bytes",
        session_id,
        chunk_idx,
        data.len()
    );

    let redis_key = format!("upload:{}:{}", user_id, session_id);
    let config = bincode::config::standard();
    let metadata_bytes: Vec<u8> = redis
        .get(&redis_key)
        .await
        .map_err(|e| AppError::Redis(e))?;

    let (mut metadata, _): (UploadMetadata, usize) =
        bincode::decode_from_slice(&metadata_bytes, config)
            .map_err(|e| AppError::Internal(format!("Bincode decode failed: {}", e)))?;

    if chunk_idx >= metadata.total_chunks {
        return Err(AppError::Validation(format!(
            "Invalid chunk index: expected 0-{}, got {}",
            metadata.total_chunks - 1,
            chunk_idx
        )));
    }

    tracing::debug!(
        "✅ Upload metadata loaded - total_chunks: {}, received: {}",
        metadata.total_chunks,
        metadata.chunks_received_count
    );

    tracing::debug!("🔐 Using session DEK to encrypt chunk...");

    if session.dek.len() != 32 {
        tracing::error!(
            "❌ Invalid DEK in session: {} bytes (expected 32)",
            session.dek.len()
        );
        return Err(AppError::Encryption(
            "Invalid DEK in session".to_string(),
        ));
    }

    let dek_array: [u8; 32] = session
        .dek
        .as_slice()
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid DEK in session".to_string()))?;

    tracing::debug!(
        "🔐 Encrypting chunk {} ({} bytes) with DEK...",
        chunk_idx,
        data.len()
    );
    let (chunk_encrypted, actual_nonce) =
        crate::crypto::aes::encrypt(&dek_array, &data).map_err(|e| {
            tracing::error!(
                "❌ Failed to encrypt chunk {}: {}",
                chunk_idx,
                e
            );
            e
        })?;

    tracing::debug!(
        "✅ Chunk {} encrypted: {} bytes → {} bytes (nonce: {:?})",
        chunk_idx,
        data.len(),
        chunk_encrypted.len(),
        &actual_nonce[..4]
    );

    tracing::debug!("💾 Saving encrypted chunk {} to disk...", chunk_idx);

    let upload_dir = PathBuf::from("uploads/files");
    tokio::fs::create_dir_all(&upload_dir).await.ok();

    let chunk_filename = format!("{}_{}.encrypted_chunk", session_id, chunk_idx);
    let chunk_path = upload_dir.join(&chunk_filename);

    let file = tokio::fs::File::create(&chunk_path).await.map_err(|e| {
        tracing::error!(
            "❌ Failed to create chunk file {}: {}",
            chunk_filename,
            e
        );
        AppError::Io(e)
    })?;

    let mut writer = BufWriter::with_capacity(dynamic_buffer, file);

    writer.write_all(&chunk_encrypted).await.map_err(|e| {
        tracing::error!(
            "❌ Failed to write chunk {}: {}",
            chunk_filename,
            e
        );
        AppError::Io(e)
    })?;

    writer.flush().await.map_err(|e| {
        tracing::error!(
            "❌ Failed to flush chunk {}: {}",
            chunk_filename,
            e
        );
        AppError::Io(e)
    })?;

    drop(writer);

    tracing::debug!(
        "✅ Chunk {} saved to disk: {}",
        chunk_idx,
        chunk_filename
    );

    tracing::debug!("📝 Updating metadata in Redis...");

    metadata.chunk_nonces[chunk_idx] = actual_nonce;
    metadata.chunks_received_count += 1;
    metadata.chunks_written_bytes += chunk_encrypted.len() as i64;

    let updated_bytes = bincode::encode_to_vec(&metadata, config).map_err(|e| {
        tracing::error!(
            "❌ Bincode encode failed: {}",
            e
        );
        AppError::Internal(format!("Bincode encode failed: {}", e))
    })?;

    let _: () = redis
        .set_ex(&redis_key, &updated_bytes, UPLOAD_EXPIRATION_SECS)
        .await
        .map_err(|e| {
            tracing::error!(
                "❌ Failed to update metadata in Redis: {}",
                e
            );
            AppError::Redis(e)
        })?;

    tracing::debug!(
        "✅ Metadata updated: {}/{}",
        metadata.chunks_received_count,
        metadata.total_chunks
    );

    let progress_percentage =
        (metadata.chunks_received_count as f64 / metadata.total_chunks as f64) * 100.0;

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "chunk_index": chunk_idx,
        "chunk_size_plaintext": data.len(),
        "chunk_size_encrypted": chunk_encrypted.len(),
        "chunks_received": metadata.chunks_received_count,
        "total_chunks": metadata.total_chunks,
        "progress_percentage": format!("{:.2}", progress_percentage)
    }))
    .map_err(|e| {
        tracing::error!(
            "❌ Failed to serialize response: {}",
            e
        );
        AppError::Internal(format!("Response serialization failed: {}", e))
    })?;

    tracing::info!(
        "✅ Upload chunk {} complete: {}/{} ({:.2}%)",
        chunk_idx,
        metadata.chunks_received_count,
        metadata.total_chunks,
        progress_percentage
    );

    Ok((StatusCode::OK, response).into_response())
}

pub async fn finalize_upload(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    axum::Json(req): axum::Json<FinalizeUploadRequest>,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    tracing::info!(
        "🔒 Finalizing upload {} for user {}",
        req.upload_session_id,
        user_id
    );

    let mut redis = state.redis.clone();
    let redis_key = format!("upload:{}:{}", user_id, req.upload_session_id);
    let config = bincode::config::standard();

    let metadata_bytes: Vec<u8> = redis
        .get(&redis_key)
        .await
        .map_err(|e| {
            tracing::error!("Redis GET error: {}", e);
            AppError::Redis(e)
        })?;

    let (metadata, _): (UploadMetadata, usize) =
        bincode::decode_from_slice(&metadata_bytes, config).map_err(|e| {
            tracing::error!("Bincode decode failed: {}", e);
            AppError::Internal(format!("Bincode decode failed: {}", e))
        })?;

    if metadata.chunks_received_count != metadata.total_chunks {
        tracing::error!(
            "❌ Incomplete upload: received {}/{} chunks",
            metadata.chunks_received_count,
            metadata.total_chunks
        );
        // ✅ LIMPAR UPLOAD INCOMPLETO (não foi debitado ainda)
        cleanup_failed_upload(&state, user_id, &req.upload_session_id, &metadata).await?;
        return Err(AppError::Validation(format!(
            "Incomplete upload: received {} chunks, expected {}",
            metadata.chunks_received_count, metadata.total_chunks
        )));
    }

    // ✅ AGORA DEBITAR A QUOTA - upload completo e validado
    let mut tx = state.db.begin().await.map_err(|e| {
        tracing::error!("Database transaction begin failed: {}", e);
        AppError::Database(e)
    })?;

    let user_quota = sqlx::query!(
        r#"
        SELECT storage_quota_bytes, storage_used_bytes
        FROM users
        WHERE id = $1
        FOR UPDATE
        "#,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch user quota: {}", e);
        AppError::Database(e)
    })?;

    let available_space = user_quota.storage_quota_bytes - user_quota.storage_used_bytes;
    if metadata.total_size > available_space {
        tx.rollback().await?;
        cleanup_failed_upload(&state, user_id, &req.upload_session_id, &metadata).await?;
        return Err(AppError::Validation(format!(
            "Insufficient storage quota at finalization. Required: {} bytes, Available: {} bytes",
            metadata.total_size, available_space
        )));
    }

    // ✅ DEBITAR A QUOTA AGORA
    sqlx::query!(
        r#"
        UPDATE users
        SET storage_used_bytes = storage_used_bytes + $1
        WHERE id = $2
        "#,
        metadata.total_size,
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update storage quota: {}", e);
        AppError::Database(e)
    })?;

    let file_id = Uuid::new_v4();
    let mut chunks_data: Vec<ChunkInfo> = Vec::new();

    for (idx, nonce) in metadata.chunk_nonces.iter().enumerate() {
        chunks_data.push(ChunkInfo::new(
            idx,
            *nonce,
            format!("{}_{}.encrypted_chunk", req.upload_session_id, idx),
            CHUNK_SIZE as i64,
        ));
    }

    let chunks_bytes = bincode::encode_to_vec(&chunks_data, bincode::config::standard())
        .map_err(|e| AppError::Internal(format!("Bincode encode failed: {}", e)))?;

    tracing::info!("✅ Chunks metadata encoded: {} bytes", chunks_bytes.len());

    let user_dek = session.dek.clone();

    if user_dek.is_empty() {
        tracing::error!("User DEK not available in session for user: {}", user_id);
        tx.rollback().await?;
        cleanup_failed_upload(&state, user_id, &req.upload_session_id, &metadata).await?;
        return Err(AppError::Encryption(
            "User DEK not available in session".to_string(),
        ));
    }

    let (kek_version, kek_bytes) = crate::crypto::kek::get_active_kek(
        &state.db,
        state.config.master_key.as_ref(),
        &state.kek_cache,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get active KEK: {}", e);
        e
    })?;

    let kek_array: [u8; 32] = kek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| {
            tracing::error!("Invalid KEK size");
            AppError::Encryption("Invalid KEK size".to_string())
        })?;

    let (encrypted_dek, dek_nonce) = crate::crypto::aes::encrypt(&kek_array, &user_dek)
        .map_err(|e| {
            tracing::error!("Failed to encrypt user DEK: {}", e);
            e
        })?;

    tracing::debug!("DEK encrypted successfully with KEK version {}", kek_version);

    sqlx::query!(
        r#"
        INSERT INTO files (
            id,
            user_id,
            folder_id,
            original_filename,
            total_chunks,
            chunks_metadata,
            encrypted_dek,
            nonce,
            dek_version,
            file_size,
            mime_type,
            checksum_sha256,
            upload_status,
            uploaded_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'completed', NOW()
        )
        "#,
        file_id,
        user_id,
        req.folder_id,
        metadata.filename,
        metadata.total_chunks as i32,
        chunks_bytes,
        encrypted_dek,
        &dek_nonce,
        kek_version,
        metadata.total_size,
        "application/octet-stream",
        metadata.expected_hash
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert file record: {}", e);
        AppError::Database(e)
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("Transaction commit failed: {}", e);
        AppError::Database(e)
    })?;

    tracing::info!(
        "⚡ Upload finalized successfully: File {} with {} chunks (quota debited)",
        file_id,
        metadata.total_chunks
    );

    let _ = redis.del::<_, ()>(&redis_key).await.ok();
    let lock_key = format!("user_uploading:{}", user_id);
    let _ = redis.del::<_, ()>(&lock_key).await.ok();

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "message": "Upload finalized successfully",
        "file_id": file_id.to_string(),
        "filename": metadata.filename,
        "total_chunks": metadata.total_chunks,
        "size_bytes": metadata.total_size,
        "ready_for_download": true,
        "can_stream": true
    }))
    .map_err(|e| {
        tracing::error!("Failed to serialize response: {}", e);
        AppError::Internal(format!("Response serialization failed: {}", e))
    })?;

    Ok((StatusCode::OK, response).into_response())
}

pub async fn cancel_upload(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    axum::Json(req): axum::Json<CancelUploadRequest>,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    tracing::info!(
        "🚫 Canceling upload {} for user {}",
        req.upload_session_id,
        user_id
    );

    let mut redis = state.redis.clone();
    let redis_key = format!("upload:{}:{}", user_id, req.upload_session_id);
    let config = bincode::config::standard();

    let metadata_bytes: Vec<u8> = redis
        .get(&redis_key)
        .await
        .map_err(|e| AppError::Redis(e))?;

    let (metadata, _): (UploadMetadata, usize) =
        bincode::decode_from_slice(&metadata_bytes, config)
            .map_err(|e| AppError::Internal(format!("Bincode decode failed: {}", e)))?;

    // ✅ LIMPAR ARQUIVO INCOMPLETO (quota nunca foi debitada)
    cleanup_failed_upload(&state, user_id, &req.upload_session_id, &metadata).await?;

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "message": "Upload canceled successfully",
        "quota_released": 0
    }))
    .unwrap();

    Ok((StatusCode::OK, response).into_response())
}

pub async fn list_files(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(params): Query<ListFilesQuery>,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    tracing::debug!("📂 Listing files - limit: {}, offset: {}", params.limit, params.offset);

    let files = sqlx::query!(
        r#"
        SELECT id, original_filename, file_size, mime_type, uploaded_at, access_count
        FROM files
        WHERE user_id = $1 AND is_deleted = false
        ORDER BY uploaded_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        params.limit,
        params.offset
    )
    .fetch_all(&state.db)
    .await?;

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "files": files.iter().map(|f| sonic_rs::json!({
            "id": f.id.to_string(),
            "filename": f.original_filename,
            "size_bytes": f.file_size,
            "mime_type": f.mime_type,
            "uploaded_at": f.uploaded_at.to_rfc3339(),
            "access_count": f.access_count
        })).collect::<Vec<_>>(),
        "count": files.len()
    }))
    .unwrap();

    Ok((StatusCode::OK, response).into_response())
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '"' | '\\' => '_',
            '\n' | '\r' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

pub async fn download_file(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    tracing::info!("📥 Download file {} (STREAMING MODE)", file_id);

    let mut redis = state.redis.clone();

    let lock_key = format!("user_downloading:{}", user_id);
    let exists_count: i64 = redis.exists(&lock_key).await.map_err(|e| AppError::Redis(e))?;
    if exists_count > 0 {
        return Err(AppError::Validation(
            "Já há um download ativo para este usuário. Aguarde a conclusão.".to_string(),
        ));
    }

    let _: () = redis
        .set_ex(&lock_key, "locked", DOWNLOAD_EXPIRATION_SECS)
        .await
        .map_err(|e| AppError::Redis(e))?;

    let _permit = state.download_limiter.acquire().await;

    let available = state.download_limiter.available_permits();
    let total_slots = DOWNLOAD_BUFFER_SLOTS;
    let concurrent_downloads = total_slots.saturating_sub(available);
    let buffer_chunks = std::cmp::max(1usize, total_slots / (concurrent_downloads.max(1) + 1));

    tracing::info!("⏳ Download buffer: {} chunks (concurrent: {}, available: {})", buffer_chunks, concurrent_downloads, available);

    let file = sqlx::query!(
        r#"
        SELECT id, original_filename, chunks_metadata,
               encrypted_dek, nonce, dek_version
        FROM files
        WHERE id = $1 AND user_id = $2 AND is_deleted = false
        "#,
        file_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let chunks_metadata_raw = file
        .chunks_metadata
        .ok_or(AppError::Internal("Missing chunks_metadata".into()))?;

    let (chunks_data, _): (Vec<ChunkInfo>, usize) =
        bincode::decode_from_slice(&chunks_metadata_raw, bincode::config::standard())
            .map_err(|e| AppError::Internal(format!("Bincode decode failed: {}", e)))?;

    let chunks_count = chunks_data.len();

    tracing::info!("✅ Decoded {} chunks from metadata", chunks_count);

    let kek_version = file.dek_version;
    let kek_bytes = crate::crypto::kek::get_kek_by_version(
        &state.db,
        kek_version,
        state.config.master_key.as_ref(),
        &state.kek_cache,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get KEK: {}", e);
        e
    })?;

    let kek_array: [u8; 32] = kek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid KEK size".into()))?;

    let dek_nonce: [u8; 12] = file
        .nonce
        .as_slice()
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid nonce size".into()))?;

    let dek_encrypted = file.encrypted_dek;
    let dek = crate::crypto::aes::decrypt(&kek_array, &dek_encrypted, &dek_nonce)
        .map_err(|e| {
            tracing::error!("Failed to decrypt DEK: {}", e);
            e
        })?;

    let dek_array: [u8; 32] = dek
        .as_slice()
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid DEK size".into()))?;

    tracing::info!("🔓 DEK decrypted successfully");

    let chunk_stream = stream::iter(chunks_data)
        .map(move |chunk_info| {
            let dek = dek_array;
            async move {
                let chunk_filename = chunk_info.get_filename()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

                let chunk_path = PathBuf::from("uploads/files").join(&chunk_filename);

                let chunk_encrypted = tokio::fs::read(&chunk_path).await.map_err(|e| {
                    tracing::error!("Failed to read chunk {}: {}", chunk_filename, e);
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                })?;

                let chunk_plaintext = crate::crypto::aes::decrypt(
                    &dek,
                    &chunk_encrypted,
                    &chunk_info.nonce,
                ).map_err(|e| {
                    tracing::error!("Failed to decrypt chunk {}: {}", chunk_info.index, e);
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                })?;

                tracing::debug!(
                    "✅ Chunk {} decrypted: {} bytes",
                    chunk_info.index,
                    chunk_plaintext.len()
                );

                Ok::<Bytes, std::io::Error>(Bytes::from(chunk_plaintext))
            }
        })
        .buffered(buffer_chunks);

    let body = Body::from_stream(chunk_stream);

    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );

    let safe_filename = sanitize_filename(&file.original_filename);
    let disposition = format!(r#"attachment; filename="{}""#, safe_filename)
        .parse()
        .unwrap_or_else(|_| "attachment".parse().unwrap());

    response_headers.insert(axum::http::header::CONTENT_DISPOSITION, disposition);

    tracing::info!(
        "✅ Download stream ready - {} chunks, buffer={} (semaphore limit: max 2GB total)",
        chunks_count,
        buffer_chunks
    );

    Ok((response_headers, body).into_response())
}

pub async fn delete_file(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    let file = sqlx::query!(
        r#"
        SELECT id, file_size, is_deleted
        FROM files
        WHERE id = $1 AND user_id = $2
        "#,
        file_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if file.is_deleted {
        return Err(AppError::Validation("File already deleted".into()));
    }

    let mut tx = state.db.begin().await?;
    sqlx::query!(
        r#"
        UPDATE files
        SET is_deleted = true, deleted_at = NOW()
        WHERE id = $1 AND user_id = $2
        "#,
        file_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        UPDATE users
        SET storage_used_bytes = GREATEST(0, storage_used_bytes - $1)
        WHERE id = $2
        "#,
        file.file_size,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        "🗑️ File deleted: {} ({} bytes quota released for user {})",
        file_id,
        file.file_size,
        user_id
    );

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "message": "File deleted successfully",
        "quota_released": file.file_size
    }))
    .unwrap();

    Ok((StatusCode::OK, response).into_response())
}

pub async fn storage_info(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    let user = sqlx::query!(
        r#"
        SELECT storage_quota_bytes, storage_used_bytes
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await?;

    let available_bytes = user.storage_quota_bytes - user.storage_used_bytes;
    let usage_percentage =
        (user.storage_used_bytes as f64 / user.storage_quota_bytes as f64) * 100.0;

    let response = sonic_rs::to_string(&sonic_rs::json!(StorageInfoResponse {
        storage_quota_bytes: user.storage_quota_bytes,
        storage_used_bytes: user.storage_used_bytes,
        available_bytes,
        usage_percentage,
    }))
    .unwrap();

    Ok((StatusCode::OK, response).into_response())
}

pub async fn recalculate_user_quota(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Result<impl IntoResponse> {
    let user_id = session.user_id;

    tracing::warn!("🔄 Recalculating storage quota for user: {}", user_id);

    let mut tx = state.db.begin().await?;
    let total_size: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(file_size), 0)
        FROM files
        WHERE user_id = $1 AND is_deleted = false
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let actual_size = total_size.unwrap_or(0);

    sqlx::query!(
        r#"
        UPDATE users
        SET storage_used_bytes = $1
        WHERE id = $2
        "#,
        actual_size,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        "✅ Storage quota recalculated: {} bytes for user {}",
        actual_size,
        user_id
    );

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "message": "Storage quota recalculated",
        "actual_storage_used": actual_size
    }))
    .unwrap();

    Ok((StatusCode::OK, response).into_response())
}

pub async fn cleanup_expired_uploads(mut state: AppState) -> Result<()> {
    tracing::info!("🧹 Checking for expired uploads...");

    let current_timestamp = Utc::now().timestamp();
    let mut cursor = 0u64;
    let mut cleaned_count = 0;

    loop {
        let mut conn = state.redis.clone();
        let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg("upload:*")
            .arg("COUNT")
            .arg(100)
            .query_async(&mut conn)
            .await
            .unwrap_or((0, vec![]));

        for key in keys {
            let mut redis_conn = state.redis.clone();
            if let Ok(metadata_bytes) = redis_conn
                .get::<_, Vec<u8>>(&key)
                .await
            {
                let config = bincode::config::standard();
                if let Ok((metadata, _)) =
                    bincode::decode_from_slice::<UploadMetadata, _>(&metadata_bytes, config)
                {
                    if current_timestamp - metadata.created_at > 86400 {
                        tracing::warn!("⏰ Expired upload found: {}", key);

                        let upload_dir = PathBuf::from("uploads/files");
                        for chunk_idx in 0..metadata.total_chunks {
                            let chunk_filename =
                                format!("{}_{}.encrypted_chunk", metadata.upload_session_id, chunk_idx);
                            let chunk_path = upload_dir.join(&chunk_filename);
                            let _ = tokio::fs::remove_file(&chunk_path).await;
                        }

                        let mut del_redis = state.redis.clone();
                        let _: () = del_redis.del::<_, ()>(&key).await.unwrap_or(());

                        let _ = sqlx::query!(
                            r#"
                            UPDATE users
                            SET storage_used_bytes = GREATEST(0, storage_used_bytes - $1)
                            WHERE id = $2
                            "#,
                            metadata.total_size,
                            metadata.user_id
                        )
                        .execute(&state.db)
                        .await;

                        let lock_key = format!("user_uploading:{}", metadata.user_id);
                        let mut lock_redis = state.redis.clone();
                        let _ = lock_redis.del::<_, ()>(&lock_key).await.ok();

                        cleaned_count += 1;
                    }
                }
            }
        }

        cursor = new_cursor;
        if cursor == 0 {
            break;
        }
    }

    tracing::info!(
        "✅ Cleanup check completed - {} expired uploads removed",
        cleaned_count
    );

    Ok(())
}
