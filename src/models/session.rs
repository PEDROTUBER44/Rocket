use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user session.
///
/// ⚠️ IMPORTANT: The `dek` field stores the DEK ENCRYPTED with the KEK.
/// Format: [ciphertext || nonce] where nonce is 12 bytes at the end.
/// NEVER use `dek` directly to encrypt/decrypt data!
/// Always decrypt with the KEK before use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// The ID of the user this session belongs to.
    pub user_id: Uuid,
    /// ⚠️ Encrypted DEK (encrypted_dek || 12-byte nonce).
    /// MUST be decrypted with KEK before any use.
    pub dek: Vec<u8>,
    /// The timestamp when the session was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp when the session expires.
    pub expires_at: DateTime<Utc>,
}
