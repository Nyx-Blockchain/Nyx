// src/lib.rs

//! # Nyx Crypto
//!
//! Cryptographic primitives for the Nyx blockchain protocol.
//!
//! This module provides quantum-resistant and privacy-preserving cryptographic
//! operations including:
//!
//! - **Hashing**: BLAKE3 (primary) and Keccak-256 (secondary)
//! - **Post-Quantum Signatures**: Dilithium-style lattice-based signatures (mock)
//! - **Ring Signatures**: Lattice-based linkable ring signatures for privacy
//! - **Stealth Addresses**: Monero-style ECDH for unlinkability
//! - **Encryption**: AES-256-GCM for confidential data
//!
//! ## Security Properties
//!
//! - **Quantum Resistance**: All signature schemes are designed to resist quantum attacks
//! - **Privacy**: Ring signatures provide sender anonymity with plausible deniability
//! - **Unlinkability**: Stealth addresses prevent transaction graph analysis
//! - **Confidentiality**: Symmetric encryption protects sensitive data
//!
//! ## Example Usage
//!
//! ```rust
//! use nyx_crypto::{hash, keys, stealth};
//!
//! // Hash data
//! let data = b"Hello Nyx";
//! let hash = hash::blake3_hash(data);
//!
//! // Generate post-quantum keypair
//! let keypair = keys::generate_keypair();
//!
//! // Sign and verify
//! let signature = keys::sign(data, &keypair.private_key).unwrap();
//! assert!(keys::verify(data, &signature, &keypair.public_key).unwrap());
//!
//! // Generate stealth address
//! let (stealth_addr, _) = stealth::generate_stealth_address(
//!     &keypair.public_key,
//!     &keypair.public_key,
//!     &[1, 2, 3, 4]
//! ).unwrap();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod errors;
pub mod hash;
pub mod keys;
pub mod ring;
pub mod stealth;
pub mod encryption;

// Re-export commonly used types
pub use crate::errors::{CryptoError, Result};
pub use crate::keys::KeyPair;
pub use crate::ring::RingSignature;

/// Standard hash output size (32 bytes / 256 bits)
pub const HASH_SIZE: usize = 32;

/// Post-quantum public key size (mock Dilithium-3: ~1952 bytes)
/// For development, we use 1952 bytes to simulate CRYSTALS-Dilithium
pub const PQ_PUBLIC_KEY_SIZE: usize = 1952;

/// Post-quantum private key size (mock Dilithium-3: ~4000 bytes)
pub const PQ_PRIVATE_KEY_SIZE: usize = 4000;

/// Post-quantum signature size (mock Dilithium-3: ~3293 bytes)
pub const PQ_SIGNATURE_SIZE: usize = 3293;

/// Ring size for privacy (16 members: 1 true + 15 decoys)
pub const RING_SIZE: usize = 16;

/// Key image size (32 bytes)
pub const KEY_IMAGE_SIZE: usize = 32;

/// AES-256 key size
pub const AES_KEY_SIZE: usize = 32;

/// AES-GCM nonce size
pub const AES_NONCE_SIZE: usize = 12;

/// Stealth address size (Ed25519 point: 32 bytes)
pub const STEALTH_ADDRESS_SIZE: usize = 32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(HASH_SIZE, 32);
        assert_eq!(RING_SIZE, 16);
        assert_eq!(AES_KEY_SIZE, 32);
    }
}
