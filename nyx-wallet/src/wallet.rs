// src/wallet.rs

//! Wallet implementation with balance tracking and transaction management.

use crate::account::Account;
use crate::builder::{TransactionBuilder, Utxo};
use crate::errors::{WalletError, Result};
use nyx_core::transaction::Transaction;
use nyx_core::Hash;
use nyx_crypto::stealth;
use std::collections::HashMap;

/// Main wallet structure
#[derive(Clone)]
pub struct Wallet {
    /// Accounts in this wallet
    accounts: Vec<Account>,

    /// Active account index
    active_account: usize,

    /// Mock UTXO set (in production, this would query the blockchain)
    utxos: HashMap<String, Vec<Utxo>>,

    /// Balance cache
    balance_cache: HashMap<String, u64>,
}

impl Wallet {
    /// Creates a new empty wallet
    ///
    /// # Example
    /// ```
    /// use nyx_wallet::Wallet;
    ///
    /// let wallet = Wallet::new();
    /// assert_eq!(wallet.account_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            active_account: 0,
            utxos: HashMap::new(),
            balance_cache: HashMap::new(),
        }
    }

    /// Creates a wallet with a default account
    pub fn with_default_account() -> Self {
        let mut wallet = Self::new();
        let account = Account::generate();
        wallet.add_account(account).unwrap();
        wallet
    }

    /// Adds an account to the wallet
    ///
    /// # Arguments
    /// * `account` - Account to add
    pub fn add_account(&mut self, account: Account) -> Result<()> {
        // Check if account already exists
        if self.accounts.iter().any(|a| a.name == account.name) {
            return Err(WalletError::AccountExists(account.name.clone()));
        }

        self.accounts.push(account);
        Ok(())
    }

    /// Gets the currently active account
    pub fn get_active_account(&self) -> Result<&Account> {
        self.accounts.get(self.active_account)
            .ok_or_else(|| WalletError::AccountNotFound("No active account".to_string()))
    }

    /// Gets a mutable reference to the active account
    pub fn get_active_account_mut(&mut self) -> Result<&mut Account> {
        self.accounts.get_mut(self.active_account)
            .ok_or_else(|| WalletError::AccountNotFound("No active account".to_string()))
    }

    /// Gets an account by name
    pub fn get_account(&self, name: &str) -> Result<&Account> {
        self.accounts.iter()
            .find(|a| a.name == name)
            .ok_or_else(|| WalletError::AccountNotFound(format!("Account '{}' not found", name)))
    }

    /// Sets the active account by index
    pub fn set_active_account(&mut self, index: usize) -> Result<()> {
        if index >= self.accounts.len() {
            return Err(WalletError::AccountNotFound(
                format!("Invalid account index: {}", index)
            ));
        }

        self.active_account = index;
        Ok(())
    }

    /// Sets the active account by name
    pub fn set_active_account_by_name(&mut self, name: &str) -> Result<()> {
        let index = self.accounts.iter()
            .position(|a| a.name == name)
            .ok_or_else(|| WalletError::AccountNotFound(format!("Account '{}' not found", name)))?;

        self.active_account = index;
        Ok(())
    }

    /// Gets the number of accounts
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Lists all account names
    pub fn list_accounts(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.name.clone()).collect()
    }

    /// Gets the balance for the active account
    ///
    /// # Returns
    /// Total balance in smallest unit
    pub fn get_balance(&self) -> u64 {
        let account = match self.get_active_account() {
            Ok(acc) => acc,
            Err(_) => return 0,
        };

        // Check cache first
        if let Some(&balance) = self.balance_cache.get(&account.name) {
            return balance;
        }

        // Calculate from UTXOs
        self.calculate_balance(&account.name)
    }

    /// Gets balance for a specific account
    pub fn get_balance_for_account(&self, name: &str) -> Result<u64> {
        let _account = self.get_account(name)?;
        Ok(self.calculate_balance(name))
    }

    /// Calculates balance from UTXOs
    fn calculate_balance(&self, account_name: &str) -> u64 {
        self.utxos.get(account_name)
            .map(|utxos| utxos.iter().map(|u| u.amount).sum())
            .unwrap_or(0)
    }

    /// Adds a mock UTXO to the wallet (for testing)
    ///
    /// In production, this would scan the blockchain for outputs
    pub fn add_utxo(&mut self, account_name: &str, utxo: Utxo) -> Result<()> {
        // Verify account exists
        self.get_account(account_name)?;

        self.utxos.entry(account_name.to_string())
            .or_insert_with(Vec::new)
            .push(utxo);

        // Invalidate cache
        self.balance_cache.remove(account_name);

        Ok(())
    }

    /// Gets available UTXOs for an account
    pub fn get_utxos(&self, account_name: &str) -> Vec<&Utxo> {
        self.utxos.get(account_name)
            .map(|utxos| utxos.iter().collect())
            .unwrap_or_default()
    }

    /// Scans for outputs belonging to the active account
    ///
    /// This is a mock implementation. In production, this would:
    /// 1. Query blockchain for new transactions
    /// 2. Check each output with stealth address detection
    /// 3. Add matching outputs as UTXOs
    pub fn scan_outputs(&mut self) -> Result<usize> {
        let account = self.get_active_account()?.clone();

        // Mock: In production, iterate through blockchain transactions
        // and use stealth::is_mine() to detect owned outputs

        let mut found = 0;

        // Mock finding an output
        let mock_utxo = Utxo {
            tx_hash: [0u8; 32],
            index: 0,
            amount: 1000,
            key_image: vec![1u8; 32],
        };

        self.add_utxo(&account.name, mock_utxo)?;
        found += 1;

        Ok(found)
    }

    /// Checks if a transaction output belongs to the active account
    ///
    /// # Arguments
    /// * `stealth_address` - The stealth address to check
    /// * `ephemeral_pubkey` - The ephemeral public key from the transaction
    pub fn is_mine(
        &self,
        stealth_address: &[u8],
        ephemeral_pubkey: &[u8],
    ) -> Result<bool> {
        let account = self.get_active_account()?;

        stealth::is_mine(
            stealth_address,
            account.view_private_key(),
            &account.address.spend_public,
            ephemeral_pubkey,
        ).map_err(|e| WalletError::CryptoError(format!("{}", e)))
    }

    /// Builds a transaction to send funds
    ///
    /// # Arguments
    /// * `to_address` - Recipient's address string
    /// * `amount` - Amount to send
    /// * `fee` - Transaction fee
    ///
    /// # Returns
    /// Built and signed transaction
    pub fn build_transaction(
        &self,
        to_address: &str,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction> {
        let account = self.get_active_account()?.clone();

        // Parse recipient address
        let recipient = crate::account::Address::from_string(to_address)?;

        // Check balance
        let balance = self.get_balance();
        let total_needed = amount + fee;

        if balance < total_needed {
            return Err(WalletError::InsufficientBalance {
                required: total_needed,
                available: balance,
            });
        }

        // Select UTXOs to spend
        let utxos = self.select_utxos(&account.name, total_needed)?;

        // Calculate change
        let total_input: u64 = utxos.iter().map(|u| u.amount).sum();
        let change = total_input - total_needed;

        // Build transaction
        let mut builder = TransactionBuilder::new()
            .sender(account.clone());

        // Add inputs
        for utxo in utxos {
            builder = builder.add_input(utxo);
        }

        // Add output to recipient
        builder = builder.add_output(
            recipient.view_public.clone(),
            recipient.spend_public.clone(),
            amount,
        );

        // Add change output if any
        if change > 0 {
            builder = builder.add_output(
                account.address.view_public.clone(),
                account.address.spend_public.clone(),
                change,
            );
        }

        // Build with mock parent hashes
        builder.build([0u8; 32], [1u8; 32])
    }

    /// Submits a transaction to the network (mock implementation)
    ///
    /// In production, this would broadcast to nyx-network
    pub fn submit_transaction(&self, _tx: &Transaction) -> Result<Hash> {
        // Mock: In production, use nyx-network to broadcast
        // For now, just return a mock transaction hash
        Ok([0u8; 32])
    }

    /// Sends funds to an address (convenience method)
    ///
    /// # Arguments
    /// * `to_address` - Recipient's address
    /// * `amount` - Amount to send
    ///
    /// # Returns
    /// Transaction hash
    pub fn send(&self, to_address: &str, amount: u64) -> Result<Hash> {
        let fee = 0; // Mock fee
        let tx = self.build_transaction(to_address, amount, fee)?;
        self.submit_transaction(&tx)
    }

    /// Selects UTXOs for spending
    fn select_utxos(&self, account_name: &str, amount: u64) -> Result<Vec<Utxo>> {
        let available_utxos = self.utxos.get(account_name)
            .ok_or_else(|| WalletError::InsufficientBalance {
                required: amount,
                available: 0,
            })?;

        // Simple selection: take UTXOs until we have enough
        let mut selected = Vec::new();
        let mut total = 0u64;

        for utxo in available_utxos {
            selected.push(utxo.clone());
            total += utxo.amount;

            if total >= amount {
                break;
            }
        }

        if total < amount {
            return Err(WalletError::InsufficientBalance {
                required: amount,
                available: total,
            });
        }

        Ok(selected)
    }

    /// Refreshes balance cache
    pub fn refresh_balance(&mut self) {
        self.balance_cache.clear();

        for account in &self.accounts {
            let balance = self.calculate_balance(&account.name);
            self.balance_cache.insert(account.name.clone(), balance);
        }
    }

    /// Gets wallet statistics
    pub fn get_stats(&self) -> WalletStats {
        WalletStats {
            total_accounts: self.accounts.len(),
            active_account: self.active_account,
            total_balance: self.get_balance(),
            total_utxos: self.utxos.values().map(|v| v.len()).sum(),
        }
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

/// Wallet statistics
#[derive(Debug, Clone)]
pub struct WalletStats {
    /// Total number of accounts
    pub total_accounts: usize,

    /// Active account index
    pub active_account: usize,

    /// Total balance across all accounts
    pub total_balance: u64,

    /// Total number of UTXOs
    pub total_utxos: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert_eq!(wallet.account_count(), 0);
        assert_eq!(wallet.get_balance(), 0);
    }

    #[test]
    fn test_wallet_with_default_account() {
        let wallet = Wallet::with_default_account();
        assert_eq!(wallet.account_count(), 1);
    }

    #[test]
    fn test_add_account() {
        let mut wallet = Wallet::new();
        let account = Account::generate_with_name("test".to_string());

        wallet.add_account(account).unwrap();
        assert_eq!(wallet.account_count(), 1);
    }

    #[test]
    fn test_duplicate_account() {
        let mut wallet = Wallet::new();
        let account1 = Account::generate_with_name("test".to_string());
        let account2 = Account::generate_with_name("test".to_string());

        wallet.add_account(account1).unwrap();
        let result = wallet.add_account(account2);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_active_account() {
        let mut wallet = Wallet::new();
        let account = Account::generate_with_name("active".to_string());

        wallet.add_account(account).unwrap();

        let active = wallet.get_active_account().unwrap();
        assert_eq!(active.name, "active");
    }

    #[test]
    fn test_set_active_account() {
        let mut wallet = Wallet::new();

        wallet.add_account(Account::generate_with_name("acc1".to_string())).unwrap();
        wallet.add_account(Account::generate_with_name("acc2".to_string())).unwrap();

        wallet.set_active_account(1).unwrap();

        let active = wallet.get_active_account().unwrap();
        assert_eq!(active.name, "acc2");
    }

    #[test]
    fn test_set_active_account_by_name() {
        let mut wallet = Wallet::new();

        wallet.add_account(Account::generate_with_name("first".to_string())).unwrap();
        wallet.add_account(Account::generate_with_name("second".to_string())).unwrap();

        wallet.set_active_account_by_name("second").unwrap();

        let active = wallet.get_active_account().unwrap();
        assert_eq!(active.name, "second");
    }

    #[test]
    fn test_list_accounts() {
        let mut wallet = Wallet::new();

        wallet.add_account(Account::generate_with_name("acc1".to_string())).unwrap();
        wallet.add_account(Account::generate_with_name("acc2".to_string())).unwrap();

        let names = wallet.list_accounts();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"acc1".to_string()));
        assert!(names.contains(&"acc2".to_string()));
    }

    #[test]
    fn test_add_utxo() {
        let mut wallet = Wallet::with_default_account();
        let account = wallet.get_active_account().unwrap().clone();

        let utxo = Utxo {
            tx_hash: [1u8; 32],
            index: 0,
            amount: 1000,
            key_image: vec![2u8; 32],
        };

        wallet.add_utxo(&account.name, utxo).unwrap();

        assert_eq!(wallet.get_balance(), 1000);
    }

    #[test]
    fn test_get_balance() {
        let mut wallet = Wallet::with_default_account();
        let account = wallet.get_active_account().unwrap().clone();

        let utxo1 = Utxo {
            tx_hash: [1u8; 32],
            index: 0,
            amount: 500,
            key_image: vec![2u8; 32],
        };

        let utxo2 = Utxo {
            tx_hash: [2u8; 32],
            index: 0,
            amount: 300,
            key_image: vec![3u8; 32],
        };

        wallet.add_utxo(&account.name, utxo1).unwrap();
        wallet.add_utxo(&account.name, utxo2).unwrap();

        assert_eq!(wallet.get_balance(), 800);
    }

    #[test]
    fn test_scan_outputs() {
        let mut wallet = Wallet::with_default_account();

        let found = wallet.scan_outputs().unwrap();

        assert!(found > 0);
        assert!(wallet.get_balance() > 0);
    }

    #[test]
    fn test_build_transaction() {
        let mut wallet = Wallet::with_default_account();
        let account = wallet.get_active_account().unwrap().clone();

        // Add sufficient balance
        let utxo = Utxo {
            tx_hash: [1u8; 32],
            index: 0,
            amount: 2000,
            key_image: vec![2u8; 32],
        };
        wallet.add_utxo(&account.name, utxo).unwrap();

        // Build transaction
        let to_address = account.address.to_string();
        let tx = wallet.build_transaction(&to_address, 1000, 0).unwrap();

        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 2); // Output + change
    }

    #[test]
    fn test_insufficient_balance() {
        let wallet = Wallet::with_default_account();
        let account = wallet.get_active_account().unwrap();

        let to_address = account.address.to_string();
        let result = wallet.build_transaction(&to_address, 1000, 0);

        assert!(result.is_err());
        match result.unwrap_err() {
            WalletError::InsufficientBalance { .. } => (),
            _ => panic!("Expected InsufficientBalance error"),
        }
    }

    #[test]
    fn test_send() {
        let mut wallet = Wallet::with_default_account();
        let account = wallet.get_active_account().unwrap().clone();

        // Add balance
        let utxo = Utxo {
            tx_hash: [1u8; 32],
            index: 0,
            amount: 2000,
            key_image: vec![2u8; 32],
        };
        wallet.add_utxo(&account.name, utxo).unwrap();

        let to_address = account.address.to_string();
        let tx_hash = wallet.send(&to_address, 1000).unwrap();

        assert_eq!(tx_hash.len(), 32);
    }

    #[test]
    fn test_get_stats() {
        let mut wallet = Wallet::new();

        wallet.add_account(Account::generate()).unwrap();
        wallet.add_account(Account::generate()).unwrap();

        let stats = wallet.get_stats();

        assert_eq!(stats.total_accounts, 2);
        assert_eq!(stats.active_account, 0);
    }

    #[test]
    fn test_refresh_balance() {
        let mut wallet = Wallet::with_default_account();
        let account = wallet.get_active_account().unwrap().clone();

        let utxo = Utxo {
            tx_hash: [1u8; 32],
            index: 0,
            amount: 1000,
            key_image: vec![2u8; 32],
        };
        wallet.add_utxo(&account.name, utxo).unwrap();

        wallet.refresh_balance();

        assert_eq!(wallet.get_balance(), 1000);
    }

    #[test]
    fn test_get_utxos() {
        let mut wallet = Wallet::with_default_account();
        let account = wallet.get_active_account().unwrap().clone();

        let utxo = Utxo {
            tx_hash: [1u8; 32],
            index: 0,
            amount: 1000,
            key_image: vec![2u8; 32],
        };
        wallet.add_utxo(&account.name, utxo).unwrap();

        let utxos = wallet.get_utxos(&account.name);
        assert_eq!(utxos.len(), 1);
        assert_eq!(utxos[0].amount, 1000);
    }
}
