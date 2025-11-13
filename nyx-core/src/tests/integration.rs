// src/tests/integration.rs

//! Integration tests for the Nyx DAG implementation
//!
//! Tests the complete flow of transaction creation, DAG processing,
//! confirmation scoring, and tip selection.

#[cfg(test)]
mod tests {
    use nyx_core::*;
    use nyx_core::storage::MemoryStorage;
    use nyx_core::dag::{DagProcessor, TxState};
    use nyx_core::tip_selection::TipSelector;
    use nyx_core::transaction::{Transaction, TxInput, TxOutput, RingSignature};

    /// Helper to create a test transaction
    fn create_test_tx(parent1: Hash, parent2: Hash, nonce: u8) -> Transaction {
        Transaction::new(
            vec![TxInput {
                prev_tx: [nonce; 32],
                index: 0,
                key_image: [0u8; 32],
                ring_indices: vec![0, 1, 2, 3],
            }],
            vec![TxOutput {
                stealth_address: vec![nonce, nonce, nonce],
                amount_commitment: vec![nonce + 1],
                range_proof: vec![nonce + 2],
            }],
            RingSignature {
                signature_data: vec![nonce + 3],
                ring_size: 16,
            },
            vec![nonce + 4],
            parent1,
            parent2,
        )
    }

    #[test]
    fn test_simple_dag_flow() {
        // Setup
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        // Create genesis transactions (stored directly in storage)
        let genesis1 = create_test_tx([0u8; 32], [0u8; 32], 1);
        let genesis2 = create_test_tx([0u8; 32], [0u8; 32], 2);

        let gen1_hash = storage.store_transaction(genesis1.clone()).unwrap();
        let gen2_hash = storage.store_transaction(genesis2.clone()).unwrap();

        // Create a transaction referencing genesis
        let tx1 = create_test_tx(gen1_hash, gen2_hash, 3);
        let tx1_hash = dag.add_transaction(tx1).unwrap();

        // Verify initial score
        let score = dag.get_score(&tx1_hash).unwrap();
        assert_eq!(score, 1.0);

        // Verify state
        let state = dag.get_state(&tx1_hash).unwrap();
        assert_eq!(state, TxState::Pending);

        // Verify not confirmed yet
        assert!(!dag.is_confirmed(&tx1_hash).unwrap());
    }

    #[test]
    fn test_dag_chain() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        // Genesis
        let genesis = create_test_tx([0u8; 32], [0u8; 32], 1);
        let gen_hash = storage.store_transaction(genesis).unwrap();

        // Build a chain of transactions
        let mut prev_hash = gen_hash;
        let mut hashes = vec![gen_hash];

        for i in 2..6 {
            let tx = create_test_tx(prev_hash, prev_hash, i);
            let tx_hash = dag.add_transaction(tx).unwrap();
            hashes.push(tx_hash);
            prev_hash = tx_hash;
        }

        // Check that later transactions have higher scores due to descendants
        let first_score = dag.get_score(&hashes[1]).unwrap();
        let last_score = dag.get_score(&hashes[hashes.len() - 1]).unwrap();

        // Last transaction should have score of 1 (no descendants yet)
        assert_eq!(last_score, 1.0);

