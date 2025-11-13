// src/transaction_builder.rs

//! Transaction builder with integrated cryptography.

use crate::transaction::{Transaction, TxInput, TxOutput};
use crate::types::Hash;
use nyx_crypto::{ring, stealth, encryption, keys};

/// Builder for creating privacy-preserving transactions
pub struct TransactionBuilder {
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    ring_members: Vec<Vec<u8>>,
    signer_keypair: Option<keys::KeyPair>,
}

impl TransactionBuilder {
    /// Creates a new transaction builder
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            ring_members: Vec::new(),
            signer_keypair: None,
        }
    }

    /// Sets the signer's keypair
    pub fn with_signer(mut self, keypair: keys::KeyPair) -> Self {
        self.signer_keypair = Some(keypair);
        self
    }

    /// Adds an input to spend
    pub fn add_input(
        mut self,
        prev_tx: Hash,
        index: u32,
        private_key: &[u8],
    ) -> Result<Self, nyx_crypto::CryptoError> {
        // Generate key image from private key
        let key_image = ring::generate_key_image(private_key);

        let input = TxInput {
            prev_tx,
            index,
            key_image,
            ring_indices: Vec::new(), // Will be populated with decoys
        };

        self.inputs.push(input);
        Ok(self)
    }

    /// Adds an output with stealth address
    pub fn add_output(
        mut self,
        view_public: &[u8],
        spend_public: &[u8],
        amount: u64,
    ) -> Result<Self, nyx_crypto::CryptoError> {
        // Generate random for stealth address
        let random = stealth::generate_random_ephemeral();

        // Create stealth address
        let (stealth_address, ephemeral_pubkey) = stealth::generate_stealth_address(
            view_public,
            spend_public,
            &random,
        )?;

        // Encrypt amount (simplified - in production use Pedersen commitments)
        let encryption_key = encryption::generate_key();
        let amount_bytes = amount.to_le_bytes();
        let amount_commitment = encryption::encrypt(&amount_bytes, &encryption_key)?;

        // TODO: Generate actual range proof (Bulletproofs+)
        let range_proof = vec![0u8; 100]; // Placeholder

        let output = TxOutput {
            stealth_address,
            amount_commitment,
            range_proof,
            ephemeral_pubkey,
        };

        self.outputs.push(output);
        Ok(self)
    }

    /// Sets the ring members for anonymity
    pub fn with_ring_members(mut self, members: Vec<Vec<u8>>) -> Self {
        self.ring_members = members;
        self
    }

    /// Builds and signs the transaction
    pub fn build(
        self,
        parent1: Hash,
        parent2: Hash,
    ) -> Result<Transaction, nyx_crypto::CryptoError> {
        let keypair = self.signer_keypair
            .ok_or_else(|| nyx_crypto::CryptoError::InvalidKey("No signer keypair".to_string()))?;

        if self.ring_members.is_empty() {
            return Err(nyx_crypto::CryptoError::RingSignatureError(
                "Ring members required".to_string()
            ));
        }

        // Create unsigned transaction
        let mut tx = Transaction {
            version: 1,
            inputs: self.inputs,
            outputs: self.outputs,
            ring_signature: ring::RingSignature {
                ring_members: self.ring_members.clone(),
                signature: Vec::new(),
                key_image: [0u8; 32],
            },
            tx_key: keypair.public_key.clone(),
            references: [parent1, parent2],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            extra: Vec::new(),
        };

        // Sign transaction
        let message = tx.signing_message();
        tx.sign(
            &message,
            keypair.private_key(),
            &keypair.public_key,
            &self.ring_members,
        )?;

        Ok(tx)
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_builder() {
        let keypair = keys::generate_keypair();

        // Create decoy ring members
        let mut ring = vec![keypair.public_key.clone()];
        for _ in 0..3 {
            ring.push(keys::generate_keypair().public_key);
        }

        let result = TransactionBuilder::new()
            .with_signer(keypair.clone())
            .add_input([1u8; 32], 0, keypair.private_key())
            .unwrap()
            .add_output(&keypair.public_key[..32], &keypair.public_key[..32], 1000)
            .unwrap()
            .with_ring_members(ring)
            .build([0u8; 32], [1u8; 32]);

        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
    }
}
