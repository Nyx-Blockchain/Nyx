// src/account.rs

//! Account management with keys and addresses.

use crate::errors::{WalletError, Result};
use nyx_crypto::keys::KeyPair;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Account with keys and address
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    /// Account name
    pub name: String,

    /// View keypair (for scanning transactions)
    view_keypair: KeyPairData,

    /// Spend keypair (for spending funds)
    spend_keypair: KeyPairData,

    /// Public address (derived from public keys)
    pub address: Address,
}

/// Keypair data with zeroization
#[derive(Clone, Zeroize, ZeroizeOnDrop, Serialize, Deserialize)]
struct KeyPairData {
    public: Vec<u8>,
    #[zeroize(skip)] // Skip because it's already protected
    private: Vec<u8>,
}

impl std::fmt::Debug for KeyPairData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPairData")
            .field("public", &"[REDACTED]")
            .field("private", &"[REDACTED]")
            .finish()
    }
}

/// Public address combining view and spend keys
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Address {
    /// Public view key
    pub view_public: Vec<u8>,

    /// Public spend key
    pub spend_public: Vec<u8>,
}

impl Account {
    /// Generates a new account with random keys
    ///
    /// # Example
    /// ```
    /// use nyx_wallet::Account;
    ///
    /// let account = Account::generate();
    /// assert!(!account.name.is_empty());
    /// ```
    pub fn generate() -> Self {
        let view_keypair = nyx_crypto::keys::generate_ed25519_keypair_as_struct();
        let spend_keypair = nyx_crypto::keys::generate_ed25519_keypair_as_struct();

        Self::from_keypairs("default".to_string(), view_keypair, spend_keypair)
    }

    /// Generates an account with a specific name
    pub fn generate_with_name(name: String) -> Self {
        let view_keypair = nyx_crypto::keys::generate_ed25519_keypair_as_struct();
        let spend_keypair = nyx_crypto::keys::generate_ed25519_keypair_as_struct();

        Self::from_keypairs(name, view_keypair, spend_keypair)
    }

    /// Creates an account from existing keypairs
    pub fn from_keypairs(name: String, view_keypair: KeyPair, spend_keypair: KeyPair) -> Self {
        let address = Address {
            view_public: view_keypair.public_key.clone(),
            spend_public: spend_keypair.public_key.clone(),
        };

        Self {
            name,
            view_keypair: KeyPairData {
                public: view_keypair.public_key.clone(),
                private: view_keypair.private_key().to_vec(),
            },
            spend_keypair: KeyPairData {
                public: spend_keypair.public_key.clone(),
                private: spend_keypair.private_key().to_vec(),
            },
            address,
        }
    }

    /// Gets the view private key
    pub fn view_private_key(&self) -> &[u8] {
        &self.view_keypair.private
    }

    /// Gets the view public key
    pub fn view_public_key(&self) -> &[u8] {
        &self.view_keypair.public
    }

    /// Gets the spend private key
    pub fn spend_private_key(&self) -> &[u8] {
        &self.spend_keypair.private
    }

    /// Gets the spend public key
    pub fn spend_public_key(&self) -> &[u8] {
        &self.spend_keypair.public
    }

    /// Gets the public address
    pub fn get_address(&self) -> &Address {
        &self.address
    }

    /// Exports account to JSON
    pub fn export_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| WalletError::SerializationError(format!("{}", e)))
    }

    /// Imports account from JSON
    pub fn import_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| WalletError::SerializationError(format!("{}", e)))
    }

    /// Exports to binary format
    pub fn export_binary(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| WalletError::SerializationError(format!("{}", e)))
    }

    /// Imports from binary format
    pub fn import_binary(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| WalletError::SerializationError(format!("{}", e)))
    }
}

impl Address {
    /// Converts address to string representation
    pub fn to_string(&self) -> String {
        format!(
            "nyx:{}:{}",
            hex::encode(&self.view_public),
            hex::encode(&self.spend_public)
        )
    }

    /// Parses address from string
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 3 || parts[0] != "nyx" {
            return Err(WalletError::InvalidAddress(
                "Invalid address format".to_string()
            ));
        }

        let view_public = hex::decode(parts[1])
            .map_err(|_| WalletError::InvalidAddress("Invalid view key".to_string()))?;

        let spend_public = hex::decode(parts[2])
            .map_err(|_| WalletError::InvalidAddress("Invalid spend key".to_string()))?;

        Ok(Self {
            view_public,
            spend_public,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_generation() {
        let account = Account::generate();
        assert_eq!(account.name, "default");
        assert!(!account.view_keypair.public.is_empty());
        assert!(!account.spend_keypair.public.is_empty());
    }

    #[test]
    fn test_account_with_name() {
        let account = Account::generate_with_name("test".to_string());
        assert_eq!(account.name, "test");
    }

    #[test]
    fn test_account_keys() {
        let account = Account::generate();

        assert!(!account.view_private_key().is_empty());
        assert!(!account.view_public_key().is_empty());
        assert!(!account.spend_private_key().is_empty());
        assert!(!account.spend_public_key().is_empty());
    }

    #[test]
    fn test_account_export_import_json() {
        let account = Account::generate_with_name("export_test".to_string());

        let json = account.export_json().unwrap();
        let imported = Account::import_json(&json).unwrap();

        assert_eq!(account.name, imported.name);
        assert_eq!(account.address, imported.address);
    }

    #[test]
    fn test_account_export_import_binary() {
        let account = Account::generate();

        let binary = account.export_binary().unwrap();
        let imported = Account::import_binary(&binary).unwrap();

        assert_eq!(account.name, imported.name);
    }

    #[test]
    fn test_address_to_string() {
        let account = Account::generate();
        let addr_str = account.address.to_string();

        assert!(addr_str.starts_with("nyx:"));
        assert!(addr_str.contains(':'));
    }

    #[test]
    fn test_address_from_string() {
        let account = Account::generate();
        let addr_str = account.address.to_string();

        let parsed = Address::from_string(&addr_str).unwrap();
        assert_eq!(parsed, account.address);
    }

    #[test]
    fn test_invalid_address() {
        let result = Address::from_string("invalid");
        assert!(result.is_err());

        let result = Address::from_string("btc:123:456");
        assert!(result.is_err());
    }
}
