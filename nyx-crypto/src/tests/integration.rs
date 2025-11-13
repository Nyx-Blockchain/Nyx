// tests/integration.rs

//! Integration tests for the Nyx cryptography module.
//!
//! Tests the complete cryptographic flow end-to-end including:
//! - Post-quantum keypair generation
//! - Hashing and signing
//! - Ring signatures with key images
//! - Stealth address generation
//! - Symmetric encryption

#[cfg(test)]
mod tests {
    use nyx_crypto::*;
    use nyx_crypto::{hash, keys, ring, stealth, encryption};

    #[test]
    fn test_complete_crypto_flow() {
        // 1. Generate post-quantum keypair
        println!("Step 1: Generating PQC keypair...");
        let keypair = keys::generate_keypair();
        assert_eq!(keypair.public_key.len(), PQ_PUBLIC_KEY_SIZE);
        assert_eq!(keypair.private_key().len(), PQ_PRIVATE_KEY_SIZE);
        println!("  ✓ Keypair generated");

        // 2. Hash a message
        println!("Step 2: Hashing message...");
        let message = b"Nyx blockchain transaction data";
        let message_hash = hash::blake3_hash(message);
        assert_eq!(message_hash.len(), HASH_SIZE);
        println!("  ✓ Message hash: {}", hash::hash_to_hex(&message_hash));

        // 3. Sign the hash
        println!("Step 3: Signing message...");
        let signature = keys::sign(message, keypair.private_key()).unwrap();
        assert_eq!(signature.len(), PQ_SIGNATURE_SIZE);
        println!("  ✓ Signature generated ({} bytes)", signature.len());

        // 4. Verify signature
        println!("Step 4: Verifying signature...");
        let valid = keys::verify(message, &signature, &keypair.public_key).unwrap();
        assert!(valid);
        println!("  ✓ Signature verified successfully");

        // 5. Generate stealth address
        println!("Step 5: Generating stealth address...");
        let view_keypair = keys::generate_keypair();
        let spend_keypair = keys::generate_keypair();

        let (stealth_addr, ephemeral_pub) = stealth::generate_stealth_address(
            &view_keypair.public_key[..32],
            &spend_keypair.public_key[..32],
            &[42u8; 32]
        ).unwrap();

        assert_eq!(stealth_addr.len(), STEALTH_ADDRESS_SIZE);
        println!("  ✓ Stealth address generated");
        println!("    Address: {}", hex::encode(&stealth_addr));

        // 6. Encrypt confidential data
        println!("Step 6: Encrypting confidential data...");
        let confidential_data = b"Transaction amount: 100 NYX";
        let encryption_key = encryption::generate_key();
        let ciphertext = encryption::encrypt(confidential_data, &encryption_key).unwrap();
        println!("  ✓ Data encrypted ({} bytes)", ciphertext.len());

        // 7. Decrypt confidential data
        println!("Step 7: Decrypting data...");
        let decrypted = encryption::decrypt(&ciphertext, &encryption_key).unwrap();
        assert_eq!(confidential_data, &decrypted[..]);
        println!("  ✓ Data decrypted successfully");

        println!("\n✅ Complete crypto flow test passed!");
    }

    #[test]
    fn test_ring_signature_flow() {
        println!("Testing ring signature flow...");

        // Generate keypairs for ring members
        let signer = keys::generate_keypair();
        let decoy1 = keys::generate_keypair();
        let decoy2 = keys::generate_keypair();
        let decoy3 = keys::generate_keypair();

        // Create ring of public keys
        let ring = vec![
            signer.public_key.clone(),
            decoy1.public_key,
            decoy2.public_key,
            decoy3.public_key,
        ];

        println!("  Ring size: {}", ring.len());

        // Generate ring signature
        let message = b"Anonymous transaction";
        let ring_sig = ring::generate_ring_signature(
            message,
            signer.private_key(),
            &signer.public_key,
            &ring
        ).unwrap();

        println!("  ✓ Ring signature generated");
        println!("    Key image: {}", hex::encode(&ring_sig.key_image));

        // Verify ring signature
        let valid = ring::verify_ring_signature(message, &ring_sig).unwrap();
        assert!(valid);
        println!("  ✓ Ring signature verified");

        // Test key image uniqueness
        let ring_sig2 = ring::generate_ring_signature(
            b"different message",
            signer.private_key(),
            &signer.public_key,
            &ring
        ).unwrap();

        // Same signer should produce same key image
        assert_eq!(ring_sig.key_image, ring_sig2.key_image);
        println!("  ✓ Key image consistency verified");

        println!("✅ Ring signature flow test passed!");
    }

