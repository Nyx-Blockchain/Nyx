// src/errors.rs

//! Error types for network operations.
//!
//! Provides comprehensive error handling for all networking operations
//! including connection management, message passing, and synchronization.

use std::fmt;
use std::io;

/// Main error type for network operations
#[derive(Debug)]
pub enum NetworkError {
    /// Connection error
    ConnectionError(String),

    /// I/O error
    IoError(io::Error),

    /// Serialization/deserialization error
    SerializationError(String),

    /// Message timeout
    Timeout(String),

    /// Invalid message format
    InvalidMessage(String),

    /// Peer not found
    PeerNotFound(String),

    /// Maximum peers reached
    MaxPeersReached,

    /// Sync error
    SyncError(String),

    /// Gossip error
    GossipError(String),

    /// Address parse error
    AddressError(String),

    /// Channel error (for async communication)
    ChannelError(String),

    /// Node shutdown
    Shutdown,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            NetworkError::IoError(err) => write!(f, "I/O error: {}", err),
            NetworkError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            NetworkError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            NetworkError::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            NetworkError::PeerNotFound(msg) => write!(f, "Peer not found: {}", msg),
            NetworkError::MaxPeersReached => write!(f, "Maximum peers reached"),
            NetworkError::SyncError(msg) => write!(f, "Sync error: {}", msg),
            NetworkError::GossipError(msg) => write!(f, "Gossip error: {}", msg),
            NetworkError::AddressError(msg) => write!(f, "Address error: {}", msg),
            NetworkError::ChannelError(msg) => write!(f, "Channel error: {}", msg),
            NetworkError::Shutdown => write!(f, "Node shutdown"),
        }
    }
}

impl std::error::Error for NetworkError {}

impl From<io::Error> for NetworkError {
    fn from(err: io::Error) -> Self {
        NetworkError::IoError(err)
    }
}

impl From<bincode::Error> for NetworkError {
    fn from(err: bincode::Error) -> Self {
        NetworkError::SerializationError(format!("Bincode error: {}", err))
    }
}

impl From<serde_json::Error> for NetworkError {
    fn from(err: serde_json::Error) -> Self {
        NetworkError::SerializationError(format!("JSON error: {}", err))
    }
}

/// Result type alias for network operations
pub type Result<T> = std::result::Result<T, NetworkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NetworkError::ConnectionError("failed".to_string());
        assert_eq!(format!("{}", err), "Connection error: failed");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let net_err: NetworkError = io_err.into();
        assert!(matches!(net_err, NetworkError::IoError(_)));
    }
}
