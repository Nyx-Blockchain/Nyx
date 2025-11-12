# nyx-core

Implements the core consensus engine and DAG transaction layer of the Nyx protocol.

## Overview
`nyx-core` contains the logic that defines how transactions are formed, linked, validated, and finalised through the hybrid DAG + Proof-of-Stake model.

## Responsibilities
- Transaction and block data structures  
- DAG construction and traversal  
- Tip-selection and confirmation algorithms  
- PoS snapshot creation and validator logic  
- Integration hooks to `nyx-network` for message propagation

## Key Modules
- `transaction.rs` – defines `Transaction`, `TxInput`, `TxOutput`
- `dag.rs` – manages adjacency lists, confirmation weights
- `pos.rs` – validator selection (VRF + stake weighting)
- `snapshot.rs` – produces PoS checkpoints
- `config.rs` – network parameters

## Future Work
- DAG sharding and checkpoint pruning  
- Consensus optimisations for 50 000+ TPS