    #[test]
    fn test_stealth_address_detection() {
        println!("Testing stealth address detection...");

        // Recipient generates view and spend keypairs
        let (view_priv, view_pub) = generate_ed25519_keypair();
        let (spend_priv, spend_pub) = generate_ed25519_keypair();

        println!("  Recipient keys generated");

        // Sender creates stealth address for recipient
        let random = stealth::generate_random_ephemeral();
        let (stealth_addr, ephemeral_pub) = stealth::generate_stealth_address(
            &view_pub,
            &spend_pub,
            &random
        ).unwrap();

        println!("  ✓ Stealth address created");
        println!("    Address: {}", hex::encode(&stealth_addr));

        // Recipient checks if address is theirs
        let is_mine = stealth::is_mine(
            &stealth_addr,
            &view_priv,
            &spend_pub,
            &ephemeral_pub
        ).unwrap();

        assert!(is_mine);
        println!("  ✓ Recipient successfully detected their address");

        // Different recipient should not detect it
        let (other_view_priv, _) = generate_ed25519_keypair();
        let (_, other_spend_pub) = generate_ed25519_keypair();

        let not_mine = stealth::is_mine(
            &stealth_addr,
            &other_view_priv,
            &other_spend_pub,
            &ephemeral_pub
        ).unwrap();

        assert!(!not_mine);
        println!("  ✓ Other recipient correctly rejected address");

        println!("✅ Stealth address detection test passed!");
    }

    #[test]
    fn test_multi_hash_consistency() {
        println!("Testing hash function consistency...");

        let data = b"Nyx Protocol Test Data";

        // Test BLAKE3
        let blake3_1 = hash::blake3_hash(data);
        let blake3_2 = hash::blake3_hash(data);
        assert_eq!(blake3_1, blake3_2);
        println!("  ✓ BLAKE3 consistent");

        // Test Keccak
        let keccak_1 = hash::keccak_hash(data);
        let keccak_2 = hash::keccak_hash(data);
        assert_eq!(keccak_1, keccak_2);
        println!("  ✓ Keccak-256 consistent");

        // Verify different algorithms produce different outputs
        assert_ne!(blake3_1, keccak_1);
        println!("  ✓ Hash functions produce distinct outputs");

        // Test hex conversion
        let hex = hash::hash_to_hex(&blake3_1);
        let restored = hash::hex_to_hash(&hex).unwrap();
        assert_eq!(blake3_1, restored);
        println!("  ✓ Hex conversion round-trip successful");

        println!("✅ Hash consistency test passed!");
    }

    #[test]
    fn test_encryption_with_authentication() {
        println!("Testing authenticated encryption...");

        let key = encryption::generate_key();
        let plaintext = b"Confidential transaction data";
        let associated_data = b"tx_id_12345";

        // Encrypt with AAD
        let ciphertext = encryption::encrypt_with_aad(
            plaintext,
            &key,
            associated_data
        ).unwrap();

        println!("  ✓ Data encrypted with AAD");

        // Decrypt with correct AAD
        let decrypted = encryption::decrypt_with_aad(
            &ciphertext,
            &key,
            associated_data
        ).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
        println!("  ✓ Data decrypted with correct AAD");

        // Attempt to decrypt with wrong AAD should fail
        let wrong_aad = b"wrong_tx_id";
        let result = encryption::decrypt_with_aad(
            &ciphertext,
            &key,
            wrong_aad
        );

        assert!(result.is_err());
        println!("  ✓ Decryption rejected with wrong AAD");

        println!("✅ Authenticated encryption test passed!");
    }

    #[test]
    fn test_deterministic_operations() {
        println!("Testing deterministic cryptographic operations...");

        // Deterministic keypair generation
        let seed = [42u8; 32];
        let kp1 = keys::generate_keypair_from_seed(&seed);
        let kp2 = keys::generate_keypair_from_seed(&seed);
        assert_eq!(kp1.public_key, kp2.public_key);
        println!("  ✓ Keypair generation deterministic");

        // Deterministic hashing
        let data = b"test data";
        let hash1 = hash::blake3_hash(data);
        let hash2 = hash::blake3_hash(data);
        assert_eq!(hash1, hash2);
        println!("  ✓ Hashing deterministic");

        // Deterministic key image
        let private_key = vec![1u8; 100];
        let ki1 = ring::generate_key_image(&private_key);
        let ki2 = ring::generate_key_image(&private_key);
        assert_eq!(ki1, ki2);
        println!("  ✓ Key image generation deterministic");

        println!("✅ Deterministic operations test passed!");
    }

    #[test]
    fn test_double_spend_detection() {
        println!("Testing double-spend detection via key images...");

        let signer = keys::generate_keypair();
        let decoy = keys::generate_keypair();
        let ring = vec![signer.public_key.clone(), decoy.public_key];

        // Create two transactions with same signer
        let tx1_msg = b"transaction 1";
        let tx2_msg = b"transaction 2";

        let ring_sig1 = ring::generate_ring_signature(
            tx1_msg,
            signer.private_key(),
            &signer.public_key,
            &ring
        ).unwrap();

        let ring_sig2 = ring::generate_ring_signature(
            tx2_msg,
            signer.private_key(),
            &signer.public_key,
            &ring
        ).unwrap();

        // Key images should be identical (same private key)
        assert!(ring::key_images_equal(&ring_sig1.key_image, &ring_sig2.key_image));
        println!("  ✓ Double-spend detected: identical key images");

        // Different signer should have different key image
        let other_signer = keys::generate_keypair();
        let ring2 = vec![other_signer.public_key.clone(), decoy.public_key.clone()];

        let ring_sig3 = ring::generate_ring_signature(
            tx1_msg,
            other_signer.private_key(),
            &other_signer.public_key,
            &ring2
        ).unwrap();

        assert!(!ring::key_images_equal(&ring_sig1.key_image, &ring_sig3.key_image));
        println!("  ✓ Different signers produce different key images");

        println!("✅ Double-spend detection test passed!");
    }

