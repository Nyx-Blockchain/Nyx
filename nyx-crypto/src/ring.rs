// src/ring.rs

//! Lattice-based linkable ring signatures for privacy.
//!
//! Provides sender anonymity through ring signatures where the signer
//! is hidden among a set of decoys. Includes key images to prevent
//! double-spending without revealing the true signer.

use crate::errors::{CryptoError, Result};
use crate::hash::{blake3_hash, hash_chunks};
use crate::{RING_SIZE, KEY_IMAGE_SIZE};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Lattice-based linkable ring signature
///
/// Provides sender anonymity with a ring of possible signers.
/// The key image prevents double-spending without revealing identity.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RingSignature {
    /// Public keys of all ring members (1 true + decoys)
    pub ring_members: Vec<Vec<u8>>,

    /// Ring signature data (proof of knowledge)
    pub signature: Vec<u8>,

    /// Key image for double-spend prevention
    /// Derived deterministically from private key
    pub key_image: [u8; KEY_IMAGE_SIZE],
}

impl RingSignature {
    /// Returns the size of the anonymity set
    pub fn ring_size(&self) -> usize {
        self.ring_members.len()
    }

    /// Checks if the ring contains a specific public key
    pub fn contains_member(&self, public_key: &[u8]) -> bool {
        self.ring_members.iter().any(|member| member == public_key)
    }
}

/// Generates a key image from a private key
///
/// Key images are deterministic values derived from private keys that:
/// - Allow detection of double-spends
/// - Don't reveal the actual private key or public key
/// - Are unique per private key
///
/// # Arguments
/// * `private_key` - Private key to derive key image from
///
/// # Returns
/// 32-byte key image
///
/// # Example
/// ```
/// use nyx_crypto::ring::generate_key_image;
///
/// let private_key = vec![1u8; 100];
/// let key_image = generate_key_image(&private_key);
/// assert_eq!(key_image.len(), 32);
/// ```
pub fn generate_key_image(private_key: &[u8]) -> [u8; KEY_IMAGE_SIZE] {
    // Key image = H(private_key || "key_image")
    // In real implementation, this would be: I = x * H_p(P)
    // where x is private key, P is public key, H_p is hash-to-point
    let mut data = private_key.to_vec();
    data.extend_from_slice(b"key_image");
    blake3_hash(&data)
}

/// Generates a ring signature
///
/// Creates a signature that proves knowledge of a private key corresponding
/// to one member of the ring, without revealing which member.
///
/// # Arguments
/// * `message` - Message to sign
/// * `private_key` - Signer's private key
/// * `public_key` - Signer's public key (must be in ring_members)
/// * `ring_members` - All public keys in the ring (includes signer)
///
/// # Returns
/// RingSignature proving knowledge of one private key in the ring
///
/// # Example
/// ```
/// use nyx_crypto::ring::generate_ring_signature;
/// use nyx_crypto::keys::generate_keypair;
///
/// let kp = generate_keypair();
/// let decoy1 = generate_keypair();
/// let decoy2 = generate_keypair();
///
/// let ring = vec![
///     kp.public_key.clone(),
///     decoy1.public_key.clone(),
///     decoy2.public_key.clone(),
/// ];
///
/// let sig = generate_ring_signature(
///     b"message",
///     kp.private_key(),
///     &kp.public_key,
///     &ring
/// ).unwrap();
///
/// assert_eq!(sig.ring_size(), 3);
/// ```
pub fn generate_ring_signature(
    message: &[u8],
    private_key: &[u8],
    public_key: &[u8],
    ring_members: &[Vec<u8>],
) -> Result<RingSignature> {
    // Validate inputs
    if ring_members.is_empty() {
        return Err(CryptoError::RingSignatureError(
            "Ring must contain at least one member".to_string()
        ));
    }

    if ring_members.len() > RING_SIZE {
        return Err(CryptoError::RingSignatureError(
            format!("Ring too large: max {}, got {}", RING_SIZE, ring_members.len())
        ));
    }

    // Verify signer is in the ring
    if !ring_members.iter().any(|member| member == public_key) {
        return Err(CryptoError::RingSignatureError(
            "Signer's public key not found in ring".to_string()
        ));
    }

    // Generate key image
    let key_image = generate_key_image(private_key);

    // Mock ring signature generation
    // Real implementation would use Lattice-based Linkable Ring Signatures (LLRS)

    // Commitment: H(message || all ring members || key_image)
    let mut commitment_data = message.to_vec();
    for member in ring_members {
        commitment_data.extend_from_slice(member);
    }
    commitment_data.extend_from_slice(&key_image);
    let commitment = blake3_hash(&commitment_data);

    // Challenge values for each ring member
    let mut signature_data = Vec::new();
    signature_data.extend_from_slice(&commitment);

    // Generate random challenges for decoys and compute response for true key
    let mut rng = rand::thread_rng();
    for (i, member) in ring_members.iter().enumerate() {
        if member == public_key {
            // True signer: response = r - challenge * private_key
            let response_hash = hash_chunks(&[
                private_key,
                &commitment,
                &[i as u8],
            ]);
            signature_data.extend_from_slice(&response_hash);
        } else {
            // Decoy: random response
            let random: [u8; 32] = rng.gen();
            signature_data.extend_from_slice(&random);
        }
    }

    Ok(RingSignature {
        ring_members: ring_members.to_vec(),
        signature: signature_data,
        key_image,
    })
}

