use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::error::{AppError, Result};

/// The size of the AES-256 key in bytes.
pub const KEY_SIZE: usize = 32;
/// The size of the AES-GCM nonce in bytes.
pub const NONCE_SIZE: usize = 12;

/// A secure key wrapper that ensures the key is zeroized on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureKey([u8; KEY_SIZE]);

impl SecureKey {
    /// Creates a new `SecureKey` from a byte array.
    ///
    /// # Arguments
    ///
    /// * `key` - A 32-byte array representing the AES-256 key.
    pub fn new(key: [u8; KEY_SIZE]) -> Self {
        Self(key)
    }

    /// Returns a reference to the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }

    /// Consumes the `SecureKey` and returns the inner key array.
    pub fn into_inner(self) -> [u8; KEY_SIZE] {
        self.0
    }
}

/// Generates a new random AES-256 key.
///
/// # Returns
///
/// A `SecureKey` containing the generated key.
pub fn generate_key() -> SecureKey {
    let mut key = [0u8; KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    SecureKey::new(key)
}

/// Generates a new random AES-GCM nonce.
///
/// # Returns
///
/// A 12-byte array representing the nonce.
pub fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Encrypts a plaintext using AES-256-GCM.
///
/// # Arguments
///
/// * `key` - The AES-256 key.
/// * `plaintext` - The data to encrypt.
///
/// # Returns
///
/// A tuple containing the ciphertext and the nonce used for encryption.
pub fn encrypt(key: &[u8; KEY_SIZE], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; NONCE_SIZE])> {
    let cipher = Aes256Gcm::new(key.into());

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| AppError::Encryption(format!("Encryption failed: {}", e)))?;

    Ok((ciphertext, nonce_bytes))
}

/// Decrypts a ciphertext using AES-256-GCM.
///
/// # Arguments
///
/// * `key` - The AES-256 key.
/// * `ciphertext` - The data to decrypt.
/// * `nonce` - The nonce used for encryption.
///
/// # Returns
///
/// The decrypted plaintext.
pub fn decrypt(key: &[u8; KEY_SIZE], ciphertext: &[u8], nonce: &[u8; NONCE_SIZE]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from(*nonce);

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| AppError::Encryption(format!("Decryption failed: {}", e)))
}
