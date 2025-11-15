// src/errors.rs

//! Error types for wallet operations.

use std::fmt;
use std::io;

/// Main error type for wallet operations
#[derive(Debug)]
pub enum WalletError {
    /// Account not found
    AccountNotFound(String),

    /// Insufficient balance
    InsufficientBalance {
        /// Required amount
        required: u64,
        /// Available balance
        available: u64,
    },

    /// Invalid address format
    InvalidAddress(String),

    /// Keystore error
    KeystoreError(String),

    /// Encryption/decryption error
    CryptoError(String),

    /// Transaction building error
    TransactionBuildError(String),

    /// Serialization error
    SerializationError(String),

    /// I/O error
    IoError(io::Error),

    /// Invalid password
    InvalidPassword,

    /// File not found
    FileNotFound(String),

    /// Account already exists
    AccountExists(String),
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::AccountNotFound(msg) => write!(f, "Account not found: {}", msg),
            WalletError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: required {}, available {}", required, available)
            }
            WalletError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            WalletError::KeystoreError(msg) => write!(f, "Keystore error: {}", msg),
            WalletError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            WalletError::TransactionBuildError(msg) => write!(f, "Transaction build error: {}", msg),
            WalletError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            WalletError::IoError(err) => write!(f, "I/O error: {}", err),
            WalletError::InvalidPassword => write!(f, "Invalid password"),
            WalletError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            WalletError::AccountExists(msg) => write!(f, "Account already exists: {}", msg),
        }
    }
}

impl std::error::Error for WalletError {}

impl From<io::Error> for WalletError {
    fn from(err: io::Error) -> Self {
        WalletError::IoError(err)
    }
}

impl From<serde_json::Error> for WalletError {
    fn from(err: serde_json::Error) -> Self {
        WalletError::SerializationError(format!("JSON error: {}", err))
    }
}

impl From<bincode::Error> for WalletError {
    fn from(err: bincode::Error) -> Self {
        WalletError::SerializationError(format!("Bincode error: {}", err))
    }
}

impl From<nyx_crypto::CryptoError> for WalletError {
    fn from(err: nyx_crypto::CryptoError) -> Self {
        WalletError::CryptoError(format!("{}", err))
    }
}

/// Result type alias for wallet operations
pub type Result<T> = std::result::Result<T, WalletError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = WalletError::AccountNotFound("test".to_string());
        assert_eq!(format!("{}", err), "Account not found: test");
    }

    #[test]
    fn test_insufficient_balance() {
        let err = WalletError::InsufficientBalance {
            required: 100,
            available: 50,
        };
        assert!(format!("{}", err).contains("100"));
        assert!(format!("{}", err).contains("50"));
    }
}
