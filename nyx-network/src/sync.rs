// src/sync.rs

//! DAG synchronization manager.
//!
//! Handles synchronization of the transaction DAG between nodes:
//! - Requests missing transactions from peers
//! - Responds to sync requests with transaction batches
//! - Maintains sync state and progress tracking

use crate::errors::Result;
use crate::message::{Message, MessageType};
use crate::peer::Peer;
use crate::MAX_SYNC_BATCH_SIZE;
use nyx_core::Transaction;
use nyx_core::dag::DagProcessor;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Manages DAG synchronization with peers
pub struct SyncManager {
    /// DAG processor for local state
    dag: Arc<RwLock<DagProcessor>>,

    /// Sync progress tracking
    sync_state: Arc<RwLock<SyncState>>,
}

/// Current synchronization state
#[derive(Debug, Clone)]
pub struct SyncState {
    /// Current DAG height
    pub current_height: u64,

    /// Target height to sync to
    pub target_height: Option<u64>,

    /// Whether we're currently syncing
    pub is_syncing: bool,

    /// Number of transactions synced in current session
    pub synced_count: usize,
}

impl SyncManager {
    /// Creates a new sync manager
    pub fn new(dag: Arc<RwLock<DagProcessor>>) -> Self {
        Self {
            dag,
            sync_state: Arc::new(RwLock::new(SyncState {
                current_height: 0,
                target_height: None,
                is_syncing: false,
                synced_count: 0,
            })),
        }
    }

    /// Handles an incoming sync request
    pub async fn handle_sync_request(
        &self,
        from_height: u64,
        peer: &mut Peer,
        stream: &mut OwnedWriteHalf,
    ) -> Result<()> {
        debug!("Handling sync request from peer {:?} from height {}", peer.id, from_height);

        // Get transactions from DAG
        let dag = self.dag.read().await;
        let transactions = self.get_transactions_from_height(&dag, from_height).await?;

        debug!("Found {} transactions to sync", transactions.len());

        // Send transactions in batches
        for batch in transactions.chunks(MAX_SYNC_BATCH_SIZE) {
            let response = Message::new(MessageType::SyncResponse {
                transactions: batch.to_vec(),
            });

            peer.send_message(stream, &response).await?;
        }

        info!("Sent {} transactions to peer {:?}", transactions.len(), peer.id);

        Ok(())
    }

    /// Handles an incoming sync response
    pub async fn handle_sync_response(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<()> {
        let mut state = self.sync_state.write().await;
        let dag = self.dag.write().await;

        debug!("Processing sync response with {} transactions", transactions.len());

        let mut added = 0;
        for tx in transactions {
            match dag.add_transaction(tx) {
                Ok(_) => {
                    added += 1;
                    state.synced_count += 1;
                }
                Err(e) => {
                    warn!("Failed to add synced transaction: {}", e);
                }
            }
        }

        info!("Added {} transactions from sync response", added);

        Ok(())
    }

    /// Requests sync from a peer
    pub async fn request_sync(
        &self,
        from_height: u64,
        peer: &mut Peer,
        stream: &mut OwnedWriteHalf,
    ) -> Result<()> {
        let mut state = self.sync_state.write().await;
        state.is_syncing = true;
        state.current_height = from_height;

        let request = Message::new(MessageType::SyncRequest { from_height });
        peer.send_message(stream, &request).await?;

        info!("Requested sync from peer {:?} starting at height {}", peer.id, from_height);

        Ok(())
    }

    /// Gets transactions from a specific height
    async fn get_transactions_from_height(
        &self,
        _dag: &DagProcessor,
        _from_height: u64,
    ) -> Result<Vec<Transaction>> {
        // In a real implementation, this would query the DAG for transactions
        // For now, we return an empty vec as the DAG doesn't have height tracking yet

        // TODO: Implement proper height-based transaction retrieval
        // This would require adding height/ordering to the DAG

        Ok(Vec::new())
    }

    /// Starts a sync process
    pub async fn start_sync(&self, target_height: u64) {
        let mut state = self.sync_state.write().await;
        state.is_syncing = true;
        state.target_height = Some(target_height);
        state.synced_count = 0;

        info!("Starting sync to height {}", target_height);
    }

    /// Completes the sync process
    pub async fn complete_sync(&self) {
        let mut state = self.sync_state.write().await;
        state.is_syncing = false;

        info!("Sync completed. Synced {} transactions", state.synced_count);
    }

    /// Gets current sync state
    pub async fn get_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    /// Checks if currently syncing
    pub async fn is_syncing(&self) -> bool {
        self.sync_state.read().await.is_syncing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyx_core::storage::MemoryStorage;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);
        let dag_arc = Arc::new(RwLock::new(dag));

        let sync = SyncManager::new(dag_arc);

        assert!(!sync.is_syncing().await);
    }

    #[tokio::test]
    async fn test_start_sync() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);
        let dag_arc = Arc::new(RwLock::new(dag));

        let sync = SyncManager::new(dag_arc);

        sync.start_sync(1000).await;

        assert!(sync.is_syncing().await);

        let state = sync.get_state().await;
        assert_eq!(state.target_height, Some(1000));
    }

    #[tokio::test]
    async fn test_complete_sync() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);
        let dag_arc = Arc::new(RwLock::new(dag));

        let sync = SyncManager::new(dag_arc);

        sync.start_sync(1000).await;
        assert!(sync.is_syncing().await);

        sync.complete_sync().await;
        assert!(!sync.is_syncing().await);
    }
}
