// src/keystore.rs

//! Encrypted keystore for secure account storage.

use crate::account::Account;
use crate::errors::{WalletError, Result};
use nyx_crypto::encryption;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Encrypted keystore
#[derive(Debug, Serialize, Deserialize)]
pub struct Keystore {
    /// Keystore version
    version: u32,

    /// Encrypted account data
    encrypted_data: Vec<u8>,

    /// Salt for password derivation (mock)
    salt: Vec<u8>,
}

impl Keystore {
    /// Creates a new keystore from an account
    ///
    /// # Arguments
    /// * `account` - Account to store
    /// * `password` - Password for encryption
    pub fn new(account: &Account, password: &str) -> Result<Self> {
        // Serialize account
        let account_data = account.export_binary()?;

        // Derive key from password (mock implementation)
        let key = Self::derive_key(password);

        // Generate salt (mock)
        let salt = vec![0u8; 32];

        // Encrypt data
        let encrypted_data = encryption::encrypt(&account_data, &key)?;

        Ok(Self {
            version: crate::WALLET_VERSION,
            encrypted_data,
            salt,
        })
    }

    /// Decrypts and returns the account
    ///
    /// # Arguments
    /// * `password` - Password for decryption
    pub fn decrypt(&self, password: &str) -> Result<Account> {
        // Derive key from password
        let key = Self::derive_key(password);

        // Decrypt data
        let account_data = encryption::decrypt(&self.encrypted_data, &key)
            .map_err(|_| WalletError::InvalidPassword)?;

        // Deserialize account
        Account::import_binary(&account_data)
    }

    /// Saves keystore to file
    ///
    /// # Arguments
    /// * `path` - File path to save to
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Loads keystore from file
    ///
    /// # Arguments
    /// * `path` - File path to load from
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = fs::read_to_string(path)
            .map_err(|_| WalletError::FileNotFound("Keystore file not found".to_string()))?;

        Ok(serde_json::from_str(&json)?)
    }

    /// Derives encryption key from password (mock implementation)
    fn derive_key(password: &str) -> Vec<u8> {
        // In production, use proper KDF like Argon2 or PBKDF2
        // For now, simple hash-based derivation
        let mut key = nyx_crypto::hash::blake3_hash(password.as_bytes()).to_vec();
        key.truncate(32); // AES-256 key size
        key
    }

    /// Gets the default keystore directory
    pub fn default_directory() -> Result<PathBuf> {
        let home = directories::UserDirs::new()
            .ok_or_else(|| WalletError::KeystoreError("Cannot find home directory".to_string()))?;

        let home_dir = home.home_dir();
        Ok(home_dir.join(crate::KEYSTORE_DIR))
    }

    /// Creates the keystore directory if it doesn't exist
    pub fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_keystore_creation() {
        let account = Account::generate_with_name("test".to_string());
        let password = "secure_password";

        let keystore = Keystore::new(&account, password).unwrap();
        assert_eq!(keystore.version, crate::WALLET_VERSION);
        assert!(!keystore.encrypted_data.is_empty());
    }

    #[test]
    fn test_keystore_encrypt_decrypt() {
        let account = Account::generate_with_name("encrypt_test".to_string());
        let password = "test123";

        let keystore = Keystore::new(&account, password).unwrap();
        let decrypted = keystore.decrypt(password).unwrap();

        assert_eq!(account.name, decrypted.name);
        assert_eq!(account.address, decrypted.address);
    }

    #[test]
    fn test_keystore_wrong_password() {
        let account = Account::generate();
        let keystore = Keystore::new(&account, "correct").unwrap();

        let result = keystore.decrypt("wrong");
        assert!(result.is_err());
    }

    #[test]
    fn test_keystore_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_keystore.json");

        let account = Account::generate_with_name("save_test".to_string());
        let password = "password123";

        let keystore = Keystore::new(&account, password).unwrap();
        keystore.save_to_file(&file_path).unwrap();

        let loaded = Keystore::load_from_file(&file_path).unwrap();
        let decrypted = loaded.decrypt(password).unwrap();

        assert_eq!(account.name, decrypted.name);
    }

    #[test]
    fn test_keystore_file_not_found() {
        let result = Keystore::load_from_file("/nonexistent/path.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_key() {
        let key1 = Keystore::derive_key("password");
        let key2 = Keystore::derive_key("password");

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }
}
