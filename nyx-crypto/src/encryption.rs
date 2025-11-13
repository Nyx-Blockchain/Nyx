// src/encryption.rs

//! Symmetric encryption for confidential data.
//!
//! Provides AES-256-GCM authenticated encryption for protecting
//! sensitive transaction data like memos and amounts.

use crate::errors::{CryptoError, Result};
use crate::{AES_KEY_SIZE, AES_NONCE_SIZE};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;

/// Encrypts data using AES-256-GCM
///
/// Provides authenticated encryption with associated data (AEAD).
/// The nonce is automatically generated and prepended to ciphertext.
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `key` - 32-byte encryption key
///
/// # Returns
/// Encrypted data (nonce || ciphertext || tag)
///
/// # Example
/// ```
/// use nyx_crypto::encryption::{encrypt, generate_key};
///
/// let key = generate_key();
/// let plaintext = b"secret message";
///
/// let ciphertext = encrypt(plaintext, &key).unwrap();
/// assert!(ciphertext.len() > plaintext.len());
/// ```
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != AES_KEY_SIZE {
        return Err(CryptoError::EncryptionError(
            format!("Invalid key size: expected {}, got {}", AES_KEY_SIZE, key.len())
        ));
    }

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptionError(format!("Failed to create cipher: {}", e)))?;

    // Generate random nonce
    let mut rng = rand::thread_rng();
    let nonce_bytes: [u8; AES_NONCE_SIZE] = rng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionError(format!("Encryption failed: {}", e)))?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(AES_NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypts data encrypted with AES-256-GCM
///
/// # Arguments
/// * `ciphertext` - Encrypted data (nonce || ciphertext || tag)
/// * `key` - 32-byte decryption key (same as encryption key)
///
/// # Returns
/// Original plaintext data
///
/// # Example
/// ```
/// use nyx_crypto::encryption::{encrypt, decrypt, generate_key};
///
/// let key = generate_key();
/// let plaintext = b"secret message";
///
/// let ciphertext = encrypt(plaintext, &key).unwrap();
/// let decrypted = decrypt(&ciphertext, &key).unwrap();
///
/// assert_eq!(plaintext, &decrypted[..]);
/// ```
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != AES_KEY_SIZE {
        return Err(CryptoError::DecryptionError(
            format!("Invalid key size: expected {}, got {}", AES_KEY_SIZE, key.len())
        ));
    }

    if ciphertext.len() < AES_NONCE_SIZE {
        return Err(CryptoError::DecryptionError(
            "Ciphertext too short".to_string()
        ));
    }

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::DecryptionError(format!("Failed to create cipher: {}", e)))?;

    // Extract nonce
    let nonce = Nonce::from_slice(&ciphertext[..AES_NONCE_SIZE]);

    // Decrypt
    let plaintext = cipher.decrypt(nonce, &ciphertext[AES_NONCE_SIZE..])
        .map_err(|e| CryptoError::DecryptionError(format!("Decryption failed: {}", e)))?;

    Ok(plaintext)
}

/// Generates a random 256-bit encryption key
///
/// # Returns
/// 32-byte random key suitable for AES-256-GCM
///
/// # Example
/// ```
/// use nyx_crypto::encryption::generate_key;
///
/// let key = generate_key();
/// assert_eq!(key.len(), 32);
/// ```
pub fn generate_key() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let key: [u8; AES_KEY_SIZE] = rng.gen();
    key.to_vec()
}

