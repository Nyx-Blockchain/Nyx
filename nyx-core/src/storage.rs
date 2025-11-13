// src/storage.rs

//! Storage layer for persisting DAG state and transactions.
//!
//! This module will eventually use a production database like RocksDB
//! for efficient key-value storage of transactions, DAG structure, and state.

use crate::errors::{NyxError, Result};
use crate::types::Hash;
use crate::transaction::Transaction;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory storage for development and testing
///
/// This will be replaced with a persistent database (RocksDB) in production.
/// For now, provides a simple in-memory implementation for testing the DAG logic.
#[derive(Clone)]
pub struct MemoryStorage {
    /// Transactions indexed by their hash
    transactions: Arc<RwLock<HashMap<Hash, Transaction>>>,

    /// Track which transactions have been confirmed
    confirmed: Arc<RwLock<HashMap<Hash, bool>>>,
}

impl MemoryStorage {
    /// Creates a new empty in-memory storage
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            confirmed: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Stores a transaction
    ///
    /// # Arguments
    /// * `tx` - The transaction to store
    ///
    /// # Returns
    /// The transaction hash if successful
    pub fn store_transaction(&self, tx: Transaction) -> Result<Hash> {
        let tx_id = tx.id();

        let mut txs = self.transactions.write()
            .map_err(|e| NyxError::StorageError(format!("Lock poisoned: {}", e)))?;

        if txs.contains_key(&tx_id) {
            return Err(NyxError::StorageError(
                "Transaction already exists".to_string()
            ));
        }

        txs.insert(tx_id, tx);
        Ok(tx_id)
    }

    /// Retrieves a transaction by hash
    ///
    /// # Arguments
    /// * `tx_hash` - The hash of the transaction to retrieve
    ///
    /// # Returns
    /// The transaction if found, error otherwise
    pub fn get_transaction(&self, tx_hash: &Hash) -> Result<Transaction> {
        let txs = self.transactions.read()
            .map_err(|e| NyxError::StorageError(format!("Lock poisoned: {}", e)))?;

        txs.get(tx_hash)
            .cloned()
            .ok_or_else(|| NyxError::TransactionNotFound(
                format!("Transaction not found: {:?}", tx_hash)
            ))
    }

    /// Marks a transaction as confirmed
    pub fn mark_confirmed(&self, tx_hash: &Hash) -> Result<()> {
        let mut confirmed = self.confirmed.write()
            .map_err(|e| NyxError::StorageError(format!("Lock poisoned: {}", e)))?;

        confirmed.insert(*tx_hash, true);
        Ok(())
    }

    /// Checks if a transaction is confirmed
    pub fn is_confirmed(&self, tx_hash: &Hash) -> Result<bool> {
        let confirmed = self.confirmed.read()
            .map_err(|e| NyxError::StorageError(format!("Lock poisoned: {}", e)))?;

        Ok(*confirmed.get(tx_hash).unwrap_or(&false))
    }

    /// Returns the total number of stored transactions
    pub fn transaction_count(&self) -> Result<usize> {
        let txs = self.transactions.read()
            .map_err(|e| NyxError::StorageError(format!("Lock poisoned: {}", e)))?;

        Ok(txs.len())
    }

    /// Checks if a transaction exists
    pub fn has_transaction(&self, tx_hash: &Hash) -> Result<bool> {
        let txs = self.transactions.read()
            .map_err(|e| NyxError::StorageError(format!("Lock poisoned: {}", e)))?;

        Ok(txs.contains_key(tx_hash))
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxInput, TxOutput};
    use nyx_crypto::RingSignature;

    fn create_test_tx() -> Transaction {
        Transaction::new(
            vec![TxInput {
                prev_tx: [1u8; 32],
                index: 0,
                key_image: [0u8; 32],
                ring_indices: vec![],
            }],
            vec![TxOutput {
                stealth_address: vec![],
                amount_commitment: vec![],
                range_proof: vec![],
            }],
            RingSignature {
                signature_data: vec![],
                ring_size: 16,
            },
            vec![],
            [0u8; 32],
            [1u8; 32],
        )
    }

    #[test]
    fn test_store_and_retrieve() {
        let storage = MemoryStorage::new();
        let tx = create_test_tx();
        let expected_id = tx.id();

        let stored_id = storage.store_transaction(tx.clone()).unwrap();
        assert_eq!(stored_id, expected_id);

        let retrieved = storage.get_transaction(&stored_id).unwrap();
        assert_eq!(retrieved.id(), expected_id);
    }

    #[test]
    fn test_confirmed_tracking() {
        let storage = MemoryStorage::new();
        let tx = create_test_tx();
        let tx_id = storage.store_transaction(tx).unwrap();

        assert!(!storage.is_confirmed(&tx_id).unwrap());

        storage.mark_confirmed(&tx_id).unwrap();

        assert!(storage.is_confirmed(&tx_id).unwrap());
    }

    #[test]
    fn test_transaction_count() {
        let storage = MemoryStorage::new();
        assert_eq!(storage.transaction_count().unwrap(), 0);

        storage.store_transaction(create_test_tx()).unwrap();
        assert_eq!(storage.transaction_count().unwrap(), 1);
    }
}
