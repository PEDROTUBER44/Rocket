use chrono::{DateTime, Utc};
use tokio_postgres::Row;
use uuid::Uuid;

/// Represents a user in the system.
#[derive(Clone, Debug)]
pub struct User {
    /// The unique identifier for the user.
    pub id: Uuid,
    /// The user's full name.
    pub name: String,
    /// The user's username.
    pub username: String,
    /// The user's email address.
    pub email: Option<String>,
    /// The user's hashed password.
    pub password: String,
    /// The user's roles.
    pub roles: Vec<String>,
    /// The user's encrypted data encryption key.
    pub encrypted_dek: Option<Vec<u8>>,
    /// The salt used to derive the key that encrypts the data encryption key.
    pub dek_salt: Option<Vec<u8>>,
    /// The version of the key encryption key used to encrypt the data encryption key.
    pub dek_kek_version: i32,
    /// The user's storage quota in bytes.
    pub storage_quota_bytes: i64,
    /// The user's used storage in bytes.
    pub storage_used_bytes: i64,
    /// The timestamp when the user was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp when the user was last updated.
    pub updated_at: DateTime<Utc>,
    /// The timestamp of the user's last password change.
    pub last_password_change: Option<DateTime<Utc>>,
    /// Whether the user is active.
    pub is_active: bool,
}

impl From<&Row> for User {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            username: row.get("username"),
            email: row.get("email"),
            password: row.get("password"),
            roles: row.get("roles"),
            encrypted_dek: row.get("encrypted_dek"),
            dek_salt: row.get("dek_salt"),
            dek_kek_version: row.get("dek_kek_version"),
            storage_quota_bytes: row.get("storage_quota_bytes"),
            storage_used_bytes: row.get("storage_used_bytes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_password_change: row.get("last_password_change"),
            is_active: row.get("is_active"),
        }
    }
}