/// Encrypts data with a deterministic nonce (for testing only!)
///
/// **WARNING**: Never use this in production! Nonce reuse breaks security.
/// This is only for deterministic testing.
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `key` - 32-byte encryption key
/// * `nonce` - 12-byte nonce (must be unique per message)
///
/// # Returns
/// Encrypted data (nonce || ciphertext || tag)
#[cfg(test)]
pub fn encrypt_with_nonce(plaintext: &[u8], key: &[u8], nonce_bytes: &[u8; AES_NONCE_SIZE]) -> Result<Vec<u8>> {
    if key.len() != AES_KEY_SIZE {
        return Err(CryptoError::EncryptionError(
            format!("Invalid key size: expected {}, got {}", AES_KEY_SIZE, key.len())
        ));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptionError(format!("Failed to create cipher: {}", e)))?;

    let nonce = Nonce::from_slice(nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionError(format!("Encryption failed: {}", e)))?;

    let mut result = Vec::with_capacity(AES_NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Encrypts with associated data for additional authentication
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `key` - 32-byte encryption key
/// * `associated_data` - Additional data to authenticate (not encrypted)
///
/// # Returns
/// Encrypted data (nonce || ciphertext || tag)
pub fn encrypt_with_aad(plaintext: &[u8], key: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
    if key.len() != AES_KEY_SIZE {
        return Err(CryptoError::EncryptionError(
            format!("Invalid key size: expected {}, got {}", AES_KEY_SIZE, key.len())
        ));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptionError(format!("Failed to create cipher: {}", e)))?;

    let mut rng = rand::thread_rng();
    let nonce_bytes: [u8; AES_NONCE_SIZE] = rng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Create payload with associated data
    use aes_gcm::aead::Payload;
    let payload = Payload {
        msg: plaintext,
        aad: associated_data,
    };

    let ciphertext = cipher.encrypt(nonce, payload)
        .map_err(|e| CryptoError::EncryptionError(format!("Encryption failed: {}", e)))?;

    let mut result = Vec::with_capacity(AES_NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypts with associated data verification
///
/// # Arguments
/// * `ciphertext` - Encrypted data (nonce || ciphertext || tag)
/// * `key` - 32-byte decryption key
/// * `associated_data` - Associated data that must match encryption
///
/// # Returns
/// Original plaintext data
pub fn decrypt_with_aad(ciphertext: &[u8], key: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
    if key.len() != AES_KEY_SIZE {
        return Err(CryptoError::DecryptionError(
            format!("Invalid key size: expected {}, got {}", AES_KEY_SIZE, key.len())
        ));
    }

    if ciphertext.len() < AES_NONCE_SIZE {
        return Err(CryptoError::DecryptionError(
            "Ciphertext too short".to_string()
        ));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::DecryptionError(format!("Failed to create cipher: {}", e)))?;

    let nonce = Nonce::from_slice(&ciphertext[..AES_NONCE_SIZE]);

    use aes_gcm::aead::Payload;
    let payload = Payload {
        msg: &ciphertext[AES_NONCE_SIZE..],
        aad: associated_data,
    };

    let plaintext = cipher.decrypt(nonce, payload)
        .map_err(|e| CryptoError::DecryptionError(format!("Decryption failed: {}", e)))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let plaintext = b"Hello, Nyx!";

        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_encrypt_different_each_time() {
        let key = generate_key();
        let plaintext = b"same message";

        let ct1 = encrypt(plaintext, &key).unwrap();
        let ct2 = encrypt(plaintext, &key).unwrap();

        // Ciphertexts should be different (different nonces)
        assert_ne!(ct1, ct2);

        // But both should decrypt to same plaintext
        assert_eq!(decrypt(&ct1, &key).unwrap(), plaintext);
        assert_eq!(decrypt(&ct2, &key).unwrap(), plaintext);
    }

    #[test]
    fn test_encrypt_with_deterministic_nonce() {
        let key = generate_key();
        let plaintext = b"deterministic test";
        let nonce = [42u8; AES_NONCE_SIZE];

        let ct1 = encrypt_with_nonce(plaintext, &key, &nonce).unwrap();
        let ct2 = encrypt_with_nonce(plaintext, &key, &nonce).unwrap();

        // Should be identical with same nonce
        assert_eq!(ct1, ct2);
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = generate_key();
        let key2 = generate_key();
        let plaintext = b"secret";

        let ciphertext = encrypt(plaintext, &key1).unwrap();
        let result = decrypt(&ciphertext, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_ciphertext() {
        let key = generate_key();
        let plaintext = b"secret";

        let mut ciphertext = encrypt(plaintext, &key).unwrap();

        // Corrupt the ciphertext
        if let Some(byte) = ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        let result = decrypt(&ciphertext, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_data() {
        let key = generate_key();
        let plaintext = b"";

        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_encrypt_large_data() {
        let key = generate_key();
        let plaintext = vec![42u8; 10000];

        let ciphertext = encrypt(&plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_key_size() {
        let invalid_key = vec![0u8; 16]; // Wrong size
        let plaintext = b"test";

        let result = encrypt(plaintext, &invalid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_ciphertext_too_short() {
        let key = generate_key();
        let short_data = vec![0u8; 5]; // Less than nonce size

        let result = decrypt(&short_data, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_key() {
        let key1 = generate_key();
        let key2 = generate_key();

        assert_eq!(key1.len(), AES_KEY_SIZE);
        assert_eq!(key2.len(), AES_KEY_SIZE);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt_with_aad() {
        let key = generate_key();
        let plaintext = b"secret message";
        let aad = b"transaction_id_12345";

        let ciphertext = encrypt_with_aad(plaintext, &key, aad).unwrap();
        let decrypted = decrypt_with_aad(&ciphertext, &key, aad).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_decrypt_with_wrong_aad() {
        let key = generate_key();
        let plaintext = b"secret";
        let aad1 = b"correct_aad";
        let aad2 = b"wrong_aad";

        let ciphertext = encrypt_with_aad(plaintext, &key, aad1).unwrap();
        let result = decrypt_with_aad(&ciphertext, &key, aad2);

        assert!(result.is_err());
    }

    #[test]
    fn test_ciphertext_longer_than_plaintext() {
        let key = generate_key();
        let plaintext = b"short";

        let ciphertext = encrypt(plaintext, &key).unwrap();

        // Ciphertext includes nonce (12 bytes) + tag (16 bytes)
        assert!(ciphertext.len() > plaintext.len());
    }
}
