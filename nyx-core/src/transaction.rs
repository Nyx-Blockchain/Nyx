// src/transaction.rs

use crate::types::{Hash, Timestamp};
use nyx_crypto::ring;  // Import crypto types
use serde::{Deserialize, Serialize};

/// Transaction input referencing a previous output
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TxInput {
    /// Hash of the previous transaction being spent
    pub prev_tx: Hash,

    /// Index of the output in the previous transaction
    pub index: u32,

    /// Key image for double-spend prevention
    pub key_image: [u8; 32],

    /// Ring signature indices (decoy outputs mixed with true input)
    pub ring_indices: Vec<u32>,
}

/// Transaction output with stealth address and confidential amount
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TxOutput {
    /// One-time stealth address
    pub stealth_address: Vec<u8>,

    /// Encrypted amount (Pedersen commitment)
    pub amount_commitment: Vec<u8>,

    /// Range proof (Bulletproofs+)
    pub range_proof: Vec<u8>,

    /// Ephemeral public key for ECDH
    pub ephemeral_pubkey: Vec<u8>,
}

/// Complete Nyx transaction structure
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Protocol version
    pub version: u8,

    /// Inputs being spent
    pub inputs: Vec<TxInput>,

    /// New outputs being created
    pub outputs: Vec<TxOutput>,

    /// Ring signature (from nyx-crypto)
    pub ring_signature: ring::RingSignature,

    /// Transaction public key for ECDH
    pub tx_key: Vec<u8>,

    /// Two parent transaction hashes (DAG references)
    pub references: [Hash; 2],

    /// Unix timestamp
    pub timestamp: Timestamp,

    /// Extra data field
    pub extra: Vec<u8>,
}

impl Transaction {
    /// Computes transaction ID using nyx-crypto's hash
    pub fn id(&self) -> Hash {
        let serialized = bincode::serialize(self)
            .expect("Transaction serialization should never fail");

        // Use nyx-crypto's BLAKE3 hash
        nyx_crypto::hash::blake3_hash(&serialized)
    }

    /// Creates a new transaction
    pub fn new(
        inputs: Vec<TxInput>,
        outputs: Vec<TxOutput>,
        ring_signature: ring::RingSignature,
        tx_key: Vec<u8>,
        parent1: Hash,
        parent2: Hash,
    ) -> Self {
        Self {
            version: 1,
            inputs,
            outputs,
            ring_signature,
            tx_key,
            references: [parent1, parent2],
            timestamp: current_timestamp(),
            extra: Vec::new(),
        }
    }

    /// Signs transaction with ring signature
    pub fn sign(
        &mut self,
        message: &[u8],
        private_key: &[u8],
        public_key: &[u8],
        ring_members: &[Vec<u8>],
    ) -> Result<(), nyx_crypto::CryptoError> {
        // Generate ring signature using nyx-crypto
        let ring_sig = nyx_crypto::ring::generate_ring_signature(
            message,
            private_key,
            public_key,
            ring_members,
        )?;

        self.ring_signature = ring_sig;
        Ok(())
    }

    /// Verifies the ring signature
    pub fn verify_signature(&self) -> Result<bool, nyx_crypto::CryptoError> {
        let message = self.signing_message();
        nyx_crypto::ring::verify_ring_signature(&message, &self.ring_signature)
    }

    /// Creates message to be signed
    pub fn signing_message(&self) -> Vec<u8> {
        // Serialize everything except the signature
        let mut data = Vec::new();
        data.extend_from_slice(&[self.version]);

        for input in &self.inputs {
            data.extend_from_slice(&input.prev_tx);
            data.extend_from_slice(&input.index.to_le_bytes());
        }

        for output in &self.outputs {
            data.extend_from_slice(&output.stealth_address);
            data.extend_from_slice(&output.amount_commitment);
        }

        data.extend_from_slice(&self.references[0]);
        data.extend_from_slice(&self.references[1]);

        nyx_crypto::hash::blake3_hash(&data).to_vec()
    }

    /// Validates transaction structure
    pub fn validate_structure(&self) -> bool {
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return false;
        }

        if self.references[0] == self.references[1] {
            return false;
        }

        let now = current_timestamp();
        let two_hours = 2 * 60 * 60;
        if self.timestamp > now + two_hours {
            return false;
        }

        // Validate ring signature structure
        if self.ring_signature.ring_size() < 2 {
            return false;
        }

        // Validate key images
        for input in &self.inputs {
            if nyx_crypto::ring::validate_key_image(&input.key_image).is_err() {
                return false;
            }
        }

        true
    }
}

fn current_timestamp() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be after Unix epoch")
        .as_secs()
}
