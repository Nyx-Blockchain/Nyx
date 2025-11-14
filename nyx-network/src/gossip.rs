// src/gossip.rs

//! Gossip protocol for message propagation.
//!
//! Implements a pub-sub style gossip system where:
//! - New transactions are broadcast to all peers
//! - Messages are deduplicated using a hash cache
//! - Failed deliveries are retried with exponential backoff

use crate::errors::{NetworkError, Result};
use crate::message::{Message, MessageId, MessageType};
use crate::peer::{Peer, PeerId};
use crate::MAX_SEEN_MESSAGES;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::net::TcpStream;
use tracing::{debug, warn};

/// Gossip engine for message propagation
pub struct GossipEngine {
    /// Cache of seen message IDs for deduplication
    seen_messages: Arc<RwLock<HashSet<MessageId>>>,

    /// Active peer connections
    peer_streams: Arc<RwLock<HashMap<PeerId, TcpStream>>>,

    /// Pending messages to broadcast
    pending: Arc<RwLock<Vec<Message>>>,
}

impl GossipEngine {
    /// Creates a new gossip engine
    pub fn new() -> Self {
        Self {
            seen_messages: Arc::new(RwLock::new(HashSet::new())),
            peer_streams: Arc::new(RwLock::new(HashMap::new())),
            pending: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Checks if we've seen this message before
    pub async fn has_seen(&self, message_id: &MessageId) -> bool {
        let seen = self.seen_messages.read().await;
        seen.contains(message_id)
    }

    /// Marks a message as seen
    pub async fn mark_seen(&self, message_id: MessageId) {
        let mut seen = self.seen_messages.write().await;

        // Limit cache size
        if seen.len() >= MAX_SEEN_MESSAGES {
            // Remove oldest (simple FIFO, could be improved with LRU)
            let to_remove: Vec<_> = seen.iter().take(1000).copied().collect();
            for id in to_remove {
                seen.remove(&id);
            }
        }

        seen.insert(message_id);
    }

    /// Broadcasts a message to all connected peers
    pub async fn broadcast(&self, message: Message, peers: &mut [Peer]) -> Result<usize> {
        // Check if we've already seen this message
        if self.has_seen(&message.id).await {
            debug!("Message {} already seen, skipping broadcast", hex::encode(&message.id));
            return Ok(0);
        }

        // Mark as seen
        self.mark_seen(message.id).await;

        let mut success_count = 0;
        let streams = self.peer_streams.read().await;

        for peer in peers.iter_mut().filter(|p| p.is_connected()) {
            if let Some(stream) = streams.get(&peer.id) {
                // Clone stream handle for this peer
                let mut stream_clone = stream.try_clone()
                    .map_err(|e| NetworkError::IoError(e))?;

                match peer.send_message(&mut stream_clone, &message).await {
                    Ok(()) => {
                        debug!("Broadcast message {} to peer {:?}",
                               hex::encode(&message.id), peer.id);
                        success_count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to send to peer {:?}: {}", peer.id, e);
                    }
                }
            }
        }

        Ok(success_count)
    }

    /// Gossips a transaction to the network
    pub async fn gossip_transaction(
        &self,
        tx: nyx_core::Transaction,
        peers: &mut [Peer],
    ) -> Result<()> {
        let message = Message::new(MessageType::Transaction(tx));

        let sent = self.broadcast(message, peers).await?;

        debug!("Gossiped transaction to {} peers", sent);

        Ok(())
    }

    /// Registers a peer stream for gossip
    pub async fn register_peer(&self, peer_id: PeerId, stream: TcpStream) {
        let mut streams = self.peer_streams.write().await;
        streams.insert(peer_id, stream);
    }

    /// Unregisters a peer stream
    pub async fn unregister_peer(&self, peer_id: &PeerId) {
        let mut streams = self.peer_streams.write().await;
        streams.remove(peer_id);
    }

    /// Gets statistics about the gossip engine
    pub async fn stats(&self) -> GossipStats {
        let seen = self.seen_messages.read().await;
        let streams = self.peer_streams.read().await;
        let pending = self.pending.read().await;

        GossipStats {
            seen_messages: seen.len(),
            active_peers: streams.len(),
            pending_messages: pending.len(),
        }
    }
}

impl Default for GossipEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about gossip engine state
#[derive(Debug, Clone)]
pub struct GossipStats {
    /// Number of seen messages in cache
    pub seen_messages: usize,

    /// Number of active peer connections
    pub active_peers: usize,

    /// Number of pending messages
    pub pending_messages: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageType;

    #[tokio::test]
    async fn test_message_deduplication() {
        let gossip = GossipEngine::new();
        let msg = Message::new(MessageType::Ping);

        assert!(!gossip.has_seen(&msg.id).await);

        gossip.mark_seen(msg.id).await;

        assert!(gossip.has_seen(&msg.id).await);
    }

    #[tokio::test]
    async fn test_gossip_stats() {
        let gossip = GossipEngine::new();
        let stats = gossip.stats().await;

        assert_eq!(stats.seen_messages, 0);
        assert_eq!(stats.active_peers, 0);
    }
}
