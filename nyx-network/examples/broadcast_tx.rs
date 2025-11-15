//! Broadcasting transactions example

use nyx_network::{Node, NodeConfig};
use nyx_core::{
    transaction::{Transaction, TxInput, TxOutput},
    RingSignature,
};
use tokio::time::{sleep, Duration};

fn create_test_transaction(nonce: u8) -> Transaction {
    Transaction::new(
        vec![TxInput {
            prev_tx: [nonce; 32],
            index: 0,
            key_image: [nonce; 32],
            ring_indices: vec![0, 1, 2, 3],
        }],
        vec![TxOutput {
            stealth_address: vec![nonce; 32],
            amount_commitment: vec![nonce; 32],
            range_proof: vec![nonce; 64],
            ephemeral_pubkey: vec![nonce; 32],
        }],
        RingSignature {
            ring_members: vec![vec![nonce; 32]; 4],
            signature: vec![nonce; 64],
            key_image: [nonce; 32],
        },
        vec![nonce; 32],
        [0u8; 32],
        [1u8; 32],
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("📡 Transaction Broadcasting Example\n");

    // Create network with 3 nodes
    println!("🌐 Setting up 3-node network...\n");

    // Node 1
    let config1 = NodeConfig {
        listen_addr: "127.0.0.1:8100".parse()?,
        node_id: vec![1u8; 32],
        ..Default::default()
    };
    let node1 = Node::new(config1.clone()).await?;
    let node1_clone = node1.clone();

    tokio::spawn(async move {
        let _ = node1.run().await;
    });
    sleep(Duration::from_millis(300)).await;
    println!("✅ Node 1 ready on {}", config1.listen_addr);

    // Node 2
    let config2 = NodeConfig {
        listen_addr: "127.0.0.1:8101".parse()?,
        node_id: vec![2u8; 32],
        bootstrap_peers: vec![config1.listen_addr],
        ..Default::default()
    };
    let node2 = Node::new(config2.clone()).await?;
    let node2_clone = node2.clone();

    tokio::spawn(async move {
        let _ = node2.run().await;
    });
    sleep(Duration::from_millis(300)).await;
    println!("✅ Node 2 ready on {}", config2.listen_addr);

    // Node 3
    let config3 = NodeConfig {
        listen_addr: "127.0.0.1:8102".parse()?,
        node_id: vec![3u8; 32],
        bootstrap_peers: vec![config1.listen_addr, config2.listen_addr],
        ..Default::default()
    };
    let node3 = Node::new(config3.clone()).await?;
    let node3_clone = node3.clone();

    tokio::spawn(async move {
        let _ = node3.run().await;
    });
    sleep(Duration::from_millis(300)).await;
    println!("✅ Node 3 ready on {}", config3.listen_addr);

    // Wait for network to stabilize
    println!("\n⏳ Waiting for network to stabilize...");
    sleep(Duration::from_secs(2)).await;

    // Broadcast transactions from Node 1
    println!("\n📡 Broadcasting 5 transactions from Node 1...\n");

    for i in 0..5 {
        let tx = create_test_transaction(i);
        let tx_id = tx.id();

        match node1_clone.broadcast_transaction(tx).await {
            Ok(()) => {
                println!("   ✅ Tx {}: {} broadcast", i + 1, hex::encode(&tx_id[..8]));
            }
            Err(e) => {
                println!("   ❌ Tx {} failed: {}", i + 1, e);
            }
        }

        sleep(Duration::from_millis(200)).await;
    }

    // Wait for propagation
    println!("\n⏳ Waiting for gossip propagation...");
    sleep(Duration::from_secs(2)).await;

    // Check final stats
    println!("\n📊 Final Network Statistics:\n");

    let stats1 = node1_clone.stats().await;
    println!("Node 1:");
    println!("   Peers: {}", stats1.peer_count);
    println!("   Seen messages: {}", stats1.gossip_stats.seen_messages);
    println!("   Active peers: {}", stats1.gossip_stats.active_peers);

    let stats2 = node2_clone.stats().await;
    println!("\nNode 2:");
    println!("   Peers: {}", stats2.peer_count);
    println!("   Seen messages: {}", stats2.gossip_stats.seen_messages);
    println!("   Active peers: {}", stats2.gossip_stats.active_peers);

    let stats3 = node3_clone.stats().await;
    println!("\nNode 3:");
    println!("   Peers: {}", stats3.peer_count);
    println!("   Seen messages: {}", stats3.gossip_stats.seen_messages);
    println!("   Active peers: {}", stats3.gossip_stats.active_peers);

    println!("\n✅ Broadcasting example completed!\n");

    Ok(())
}

