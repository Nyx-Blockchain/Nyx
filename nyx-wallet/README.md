# nyx-wallet

Reference wallet and CLI client for the Nyx protocol.

## Overview
Provides key management, transaction creation, staking, and explorer utilities for end users and developers.

## Responsibilities
- Wallet creation & recovery (Dilithium/Ed25519 hybrid)
- Transaction signing & broadcast
- Staking interface for validators
- Basic explorer view (balance, history, network stats)
- Integration testing harness for other crates

## Key Modules
- `main.rs` – CLI entrypoint
- `wallet.rs` – key storage and encryption
- `tx_builder.rs` – transaction assembly
- `api.rs` – RPC / gRPC client for network nodes

## Future Work
- GUI wallet (React + Tauri)
- Mobile SDK for NyxPay integration
