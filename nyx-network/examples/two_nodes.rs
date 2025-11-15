//! Two nodes example - demonstrates peer connection

use nyx_network::{Node, NodeConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("🌐 Starting Two-Node Network Example\n");

    // ===== Node 1 (Bootstrap) =====
    println!("📍 Creating Node 1 (Bootstrap)...");
    let config1 = NodeConfig {
        listen_addr: "127.0.0.1:8000".parse()?,
        max_peers: 50,
        min_peers: 1,
        bootstrap_peers: vec![],
        node_id: vec![1u8; 32],
    };

    let node1 = Node::new(config1.clone()).await?;
    println!("   ✅ Node 1 ready on {}\n", config1.listen_addr);

    // Start node1 in background
    let node1_clone = node1.clone();
    let node1_handle = tokio::spawn(async move {
        if let Err(e) = node1.run().await {
            eprintln!("Node 1 error: {}", e);
        }
    });

    // Give node1 time to start listening
    sleep(Duration::from_millis(500)).await;

    // ===== Node 2 (Connects to Node 1) =====
    println!("📍 Creating Node 2...");
    let config2 = NodeConfig {
        listen_addr: "127.0.0.1:8001".parse()?,
        max_peers: 50,
        min_peers: 1,
        bootstrap_peers: vec![config1.listen_addr],
        node_id: vec![2u8; 32],
    };

    let node2 = Node::new(config2.clone()).await?;
    println!("   ✅ Node 2 ready on {}", config2.listen_addr);
    println!("   🔗 Bootstrap peer: {}\n", config1.listen_addr);

    // Start node2 in background
    let node2_clone = node2.clone();
    let node2_handle = tokio::spawn(async move {
        if let Err(e) = node2.run().await {
            eprintln!("Node 2 error: {}", e);
        }
    });

    // Wait for connection
    sleep(Duration::from_secs(1)).await;
    println!("⏳ Waiting for nodes to connect...\n");
    sleep(Duration::from_secs(1)).await;

    // Check stats
    println!("📊 Network Statistics:\n");

    let stats1 = node1_clone.stats().await;
    println!("Node 1:");
    println!("   Connected peers: {}", stats1.peer_count);
    println!("   Active gossip peers: {}", stats1.gossip_stats.active_peers);

    let stats2 = node2_clone.stats().await;
    println!("\nNode 2:");
    println!("   Connected peers: {}", stats2.peer_count);
    println!("   Active gossip peers: {}", stats2.gossip_stats.active_peers);

    // Keep running for a bit
    println!("\n🔄 Network running for 10 seconds...\n");
    sleep(Duration::from_secs(10)).await;

    // Cleanup
    println!("🛑 Shutting down nodes...");
    node1_handle.abort();
    node2_handle.abort();
    sleep(Duration::from_millis(200)).await;

    println!("✅ Example completed successfully!\n");

    Ok(())
}
