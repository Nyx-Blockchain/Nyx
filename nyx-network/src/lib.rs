// src/lib.rs

//! # Nyx Network
//!
//! P2P networking layer for the Nyx blockchain protocol.
//!
//! This module provides:
//! - **Peer Discovery**: Find and connect to network peers
//! - **Message Propagation**: Gossip protocol for transaction broadcasting
//! - **DAG Synchronization**: Sync transaction DAG with other nodes
//! - **Connection Management**: Maintain healthy peer connections
//!
//! ## Architecture
//!
//! The network layer uses a gossip-based approach where:
//! 1. Nodes maintain connections to multiple peers
//! 2. New transactions are broadcast to all connected peers
//! 3. Peers deduplicate messages and forward to their neighbors
//! 4. DAG sync ensures nodes have consistent state
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use nyx_network::{Node, NodeConfig};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = NodeConfig {
//!         listen_addr: "127.0.0.1:8000".parse().unwrap(),
//!         max_peers: 50,
//!         ..Default::default()
//!     };
//!
//!     let node = Node::new(config).await.unwrap();
//!     node.run().await.unwrap();
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod errors;
pub mod message;
pub mod peer;
pub mod gossip;
pub mod sync;
pub mod node;

// Re-export commonly used types
pub use crate::errors::{NetworkError, Result};
pub use crate::message::{Message, MessageType};
pub use crate::peer::{Peer, PeerId};
pub use crate::gossip::GossipEngine;
pub use crate::sync::SyncManager;
pub use crate::node::{Node, NodeConfig};

/// Default P2P network port
pub const DEFAULT_PORT: u16 = 8000;

/// Maximum number of peer connections
pub const MAX_PEERS: usize = 50;

/// Minimum number of peer connections to maintain
pub const MIN_PEERS: usize = 8;

/// Heartbeat interval in seconds
pub const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Connection timeout in seconds
pub const CONNECTION_TIMEOUT_SECS: u64 = 10;

/// Message size limit (10 MB)
pub const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of messages to cache for deduplication
pub const MAX_SEEN_MESSAGES: usize = 10000;

/// Sync interval in seconds
pub const SYNC_INTERVAL_SECS: u64 = 60;

/// Maximum transactions per sync response
pub const MAX_SYNC_BATCH_SIZE: usize = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_PORT, 8000);
        assert!(MAX_PEERS > MIN_PEERS);
        assert!(HEARTBEAT_INTERVAL_SECS > 0);
    }
}
