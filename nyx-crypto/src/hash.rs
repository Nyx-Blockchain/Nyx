// src/hash.rs

//! Cryptographic hash functions for the Nyx protocol.
//!
//! Provides BLAKE3 (primary) and Keccak-256 (secondary) hashing.
//! BLAKE3 is used for general-purpose hashing due to its speed and
//! quantum resistance. Keccak is provided for Ethereum compatibility.

use crate::errors::{CryptoError, Result};
use crate::HASH_SIZE;
use sha3::{Digest, Keccak256};

/// Computes BLAKE3 hash of input data
///
/// BLAKE3 is the primary hash function in Nyx due to its:
/// - Extreme speed (faster than SHA-256)
/// - Quantum resistance
/// - Security properties (no known weaknesses)
///
/// # Arguments
/// * `data` - Input data to hash
///
/// # Returns
/// 32-byte hash digest
///
/// # Example
/// ```
/// use nyx_crypto::hash::blake3_hash;
///
/// let data = b"Hello Nyx";
/// let hash = blake3_hash(data);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn blake3_hash(data: &[u8]) -> [u8; HASH_SIZE] {
    let hash = blake3::hash(data);
    let mut out = [0u8; HASH_SIZE];
    out.copy_from_slice(&hash.as_bytes()[..HASH_SIZE]);
    out
}

/// Computes Keccak-256 hash of input data
///
/// Keccak-256 (SHA-3) is provided for:
/// - Ethereum compatibility
/// - Cross-chain bridge support
/// - Alternative hash function diversity
///
/// # Arguments
/// * `data` - Input data to hash
///
/// # Returns
/// 32-byte hash digest
///
/// # Example
/// ```
/// use nyx_crypto::hash::keccak_hash;
///
/// let data = b"Hello Ethereum";
/// let hash = keccak_hash(data);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn keccak_hash(data: &[u8]) -> [u8; HASH_SIZE] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; HASH_SIZE];
    out.copy_from_slice(&result[..HASH_SIZE]);
    out
}

/// Converts a hash to hexadecimal string
///
/// # Arguments
/// * `hash` - 32-byte hash array
///
/// # Returns
/// 64-character hexadecimal string
pub fn hash_to_hex(hash: &[u8; HASH_SIZE]) -> String {
    hex::encode(hash)
}

/// Parses hexadecimal string to hash
///
/// # Arguments
/// * `hex_str` - 64-character hexadecimal string
///
/// # Returns
/// 32-byte hash array or error
pub fn hex_to_hash(hex_str: &str) -> Result<[u8; HASH_SIZE]> {
    if hex_str.len() != HASH_SIZE * 2 {
        return Err(CryptoError::HashError(
            format!("Invalid hex length: expected {}, got {}", HASH_SIZE * 2, hex_str.len())
        ));
    }

    let bytes = hex::decode(hex_str)
        .map_err(|e| CryptoError::HashError(format!("Hex decode failed: {}", e)))?;

    let mut hash = [0u8; HASH_SIZE];
    hash.copy_from_slice(&bytes);
    Ok(hash)
}

/// Hashes multiple data chunks together
///
/// Efficiently hashes concatenated data without allocation
///
/// # Arguments
/// * `chunks` - Slice of data chunks to hash
///
/// # Returns
/// 32-byte hash of concatenated chunks
pub fn hash_chunks(chunks: &[&[u8]]) -> [u8; HASH_SIZE] {
    let mut hasher = blake3::Hasher::new();
    for chunk in chunks {
        hasher.update(chunk);
    }
    let hash = hasher.finalize();
    let mut out = [0u8; HASH_SIZE];
    out.copy_from_slice(&hash.as_bytes()[..HASH_SIZE]);
    out
}

/// Double hash (hash of hash) for additional security
///
/// Used in some protocols to prevent length extension attacks
pub fn double_blake3(data: &[u8]) -> [u8; HASH_SIZE] {
    let first = blake3_hash(data);
    blake3_hash(&first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_deterministic() {
        let data = b"test data";
        let hash1 = blake3_hash(data);
        let hash2 = blake3_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_blake3_different_input() {
        let hash1 = blake3_hash(b"data1");
        let hash2 = blake3_hash(b"data2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_keccak_deterministic() {
        let data = b"test data";
        let hash1 = keccak_hash(data);
        let hash2 = keccak_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_blake3_vs_keccak() {
        let data = b"same input";
        let blake3 = blake3_hash(data);
        let keccak = keccak_hash(data);
        // Should produce different hashes
        assert_ne!(blake3, keccak);
    }

    #[test]
    fn test_hash_to_hex_and_back() {
        let original = blake3_hash(b"test");
        let hex = hash_to_hex(&original);
        assert_eq!(hex.len(), 64);

        let restored = hex_to_hash(&hex).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_hex_to_hash_invalid_length() {
        let result = hex_to_hash("abcd");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_to_hash_invalid_chars() {
        let invalid = "g".repeat(64);
        let result = hex_to_hash(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_chunks() {
        let chunk1 = b"hello";
        let chunk2 = b"world";

        let hash1 = hash_chunks(&[chunk1, chunk2]);
        let hash2 = blake3_hash(b"helloworld");

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_double_blake3() {
        let data = b"test";
        let single = blake3_hash(data);
        let double = double_blake3(data);

        assert_ne!(single, double);
        assert_eq!(double, blake3_hash(&single));
    }

    #[test]
    fn test_empty_input() {
        let hash = blake3_hash(&[]);
        assert_eq!(hash.len(), 32);
    }
}
