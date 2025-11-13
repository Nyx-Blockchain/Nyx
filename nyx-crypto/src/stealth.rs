// src/stealth.rs

//! Stealth address generation for transaction unlinkability.
//!
//! Implements Monero-style ECDH stealth addresses where each transaction
//! output uses a unique one-time address that only the recipient can detect.

use crate::errors::{CryptoError, Result};
use crate::hash::blake3_hash;
use crate::STEALTH_ADDRESS_SIZE;
use curve25519_dalek::{
    edwards::CompressedEdwardsY,
    scalar::Scalar,
    constants::ED25519_BASEPOINT_TABLE,
};
use rand::Rng;

/// Generates a stealth address using Diffie-Hellman key exchange
///
/// Creates a one-time address: P = H(rA)G + B
/// where:
/// - r = random scalar (sender's ephemeral key)
/// - A = recipient's view public key
/// - B = recipient's spend public key
/// - G = base point
///
/// Only the recipient can detect this output belongs to them by computing
/// H(aR)G + B where a is their private view key and R = rG is published.
///
/// # Arguments
/// * `view_public` - Recipient's public view key
/// * `spend_public` - Recipient's public spend key
/// * `random_data` - Random bytes for ephemeral key generation
///
/// # Returns
/// Tuple of (stealth_address, ephemeral_public_key)
///
/// # Example
/// ```
/// use nyx_crypto::stealth::generate_stealth_address;
/// use nyx_crypto::keys::generate_keypair_ed25519;
///
/// let (_, view_pub) = generate_keypair_ed25519();
/// let (_, spend_pub) = generate_keypair_ed25519();
/// let random = vec![3u8; 32];
///
/// let (stealth_addr, ephemeral_pub) = generate_stealth_address(
///     &view_pub,
///     &spend_pub,
///     &random
/// ).unwrap();
///
/// assert_eq!(stealth_addr.len(), 32);
/// assert_eq!(ephemeral_pub.len(), 32);
/// ```
pub fn generate_stealth_address(
    view_public: &[u8],
    spend_public: &[u8],
    random_data: &[u8],
) -> Result<(Vec<u8>, Vec<u8>)> {
    // Validate inputs
    if view_public.len() != 32 {
        return Err(CryptoError::StealthAddressError(
            "View public key must be 32 bytes".to_string()
        ));
    }

    if spend_public.len() != 32 {
        return Err(CryptoError::StealthAddressError(
            "Spend public key must be 32 bytes".to_string()
        ));
    }

    // Generate ephemeral keypair from random data
    let ephemeral_scalar = Scalar::from_bytes_mod_order(
        hash_to_scalar(random_data)
    );
    let ephemeral_public = (&ephemeral_scalar * ED25519_BASEPOINT_TABLE).compress();

    // Parse recipient's view public key
    let view_point = CompressedEdwardsY::from_slice(view_public)
        .map_err(|_| CryptoError::StealthAddressError("Invalid view public key".to_string()))?
        .decompress()
        .ok_or_else(|| CryptoError::StealthAddressError("Failed to decompress view key".to_string()))?;

    // Compute shared secret: rA
    let shared_secret = ephemeral_scalar * view_point;

    // Hash shared secret: H(rA)
    let shared_secret_hash = blake3_hash(&shared_secret.compress().to_bytes());
    let shared_scalar = Scalar::from_bytes_mod_order(shared_secret_hash);

    // Parse spend public key
    let spend_point = CompressedEdwardsY::from_slice(spend_public)
        .map_err(|_| CryptoError::StealthAddressError("Invalid spend public key".to_string()))?
        .decompress()
        .ok_or_else(|| CryptoError::StealthAddressError("Failed to decompress spend key".to_string()))?;

    // Compute stealth address: P = H(rA)G + B
    let stealth_point = (&shared_scalar * ED25519_BASEPOINT_TABLE) + spend_point;
    let stealth_address = stealth_point.compress().to_bytes().to_vec();

    Ok((stealth_address, ephemeral_public.to_bytes().to_vec()))
}

/// Derives shared secret from private view key and ephemeral public key
///
/// Recipient uses this to check if an output belongs to them.
/// Computes: aR where a is private view key, R is ephemeral public key
///
/// # Arguments
/// * `view_private` - Recipient's private view key
/// * `ephemeral_public` - Sender's ephemeral public key (from transaction)
///
/// # Returns
/// Shared secret bytes
///
/// # Example
/// ```
/// use nyx_crypto::stealth::{generate_stealth_address, derive_shared_secret};
/// use nyx_crypto::keys::generate_keypair_ed25519;
///
/// let (view_priv, view_pub) = generate_keypair_ed25519();
/// let (_, spend_pub) = generate_keypair_ed25519();
///
/// let (stealth, ephemeral) = generate_stealth_address(&view_pub, &spend_pub, &[4u8; 32]).unwrap();
/// let secret = derive_shared_secret(&view_priv, &ephemeral).unwrap();
///
/// assert_eq!(secret.len(), 32);
/// ```
pub fn derive_shared_secret(
    view_private: &[u8],
    ephemeral_public: &[u8],
) -> Result<Vec<u8>> {
    if view_private.len() != 32 {
        return Err(CryptoError::StealthAddressError(
            "View private key must be 32 bytes".to_string()
        ));
    }

    if ephemeral_public.len() != 32 {
        return Err(CryptoError::StealthAddressError(
            "Ephemeral public key must be 32 bytes".to_string()
        ));
    }

    // Parse private key as scalar
    let view_scalar = Scalar::from_bytes_mod_order(
        hash_to_scalar(view_private)
    );

    // Parse ephemeral public key
    let ephemeral_point = CompressedEdwardsY::from_slice(ephemeral_public)
        .map_err(|_| CryptoError::StealthAddressError("Invalid ephemeral public key".to_string()))?
        .decompress()
        .ok_or_else(|| CryptoError::StealthAddressError("Failed to decompress ephemeral key".to_string()))?;

    // Compute shared secret: aR
    let shared_secret = view_scalar * ephemeral_point;

    Ok(shared_secret.compress().to_bytes().to_vec())
}

