// src/errors.rs

//! Error types for node operations.

use std::fmt;

/// Main error type for node operations
#[derive(Debug)]
pub enum NodeError {
    /// Configuration error
    ConfigError(String),

    /// Network error
    NetworkError(String),

    /// Core/DAG error
    CoreError(String),

    /// Wallet error
    WalletError(String),

    /// Mempool error
    MempoolError(String),

    /// RPC error
    RpcError(String),

    /// I/O error
    IoError(std::io::Error),

    /// Initialization error
    InitializationError(String),
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            NodeError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            NodeError::CoreError(msg) => write!(f, "Core error: {}", msg),
            NodeError::WalletError(msg) => write!(f, "Wallet error: {}", msg),
            NodeError::MempoolError(msg) => write!(f, "Mempool error: {}", msg),
            NodeError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            NodeError::IoError(err) => write!(f, "I/O error: {}", err),
            NodeError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
        }
    }
}

impl std::error::Error for NodeError {}

impl From<std::io::Error> for NodeError {
    fn from(err: std::io::Error) -> Self {
        NodeError::IoError(err)
    }
}

impl From<nyx_network::NetworkError> for NodeError {
    fn from(err: nyx_network::NetworkError) -> Self {
        NodeError::NetworkError(format!("{}", err))
    }
}

impl From<nyx_core::NyxError> for NodeError {
    fn from(err: nyx_core::NyxError) -> Self {
        NodeError::CoreError(format!("{}", err))
    }
}

impl From<nyx_wallet::WalletError> for NodeError {
    fn from(err: nyx_wallet::WalletError) -> Self {
        NodeError::WalletError(format!("{}", err))
    }
}

/// Result type alias for node operations
pub type Result<T> = std::result::Result<T, NodeError>;
