use argon2::Argon2;
use rand::{rngs::OsRng, RngCore};
use crate::error::{AppError, Result};

/// Derives a key from a password and salt using Argon2.
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| AppError::Encryption(format!("Argon2 key derivation error: {}", e)))?;
    Ok(key)
}

/// Creates a new user data encryption key (DEK).
pub fn create_user_dek(password: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut dek = [0u8; 32];
    OsRng.fill_bytes(&mut dek);

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let (encrypted_dek, nonce) = crate::crypto::aes::encrypt(&key, &dek)?;

    let mut result = Vec::with_capacity(encrypted_dek.len() + nonce.len());
    result.extend_from_slice(&encrypted_dek);
    result.extend_from_slice(&nonce);

    Ok((result, salt.to_vec()))
}

/// Changes a user's password and re-encrypts the DEK.
pub fn change_user_password_dek(
    encrypted_dek_with_nonce: &[u8],
    salt: &[u8],
    old_password: &str,
    new_password: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let old_key = derive_key(old_password, salt)?;
    let (encrypted_dek, nonce) = encrypted_dek_with_nonce.split_at(encrypted_dek_with_nonce.len() - 12);
    let nonce_arr: [u8; 12] = nonce.try_into().unwrap();

    let dek = crate::crypto::aes::decrypt(&old_key, encrypted_dek, &nonce_arr)?;

    let mut new_salt = [0u8; 16];
    OsRng.fill_bytes(&mut new_salt);

    let new_key = derive_key(new_password, &new_salt)?;
    let (new_encrypted_dek, new_nonce) = crate::crypto::aes::encrypt(&new_key, &dek)?;

    let mut result = Vec::with_capacity(new_encrypted_dek.len() + new_nonce.len());
    result.extend_from_slice(&new_encrypted_dek);
    result.extend_from_slice(&new_nonce);

    Ok((result, new_salt.to_vec()))
}

/// Decrypts a user's data encryption key (DEK).
pub fn decrypt_user_dek(
    encrypted_dek_with_nonce: &[u8],
    salt: &[u8],
    password: &str,
) -> Result<zeroize::Zeroizing<String>> {
    let key = derive_key(password, salt)?;
    let (encrypted_dek, nonce) = encrypted_dek_with_nonce.split_at(encrypted_dek_with_nonce.len() - 12);
    let nonce_arr: [u8; 12] = nonce.try_into().unwrap();

    let dek = crate::crypto::aes::decrypt(&key, encrypted_dek, &nonce_arr)?;

    Ok(zeroize::Zeroizing::new(hex::encode(dek)))
}
