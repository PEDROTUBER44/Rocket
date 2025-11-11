use crate::crypto::aes::{self, KEY_SIZE, SecureKey};
use crate::error::{AppError, Result};
use rand::rngs::OsRng;
use rand::TryRngCore;
use zeroize::Zeroize;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, SaltString},
    Argon2, ParamsBuilder,
};

/// The memory cost for Argon2 in MB.
const ARGON2_MEMORY_MB: u32 = 19;
/// The number of iterations for Argon2.
const ARGON2_ITERATIONS: u32 = 3;
/// The parallelism factor for Argon2.
const ARGON2_PARALLELISM: u32 = 6;

/// Creates a new user Data Encryption Key (DEK) and encrypts it with a key derived from the user's password.
///
/// # Arguments
///
/// * `user_password` - The user's password.
///
/// # Returns
///
/// A tuple containing the encrypted DEK (with nonce appended) and the salt used for key derivation.
pub fn create_user_dek(user_password: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    tracing::debug!("Creating user DEK with Argon2");
    
    let dek = aes::generate_key();

    let mut salt_bytes = [0u8; 16];
    OsRng.try_fill_bytes(&mut salt_bytes)
        .map_err(|e| AppError::Internal(format!("Failed to generate salt: {}", e)))?;
    
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| AppError::Encryption(format!("Salt encoding error: {}", e)))?;
    
    let password_key = derive_key_from_password(user_password, &salt)?;
    let (encrypted_dek, nonce) = aes::encrypt(password_key.as_bytes(), &dek.as_bytes()[..])?;

    let mut combined = encrypted_dek;
    combined.extend_from_slice(&nonce);

    let salt_bytes_vec = salt.to_string().as_bytes().to_vec();
    
    tracing::debug!("User DEK created successfully");
    
    Ok((combined, salt_bytes_vec))
}

/// Decrypts the user's Data Encryption Key (DEK).
///
/// # Arguments
///
/// * `encrypted_dek_with_nonce` - The encrypted DEK with the nonce appended.
/// * `salt_bytes` - The salt used for key derivation.
/// * `user_password` - The user's password.
///
/// # Returns
///
/// The decrypted DEK as a `SecureKey`.
pub fn decrypt_user_dek(
    encrypted_dek_with_nonce: &[u8],
    salt_bytes: &[u8],
    user_password: &str,
) -> Result<SecureKey> {
    tracing::debug!("Decrypting user DEK");
    
    let salt_str = std::str::from_utf8(salt_bytes)
        .map_err(|_| AppError::Encryption("Invalid salt encoding".to_string()))?;

    let salt = SaltString::new(salt_str)
        .map_err(|e| AppError::Encryption(format!("Salt parse error: {}", e)))?;

    if encrypted_dek_with_nonce.len() < 60 {
        return Err(AppError::Encryption(format!(
            "Invalid encrypted DEK size: expected at least 60, got {}",
            encrypted_dek_with_nonce.len()
        )));
    }

    let password_key = derive_key_from_password(user_password, &salt)?;

    let (ciphertext, nonce_bytes) =
        encrypted_dek_with_nonce.split_at(encrypted_dek_with_nonce.len() - 12);
    let nonce: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid nonce".to_string()))?;

    let mut decrypted = aes::decrypt(password_key.as_bytes(), ciphertext, &nonce)?;

    let dek_array: [u8; KEY_SIZE] = decrypted[..KEY_SIZE]
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid DEK size".to_string()))?;

    decrypted.zeroize();

    tracing::debug!("User DEK decrypted successfully");

    Ok(SecureKey::new(dek_array))
}

/// Changes the user's password and re-encrypts the Data Encryption Key (DEK).
///
/// # Arguments
///
/// * `encrypted_dek_with_nonce` - The currently encrypted DEK with the nonce appended.
/// * `old_salt` - The salt used with the old password.
/// * `old_password` - The user's old password.
/// * `new_password` - The user's new password.
///
/// # Returns
///
/// A tuple containing the newly encrypted DEK (with nonce appended) and the new salt.
pub fn change_user_password_dek(
    encrypted_dek_with_nonce: &[u8],
    old_salt: &[u8],
    old_password: &str,
    new_password: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    tracing::info!("Changing user password and re-encrypting DEK");
    
    let dek = decrypt_user_dek(encrypted_dek_with_nonce, old_salt, old_password)?;

    let mut new_salt_bytes = [0u8; 16];
    OsRng.try_fill_bytes(&mut new_salt_bytes)
        .map_err(|e| AppError::Internal(format!("Failed to generate new salt: {}", e)))?;

    let new_salt = SaltString::encode_b64(&new_salt_bytes)
        .map_err(|e| AppError::Encryption(format!("Salt encoding error: {}", e)))?;

    let new_password_key = derive_key_from_password(new_password, &new_salt)?;

    let (new_encrypted_dek, new_nonce) =
        aes::encrypt(new_password_key.as_bytes(), &dek.as_bytes()[..])?;

    let mut combined = new_encrypted_dek;
    combined.extend_from_slice(&new_nonce);

    let new_salt_bytes_vec = new_salt.to_string().as_bytes().to_vec();
    
    tracing::info!("User password changed and DEK re-encrypted successfully");

    Ok((combined, new_salt_bytes_vec))
}

/// Derives a key from a password using Argon2id.
///
/// # Arguments
///
/// * `password` - The password to derive the key from.
/// * `salt` - The salt to use for key derivation.
///
/// # Returns
///
/// The derived key as a `SecureKey`.
fn derive_key_from_password(password: &str, salt: &SaltString) -> Result<SecureKey> {
    let mut password_bytes = password.as_bytes().to_vec();

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        ParamsBuilder::new()
            .m_cost(ARGON2_MEMORY_MB * 1024)
            .t_cost(ARGON2_ITERATIONS)
            .p_cost(ARGON2_PARALLELISM)
            .build()
            .map_err(|e| AppError::Encryption(format!("Argon2 params error: {}", e)))?,
    );

    let password_hash = argon2
        .hash_password(&password_bytes, salt)
        .map_err(|e| AppError::Encryption(format!("Argon2 hashing error: {}", e)))?;

    let hash_value = password_hash
        .hash
        .ok_or_else(|| AppError::Encryption("No hash generated".to_string()))?;
    
    let hash_bytes = hash_value.as_bytes();

    let key_bytes: [u8; KEY_SIZE] = if hash_bytes.len() >= KEY_SIZE {
        hash_bytes[..KEY_SIZE].try_into().unwrap()
    } else {
        return Err(AppError::Encryption("Hash too short".to_string()));
    };

    password_bytes.zeroize();

    Ok(SecureKey::new(key_bytes))
}
