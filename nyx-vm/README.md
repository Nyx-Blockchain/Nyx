# nyx-vm

Privacy-preserving smart-contract virtual machine for Nyx.

## Overview
Executes encrypted smart contracts using FHE and Zero-Knowledge Proofs while maintaining compatibility with EVM-style bytecode where possible.

## Responsibilities
- Encrypted state execution (FHE operations)
- zk-proof generation for state transitions
- Gas metering and fee accounting
- Contract deployment and invocation interface

## Key Modules
- `engine.rs` – main execution engine
- `fhe.rs` – fully homomorphic encryption helpers
- `zk.rs` – zero-knowledge proof utilities
- `abi.rs` – contract I/O encoding

## Future Work
- zk-Rollups for Layer 2 privacy  
- WASM runtime for user-defined contracts