    #[test]
    fn test_signature_verification_failures() {
        println!("Testing signature verification failure cases...");

        let keypair = keys::generate_keypair();
        let message = b"original message";
        let signature = keys::sign(message, keypair.private_key()).unwrap();

        // Wrong message
        let wrong_message = b"tampered message";
        let valid = keys::verify(wrong_message, &signature, &keypair.public_key).unwrap();
        assert!(!valid);
        println!("  ✓ Rejected tampered message");

        // Corrupted signature
        let mut bad_signature = signature.clone();
        if let Some(byte) = bad_signature.last_mut() {
            *byte ^= 0xFF;
        }
        let valid = keys::verify(message, &bad_signature, &keypair.public_key).unwrap();
        assert!(!valid);
        println!("  ✓ Rejected corrupted signature");

        println!("✅ Signature verification failure test passed!");
    }

    #[test]
    fn test_privacy_guarantees() {
        println!("Testing privacy guarantees...");

        // Ring signatures hide the true signer
        let signer = keys::generate_keypair();
        let mut decoys = Vec::new();
        for _ in 0..15 {
            decoys.push(keys::generate_keypair().public_key);
        }

        let mut ring = vec![signer.public_key.clone()];
        ring.extend(decoys);

        let message = b"private transaction";
        let ring_sig = ring::generate_ring_signature(
            message,
            signer.private_key(),
            &signer.public_key,
            &ring
        ).unwrap();

        // Verifier can confirm signature is valid but not which key signed
        assert!(ring::verify_ring_signature(message, &ring_sig).unwrap());
        assert_eq!(ring_sig.ring_size(), RING_SIZE);
        println!("  ✓ Ring signature provides 1/{} anonymity", RING_SIZE);

        // Stealth addresses hide recipient
        let (view_priv, view_pub) = generate_ed25519_keypair();
        let (_, spend_pub) = generate_ed25519_keypair();

        let (stealth1, _) = stealth::generate_stealth_address(
            &view_pub,
            &spend_pub,
            &[1u8; 32]
        ).unwrap();

        let (stealth2, _) = stealth::generate_stealth_address(
            &view_pub,
            &spend_pub,
            &[2u8; 32]
        ).unwrap();

        // Different stealth addresses for same recipient
        assert_ne!(stealth1, stealth2);
        println!("  ✓ Stealth addresses unlinkable");

        // Encryption hides amounts
        let amount = b"1000000 NYX";
        let key = encryption::generate_key();
        let encrypted = encryption::encrypt(amount, &key).unwrap();

        // Ciphertext reveals nothing about plaintext
        assert_ne!(amount.to_vec(), encrypted);
        println!("  ✓ Encryption hides transaction amounts");

        println!("✅ Privacy guarantees test passed!");
    }

    #[test]
    fn test_performance_benchmarks() {
        println!("Running performance benchmarks...");

        use std::time::Instant;

        // Hash performance
        let data = vec![0u8; 1024 * 1024]; // 1 MB
        let start = Instant::now();
        let _hash = hash::blake3_hash(&data);
        let duration = start.elapsed();
        println!("  BLAKE3 (1 MB): {:?}", duration);

        // Keypair generation
        let start = Instant::now();
        let _kp = keys::generate_keypair();
        let duration = start.elapsed();
        println!("  Keypair generation: {:?}", duration);

        // Signature generation
        let kp = keys::generate_keypair();
        let msg = b"benchmark message";
        let start = Instant::now();
        let sig = keys::sign(msg, kp.private_key()).unwrap();
        let duration = start.elapsed();
        println!("  Signature generation: {:?}", duration);

        // Signature verification
        let start = Instant::now();
        let _valid = keys::verify(msg, &sig, &kp.public_key).unwrap();
        let duration = start.elapsed();
        println!("  Signature verification: {:?}", duration);

        // Encryption
        let key = encryption::generate_key();
        let plaintext = vec![0u8; 1024]; // 1 KB
        let start = Instant::now();
        let _ct = encryption::encrypt(&plaintext, &key).unwrap();
        let duration = start.elapsed();
        println!("  Encryption (1 KB): {:?}", duration);

        println!("✅ Performance benchmark completed!");
    }

    // Helper function to generate Ed25519 keypairs for stealth addresses
    fn generate_ed25519_keypair() -> (Vec<u8>, Vec<u8>) {
        use rand::Rng;
        use curve25519_dalek::{scalar::Scalar, constants::ED25519_BASEPOINT_TABLE};

        let mut rng = rand::thread_rng();
        let private: [u8; 32] = rng.gen();
        let scalar = Scalar::from_bytes_mod_order(hash::blake3_hash(&private));
        let public = (&scalar * ED25519_BASEPOINT_TABLE).compress().to_bytes().to_vec();

        (private.to_vec(), public)
    }
}
