use crate::error::{AppError, Result};
use rand::TryRngCore;
use rand::rngs::OsRng;
use base64::{Engine as _, engine::general_purpose};

/// The size of the CSRF token in bytes.
const CSRF_TOKEN_SIZE: usize = 32;

/// Generates a new random CSRF token.
///
/// # Returns
///
/// A URL-safe base64-encoded CSRF token.
pub fn generate_csrf_token() -> Result<String> {
    let mut token = [0u8; CSRF_TOKEN_SIZE];
    OsRng
        .try_fill_bytes(&mut token)
        .map_err(|e| AppError::Internal(format!("Failed to generate CSRF token: {}", e)))?;
    
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(token))
}
