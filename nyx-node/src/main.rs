// src/main.rs

//! Nyx node binary entry point.

use nyx_node::{NodeConfig, NyxNode};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Nyx Blockchain Node");
    tracing::info!("Version: {}", nyx_node::NODE_VERSION);

    // Load or create configuration
    let config = NodeConfig::default();

    // Create and start node
    let node = NyxNode::new(config).await?;
    node.start().await?;

    Ok(())
}
