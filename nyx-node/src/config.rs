// src/config.rs

//! Node configuration.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Complete node configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Network configuration
    pub network: NetworkConfig,

    /// RPC server configuration
    pub rpc: RpcConfig,

    /// DAG configuration
    pub dag: DagConfig,

    /// Wallet configuration
    pub wallet: WalletConfig,

    /// Data directory
    pub data_dir: PathBuf,
}

/// Network configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P listen address
    pub listen_addr: SocketAddr,

    /// Maximum peers
    pub max_peers: usize,

    /// Bootstrap peers
    pub bootstrap_peers: Vec<SocketAddr>,
}

/// RPC server configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcConfig {
    /// RPC listen address
    pub listen_addr: SocketAddr,

    /// Enable RPC server
    pub enabled: bool,
}

/// DAG configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DagConfig {
    /// Confirmation threshold
    pub confirmation_threshold: f64,

    /// Sync interval in seconds
    pub sync_interval: u64,
}

/// Wallet configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletConfig {
    /// Enable built-in wallet
    pub enabled: bool,

    /// Wallet data directory
    pub wallet_dir: PathBuf,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                listen_addr: "127.0.0.1:8000".parse().unwrap(),
                max_peers: 50,
                bootstrap_peers: Vec::new(),
            },
            rpc: RpcConfig {
                listen_addr: "127.0.0.1:9000".parse().unwrap(),
                enabled: true,
            },
            dag: DagConfig {
                confirmation_threshold: 100.0,
                sync_interval: 60,
            },
            wallet: WalletConfig {
                enabled: true,
                wallet_dir: PathBuf::from(".nyx-wallet"),
            },
            data_dir: PathBuf::from(".nyx-data"),
        }
    }
}

impl NodeConfig {
    /// Loads configuration from file
    pub fn from_file(path: &std::path::Path) -> crate::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e| crate::NodeError::ConfigError(format!("Failed to parse config: {}", e)))
    }

    /// Saves configuration to file
    pub fn save_to_file(&self, path: &std::path::Path) -> crate::Result<()> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| crate::NodeError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}
