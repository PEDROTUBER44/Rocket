use crate::error::{AppError, Result};

/// Validates a username.
///
/// # Arguments
///
/// * `username` - The username to validate.
///
/// # Returns
///
/// A `Result<()>` indicating whether the username is valid.
pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() || username.len() < 3 {
        return Err(AppError::Validation(
            "Username must be at least 3 characters long".to_string(),
        ));
    }

    if username.len() > 255 {
        return Err(AppError::Validation(
            "Username must be at most 255 characters".to_string(),
        ));
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(AppError::Validation(
            "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
        ));
    }

    Ok(())
}

/// Validates a password.
///
/// # Arguments
///
/// * `password` - The password to validate.
///
/// # Returns
///
/// A `Result<()>` indicating whether the password is valid.
pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if password.len() > 128 {
        return Err(AppError::Validation(
            "Password must be at most 128 characters".to_string(),
        ));
    }

    Ok(())
}
