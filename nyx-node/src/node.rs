// src/node.rs

//! Main node orchestration.

use crate::{NodeConfig, Mempool, RpcServer, Result};
use nyx_core::storage::MemoryStorage;
use nyx_core::dag::DagProcessor;
use nyx_network::{Node as NetworkNode, NodeConfig as NetConfig};
use nyx_wallet::Wallet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Main Nyx blockchain node
pub struct NyxNode {
    /// Node configuration
    config: NodeConfig,

    /// DAG processor
    dag: Arc<RwLock<DagProcessor>>,

    /// Network node
    network: Arc<RwLock<NetworkNode>>,

    /// Mempool
    mempool: Mempool,

    /// Built-in wallet
    wallet: Option<Wallet>,
}

impl NyxNode {
    /// Creates a new Nyx node
    pub async fn new(config: NodeConfig) -> Result<Self> {
        info!("Initializing Nyx node...");

        // Initialize storage and DAG
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);
        let dag = Arc::new(RwLock::new(dag));

        // Initialize network
        let net_config = NetConfig {
            listen_addr: config.network.listen_addr,
            max_peers: config.network.max_peers,
            min_peers: 8,
            bootstrap_peers: config.network.bootstrap_peers.clone(),
            node_id: vec![0u8; 32],
        };

        let network = NetworkNode::new(net_config).await?;
        let network = Arc::new(RwLock::new(network));

        // Initialize mempool
        let mempool = Mempool::new(1000);

        // Initialize wallet if enabled
        let wallet = if config.wallet.enabled {
            let mut wallet = Wallet::with_default_account();
            // Mock: add some balance for testing
            wallet.scan_outputs().ok();
            Some(wallet)
        } else {
            None
        };

        info!("✅ Nyx node initialized");

        Ok(Self {
            config,
            dag,
            network,
            mempool,
            wallet,
        })
    }

    /// Starts the node
    pub async fn start(self) -> Result<()> {
        info!("🚀 Starting Nyx node...");

        let node_arc = Arc::new(RwLock::new(self));

        // Start RPC server if enabled
        if node_arc.read().await.config.rpc.enabled {
            let rpc_addr = node_arc.read().await.config.rpc.listen_addr;
            let rpc_server = RpcServer::new(rpc_addr, node_arc.clone());

            tokio::spawn(async move {
                if let Err(e) = rpc_server.start().await {
                    tracing::error!("RPC server error: {}", e);
                }
            });
        }

        // Start network node
        // TODO: Use this handle to manage the network task
        let _network = node_arc.read().await.network.clone();
        tokio::spawn(async move {
            // Note: This will block, so run in separate task
            // In production, we'd handle this better
        });

        info!("✅ Nyx node started successfully");

        // Keep running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    /// Gets mempool size
    pub async fn mempool_size(&self) -> usize {
        self.mempool.size().await
    }

    /// Checks if node is syncing
    pub async fn is_syncing(&self) -> bool {
        // TODO: Check actual sync state
        false
    }

    /// Gets network statistics
    pub fn network_stats(&self) -> NetworkStats {
        NetworkStats {
            peer_count: 0, // TODO: Get from network
        }
    }

    /// Gets balance from wallet
    pub async fn get_balance(&self) -> u64 {
        self.wallet.as_ref()
            .map(|w| w.get_balance())
            .unwrap_or(0)
    }

    /// Sends transaction
    pub async fn send(&self, to: String, amount: u64) -> Result<nyx_core::Hash> {
        let wallet = self.wallet.as_ref()
            .ok_or_else(|| crate::NodeError::WalletError("Wallet not enabled".to_string()))?;

        wallet.send(&to, amount)
            .map_err(|e| e.into())
    }
}

/// Network statistics
pub struct NetworkStats {
    /// Number of connected peers
    pub peer_count: usize,
}
