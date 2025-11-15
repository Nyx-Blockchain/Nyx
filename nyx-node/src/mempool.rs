// src/mempool.rs

//! Transaction mempool for pending transactions.

use nyx_core::transaction::Transaction;
use nyx_core::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Transaction mempool
#[derive(Clone)]
pub struct Mempool {
    /// Pending transactions
    transactions: Arc<RwLock<HashMap<Hash, Transaction>>>,

    /// Maximum mempool size
    max_size: usize,
}

impl Mempool {
    /// Creates a new mempool
    pub fn new(max_size: usize) -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    /// Adds a transaction to the mempool
    pub async fn add_transaction(&self, tx: Transaction) -> crate::Result<Hash> {
        let tx_id = tx.id();

        let mut txs = self.transactions.write().await;

        // Check if mempool is full
        if txs.len() >= self.max_size {
            return Err(crate::NodeError::MempoolError(
                "Mempool is full".to_string()
            ));
        }

        // Check if transaction already exists
        if txs.contains_key(&tx_id) {
            return Ok(tx_id); // Already in mempool
        }

        txs.insert(tx_id, tx);

        Ok(tx_id)
    }

    /// Gets a transaction from the mempool
    pub async fn get_transaction(&self, tx_id: &Hash) -> Option<Transaction> {
        let txs = self.transactions.read().await;
        txs.get(tx_id).cloned()
    }

    /// Removes a transaction from the mempool
    pub async fn remove_transaction(&self, tx_id: &Hash) -> Option<Transaction> {
        let mut txs = self.transactions.write().await;
        txs.remove(tx_id)
    }

    /// Gets all transactions in the mempool
    pub async fn get_all_transactions(&self) -> Vec<Transaction> {
        let txs = self.transactions.read().await;
        txs.values().cloned().collect()
    }

    /// Gets mempool size
    pub async fn size(&self) -> usize {
        let txs = self.transactions.read().await;
        txs.len()
    }

    /// Clears the mempool
    pub async fn clear(&self) {
        let mut txs = self.transactions.write().await;
        txs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyx_core::transaction::{TxInput, TxOutput};
    use nyx_crypto::ring::RingSignature;

    fn create_test_tx(nonce: u8) -> Transaction {
        Transaction::new(
            vec![TxInput {
                prev_tx: [nonce; 32],
                index: 0,
                key_image: [nonce; 32],
                ring_indices: vec![],
            }],
            vec![TxOutput {
                stealth_address: vec![nonce],
                amount_commitment: vec![],
                range_proof: vec![],
                ephemeral_pubkey: vec![],
            }],
            RingSignature {
                ring_members: vec![],
                signature: vec![nonce],
                key_image: [nonce; 32],
            },
            vec![nonce],
            [0u8; 32],
            [1u8; 32],
        )
    }

    #[tokio::test]
    async fn test_mempool_add() {
        let mempool = Mempool::new(100);
        let tx = create_test_tx(1);

        let tx_id = mempool.add_transaction(tx).await.unwrap();
        assert_eq!(mempool.size().await, 1);

        let retrieved = mempool.get_transaction(&tx_id).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_mempool_remove() {
        let mempool = Mempool::new(100);
        let tx = create_test_tx(1);

        let tx_id = mempool.add_transaction(tx).await.unwrap();
        assert_eq!(mempool.size().await, 1);

        mempool.remove_transaction(&tx_id).await;
        assert_eq!(mempool.size().await, 0);
    }

    #[tokio::test]
    async fn test_mempool_full() {
        let mempool = Mempool::new(2);

        mempool.add_transaction(create_test_tx(1)).await.unwrap();
        mempool.add_transaction(create_test_tx(2)).await.unwrap();

        let result = mempool.add_transaction(create_test_tx(3)).await;
        assert!(result.is_err());
    }
}
