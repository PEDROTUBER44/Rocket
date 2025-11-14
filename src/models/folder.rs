use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

/// Represents a folder in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    /// The unique identifier for the folder.
    pub id: Uuid,
    /// The ID of the user who owns the folder.
    pub user_id: Uuid,
    /// The ID of the parent folder, if any.
    pub parent_folder_id: Option<Uuid>,
    /// The name of the folder.
    pub name: String,
    /// The description of the folder.
    pub description: Option<String>,
    /// Whether the folder is deleted.
    pub is_deleted: bool,
    /// The timestamp when the folder was deleted.
    pub deleted_at: Option<DateTime<Utc>>,
    /// The timestamp when the folder was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp when the folder was last updated.
    pub updated_at: DateTime<Utc>,
}

impl From<&Row> for Folder {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            parent_folder_id: row.get("parent_folder_id"),
            name: row.get("name"),
            description: row.get("description"),
            is_deleted: row.get("is_deleted"),
            deleted_at: row.get("deleted_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Represents a folder with its statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderWithStats {
    /// The unique identifier for the folder.
    pub id: Uuid,
    /// The name of the folder.
    pub name: String,
    /// The description of the folder.
    pub description: Option<String>,
    /// The timestamp when the folder was created.
    pub created_at: DateTime<Utc>,
    /// The number of files in the folder.
    pub file_count: i64,
    /// The number of subfolders in the folder.
    pub subfolder_count: i64,
    /// The total size of the files in the folder in bytes.
    pub total_size: i64,
}
