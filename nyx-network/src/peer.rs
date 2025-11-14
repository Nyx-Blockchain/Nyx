// src/peer.rs

//! Peer connection management.
//!
//! Handles individual peer connections including TCP communication,
//! message sending/receiving, and connection state management.

use crate::errors::{NetworkError, Result};
use crate::message::Message;
use crate::{CONNECTION_TIMEOUT_SECS, MAX_MESSAGE_SIZE};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Unique peer identifier
pub type PeerId = Vec<u8>;

/// Represents a connected peer in the network
#[derive(Clone, Debug)]
pub struct Peer {
    /// Unique peer identifier
    pub id: PeerId,

    /// Peer's network address
    pub address: SocketAddr,

    /// Last time we received a message from this peer
    pub last_seen: Instant,

    /// Average latency to this peer (milliseconds)
    pub latency_ms: Option<u64>,

    /// Connection state
    pub state: PeerState,
}

/// Peer connection state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PeerState {
    /// Attempting to connect
    Connecting,

    /// Connected and active
    Connected,

    /// Disconnected
    Disconnected,

    /// Banned (misbehaving)
    Banned,
}

impl Peer {
    /// Creates a new peer
    pub fn new(id: PeerId, address: SocketAddr) -> Self {
        Self {
            id,
            address,
            last_seen: Instant::now(),
            latency_ms: None,
            state: PeerState::Connecting,
        }
    }

    /// Connects to the peer
    pub async fn connect(&mut self) -> Result<TcpStream> {
        self.state = PeerState::Connecting;

        let stream = timeout(
            Duration::from_secs(CONNECTION_TIMEOUT_SECS),
            TcpStream::connect(self.address)
        )
        .await
        .map_err(|_| NetworkError::Timeout(format!("Connection timeout to {}", self.address)))?
        .map_err(|e| NetworkError::ConnectionError(format!("Failed to connect: {}", e)))?;

        self.state = PeerState::Connected;
        self.last_seen = Instant::now();

        Ok(stream)
    }

    /// Sends a message to this peer
    pub async fn send_message(
        &mut self,
        stream: &mut OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        // Serialize message
        let data = message.to_bytes()?;

        if data.len() > MAX_MESSAGE_SIZE {
            return Err(NetworkError::InvalidMessage(
                format!("Message too large: {} bytes", data.len())
            ));
        }

        // Send length prefix (4 bytes)
        let len = (data.len() as u32).to_be_bytes();
        stream.write_all(&len).await?;

        // Send message data
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Receives a message from this peer
    pub async fn receive_message(
        &mut self,
        stream: &mut OwnedReadHalf,
    ) -> Result<Message> {
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(NetworkError::InvalidMessage(
                format!("Message too large: {} bytes", len)
            ));
        }

        // Read message data
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await?;

        // Deserialize message
        let message = Message::from_bytes(&data)?;

        self.last_seen = Instant::now();

        Ok(message)
    }

    /// Updates peer state
    pub fn update_state(&mut self, new_state: PeerState) {
        self.state = new_state;
    }

    /// Checks if peer is connected
    pub fn is_connected(&self) -> bool {
        self.state == PeerState::Connected
    }

    /// Measures latency by timing a ping
    pub async fn measure_latency(
        &mut self,
        writer: &mut OwnedWriteHalf,
        reader: &mut OwnedReadHalf,
    ) -> Result<u64> {
        let start = Instant::now();

        // Send ping
        let ping = Message::new(crate::message::MessageType::Ping);
        self.send_message(writer, &ping).await?;

        // Wait for pong
        let response = self.receive_message(reader).await?;

        if !matches!(response.message_type, crate::message::MessageType::Pong) {
            return Err(NetworkError::InvalidMessage("Expected Pong".to_string()));
        }

        let latency = start.elapsed().as_millis() as u64;
        self.latency_ms = Some(latency);

        Ok(latency)
    }
}

/// Peer manager for handling multiple peer connections
pub struct PeerManager {
    /// Connected peers
    peers: Vec<Peer>,

    /// Maximum number of peers
    max_peers: usize,
}

impl PeerManager {
    /// Creates a new peer manager
    pub fn new(max_peers: usize) -> Self {
        Self {
            peers: Vec::new(),
            max_peers,
        }
    }

    /// Adds a new peer
    pub fn add_peer(&mut self, peer: Peer) -> Result<()> {
        if self.peers.len() >= self.max_peers {
            return Err(NetworkError::MaxPeersReached);
        }

        // Check if peer already exists
        if self.peers.iter().any(|p| p.id == peer.id) {
            return Ok(()); // Already connected
        }

        self.peers.push(peer);
        Ok(())
    }

    /// Removes a peer
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.retain(|p| &p.id != peer_id);
    }

    /// Gets a peer by ID
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&Peer> {
        self.peers.iter().find(|p| &p.id == peer_id)
    }

    /// Gets a mutable peer by ID
    pub fn get_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut Peer> {
        self.peers.iter_mut().find(|p| &p.id == peer_id)
    }

    /// Gets all connected peers
    pub fn connected_peers(&self) -> Vec<&Peer> {
        self.peers.iter()
            .filter(|p| p.is_connected())
            .collect()
    }

    /// Gets number of connected peers
    pub fn peer_count(&self) -> usize {
        self.connected_peers().len()
    }

    /// Checks if we can accept more peers
    pub fn can_accept_more(&self) -> bool {
        self.peers.len() < self.max_peers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_creation() {
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let peer = Peer::new(vec![1, 2, 3], addr);

        assert_eq!(peer.id, vec![1, 2, 3]);
        assert_eq!(peer.address, addr);
        assert_eq!(peer.state, PeerState::Connecting);
    }

    #[test]
    fn test_peer_manager() {
        let mut manager = PeerManager::new(10);
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();

        let peer = Peer::new(vec![1], addr);
        manager.add_peer(peer).unwrap();

        assert_eq!(manager.peer_count(), 0); // Not connected yet
        assert_eq!(manager.peers.len(), 1);
    }

    #[test]
    fn test_peer_manager_max_peers() {
        let mut manager = PeerManager::new(2);
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();

        manager.add_peer(Peer::new(vec![1], addr)).unwrap();
        manager.add_peer(Peer::new(vec![2], addr)).unwrap();

        let result = manager.add_peer(Peer::new(vec![3], addr));
        assert!(result.is_err());
    }
}
