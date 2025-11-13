// src/errors.rs

//! Error types for the Nyx protocol.
//!
//! Provides comprehensive error handling for all Nyx operations
//! including DAG processing, consensus, and cryptographic validation.

use std::fmt;

/// Main error type for Nyx operations
#[derive(Debug, Clone, PartialEq)]
pub enum NyxError {
    /// Transaction validation failed
    InvalidTransaction(String),

    /// DAG operation error
    DagError(String),

    /// Storage/database error
    StorageError(String),

    /// Cryptographic operation failed
    CryptoError(String),

    /// Transaction not found
    TransactionNotFound(String),

    /// Double-spend detected
    DoubleSpend(String),

    /// Invalid parent reference
    InvalidParent(String),

    /// Tip selection failed
    TipSelectionError(String),

    /// Network/consensus error
    ConsensusError(String),

    /// Serialization/deserialization error
    SerializationError(String),
}

impl fmt::Display for NyxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NyxError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            NyxError::DagError(msg) => write!(f, "DAG error: {}", msg),
            NyxError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            NyxError::CryptoError(msg) => write!(f, "Cryptographic error: {}", msg),
            NyxError::TransactionNotFound(msg) => write!(f, "Transaction not found: {}", msg),
            NyxError::DoubleSpend(msg) => write!(f, "Double-spend detected: {}", msg),
            NyxError::InvalidParent(msg) => write!(f, "Invalid parent reference: {}", msg),
            NyxError::TipSelectionError(msg) => write!(f, "Tip selection error: {}", msg),
            NyxError::ConsensusError(msg) => write!(f, "Consensus error: {}", msg),
            NyxError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for NyxError {}

/// Result type alias for Nyx operations
pub type Result<T> = std::result::Result<T, NyxError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NyxError::InvalidTransaction("missing inputs".to_string());
        assert_eq!(
            format!("{}", err),
            "Invalid transaction: missing inputs"
        );
    }
}
