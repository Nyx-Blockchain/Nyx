// src/types.rs

//! Core type definitions using nyx-crypto

pub use nyx_crypto::hash::blake3_hash as hash_bytes_to_hash;
pub use nyx_crypto::HASH_SIZE;

/// 32-byte hash type
pub type Hash = [u8; 32];

/// Unix timestamp in seconds
pub type Timestamp = u64;

/// Converts hash to hex using nyx-crypto
pub fn hash_to_hex(hash: &Hash) -> String {
    nyx_crypto::hash::hash_to_hex(hash)
}

/// Parses hex to hash using nyx-crypto
pub fn hex_to_hash(hex_str: &str) -> Option<Hash> {
    nyx_crypto::hash::hex_to_hash(hex_str).ok()
}