/// Checks if a stealth address belongs to the recipient
///
/// # Arguments
/// * `stealth_address` - The stealth address to check
/// * `view_private` - Recipient's private view key
/// * `spend_public` - Recipient's public spend key
/// * `ephemeral_public` - Ephemeral public key from transaction
///
/// # Returns
/// true if the address belongs to the recipient
pub fn is_mine(
    stealth_address: &[u8],
    view_private: &[u8],
    spend_public: &[u8],
    ephemeral_public: &[u8],
) -> Result<bool> {
    // Derive shared secret
    let shared_secret = derive_shared_secret(view_private, ephemeral_public)?;

    // Hash shared secret
    let shared_secret_hash = blake3_hash(&shared_secret);
    let shared_scalar = Scalar::from_bytes_mod_order(shared_secret_hash);

    // Parse spend public key
    let spend_point = CompressedEdwardsY::from_slice(spend_public)
        .map_err(|_| CryptoError::StealthAddressError("Invalid spend public key".to_string()))?
        .decompress()
        .ok_or_else(|| CryptoError::StealthAddressError("Failed to decompress spend key".to_string()))?;

    // Reconstruct stealth address: H(aR)G + B
    let reconstructed = (&shared_scalar * ED25519_BASEPOINT_TABLE) + spend_point;
    let reconstructed_bytes = reconstructed.compress().to_bytes();

    Ok(stealth_address == &reconstructed_bytes[..])
}

/// Generates random bytes for ephemeral key generation
pub fn generate_random_ephemeral() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let random: [u8; 32] = rng.gen();
    random.to_vec()
}

/// Helper: hash data to scalar for Ed25519
fn hash_to_scalar(data: &[u8]) -> [u8; 32] {
    blake3_hash(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::generate_keypair_ed25519;

    #[test]
    fn test_generate_stealth_address() {
        let (_, view_pub) = generate_keypair_ed25519();
        let (_, spend_pub) = generate_keypair_ed25519();
        let random = generate_random_ephemeral();

        let result = generate_stealth_address(&view_pub, &spend_pub, &random);
        assert!(result.is_ok());

        let (stealth, ephemeral) = result.unwrap();
        assert_eq!(stealth.len(), STEALTH_ADDRESS_SIZE);
        assert_eq!(ephemeral.len(), 32);
    }

    #[test]
    fn test_stealth_address_different_random() {
        let (_, view_pub) = generate_keypair_ed25519();
        let (_, spend_pub) = generate_keypair_ed25519();

        let (stealth1, _) = generate_stealth_address(&view_pub, &spend_pub, &[1u8; 32]).unwrap();
        let (stealth2, _) = generate_stealth_address(&view_pub, &spend_pub, &[2u8; 32]).unwrap();

        assert_ne!(stealth1, stealth2);
    }

    #[test]
    fn test_derive_shared_secret() {
        let (view_priv, view_pub) = generate_keypair_ed25519();
        let (_, spend_pub) = generate_keypair_ed25519();

        let (_, ephemeral) = generate_stealth_address(&view_pub, &spend_pub, &[1u8; 32]).unwrap();
        let secret = derive_shared_secret(&view_priv, &ephemeral).unwrap();

        assert_eq!(secret.len(), 32);
    }

    #[test]
    fn test_is_mine() {
        let (view_priv, view_pub) = generate_keypair_ed25519();
        let (_, spend_pub) = generate_keypair_ed25519();
        let random = [42u8; 32];

        let (stealth, ephemeral) = generate_stealth_address(&view_pub, &spend_pub, &random).unwrap();

        // Should recognize own address
        let mine = is_mine(&stealth, &view_priv, &spend_pub, &ephemeral).unwrap();
        assert!(mine);
    }

    #[test]
    fn test_is_not_mine() {
        let (_view_priv1, view_pub1) = generate_keypair_ed25519();
        let (_, spend_pub1) = generate_keypair_ed25519();

        let (view_priv2, _) = generate_keypair_ed25519();
        let (_, spend_pub2) = generate_keypair_ed25519();

        // Generate stealth for recipient 1
        let (stealth, ephemeral) = generate_stealth_address(&view_pub1, &spend_pub1, &[1u8; 32]).unwrap();

        // Recipient 2 checks if it's theirs (should be false)
        let mine = is_mine(&stealth, &view_priv2, &spend_pub2, &ephemeral).unwrap();
        assert!(!mine);
    }

    #[test]
    fn test_invalid_key_sizes() {
        let invalid_key = vec![1u8; 16]; // Wrong size
        let valid_key = vec![2u8; 32];

        let result = generate_stealth_address(&invalid_key, &valid_key, &valid_key);
        assert!(result.is_err());

        let result = generate_stealth_address(&valid_key, &invalid_key, &valid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_random_ephemeral() {
        let rand1 = generate_random_ephemeral();
        let rand2 = generate_random_ephemeral();

        assert_eq!(rand1.len(), 32);
        assert_eq!(rand2.len(), 32);
        assert_ne!(rand1, rand2);
    }
}
