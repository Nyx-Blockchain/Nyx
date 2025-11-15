// src/lib.rs

//! # Nyx Node
//!
//! Complete blockchain node implementation that orchestrates all Nyx modules.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           Nyx Node                      │
//! ├─────────────────────────────────────────┤
//! │  RPC Server  │  Mempool  │  Config      │
//! ├──────────────┴───────────┴──────────────┤
//! │           Network Layer                 │
//! │         (nyx-network)                   │
//! ├─────────────────────────────────────────┤
//! │  DAG Processor │ Transaction Builder    │
//! │  (nyx-core)    │  (nyx-wallet)         │
//! ├─────────────────────────────────────────┤
//! │         Cryptography Layer              │
//! │         (nyx-crypto)                    │
//! └─────────────────────────────────────────┘
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod errors;
pub mod config;
pub mod mempool;
pub mod rpc;
pub mod node;

// Re-export commonly used types
pub use crate::errors::{NodeError, Result};
pub use crate::config::NodeConfig;
pub use crate::mempool::Mempool;
pub use crate::rpc::RpcServer;
pub use crate::node::NyxNode;

/// Node version
pub const NODE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_version() {
        assert!(!NODE_VERSION.is_empty());
    }
}
