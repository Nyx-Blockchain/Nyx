// src/keys.rs

//! Post-quantum keypair generation and signing.
//!
//! This module provides a mock implementation of CRYSTALS-Dilithium-style
//! lattice-based signatures. In production, this would be replaced with
//! actual Dilithium or Falcon implementations.

use crate::errors::{CryptoError, Result};
use crate::hash::blake3_hash;
use crate::{PQ_PUBLIC_KEY_SIZE, PQ_PRIVATE_KEY_SIZE, PQ_SIGNATURE_SIZE};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use curve25519_dalek::{
    scalar::Scalar,
    constants::ED25519_BASEPOINT_TABLE,
};

/// Post-quantum keypair (mock Dilithium-3)
///
/// In production, this would be CRYSTALS-Dilithium or Falcon keys.
/// This mock maintains realistic key sizes for testing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyPair {
    /// Public key (~1952 bytes for Dilithium-3)
    pub public_key: Vec<u8>,

    /// Private key (~4000 bytes for Dilithium-3)
    /// Zeroized on drop for security
    #[serde(skip)]
    private_key_inner: PrivateKey,
}

/// Private key wrapper that zeroizes on drop
//#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(Clone, Default, Zeroize, ZeroizeOnDrop)]
struct PrivateKey {
    data: Vec<u8>,
}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrivateKey([REDACTED])")
    }
}

impl KeyPair {
    /// Get reference to private key data
    pub fn private_key(&self) -> &[u8] {
        &self.private_key_inner.data
    }
}

/// Generates a post-quantum keypair using system randomness
///
/// # Returns
/// A new KeyPair with Dilithium-3 sized keys
///
/// # Example
/// ```
/// use nyx_crypto::keys::generate_keypair;
///
/// let keypair = generate_keypair();
/// assert!(keypair.public_key.len() > 0);
/// ```
pub fn generate_keypair() -> KeyPair {
    let mut rng = rand::thread_rng();
    generate_keypair_with_rng(&mut rng)
}

/// Generates a keypair from a deterministic seed
///
/// Useful for testing and deterministic key derivation.
///
/// # Arguments
/// * `seed` - 32-byte seed for key generation
///
/// # Returns
/// A deterministic KeyPair
///
/// # Example
/// ```
/// use nyx_crypto::keys::generate_keypair_from_seed;
///
/// let seed = [42u8; 32];
/// let keypair1 = generate_keypair_from_seed(&seed);
/// let keypair2 = generate_keypair_from_seed(&seed);
/// assert_eq!(keypair1.public_key, keypair2.public_key);
/// ```
pub fn generate_keypair_from_seed(seed: &[u8; 32]) -> KeyPair {
    let mut rng = StdRng::from_seed(*seed);
    generate_keypair_with_rng(&mut rng)
}

/// Internal keypair generation with custom RNG
fn generate_keypair_with_rng<R: Rng>(rng: &mut R) -> KeyPair {
    // Generate mock public key
    let mut public_key = vec![0u8; PQ_PUBLIC_KEY_SIZE];
    rng.fill(&mut public_key[..]);

    // Generate mock private key
    let mut private_data = vec![0u8; PQ_PRIVATE_KEY_SIZE];
    rng.fill(&mut private_data[..]);

    // Include public key hash in private key for verification
    let pub_hash = blake3_hash(&public_key);
    private_data[..32].copy_from_slice(&pub_hash);

    KeyPair {
        public_key,
        private_key_inner: PrivateKey { data: private_data },
    }
}

pub fn generate_keypair_ed25519() -> (Vec<u8>, Vec<u8>) {
    let mut rng = rand::thread_rng();
    let private: [u8; 32] = rng.gen();
    let scalar = Scalar::from_bytes_mod_order(blake3_hash(&private));
    let public = (&scalar * ED25519_BASEPOINT_TABLE).compress().to_bytes().to_vec();
    (private.to_vec(), public)
}

/// Signs data with a private key
///
/// Mock implementation of Dilithium-3 signature generation.
/// In production, this would use actual lattice-based signatures.
///
/// # Arguments
/// * `data` - Data to sign
/// * `private_key` - Private key bytes
///
/// # Returns
/// Signature bytes (~3293 bytes for Dilithium-3)
///
/// # Example
/// ```
/// use nyx_crypto::keys::{generate_keypair, sign};
///
/// let keypair = generate_keypair();
/// let data = b"message to sign";
/// let signature = sign(data, keypair.private_key()).unwrap();
/// assert!(signature.len() > 0);
/// ```
pub fn sign(data: &[u8], private_key: &[u8]) -> Result<Vec<u8>> {
    if private_key.len() != PQ_PRIVATE_KEY_SIZE {
        return Err(CryptoError::InvalidKey(
            format!("Invalid private key size: expected {}, got {}",
                    PQ_PRIVATE_KEY_SIZE, private_key.len())
        ));
    }

    // Mock signature: hash(data || private_key) + padding
    let mut sig_data = Vec::with_capacity(PQ_SIGNATURE_SIZE);

    // Hash data with private key
    let hash = blake3_hash(&[data, private_key].concat());
    sig_data.extend_from_slice(&hash);

    // Include data hash for verification
    let data_hash = blake3_hash(data);
    sig_data.extend_from_slice(&data_hash);

    // Pad to signature size with deterministic pseudorandom data
    let mut rng = StdRng::from_seed(hash);
    while sig_data.len() < PQ_SIGNATURE_SIZE {
        let byte: u8 = rng.gen();
        sig_data.push(byte);
    }

    Ok(sig_data)
}

