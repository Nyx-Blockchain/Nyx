// src/message.rs

//! Network message types and serialization.
//!
//! Defines all message types used in the Nyx P2P protocol including
//! transaction broadcasts, sync requests, and peer discovery.

use nyx_core::Transaction;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Unique message identifier
pub type MessageId = [u8; 32];

/// Network message envelope
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID (hash of content)
    pub id: MessageId,

    /// Message type and payload
    pub message_type: MessageType,

    /// Timestamp when message was created
    pub timestamp: u64,

    /// Sender's peer ID
    pub sender: Option<Vec<u8>>,
}

impl Message {
    /// Creates a new message
    pub fn new(message_type: MessageType) -> Self {
        let timestamp = current_timestamp();
        let id = Self::compute_id(&message_type, timestamp);

        Self {
            id,
            message_type,
            timestamp,
            sender: None,
        }
    }

    /// Creates a message with sender ID
    pub fn with_sender(mut self, sender: Vec<u8>) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Computes message ID
    fn compute_id(message_type: &MessageType, timestamp: u64) -> MessageId {
        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());

        match message_type {
            MessageType::Transaction(tx) => {
                let tx_id = tx.id();
                data.extend_from_slice(&tx_id);
            }
            MessageType::Ping => data.extend_from_slice(b"ping"),
            MessageType::Pong => data.extend_from_slice(b"pong"),
            MessageType::SyncRequest { from_height } => {
                data.extend_from_slice(&from_height.to_le_bytes());
            }
            MessageType::SyncResponse { transactions } => {
                data.extend_from_slice(&transactions.len().to_le_bytes());
            }
            MessageType::PeerDiscovery { peers } => {
                data.extend_from_slice(&peers.len().to_le_bytes());
            }
        }

        nyx_core::hash::blake3_hash(&data)
    }

    /// Serializes message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserializes message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

/// Message types in the Nyx network protocol
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    /// Transaction broadcast
    Transaction(Transaction),

    /// Ping message (keepalive)
    Ping,

    /// Pong message (response to ping)
    Pong,

    /// Request to sync DAG from height
    SyncRequest {
        /// Starting height to sync from
        from_height: u64,
    },

    /// Response with batch of transactions
    SyncResponse {
        /// Batch of transactions
        transactions: Vec<Transaction>,
    },

    /// Peer discovery message
    PeerDiscovery {
        /// Known peer addresses
        peers: Vec<SocketAddr>,
    },
}

impl MessageType {
    /// Returns the type name
    pub fn type_name(&self) -> &'static str {
        match self {
            MessageType::Transaction(_) => "Transaction",
            MessageType::Ping => "Ping",
            MessageType::Pong => "Pong",
            MessageType::SyncRequest { .. } => "SyncRequest",
            MessageType::SyncResponse { .. } => "SyncResponse",
            MessageType::PeerDiscovery { .. } => "PeerDiscovery",
        }
    }
}

/// Helper to get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be after Unix epoch")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyx_core::{RingSignature, transaction::{TxInput, TxOutput}};

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
                ephemeral_pubkey: vec![],
            }],
            RingSignature {
                ring_members: vec![],
                signature: vec![],
                key_image: [0u8; 32],
            },
            vec![],
            [0u8; 32],
            [1u8; 32],
        )
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new(MessageType::Ping);
        assert_eq!(msg.id.len(), 32);
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_message_serialization() {
        let tx = create_test_tx();
        let msg = Message::new(MessageType::Transaction(tx));

        let bytes = msg.to_bytes().unwrap();
        let decoded = Message::from_bytes(&bytes).unwrap();

        assert_eq!(msg.id, decoded.id);
    }

    #[test]
    fn test_message_with_sender() {
        let msg = Message::new(MessageType::Ping)
            .with_sender(vec![1, 2, 3]);

        assert!(msg.sender.is_some());
        assert_eq!(msg.sender.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_message_type_name() {
        assert_eq!(MessageType::Ping.type_name(), "Ping");
        assert_eq!(MessageType::Pong.type_name(), "Pong");
    }
}
