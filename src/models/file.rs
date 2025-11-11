use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub user_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub original_filename: String,
    pub total_chunks: Option<i32>,
    pub chunks_metadata: Option<Vec<u8>>,
    pub encrypted_dek: Vec<u8>,
    pub nonce: Vec<u8>,
    pub dek_version: i32,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub checksum_sha256: Option<String>,
    pub upload_status: String,
    pub uploaded_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub access_count: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct FileListItem {
    pub id: Uuid,
    pub original_filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub uploaded_at: DateTime<Utc>,
}