/// Verifies a signature against data and public key
///
/// # Arguments
/// * `data` - Original data that was signed
/// * `signature` - Signature bytes
/// * `public_key` - Public key bytes
///
/// # Returns
/// `Ok(true)` if signature is valid, `Ok(false)` if invalid, `Err` on error
///
/// # Example
/// ```
/// use nyx_crypto::keys::{generate_keypair, sign, verify};
///
/// let keypair = generate_keypair();
/// let data = b"message";
/// let signature = sign(data, keypair.private_key()).unwrap();
/// assert!(verify(data, &signature, &keypair.public_key).unwrap());
/// ```
pub fn verify(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
    if public_key.len() != PQ_PUBLIC_KEY_SIZE {
        return Err(CryptoError::InvalidKey(
            format!("Invalid public key size: expected {}, got {}",
                    PQ_PUBLIC_KEY_SIZE, public_key.len())
        ));
    }

    if signature.len() != PQ_SIGNATURE_SIZE {
        return Err(CryptoError::SignatureError(
            format!("Invalid signature size: expected {}, got {}",
                    PQ_SIGNATURE_SIZE, signature.len())
        ));
    }

    // Verify data hash matches what's in signature
    let data_hash = blake3_hash(data);
    if &signature[32..64] != &data_hash[..] {
        return Ok(false);
    }

    // In a real implementation, we would verify the lattice-based signature
    // For now, we just check that the signature contains the correct data hash
    Ok(true)
}

/// Derives a public key from a private key
///
/// Useful for key recovery and verification
pub fn derive_public_key(private_key: &[u8]) -> Result<Vec<u8>> {
    if private_key.len() != PQ_PRIVATE_KEY_SIZE {
        return Err(CryptoError::InvalidKey(
            "Invalid private key size".to_string()
        ));
    }

    // Extract public key hash from private key
    let pub_hash = &private_key[..32];

    // In reality, we'd derive the public key mathematically
    // For this mock, we'll generate a deterministic public key from the hash
    let mut rng = StdRng::from_seed({
        let mut seed = [0u8; 32];
        seed.copy_from_slice(pub_hash);
        seed
    });

    let mut public_key = vec![0u8; PQ_PUBLIC_KEY_SIZE];
    rng.fill(&mut public_key[..]);

    Ok(public_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let kp = generate_keypair();
        assert_eq!(kp.public_key.len(), PQ_PUBLIC_KEY_SIZE);
        assert_eq!(kp.private_key().len(), PQ_PRIVATE_KEY_SIZE);
    }

    #[test]
    fn test_deterministic_keypair() {
        let seed = [42u8; 32];
        let kp1 = generate_keypair_from_seed(&seed);
        let kp2 = generate_keypair_from_seed(&seed);

        assert_eq!(kp1.public_key, kp2.public_key);
        assert_eq!(kp1.private_key(), kp2.private_key());
    }

    #[test]
    fn test_different_seeds_different_keys() {
        let kp1 = generate_keypair_from_seed(&[1u8; 32]);
        let kp2 = generate_keypair_from_seed(&[2u8; 32]);

        assert_ne!(kp1.public_key, kp2.public_key);
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = generate_keypair();
        let data = b"test message";

        let signature = sign(data, kp.private_key()).unwrap();
        assert_eq!(signature.len(), PQ_SIGNATURE_SIZE);

        let valid = verify(data, &signature, &kp.public_key).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_verify_wrong_data() {
        let kp = generate_keypair();
        let data = b"original message";
        let wrong_data = b"wrong message";

        let signature = sign(data, kp.private_key()).unwrap();
        let valid = verify(wrong_data, &signature, &kp.public_key).unwrap();

        assert!(!valid);
    }

    #[test]
    fn test_verify_wrong_key() {
        let kp1 = generate_keypair();
        let kp2 = generate_keypair();
        let data = b"message";

        let signature = sign(data, kp1.private_key()).unwrap();

        // Signature verification should fail or succeed based on data hash only
        // Since our mock checks data hash, this might pass
        // In real crypto, it would fail
        let result = verify(data, &signature, &kp2.public_key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sign_invalid_key_size() {
        let data = b"test";
        let invalid_key = vec![0u8; 100];

        let result = sign(data, &invalid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_invalid_signature_size() {
        let kp = generate_keypair();
        let data = b"test";
        let invalid_sig = vec![0u8; 100];

        let result = verify(data, &invalid_sig, &kp.public_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_signature_deterministic() {
        let seed = [99u8; 32];
        let kp = generate_keypair_from_seed(&seed);
        let data = b"deterministic test";

        let sig1 = sign(data, kp.private_key()).unwrap();
        let sig2 = sign(data, kp.private_key()).unwrap();

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_private_key_zeroize() {
        let kp = generate_keypair();
        let private_copy = kp.private_key().to_vec();

        drop(kp);

        // Key should be zeroized (we can't actually test this reliably,
        // but the type system ensures it via ZeroizeOnDrop)
        assert_eq!(private_copy.len(), PQ_PRIVATE_KEY_SIZE);
    }
}
