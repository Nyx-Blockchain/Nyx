// src/builder.rs

//! Transaction builder for creating privacy-preserving transactions.

use crate::account::Account;
use crate::errors::{WalletError, Result};
use nyx_core::transaction::{Transaction, TxInput, TxOutput, RingSignature};
use nyx_core::Hash;
use nyx_crypto::{ring, stealth};

/// UTXO (Unspent Transaction Output)
#[derive(Clone, Debug)]
pub struct Utxo {
    /// Transaction hash
    pub tx_hash: Hash,

    /// Output index
    pub index: u32,

    /// Amount
    pub amount: u64,

    /// Key image (for preventing double-spends)
    pub key_image: Vec<u8>,
}

/// Transaction builder
pub struct TransactionBuilder {
    /// Inputs to spend
    inputs: Vec<Utxo>,

    /// Outputs to create
    outputs: Vec<(Vec<u8>, Vec<u8>, u64)>, // (view_pub, spend_pub, amount)

    /// Ring members for privacy (mock decoys)
    ring_members: Vec<Vec<u8>>,

    /// Sender account
    sender: Option<Account>,
}

impl TransactionBuilder {
    /// Creates a new transaction builder
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            ring_members: Vec::new(),
            sender: None,
        }
    }

    /// Sets the sender account
    pub fn sender(mut self, account: Account) -> Self {
        self.sender = Some(account);
        self
    }

    /// Adds an input to spend
    pub fn add_input(mut self, utxo: Utxo) -> Self {
        self.inputs.push(utxo);
        self
    }

    /// Adds an output
    ///
    /// # Arguments
    /// * `view_public` - Recipient's view public key
    /// * `spend_public` - Recipient's spend public key
    /// * `amount` - Amount to send
    pub fn add_output(
        mut self,
        view_public: Vec<u8>,
        spend_public: Vec<u8>,
        amount: u64,
    ) -> Self {
        self.outputs.push((view_public, spend_public, amount));
        self
    }

    /// Adds ring members for privacy (decoys)
    pub fn with_ring_members(mut self, members: Vec<Vec<u8>>) -> Self {
        self.ring_members = members;
        self
    }

    /// Builds and signs the transaction
    ///
    /// # Arguments
    /// * `parent1` - First parent transaction hash (for DAG)
    /// * `parent2` - Second parent transaction hash (for DAG)
    pub fn build(self, parent1: Hash, parent2: Hash) -> Result<Transaction> {
        let sender = self.sender
            .ok_or_else(|| WalletError::TransactionBuildError("No sender set".to_string()))?;

        if self.inputs.is_empty() {
            return Err(WalletError::TransactionBuildError(
                "No inputs provided".to_string()
            ));
        }

        if self.outputs.is_empty() {
            return Err(WalletError::TransactionBuildError(
                "No outputs provided".to_string()
            ));
        }

        // Build inputs
        let mut tx_inputs = Vec::new();
        for utxo in &self.inputs {
            let input = TxInput {
                prev_tx: utxo.tx_hash,
                index: utxo.index,
                key_image: utxo.key_image.clone(),
                ring_indices: vec![0, 1, 2, 3], // Mock indices
            };
            tx_inputs.push(input);
        }

        // Build outputs with stealth addresses
        let mut tx_outputs = Vec::new();
        for (view_pub, spend_pub, amount) in &self.outputs {
            // Generate stealth address
            let random = stealth::generate_random_ephemeral();
            let (stealth_address, _ephemeral_pubkey) = stealth::generate_stealth_address(
                view_pub,
                spend_pub,
                &random,
            )?;

            // Mock amount commitment (in production, use Pedersen commitments)
            let amount_commitment = Self::mock_amount_commitment(*amount);

            // Mock range proof
            let range_proof = vec![0u8; 100]; // Placeholder

            let output = TxOutput {
                stealth_address,
                amount_commitment,
                range_proof,
            };
            tx_outputs.push(output);
        }

        // Generate ring signature
        let mut ring = vec![sender.spend_public_key().to_vec()];
        ring.extend(self.ring_members.clone());

        // If not enough ring members, add mocks
        while ring.len() < 16 {
            ring.push(vec![0u8; 32]);
        }

        let message = b"transaction_data"; // In production, serialize tx data
        let ring_sig = ring::generate_ring_signature(
            message,
            sender.spend_private_key(),
            sender.spend_public_key(),
            &ring,
        )?;

        // Build transaction
        let tx = Transaction::new(
            tx_inputs,
            tx_outputs,
            ring_sig,
            sender.spend_public_key().to_vec(),
            parent1,
            parent2,
        );

        Ok(tx)
    }

    /// Mock amount commitment (in production, use Pedersen commitment)
    fn mock_amount_commitment(amount: u64) -> Vec<u8> {
        let amount_bytes = amount.to_le_bytes();
        nyx_crypto::hash::blake3_hash(&amount_bytes).to_vec()
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

    fn create_mock_utxo(amount: u64) -> Utxo {
        Utxo {
            tx_hash: [1u8; 32],
            index: 0,
            amount,
            key_image: vec![2u8; 32],
        }
    }

    #[test]
    fn test_builder_creation() {
        let builder = TransactionBuilder::new();
        assert!(builder.inputs.is_empty());
        assert!(builder.outputs.is_empty());
    }

    #[test]
    fn test_builder_add_input() {
        let utxo = create_mock_utxo(1000);
        let builder = TransactionBuilder::new()
            .add_input(utxo);

        assert_eq!(builder.inputs.len(), 1);
    }

    #[test]
    fn test_builder_add_output() {
        let builder = TransactionBuilder::new()
            .add_output(vec![1u8; 32], vec![2u8; 32], 500);

        assert_eq!(builder.outputs.len(), 1);
    }

    #[test]
    fn test_builder_build() {
        let account = Account::generate();
        let utxo = create_mock_utxo(1000);

        let tx = TransactionBuilder::new()
            .sender(account.clone())
            .add_input(utxo)
            .add_output(
                account.view_public_key().to_vec(),
                account.spend_public_key().to_vec(),
                900
            )
            .build([0u8; 32], [1u8; 32])
            .unwrap();

        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
    }

    #[test]
    fn test_builder_no_sender() {
        let utxo = create_mock_utxo(1000);

        let result = TransactionBuilder::new()
            .add_input(utxo)
            .add_output(vec![1u8; 32], vec![2u8; 32], 500)
            .build([0u8; 32], [1u8; 32]);

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_no_inputs() {
        let account = Account::generate();

        let result = TransactionBuilder::new()
            .sender(account)
            .add_output(vec![1u8; 32], vec![2u8; 32], 500)
            .build([0u8; 32], [1u8; 32]);

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_no_outputs() {
        let account = Account::generate();
        let utxo = create_mock_utxo(1000);

        let result = TransactionBuilder::new()
            .sender(account)
            .add_input(utxo)
            .build([0u8; 32], [1u8; 32]);

        assert!(result.is_err());
    }

    #[test]
    fn test_mock_amount_commitment() {
        let commitment1 = TransactionBuilder::mock_amount_commitment(1000);
        let commitment2 = TransactionBuilder::mock_amount_commitment(1000);

        assert_eq!(commitment1, commitment2);
        assert_eq!(commitment1.len(), 32);
    }

    #[test]
    fn test_builder_with_ring_members() {
        let account = Account::generate();
        let utxo = create_mock_utxo(1000);

        let ring_members = vec![
            vec![3u8; 32],
            vec![4u8; 32],
            vec![5u8; 32],
        ];

        let tx = TransactionBuilder::new()
            .sender(account.clone())
            .add_input(utxo)
            .add_output(
                account.view_public_key().to_vec(),
                account.spend_public_key().to_vec(),
                900
            )
            .with_ring_members(ring_members)
            .build([0u8; 32], [1u8; 32])
            .unwrap();

        assert_eq!(tx.inputs.len(), 1);
    }

    #[test]
    fn test_builder_multiple_inputs_outputs() {
        let account = Account::generate();

        let utxo1 = create_mock_utxo(500);
        let utxo2 = create_mock_utxo(500);

        let tx = TransactionBuilder::new()
            .sender(account.clone())
            .add_input(utxo1)
            .add_input(utxo2)
            .add_output(
                account.view_public_key().to_vec(),
                account.spend_public_key().to_vec(),
                400
            )
            .add_output(
                account.view_public_key().to_vec(),
                account.spend_public_key().to_vec(),
                500
            )
            .build([0u8; 32], [1u8; 32])
            .unwrap();

        assert_eq!(tx.inputs.len(), 2);
        assert_eq!(tx.outputs.len(), 2);
    }
}
