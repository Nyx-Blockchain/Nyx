// src/dag.rs

//! DAG (Directed Acyclic Graph) transaction processing layer.
//!
//! Implements the core DAG logic from the Nyx whitepaper including:
//! - Transaction confirmation scoring
//! - Conflict resolution
//! - Finality determination

use crate::errors::{NyxError, Result};
use crate::types::Hash;
use crate::transaction::Transaction;
use crate::storage::MemoryStorage;
use crate::{CONFIRMATION_THRESHOLD, SCORE_DECAY_FACTOR};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Represents the state of a transaction in the DAG
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxState {
    /// Transaction is pending confirmation
    Pending,
    /// Transaction has reached confirmation threshold
    Confirmed,
    /// Transaction is finalized by a PoS snapshot
    Finalized,
    /// Transaction is conflicted (double-spend detected)
    Conflicted,
}

/// DAG processor managing the transaction graph
#[derive(Clone)]
pub struct DagProcessor {
    /// Storage backend
    storage: MemoryStorage,

    /// Transaction confirmation scores
    scores: Arc<RwLock<HashMap<Hash, f64>>>,

    /// Transaction states
    states: Arc<RwLock<HashMap<Hash, TxState>>>,

    /// Children map: tx_hash -> set of children hashes
    children: Arc<RwLock<HashMap<Hash, HashSet<Hash>>>>,

    /// Current tips (unconfirmed transactions with no children)
    tips: Arc<RwLock<HashSet<Hash>>>,
}

