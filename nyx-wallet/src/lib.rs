// src/lib.rs

//! # Nyx Wallet
//!
//! Complete wallet implementation for the Nyx blockchain.
//!
//! This module provides:
//! - **Account Management**: Key generation and storage
//! - **Transaction Building**: Create privacy-preserving transactions
//! - **Balance Tracking**: Monitor wallet balances
//! - **Keystore**: Encrypted key storage
//!
//! ## Example Usage
//!
//! ```rust
//! use nyx_wallet::{Wallet, Account};
//!
//! // Create a new wallet
//! let mut wallet = Wallet::new();
//!
//! // Create account
//! let account = Account::generate();
//! wallet.add_account(account);
//!
//! // Check balance
//! let balance = wallet.get_balance();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod errors;
pub mod account;
pub mod keystore;
pub mod builder;
pub mod wallet;

// Re-export commonly used types
pub use crate::errors::{WalletError, Result};
pub use crate::account::Account;
pub use crate::keystore::Keystore;
pub use crate::builder::TransactionBuilder;
pub use crate::wallet::Wallet;

/// Wallet version for compatibility
pub const WALLET_VERSION: u32 = 1;

/// Default keystore directory name
pub const KEYSTORE_DIR: &str = ".nyx-wallet";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_version() {
        assert_eq!(WALLET_VERSION, 1);
    }
}
