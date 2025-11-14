// src/node.rs

//! Network node implementation.
//!
//! Combines all network components into a running node that:
//! - Accepts incoming connections
//! - Maintains peer connections
//! - Handles message routing
//! - Manages gossip and sync

use crate::errors::{NetworkError, Result};
use crate::message::{Message, MessageType};
use crate::peer::{Peer, PeerManager, PeerState};
use crate::gossip::GossipEngine;
use crate::sync::SyncManager;
use crate::{MAX_PEERS, MIN_PEERS, HEARTBEAT_INTERVAL_SECS, SYNC_INTERVAL_SECS};
use nyx_core::storage::MemoryStorage;
use nyx_core::dag::DagProcessor;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn, error};

/// Node configuration
#[derive(Clone, Debug)]
pub struct NodeConfig {
    /// Address to listen on
    pub listen_addr: SocketAddr,

    /// Maximum number of peer connections
    pub max_peers: usize,

    /// Minimum number of peers to maintain
    pub min_peers: usize,

    /// Bootstrap peer addresses
    pub bootstrap_peers: Vec<SocketAddr>,

    /// Node identifier
    pub node_id: Vec<u8>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8000".parse().unwrap(),
            max_peers: MAX_PEERS,
            min_peers: MIN_PEERS,
            bootstrap_peers: Vec::new(),
            node_id: vec![0u8; 32],
        }
    }
}

/// Network node
pub struct Node {
    /// Node configuration
    config: NodeConfig,

    /// Peer manager
    peer_manager: Arc<RwLock<PeerManager>>,

    /// Gossip engine
    gossip: Arc<GossipEngine>,

    /// Sync manager
    sync: Arc<SyncManager>,

    /// DAG processor
    dag: Arc<RwLock<DagProcessor>>,

    /// Shutdown signal
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: mpsc::Receiver<()>,
}

impl Node {
    /// Creates a new network node
    pub async fn new(config: NodeConfig) -> Result<Self> {
        // Initialize storage and DAG
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);
        let dag = Arc::new(RwLock::new(dag));

        // Initialize components
        let peer_manager = Arc::new(RwLock::new(PeerManager::new(config.max_peers)));
        let gossip = Arc::new(GossipEngine::new());
        let sync = Arc::new(SyncManager::new(dag.clone()));

        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        info!("Created node with ID: {:?}", hex::encode(&config.node_id));

        Ok(Self {
            config,
            peer_manager,
            gossip,
            sync,
            dag,
            shutdown_tx,
            shutdown_rx,
        })
    }

    /// Runs the network node
    pub async fn run(mut self) -> Result<()> {
        info!("Starting node on {}", self.config.listen_addr);

        // Start listening for connections
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        info!("Listening on {}", self.config.listen_addr);

        // Connect to bootstrap peers
        self.connect_to_bootstrap_peers().await?;

        // Spawn background tasks
        let heartbeat_handle = self.spawn_heartbeat_task();
        let sync_handle = self.spawn_sync_task();

        // Main accept loop
        loop {
            tokio::select! {
                // Accept new connections
                Ok((stream, addr)) = listener.accept() => {
                    debug!("Accepted connection from {}", addr);
                    self.handle_new_connection(stream, addr).await;
                }

                // Shutdown signal
                _ = self.shutdown_rx.recv() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        // Cleanup
        heartbeat_handle.abort();
        sync_handle.abort();

        info!("Node stopped");
        Ok(())
    }

    /// Handles a new incoming connection
    async fn handle_new_connection(&self, mut stream: TcpStream, addr: SocketAddr) {
        let peer_manager = self.peer_manager.clone();
        let gossip = self.gossip.clone();
        let sync = self.sync.clone();
        let node_id = self.config.node_id.clone();

        tokio::spawn(async move {
            // Check if we can accept more peers
            {
                let manager = peer_manager.read().await;
                if !manager.can_accept_more() {
                    warn!("Max peers reached, rejecting connection from {}", addr);
                    return;
                }
            }

            // Create peer
            let peer_id = generate_peer_id(&addr);
            let mut peer = Peer::new(peer_id.clone(), addr);
            peer.update_state(PeerState::Connected);

            // Add to peer manager
            {
                let mut manager = peer_manager.write().await;
                if let Err(e) = manager.add_peer(peer.clone()) {
                    warn!("Failed to add peer: {}", e);
                    return;
                }
            }

            // Register with gossip
            gossip.register_peer(peer_id.clone(), stream.try_clone().unwrap()).await;

            info!("Connected to peer {:?} at {}", peer_id, addr);

            // Handle peer messages
            loop {
                match peer.receive_message(&mut stream).await {
                    Ok(message) => {
                        if let Err(e) = handle_message(
                            message,
                            &mut peer,
                            &mut stream,
                            &gossip,
                            &sync,
                            &peer_manager,
                        ).await {
                            warn!("Error handling message from {:?}: {}", peer_id, e);
                        }
                    }
                    Err(e) => {
                        warn!("Error receiving message from {:?}: {}", peer_id, e);
                        break;
                    }
                }
            }

            // Cleanup
            gossip.unregister_peer(&peer_id).await;
            {
                let mut manager = peer_manager.write().await;
                manager.remove_peer(&peer_id);
            }

            info!("Disconnected from peer {:?}", peer_id);
        });
    }

    /// Connects to bootstrap peers
    async fn connect_to_bootstrap_peers(&self) -> Result<()> {
        for addr in &self.config.bootstrap_peers {
            match self.connect_to_peer(*addr).await {
                Ok(()) => info!("Connected to bootstrap peer {}", addr),
                Err(e) => warn!("Failed to connect to bootstrap peer {}: {}", addr, e),
            }
        }
        Ok(())
    }

    /// Connects to a specific peer
    async fn connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        let peer_id = generate_peer_id(&addr);
        let mut peer = Peer::new(peer_id.clone(), addr);

        let mut stream = peer.connect().await?;

        // Add to peer manager
        {
            let mut manager = self.peer_manager.write().await;
            manager.add_peer(peer.clone())?;
        }

        // Register with gossip
        self.gossip.register_peer(peer_id, stream.try_clone().unwrap()).await;

        Ok(())
    }

    /// Spawns heartbeat task to maintain peer connections
    fn spawn_heartbeat_task(&self) -> tokio::task::JoinHandle<()> {
        let peer_manager = self.peer_manager.clone();
        let gossip = self.gossip.clone();

        tokio::spawn(async move {
            let mut timer = interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

            loop {
                timer.tick().await;

                debug!("Running heartbeat check");

                let mut manager = peer_manager.write().await;
                let connected = manager.connected_peers();

                debug!("Connected peers: {}", connected.len());

                // TODO: Send ping to all peers and check responses
            }
        })
    }

    /// Spawns sync task to periodically sync with peers
    fn spawn_sync_task(&self) -> tokio::task::JoinHandle<()> {
        let sync = self.sync.clone();
        let peer_manager = self.peer_manager.clone();

        tokio::spawn(async move {
            let mut timer = interval(Duration::from_secs(SYNC_INTERVAL_SECS));

            loop {
                timer.tick().await;

                if sync.is_syncing().await {
                    debug!("Already syncing, skipping sync cycle");
                    continue;
                }

                debug!("Running sync cycle");

                // TODO: Implement periodic sync with random peer
            }
        })
    }

    /// Broadcasts a transaction to the network
    pub async fn broadcast_transaction(&self, tx: nyx_core::Transaction) -> Result<()> {
        let mut manager = self.peer_manager.write().await;
        let mut peers: Vec<Peer> = manager.connected_peers()
            .into_iter()
            .cloned()
            .collect();

        self.gossip.gossip_transaction(tx, &mut peers).await
    }

    /// Gets node statistics
    pub async fn stats(&self) -> NodeStats {
        let peer_manager = self.peer_manager.read().await;
        let gossip_stats = self.gossip.stats().await;
        let sync_state = self.sync.get_state().await;

        NodeStats {
            peer_count: peer_manager.peer_count(),
            gossip_stats,
            sync_state,
        }
    }

    /// Initiates graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        self.shutdown_tx.send(()).await
            .map_err(|_| NetworkError::ChannelError("Failed to send shutdown signal".to_string()))?;
        Ok(())
    }
}