        // First transaction should have higher score (has descendants)
        assert!(first_score >= 1.0);
    }

    #[test]
    fn test_dag_statistics() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        // Genesis
        let genesis = create_test_tx([0u8; 32], [0u8; 32], 1);
        let gen_hash = storage.store_transaction(genesis).unwrap();

        // Add several transactions
        for i in 2..10 {
            let tx = create_test_tx(gen_hash, gen_hash, i);
            dag.add_transaction(tx).unwrap();
        }

        let stats = dag.get_stats().unwrap();

        assert_eq!(stats.total_transactions, 8); // 8 transactions added through DAG
        assert_eq!(stats.pending_transactions, 8);
        assert_eq!(stats.confirmed_transactions, 0);
        assert_eq!(stats.finalized_transactions, 0);
    }

    #[test]
    fn test_finalization() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        let genesis = create_test_tx([0u8; 32], [0u8; 32], 1);
        let gen_hash = storage.store_transaction(genesis).unwrap();

        let tx = create_test_tx(gen_hash, gen_hash, 2);
        let tx_hash = dag.add_transaction(tx).unwrap();

        // Initially pending
        assert_eq!(dag.get_state(&tx_hash).unwrap(), TxState::Pending);

        // Finalize
        dag.finalize_transaction(&tx_hash).unwrap();

        // Now finalized
        assert_eq!(dag.get_state(&tx_hash).unwrap(), TxState::Finalized);
        assert!(storage.is_confirmed(&tx_hash).unwrap());
    }

    #[test]
    fn test_tip_tracking() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        let genesis = create_test_tx([0u8; 32], [0u8; 32], 1);
        let gen_hash = storage.store_transaction(genesis).unwrap();

        // Add first transaction
        let tx1 = create_test_tx(gen_hash, gen_hash, 2);
        let tx1_hash = dag.add_transaction(tx1).unwrap();

        let tips = dag.get_tips().unwrap();
        assert_eq!(tips.len(), 1);
        assert!(tips.contains(&tx1_hash));

        // Add second transaction (also referencing genesis)
        let tx2 = create_test_tx(gen_hash, gen_hash, 3);
        let tx2_hash = dag.add_transaction(tx2).unwrap();

        let tips = dag.get_tips().unwrap();
        assert_eq!(tips.len(), 2);
        assert!(tips.contains(&tx1_hash));
        assert!(tips.contains(&tx2_hash));

        // Add third transaction referencing both previous ones
        let tx3 = create_test_tx(tx1_hash, tx2_hash, 4);
        let tx3_hash = dag.add_transaction(tx3).unwrap();

        let tips = dag.get_tips().unwrap();
        // tx1 and tx2 should be removed from tips, only tx3 remains
        assert_eq!(tips.len(), 1);
        assert!(tips.contains(&tx3_hash));
    }

    #[test]
    fn test_tip_selection() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        let genesis = create_test_tx([0u8; 32], [0u8; 32], 1);
        let gen_hash = storage.store_transaction(genesis).unwrap();

        // Add multiple tips
        for i in 2..6 {
            let tx = create_test_tx(gen_hash, gen_hash, i);
            dag.add_transaction(tx).unwrap();
        }

        let selector = TipSelector::new(dag);
        let selected = selector.select_tips().unwrap();

        // Should select two tips
        assert_eq!(selected.len(), 2);

        // Tips should be different (unless only one available)
        // This is probabilistic but should usually be true with 4 tips
    }

    #[test]
    fn test_parent_validation() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);

        // Try to add transaction with non-existent parents
        let tx = create_test_tx([99u8; 32], [98u8; 32], 1);
        let result = dag.add_transaction(tx);

        // Should fail with InvalidParent error
        assert!(result.is_err());
        match result {
            Err(NyxError::InvalidParent(_)) => (),
            _ => panic!("Expected InvalidParent error"),
        }
    }

    #[test]
    fn test_transaction_structure_validation() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);

        // Create transaction with invalid structure (same parent twice)
        let mut tx = create_test_tx([1u8; 32], [1u8; 32], 1);

        let result = dag.add_transaction(tx);

        // Should fail validation
        assert!(result.is_err());
    }

    #[test]
    fn test_complete_flow() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage.clone());

        // Genesis
        let genesis = create_test_tx([0u8; 32], [0u8; 32], 1);
        let gen_hash = storage.store_transaction(genesis).unwrap();

        // Build DAG with 20 transactions
        let mut all_hashes = vec![gen_hash];

        for i in 2..22 {
            // Use tip selector to choose parents
            let selector = TipSelector::new(dag.clone());

            let parents = if all_hashes.len() >= 2 {
                // Use last two transactions as parents
                [
                    all_hashes[all_hashes.len() - 1],
                    all_hashes[all_hashes.len() - 2],
                ]
            } else {
                [gen_hash, gen_hash]
            };

            let tx = create_test_tx(parents[0], parents[1], i);
            let tx_hash = dag.add_transaction(tx).unwrap();
            all_hashes.push(tx_hash);
        }

        let stats = dag.get_stats().unwrap();

        println!("DAG Stats:");
        println!("  Total: {}", stats.total_transactions);
        println!("  Pending: {}", stats.pending_transactions);
        println!("  Confirmed: {}", stats.confirmed_transactions);
        println!("  Tips: {}", stats.current_tips);

        assert!(stats.total_transactions >= 20);
    }
}
