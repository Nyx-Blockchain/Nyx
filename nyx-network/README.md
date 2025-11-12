# nyx-network

Peer-to-peer networking and message propagation layer.

## Overview
Provides connectivity, peer discovery, and transaction/block gossip for the Nyx protocol.

## Responsibilities
- Dandelion++ transaction propagation
- Kademlia-based peer discovery (DHT)
- Node handshake and identity management
- Message serialisation / deserialisation
- Bandwidth and latency monitoring

## Key Modules
- `peer.rs` – peer data structures
- `protocol.rs` – network message formats
- `dandelion.rs` – privacy-preserving broadcast
- `discovery.rs` – peer routing logic

## Future Work
- NAT traversal and hole-punching  
- Encrypted overlay network using Kyber sessions