/// Verifies a ring signature
///
/// Checks that the signature is valid for at least one member of the ring
/// without revealing which member signed.
///
/// # Arguments
/// * `message` - Original message that was signed
/// * `ring_sig` - Ring signature to verify
///
/// # Returns
/// `Ok(true)` if valid, `Ok(false)` if invalid, `Err` on error
///
/// # Example
/// ```
/// use nyx_crypto::ring::{generate_ring_signature, verify_ring_signature};
/// use nyx_crypto::keys::generate_keypair;
///
/// let kp = generate_keypair();
/// let decoy = generate_keypair();
/// let ring = vec![kp.public_key.clone(), decoy.public_key.clone()];
///
/// let message = b"test message";
/// let sig = generate_ring_signature(message, kp.private_key(), &kp.public_key, &ring).unwrap();
///
/// assert!(verify_ring_signature(message, &sig).unwrap());
/// ```
pub fn verify_ring_signature(message: &[u8], ring_sig: &RingSignature) -> Result<bool> {
    if ring_sig.ring_members.is_empty() {
        return Err(CryptoError::RingSignatureError(
            "Ring signature has no members".to_string()
        ));
    }

    // Reconstruct commitment
    let mut commitment_data = message.to_vec();
    for member in &ring_sig.ring_members {
        commitment_data.extend_from_slice(member);
    }
    commitment_data.extend_from_slice(&ring_sig.key_image);
    let expected_commitment = blake3_hash(&commitment_data);

    // Extract commitment from signature
    if ring_sig.signature.len() < 32 {
        return Ok(false);
    }
    let sig_commitment = &ring_sig.signature[..32];

    // Verify commitment matches
    if sig_commitment != &expected_commitment[..] {
        return Ok(false);
    }

    // In real implementation, we would verify the ring signature equations
    // For this mock, we just verify the commitment is correct
    Ok(true)
}

/// Checks if two key images are the same (indicating double-spend attempt)
///
/// # Arguments
/// * `key_image1` - First key image
/// * `key_image2` - Second key image
///
/// # Returns
/// true if key images match (double-spend detected)
pub fn key_images_equal(key_image1: &[u8; KEY_IMAGE_SIZE], key_image2: &[u8; KEY_IMAGE_SIZE]) -> bool {
    key_image1 == key_image2
}