impl DagProcessor {
    /// Creates a new DAG processor with the given storage
    pub fn new(storage: MemoryStorage) -> Self {
        Self {
            storage,
            scores: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            children: Arc::new(RwLock::new(HashMap::new())),
            tips: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Adds a transaction to the DAG
    ///
    /// # Arguments
    /// * `tx` - The transaction to add
    ///
    /// # Returns
    /// Transaction hash if successful
    pub fn add_transaction(&self, tx: Transaction) -> Result<Hash> {
        // Validate transaction structure
        if !tx.validate_structure() {
            return Err(NyxError::InvalidTransaction(
                "Invalid transaction structure".to_string()
            ));
        }

        // Check that parent transactions exist
        for parent_hash in &tx.references {
            if !self.storage.has_transaction(parent_hash)? {
                return Err(NyxError::InvalidParent(
                    format!("Parent transaction not found: {:?}", parent_hash)
                ));
            }
        }

        // Store transaction
        let tx_hash = self.storage.store_transaction(tx.clone())?;

        // Initialize score and state
        {
            let mut scores = self.scores.write()
                .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;
            scores.insert(tx_hash, 1.0);
        }

        {
            let mut states = self.states.write()
                .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;
            states.insert(tx_hash, TxState::Pending);
        }

        // Update parent-child relationships
        self.update_children(&tx_hash, &tx.references)?;

        // Update tips
        self.update_tips(&tx_hash, &tx.references)?;

        // Update confirmation scores for ancestors
        self.update_scores(&tx_hash)?;

        Ok(tx_hash)
    }

    /// Gets the confirmation score of a transaction
    ///
    /// Score calculation from whitepaper:
    /// Score(T) = 1 + Σ(Score(Di) × decay_factor)
    /// where Di are direct descendants
    pub fn get_score(&self, tx_hash: &Hash) -> Result<f64> {
        let scores = self.scores.read()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;

        Ok(*scores.get(tx_hash).unwrap_or(&0.0))
    }

    /// Gets the state of a transaction
    pub fn get_state(&self, tx_hash: &Hash) -> Result<TxState> {
        let states = self.states.read()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;

        Ok(*states.get(tx_hash).unwrap_or(&TxState::Pending))
    }

    /// Checks if a transaction has reached the confirmation threshold
    pub fn is_confirmed(&self, tx_hash: &Hash) -> Result<bool> {
        let score = self.get_score(tx_hash)?;
        let state = self.get_state(tx_hash)?;

        Ok(score >= CONFIRMATION_THRESHOLD && state != TxState::Conflicted)
    }

    /// Gets the current tips (unconfirmed transactions with no children)
    pub fn get_tips(&self) -> Result<Vec<Hash>> {
        let tips = self.tips.read()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;

        Ok(tips.iter().copied().collect())
    }

    /// Updates parent-child relationships
    fn update_children(&self, tx_hash: &Hash, parent_hashes: &[Hash; 2]) -> Result<()> {
        let mut children = self.children.write()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;

        for parent_hash in parent_hashes {
            children.entry(*parent_hash)
                .or_insert_with(HashSet::new)
                .insert(*tx_hash);
        }

        Ok(())
    }

    /// Updates the tips set when a new transaction is added
    fn update_tips(&self, tx_hash: &Hash, parent_hashes: &[Hash; 2]) -> Result<()> {
        let mut tips = self.tips.write()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;

        // Remove parents from tips (they now have children)
        for parent_hash in parent_hashes {
            tips.remove(parent_hash);
        }

        // Add this transaction as a new tip
        tips.insert(*tx_hash);

        Ok(())
    }

    /// Recursively updates confirmation scores for ancestors
    fn update_scores(&self, tx_hash: &Hash) -> Result<()> {
        let tx = self.storage.get_transaction(tx_hash)?;

        // Update scores for both parents
        for parent_hash in &tx.references {
            self.update_score_recursive(parent_hash)?;
        }

        Ok(())
    }

    /// Recursively calculates and updates the score for a transaction
    fn update_score_recursive(&self, tx_hash: &Hash) -> Result<f64> {
        // Base score is 1
        let mut score = 1.0;

        // Get children
        let children_set = {
            let children = self.children.read()
                .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;
            children.get(tx_hash).cloned().unwrap_or_default()
        };

        // Add weighted scores from all children
        for child_hash in children_set {
            let child_score = self.get_score(&child_hash)?;
            score += child_score * SCORE_DECAY_FACTOR;
        }

        // Update stored score
        {
            let mut scores = self.scores.write()
                .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;
            scores.insert(*tx_hash, score);
        }

        // Update state if threshold reached
        if score >= CONFIRMATION_THRESHOLD {
            let mut states = self.states.write()
                .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;
            if states.get(tx_hash) == Some(&TxState::Pending) {
                states.insert(*tx_hash, TxState::Confirmed);
            }
        }

        Ok(score)
    }

    /// Marks a transaction as finalized (by PoS snapshot)
    pub fn finalize_transaction(&self, tx_hash: &Hash) -> Result<()> {
        let mut states = self.states.write()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;

        states.insert(*tx_hash, TxState::Finalized);
        self.storage.mark_confirmed(tx_hash)?;

        Ok(())
    }

    /// Returns statistics about the DAG
    pub fn get_stats(&self) -> Result<DagStats> {
        let scores = self.scores.read()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;
        let states = self.states.read()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;
        let tips = self.tips.read()
            .map_err(|e| NyxError::DagError(format!("Lock poisoned: {}", e)))?;

        let total = scores.len();
        let pending = states.values().filter(|s| **s == TxState::Pending).count();
        let confirmed = states.values().filter(|s| **s == TxState::Confirmed).count();
        let finalized = states.values().filter(|s| **s == TxState::Finalized).count();

        Ok(DagStats {
            total_transactions: total,
            pending_transactions: pending,
            confirmed_transactions: confirmed,
            finalized_transactions: finalized,
            current_tips: tips.len(),
        })
    }
}

/// Statistics about the DAG state
#[derive(Debug, Clone)]
pub struct DagStats {
    /// Total number of transactions
    pub total_transactions: usize,
    /// Pending transactions
    pub pending_transactions: usize,
    /// Confirmed transactions
    pub confirmed_transactions: usize,
    /// Finalized transactions
    pub finalized_transactions: usize,
    /// Current number of tips
    pub current_tips: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxInput, TxOutput};
    use nyx_crypto::RingSignature;


    fn create_test_tx(parent1: Hash, parent2: Hash, nonce: u8) -> Transaction {
        Transaction::new(
            vec![TxInput {
                prev_tx: [nonce; 32],
                index: 0,
                key_image: [0u8; 32],
                ring_indices: vec![],
            }],
            vec![TxOutput {
                stealth_address: vec![nonce],
                amount_commitment: vec![],
                range_proof: vec![],
            }],
            RingSignature {
                signature_data: vec![nonce],
                ring_size: 16,
            },
            vec![nonce],
            parent1,
            parent2,
        )
    }

    #[test]
    fn test_genesis_transaction() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        // Create two DIFFERENT genesis transactions by using different nonce values
        let genesis1 = create_test_tx([0u8; 32], [0u8; 32], 1);
        let genesis2 = create_test_tx([0u8; 32], [0u8; 32], 2);

        let gen1_hash = storage.store_transaction(genesis1.clone()).unwrap();
        let gen2_hash = storage.store_transaction(genesis2.clone()).unwrap();

        // Verify they have different hashes
        assert_ne!(gen1_hash, gen2_hash);

        // Now create a transaction referencing both
        let tx = create_test_tx(gen1_hash, gen2_hash, 3);
        let tx_hash = dag.add_transaction(tx).unwrap();

        let score = dag.get_score(&tx_hash).unwrap();
        assert_eq!(score, 1.0);

        let state = dag.get_state(&tx_hash).unwrap();
        assert_eq!(state, TxState::Pending);
    }

    #[test]
    fn test_tip_tracking() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        let genesis = create_test_tx([0u8; 32], [0u8; 32], 1);
        storage.store_transaction(genesis.clone()).unwrap();

        let tips = dag.get_tips().unwrap();
        assert_eq!(tips.len(), 0); // No tips yet (genesis not added through dag)
    }
}
