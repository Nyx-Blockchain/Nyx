// src/errors.rs

//! Error types for cryptographic operations.
//!
//! Provides comprehensive error handling for all cryptographic primitives
//! in the Nyx protocol.

use std::fmt;

/// Main error type for cryptographic operations
#[derive(Debug, Clone, PartialEq)]
pub enum CryptoError {
    /// Invalid key format or size
    InvalidKey(String),

    /// Hash operation failed
    HashError(String),

    /// Signature generation or verification failed
    SignatureError(String),

    /// Ring signature error
    RingSignatureError(String),

    /// Invalid key image
    InvalidKeyImage(String),

    /// Stealth address generation failed
    StealthAddressError(String),

    /// Encryption failed
    EncryptionError(String),

    /// Decryption failed
    DecryptionError(String),

    /// Invalid input data
    InvalidInput(String),

    /// Serialization/deserialization error
    SerializationError(String),

    /// Random number generation failed
    RandomError(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            CryptoError::HashError(msg) => write!(f, "Hash error: {}", msg),
            CryptoError::SignatureError(msg) => write!(f, "Signature error: {}", msg),
            CryptoError::RingSignatureError(msg) => write!(f, "Ring signature error: {}", msg),
            CryptoError::InvalidKeyImage(msg) => write!(f, "Invalid key image: {}", msg),
            CryptoError::StealthAddressError(msg) => write!(f, "Stealth address error: {}", msg),
            CryptoError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            CryptoError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            CryptoError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CryptoError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CryptoError::RandomError(msg) => write!(f, "Random generation error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Result type alias for cryptographic operations
pub type Result<T> = std::result::Result<T, CryptoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CryptoError::InvalidKey("key too short".to_string());
        assert_eq!(format!("{}", err), "Invalid key: key too short");
    }

    #[test]
    fn test_error_clone() {
        let err1 = CryptoError::SignatureError("failed".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
