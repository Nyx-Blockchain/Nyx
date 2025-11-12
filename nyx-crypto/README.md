# nyx-crypto

Post-quantum and classical cryptographic primitives used across Nyx.

## Overview
Implements lattice-based cryptography and related key operations required by the protocol.

## Responsibilities
- CRYSTALS-Dilithium and Falcon digital signatures  
- CRYSTALS-Kyber key-exchange  
- BLAKE3 / SHA3 hashing utilities  
- Pedersen commitments and Bulletproofs+ helpers  
- Randomness and VRF generation

## Key Modules
- `dilithium.rs` – Dilithium signature implementation
- `falcon.rs` – compact validator signatures
- `kyber.rs` – key encapsulation & stealth-address support
- `hash.rs` – hashing and Merkle helpers
- `vrf.rs` – validator random selection

## Future Work
- Hardware-accelerated PQC back-ends  
- FIPS/NIST compliance testing
