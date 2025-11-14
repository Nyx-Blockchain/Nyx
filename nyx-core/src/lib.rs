// src/lib.rs

//! # Nyx Core
//!
//! Core implementation of the Nyx blockchain protocol - a quantum-resistant,
//! privacy-preserving dual-layer blockchain architecture.
//!
//! ## Architecture
//!
//! Nyx combines two layers:
//! - **Layer 1 (DAG)**: Handles user transactions asynchronously with instant confirmations
//! - **Layer 2 (PoS)**: Provides finality and canonical ordering through validator consensus
//!
//! ## Features
//!
//! - 🔒 **Quantum Resistance**: Post-quantum cryptography (CRYSTALS-Dilithium, Falcon)
//! - 🕵️ **Monero-Grade Privacy**: Ring signatures, stealth addresses, confidential amounts
//! - ⚡ **High Performance**: 10,000+ TPS theoretical, 3,000+ TPS observed
//! - 💸 **Zero Fees**: Layer 1 transactions are completely free
//! - 🎯 **Fair Economics**: Inverse inflation mechanism prevents wealth concentration
//!
//! ## Example Usage
//!
//! ```rust
//! use nyx_core::{Transaction, TxInput, TxOutput, RingSignature};
//!
//! // Create a new transaction
//! let tx = Transaction::new(
//!     vec![/* inputs */],
//!     vec![/* outputs */],
//!     RingSignature { ring_members: vec![], signature: vec![], key_image: [0u8; 32] },
//!     vec![/* tx_key */],
//!     [0u8; 32], // parent 1
//!     [1u8; 32], // parent 2
//! );
//!
//! // Get transaction ID
//! let tx_id = tx.id();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod types;
///! Transaction structure and validation logic.
pub mod transaction;
pub mod transaction_builder;
pub mod dag;
pub mod tip_selection;
pub mod storage;
pub mod errors;

// Re-export crypto for convenience
pub use nyx_crypto;

// Re-export commonly used types
pub use crate::transaction::{Transaction, TxInput, TxOutput};
pub use crate::transaction_builder::TransactionBuilder;
pub use crate::types::{Hash, Timestamp, hash_bytes_to_hash};
pub use crate::errors::{NyxError, Result};

// Re-export crypto types that are commonly used
pub use nyx_crypto::{
    keys::KeyPair,
    ring::RingSignature,
    hash,
    encryption,
};

// ... rest of your constants ...
/// Protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Default ring size for privacy (16 total: 1 true + 15 decoys)
pub const DEFAULT_RING_SIZE: u8 = 16;

/// DAG confirmation threshold (from whitepaper: Score > 100)
pub const CONFIRMATION_THRESHOLD: f64 = 100.0;

/// PoS snapshot interval in seconds (from whitepaper: every 10 seconds)
pub const SNAPSHOT_INTERVAL_SECS: u64 = 10;

/// Decay factor for confirmation score calculation (from whitepaper: 0.9)
pub const SCORE_DECAY_FACTOR: f64 = 0.9;

/// Alpha parameter for tip selection (from whitepaper: 0.5)
pub const TIP_SELECTION_ALPHA: f64 = 0.5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(PROTOCOL_VERSION, 1);
        assert_eq!(DEFAULT_RING_SIZE, 16);
        assert_eq!(CONFIRMATION_THRESHOLD, 100.0);
    }
}
