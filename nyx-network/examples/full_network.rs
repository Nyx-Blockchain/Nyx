//! Full network example - 5 nodes with transaction broadcasting

use nyx_network::{Node, NodeConfig};
use nyx_core::{
    transaction::{Transaction, TxInput, TxOutput},
    RingSignature,
};
use tokio::time::{sleep, Duration};

fn create_transaction(id: u8) -> Transaction {
    Transaction::new(
        vec![TxInput {
            prev_tx: [id; 32],
            index: 0,
            key_image: [id; 32],
            ring_indices: vec![0, 1, 2, 3],
        }],
        vec![TxOutput {
            stealth_address: vec![id; 32],
            amount_commitment: vec![id; 32],
            range_proof: vec![id; 64],
            ephemeral_pubkey: vec![id; 32],
        }],
        RingSignature {
            ring_members: vec![vec![id; 32]; 4],
            signature: vec![id; 64],
            key_image: [id; 32],
        },
        vec![id; 32],
        [0u8; 32],
        [1u8; 32],
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("🌐 Full Network Example - 5 Nodes\n");
    println!("{}", "=".repeat(50));

    let mut nodes = Vec::new();
    let mut bootstrap_peers = Vec::new();

    // Create 5 nodes
    for i in 0..5 {
        let port = 8200 + i;
        let addr = format!("127.0.0.1:{}", port).parse()?;

        let config = NodeConfig {
            listen_addr: addr,
            node_id: vec![i as u8; 32],
            bootstrap_peers: bootstrap_peers.clone(),
            max_peers: 10,
            min_peers: 2,
        };

        let node = Node::new(config.clone()).await?;
        println!("✅ Node {} created on {}", i + 1, addr);

        // Store for later use
        let node_clone = node.clone();
        nodes.push(node_clone);

        // Start node
        tokio::spawn(async move {
            let _ = node.run().await;
        });

        // Add to bootstrap peers for next nodes
        bootstrap_peers.push(addr);

        // Wait between node startups
        sleep(Duration::from_millis(300)).await;
    }

    println!("\n⏳ Waiting for network to stabilize...");
    sleep(Duration::from_secs(3)).await;

    // Show network status
    println!("\n📊 Network Status:");
    println!("{}", "=".repeat(50));
    for (i, node) in nodes.iter().enumerate() {
        let stats = node.stats().await;
        println!("\nNode {}:", i + 1);
        println!("   Connected peers: {}", stats.peer_count);
        println!("   Gossip active: {}", stats.gossip_stats.active_peers);
        println!("   Messages seen: {}", stats.gossip_stats.seen_messages);
    }

    // Broadcast transactions from different nodes
    println!("\n\n📡 Broadcasting Transactions:");
    println!("{}", "=".repeat(50));

    // Node 0 broadcasts 3 transactions
    println!("\n🔵 Node 1 broadcasting...");
    for i in 0..3 {
        let tx = create_transaction(i);
        let tx_id = tx.id();
        nodes[0].broadcast_transaction(tx).await?;
        println!("   ✅ Tx {}: {}", i + 1, hex::encode(&tx_id[..8]));
        sleep(Duration::from_millis(100)).await;
    }

    sleep(Duration::from_millis(500)).await;

    // Node 2 broadcasts 2 transactions
    println!("\n🟢 Node 3 broadcasting...");
    for i in 10..12 {
        let tx = create_transaction(i);
        let tx_id = tx.id();
        nodes[2].broadcast_transaction(tx).await?;
        println!("   ✅ Tx {}: {}", i - 9, hex::encode(&tx_id[..8]));
        sleep(Duration::from_millis(100)).await;
    }

    // Wait for propagation
    println!("\n⏳ Waiting for gossip propagation...");
    sleep(Duration::from_secs(2)).await;

    // Final statistics
    println!("\n\n📈 Final Network Statistics:");
    println!("{}", "=".repeat(50));

    let mut total_messages = 0;
    let mut total_peers = 0;

    for (i, node) in nodes.iter().enumerate() {
        let stats = node.stats().await;
        total_messages += stats.gossip_stats.seen_messages;
        total_peers += stats.peer_count;

        println!("\nNode {}:", i + 1);
        println!("   Peers: {}", stats.peer_count);
        println!("   Messages: {}", stats.gossip_stats.seen_messages);
        println!("   Active: {}", stats.gossip_stats.active_peers);
    }

    println!("\n📊 Aggregate Stats:");
    println!("   Total messages propagated: {}", total_messages);
    println!("   Average peers per node: {:.1}", total_peers as f64 / nodes.len() as f64);

    println!("\n✅ Full network example completed successfully!\n");

    Ok(())
}
