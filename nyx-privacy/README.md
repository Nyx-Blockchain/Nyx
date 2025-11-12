# nyx-privacy

Implements Nyx’s privacy layer: RingCT, stealth addresses, and confidential amounts.

## Overview
Provides Monero-grade privacy upgraded with post-quantum security.

## Responsibilities
- Lattice-based Linkable Ring Signatures (LLRS)
- Ring Confidential Transactions (RingCT)
- Stealth-address generation via Kyber
- Pedersen commitments & Bulletproofs+
- Integration with DAG transactions for hidden inputs/outputs

## Key Modules
- `ringct.rs` – confidential transaction construction
- `stealth.rs` – one-time output address creation
- `rangeproof.rs` – Bulletproofs+ range proofs
- `linkable_sig.rs` – LLRS signature scheme

## Future Work
- zk-SNARK-based privacy proofs  
- Optional auditability for CBDC layer