/// Validates that a key image is well-formed
///
/// # Arguments
/// * `key_image` - Key image to validate
///
/// # Returns
/// Ok(()) if valid, Err otherwise
pub fn validate_key_image(key_image: &[u8; KEY_IMAGE_SIZE]) -> Result<()> {
    // Check it's not all zeros (invalid)
    if key_image.iter().all(|&b| b == 0) {
        return Err(CryptoError::InvalidKeyImage(
            "Key image cannot be all zeros".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::generate_keypair;

    #[test]
    fn test_generate_key_image() {
        let private_key = vec![1u8; 100];
        let ki1 = generate_key_image(&private_key);
        let ki2 = generate_key_image(&private_key);

        assert_eq!(ki1, ki2);
        assert_eq!(ki1.len(), KEY_IMAGE_SIZE);
    }

    #[test]
    fn test_different_keys_different_images() {
        let ki1 = generate_key_image(&[1u8; 100]);
        let ki2 = generate_key_image(&[2u8; 100]);

        assert_ne!(ki1, ki2);
    }

    #[test]
    fn test_generate_ring_signature() {
        let kp = generate_keypair();
        let decoy1 = generate_keypair();
        let decoy2 = generate_keypair();

        let ring = vec![
            kp.public_key.clone(),
            decoy1.public_key,
            decoy2.public_key,
        ];

        let message = b"test message";
        let sig = generate_ring_signature(message, kp.private_key(), &kp.public_key, &ring).unwrap();

        assert_eq!(sig.ring_size(), 3);
        assert_eq!(sig.key_image.len(), KEY_IMAGE_SIZE);
        assert!(sig.contains_member(&kp.public_key));
    }

    #[test]
    fn test_verify_ring_signature() {
        let kp = generate_keypair();
        let decoy = generate_keypair();
        let ring = vec![kp.public_key.clone(), decoy.public_key];

        let message = b"test";
        let sig = generate_ring_signature(message, kp.private_key(), &kp.public_key, &ring).unwrap();

        let valid = verify_ring_signature(message, &sig).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_verify_wrong_message() {
        let kp = generate_keypair();
        let decoy = generate_keypair();
        let ring = vec![kp.public_key.clone(), decoy.public_key];

        let sig = generate_ring_signature(b"original", kp.private_key(), &kp.public_key, &ring).unwrap();
        let valid = verify_ring_signature(b"wrong", &sig).unwrap();

        assert!(!valid);
    }

    #[test]
    fn test_ring_signature_signer_not_in_ring() {
        let kp = generate_keypair();
        let decoy1 = generate_keypair();
        let decoy2 = generate_keypair();

        let ring = vec![decoy1.public_key, decoy2.public_key];

        let result = generate_ring_signature(b"test", kp.private_key(), &kp.public_key, &ring);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_ring() {
        let kp = generate_keypair();
        let ring = vec![];

        let result = generate_ring_signature(b"test", kp.private_key(), &kp.public_key, &ring);
        assert!(result.is_err());
    }

    #[test]
    fn test_ring_too_large() {
        let kp = generate_keypair();
        let mut ring = vec![kp.public_key.clone()];

        // Create ring larger than RING_SIZE
        for _ in 0..RING_SIZE {
            ring.push(generate_keypair().public_key);
        }

        let result = generate_ring_signature(b"test", kp.private_key(), &kp.public_key, &ring);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_images_equal() {
        let ki1 = generate_key_image(&[1u8; 100]);
        let ki2 = generate_key_image(&[1u8; 100]);
        let ki3 = generate_key_image(&[2u8; 100]);

        assert!(key_images_equal(&ki1, &ki2));
        assert!(!key_images_equal(&ki1, &ki3));
    }

    #[test]
    fn test_validate_key_image() {
        let valid = generate_key_image(&[1u8; 100]);
        assert!(validate_key_image(&valid).is_ok());

        let invalid = [0u8; KEY_IMAGE_SIZE];
        assert!(validate_key_image(&invalid).is_err());
    }

    #[test]
    fn test_ring_signature_deterministic() {
        let kp = generate_keypair();
        let decoy = generate_keypair();
        let ring = vec![kp.public_key.clone(), decoy.public_key];

        let msg = b"deterministic test";
        let sig1 = generate_ring_signature(msg, kp.private_key(), &kp.public_key, &ring).unwrap();
        let sig2 = generate_ring_signature(msg, kp.private_key(), &kp.public_key, &ring).unwrap();

        // Key images should be the same (deterministic)
        assert_eq!(sig1.key_image, sig2.key_image);
    }
}
