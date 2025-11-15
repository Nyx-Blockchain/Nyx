//! Simple node example - starts a single network node

use nyx_network::{Node, NodeConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Nyx Network Node...\n");

    // Configure node
    let config = NodeConfig {
        listen_addr: "127.0.0.1:8000".parse()?,
        max_peers: 50,
        min_peers: 8,
        bootstrap_peers: vec![],
        node_id: vec![1u8; 32],
    };

    println!("📍 Node configuration:");
    println!("   Address: {}", config.listen_addr);
    println!("   Max peers: {}", config.max_peers);
    println!("   Node ID: {}\n", hex::encode(&config.node_id));

    // Create node
    let node = Node::new(config.clone()).await?;
    println!("✅ Node created successfully!\n");

    // Get initial stats
    let stats = node.stats().await;
    println!("📊 Initial stats:");
    println!("   Peers: {}", stats.peer_count);
    println!("   Gossip messages: {}", stats.gossip_stats.seen_messages);
    println!("   Is syncing: {}\n", stats.sync_state.is_syncing);

    // Run node in background
    println!("🔄 Starting node (press Ctrl+C to stop)...\n");

    let node_handle = tokio::spawn(async move {
        if let Err(e) = node.run().await {
            eprintln!("❌ Node error: {}", e);
        }
    });

    // Monitor stats every 5 seconds
    for i in 1..=6 {
        sleep(Duration::from_secs(5)).await;
        println!("⏱️  Uptime: {} seconds", i * 5);
    }

    println!("\n✅ Example completed! Shutting down...");
    node_handle.abort();

    Ok(())
}
