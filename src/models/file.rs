use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

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

impl From<&Row> for File {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            folder_id: row.get("folder_id"),
            original_filename: row.get("original_filename"),
            total_chunks: row.get("total_chunks"),
            chunks_metadata: row.get("chunks_metadata"),
            encrypted_dek: row.get("encrypted_dek"),
            nonce: row.get("nonce"),
            dek_version: row.get("dek_version"),
            file_size: row.get("file_size"),
            mime_type: row.get("mime_type"),
            checksum_sha256: row.get("checksum_sha256"),
            upload_status: row.get("upload_status"),
            uploaded_at: row.get("uploaded_at"),
            is_deleted: row.get("is_deleted"),
            deleted_at: row.get("deleted_at"),
            access_count: row.get("access_count"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FileListItem {
    pub id: Uuid,
    pub original_filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub uploaded_at: DateTime<Utc>,
}