/// Node statistics
#[derive(Debug, Clone)]
pub struct NodeStats {
    /// Number of connected peers
    pub peer_count: usize,

    /// Gossip engine statistics
    pub gossip_stats: crate::gossip::GossipStats,

    /// Sync state
    pub sync_state: crate::sync::SyncState,
}

/// Handles an incoming message
async fn handle_message(
    message: Message,
    peer: &mut Peer,
    stream: &mut TcpStream,
    gossip: &Arc<GossipEngine>,
    sync: &Arc<SyncManager>,
    peer_manager: &Arc<RwLock<PeerManager>>,
) -> Result<()> {
    debug!("Handling message type: {}", message.message_type.type_name());

    match message.message_type {
        MessageType::Transaction(tx) => {
            // Add to DAG and gossip to other peers
            info!("Received transaction: {:?}", hex::encode(tx.id()));

            // Gossip to other peers
            let mut manager = peer_manager.write().await;
            let mut peers: Vec<Peer> = manager.connected_peers()
                .into_iter()
                .filter(|p| p.id != peer.id) // Don't send back to sender
                .cloned()
                .collect();

            drop(manager);

            gossip.gossip_transaction(tx, &mut peers).await?;
        }

        MessageType::Ping => {
            // Respond with pong
            let pong = Message::new(MessageType::Pong);
            peer.send_message(stream, &pong).await?;
        }

        MessageType::Pong => {
            // Update peer latency
            debug!("Received pong from peer {:?}", peer.id);
        }

        MessageType::SyncRequest { from_height } => {
            // Handle sync request
            sync.handle_sync_request(from_height, peer, stream).await?;
        }

        MessageType::SyncResponse { transactions } => {
            // Handle sync response
            sync.handle_sync_response(transactions).await?;
        }

        MessageType::PeerDiscovery { peers: peer_addrs } => {
            // Handle peer discovery
            debug!("Received {} peer addresses", peer_addrs.len());
            // TODO: Connect to new peers
        }
    }

    Ok(())
}

/// Generates a peer ID from an address
fn generate_peer_id(addr: &SocketAddr) -> Vec<u8> {
    use nyx_core::hash::blake3_hash;
    let addr_str = addr.to_string();
    blake3_hash(addr_str.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_config_default() {
        let config = NodeConfig::default();
        assert_eq!(config.max_peers, MAX_PEERS);
        assert_eq!(config.min_peers, MIN_PEERS);
    }

    #[tokio::test]
    async fn test_node_creation() {
        let config = NodeConfig::default();
        let node = Node::new(config).await.unwrap();

        let stats = node.stats().await;
        assert_eq!(stats.peer_count, 0);
    }

    #[test]
    fn test_generate_peer_id() {
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let id1 = generate_peer_id(&addr);
        let id2 = generate_peer_id(&addr);

        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 32);
    }
}
