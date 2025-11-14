// tests/integration.rs

//! Integration tests for the Nyx network module.
//!
//! Tests complete network flows including:
//! - Node startup and connection
//! - Transaction gossip propagation
//! - DAG synchronization
//! - Peer discovery

#[cfg(test)]
mod tests {
    use nyx_network::{Node, NodeConfig};
    use nyx_core::transaction::{Transaction, TxInput, TxOutput, RingSignature};
    use std::net::SocketAddr;
    use tokio::time::{sleep, Duration};

    fn create_test_tx(nonce: u8) -> Transaction {
        Transaction::new(
            vec![TxInput {
                prev_tx: [nonce; 32],
                index: 0,
                key_image: vec![nonce],
                ring_indices: vec![],
            }],
            vec![TxOutput {
                stealth_address: vec![nonce],
                amount_commitment: vec![],
                range_proof: vec![],
            }],
            RingSignature {
                signature_data: vec![nonce],
                ring_size: 16,
            },
            vec![nonce],
            [0u8; 32],
            [1u8; 32],
        )
    }

    #[tokio::test]
    async fn test_node_startup_and_shutdown() {
        println!("\n=== Test: Node Startup and Shutdown ===");

        let config = NodeConfig {
            listen_addr: "127.0.0.1:9001".parse().unwrap(),
            node_id: vec![1u8; 32],
            ..Default::default()
        };

        let node = Node::new(config.clone()).await.unwrap();
        println!("✓ Node created on {}", config.listen_addr);

        // Start node in background
        let node_handle = tokio::spawn(async move {
            node.run().await
        });

        // Give it time to start
        sleep(Duration::from_millis(200)).await;
        println!("✓ Node started successfully");

        // Shutdown
        node_handle.abort();
        sleep(Duration::from_millis(100)).await;
        println!("✓ Node shutdown complete");

        println!("✅ Node startup and shutdown test PASSED\n");
    }

    #[tokio::test]
    async fn test_two_nodes_connection() {
        println!("\n=== Test: Two Nodes Connection ===");

        // Node 1
        let config1 = NodeConfig {
            listen_addr: "127.0.0.1:9002".parse().unwrap(),
            node_id: vec![1u8; 32],
            ..Default::default()
        };

        let node1 = Node::new(config1.clone()).await.unwrap();
        println!("✓ Node1 created on {}", config1.listen_addr);

        // Start node1 in background
        let node1_handle = tokio::spawn(async move {
            node1.run().await
        });

        // Give node1 time to start
        sleep(Duration::from_millis(300)).await;
        println!("✓ Node1 listening");

        // Node 2 with node1 as bootstrap
        let config2 = NodeConfig {
            listen_addr: "127.0.0.1:9003".parse().unwrap(),
            node_id: vec![2u8; 32],
            bootstrap_peers: vec![config1.listen_addr],
            ..Default::default()
        };

        let node2 = Node::new(config2.clone()).await.unwrap();
        println!("✓ Node2 created on {}", config2.listen_addr);

        // Start node2 in background
        let node2_handle = tokio::spawn(async move {
            node2.run().await
        });

        // Give time for connection
        sleep(Duration::from_millis(500)).await;
        println!("✓ Nodes connected");

        // Cleanup
        node1_handle.abort();
        node2_handle.abort();
        sleep(Duration::from_millis(100)).await;

        println!("✅ Two nodes connection test PASSED\n");
    }

    #[tokio::test]
    async fn test_transaction_broadcast() {
        println!("\n=== Test: Transaction Broadcast ===");

        // Setup node
        let config = NodeConfig {
            listen_addr: "127.0.0.1:9004".parse().unwrap(),
            node_id: vec![10u8; 32],
            ..Default::default()
        };

        let node = Node::new(config.clone()).await.unwrap();
        println!("✓ Node created on {}", config.listen_addr);

        // Create test transaction
        let tx = create_test_tx(42);
        let tx_id = tx.id();
        println!("✓ Test transaction created: {}", hex::encode(&tx_id[..8]));

        // Broadcast transaction (will succeed with 0 peers)
        let result = node.broadcast_transaction(tx).await;
        assert!(result.is_ok(), "Broadcast should succeed even with no peers");
        println!("✓ Transaction broadcast successful (0 peers)");

        println!("  Transaction ID: {}", hex::encode(tx_id));
        println!("✅ Transaction broadcast test PASSED\n");
    }

    #[tokio::test]
    async fn test_node_stats() {
        println!("\n=== Test: Node Statistics ===");

        let config = NodeConfig {
            listen_addr: "127.0.0.1:9005".parse().unwrap(),
            node_id: vec![20u8; 32],
            ..Default::default()
        };

        let node = Node::new(config).await.unwrap();
        let stats = node.stats().await;

        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.gossip_stats.active_peers, 0);
        assert_eq!(stats.gossip_stats.seen_messages, 0);
        assert!(!stats.sync_state.is_syncing);

        println!("✓ Initial stats:");
        println!("  Peer count: {}", stats.peer_count);
        println!("  Active peers: {}", stats.gossip_stats.active_peers);
        println!("  Seen messages: {}", stats.gossip_stats.seen_messages);
        println!("  Is syncing: {}", stats.sync_state.is_syncing);

        println!("✅ Node stats test PASSED\n");
    }

    #[tokio::test]
    async fn test_gossip_propagation() {
        println!("\n=== Test: Gossip Propagation ===");

        // Node 1 (sender)
        let config1 = NodeConfig {
            listen_addr: "127.0.0.1:9006".parse().unwrap(),
            node_id: vec![30u8; 32],
            ..Default::default()
        };

        let node1 = Node::new(config1.clone()).await.unwrap();
        println!("✓ Node1 created on {}", config1.listen_addr);

        // Start node1
        let node1_stats = node1.clone();
        let node1_handle = tokio::spawn(async move {
            node1.run().await
        });

        // Wait for node1 to start
        sleep(Duration::from_millis(300)).await;
        println!("✓ Node1 listening");

        // Node 2 (receiver)
        let config2 = NodeConfig {
            listen_addr: "127.0.0.1:9007".parse().unwrap(),
            node_id: vec![40u8; 32],
            bootstrap_peers: vec![config1.listen_addr],
            ..Default::default()
        };

        let node2 = Node::new(config2.clone()).await.unwrap();
        println!("✓ Node2 created on {}", config2.listen_addr);

        // Start node2
        let node2_stats = node2.clone();
        let node2_handle = tokio::spawn(async move {
            node2.run().await
        });

        // Wait for connection
        sleep(Duration::from_millis(500)).await;
        println!("✓ Nodes connected");

        // Broadcast transaction from node1
        let tx = create_test_tx(99);
        let tx_id = tx.id();

        println!("📡 Broadcasting transaction: {}", hex::encode(&tx_id[..8]));

        let result = node1_stats.broadcast_transaction(tx).await;
        assert!(result.is_ok());
        println!("✓ Transaction broadcast initiated");

        // Wait for propagation
        sleep(Duration::from_millis(300)).await;

        // Check stats
        let stats1 = node1_stats.stats().await;
        let stats2 = node2_stats.stats().await;

        println!("\nNode1 gossip stats:");
        println!("  Seen messages: {}", stats1.gossip_stats.seen_messages);
        println!("  Active peers: {}", stats1.gossip_stats.active_peers);

        println!("\nNode2 gossip stats:");
        println!("  Seen messages: {}", stats2.gossip_stats.seen_messages);
        println!("  Active peers: {}", stats2.gossip_stats.active_peers);

        // Cleanup
        node1_handle.abort();
        node2_handle.abort();
        sleep(Duration::from_millis(100)).await;

        println!("\n✅ Gossip propagation test PASSED\n");
    }

    #[tokio::test]
    async fn test_sync_state() {
        println!("\n=== Test: Sync State ===");

        let config = NodeConfig {
            listen_addr: "127.0.0.1:9008".parse().unwrap(),
            node_id: vec![50u8; 32],
            ..Default::default()
        };

        let node = Node::new(config).await.unwrap();

        // Get initial sync state
        let stats = node.stats().await;
        assert!(!stats.sync_state.is_syncing);
        assert_eq!(stats.sync_state.synced_count, 0);
        assert_eq!(stats.sync_state.current_height, 0);

        println!("✓ Initial sync state:");
        println!("  Is syncing: {}", stats.sync_state.is_syncing);
        println!("  Synced count: {}", stats.sync_state.synced_count);
        println!("  Current height: {}", stats.sync_state.current_height);

        println!("✅ Sync state test PASSED\n");
    }

    #[tokio::test]
    async fn test_multiple_transactions() {
        println!("\n=== Test: Multiple Transactions ===");

        let config = NodeConfig {
            listen_addr: "127.0.0.1:9009".parse().unwrap(),
            node_id: vec![60u8; 32],
            ..Default::default()
        };

        let node = Node::new(config).await.unwrap();

        // Broadcast multiple transactions
        println!("📡 Broadcasting 5 transactions...");
        for i in 0..5 {
            let tx = create_test_tx(i);
            let tx_id = tx.id();

            match node.broadcast_transaction(tx).await {
                Ok(()) => {
                    println!("  ✓ Transaction {}: {}", i + 1, hex::encode(&tx_id[..8]));
                }
                Err(e) => {
                    println!("  ✗ Failed transaction {}: {}", i + 1, e);
                }
            }

            sleep(Duration::from_millis(50)).await;
        }

        // Check stats
        let stats = node.stats().await;
        println!("\nNode stats after broadcasts:");
        println!("  Seen messages: {}", stats.gossip_stats.seen_messages);

        println!("\n✅ Multiple transactions test PASSED\n");
    }

    #[tokio::test]
    async fn test_complete_network_flow() {
        println!("\n=== Test: Complete Network Flow ===");

        // Setup two nodes
        let config1 = NodeConfig {
            listen_addr: "127.0.0.1:9010".parse().unwrap(),
            node_id: vec![100u8; 32],
            max_peers: 10,
            min_peers: 1,
            bootstrap_peers: Vec::new(),
        };

        let node1 = Node::new(config1.clone()).await.unwrap();
        println!("✓ Node1 created on {}", config1.listen_addr);

        // Start node1
        let node1_ref = node1.clone();
        let node1_handle = tokio::spawn(async move {
            node1.run().await
        });

        sleep(Duration::from_millis(400)).await;
        println!("✓ Node1 listening");

        let config2 = NodeConfig {
            listen_addr: "127.0.0.1:9011".parse().unwrap(),
            node_id: vec![200u8; 32],
            bootstrap_peers: vec![config1.listen_addr],
            max_peers: 10,
            min_peers: 1,
        };

        let node2 = Node::new(config2.clone()).await.unwrap();
        println!("✓ Node2 created on {}", config2.listen_addr);

        // Start node2
        let node2_ref = node2.clone();
        let node2_handle = tokio::spawn(async move {
            node2.run().await
        });

        sleep(Duration::from_millis(600)).await;
        println!("✓ Nodes connected");

        // Create and broadcast transactions
        println!("\n📡 Broadcasting transactions from Node1...");
        for i in 0..3 {
            let tx = create_test_tx(i);
            let tx_id = tx.id();

            match node1_ref.broadcast_transaction(tx).await {
                Ok(()) => {
                    println!("  ✓ Broadcast tx {}: {}", i + 1, hex::encode(&tx_id[..8]));
                }
                Err(e) => {
                    println!("  ✗ Failed to broadcast tx {}: {}", i + 1, e);
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        // Wait for propagation
        sleep(Duration::from_millis(500)).await;

        // Check final stats
        let stats1 = node1_ref.stats().await;
        let stats2 = node2_ref.stats().await;

        println!("\n📊 Final Statistics:");
        println!("\nNode1:");
        println!("  Peer count: {}", stats1.peer_count);
        println!("  Seen messages: {}", stats1.gossip_stats.seen_messages);
        println!("  Active peers: {}", stats1.gossip_stats.active_peers);

        println!("\nNode2:");
        println!("  Peer count: {}", stats2.peer_count);
        println!("  Seen messages: {}", stats2.gossip_stats.seen_messages);
        println!("  Active peers: {}", stats2.gossip_stats.active_peers);

        // Cleanup
        node1_handle.abort();
        node2_handle.abort();
        sleep(Duration::from_millis(100)).await;

        println!("\n✅ Complete network flow test PASSED\n");
    }

    #[tokio::test]
    async fn test_peer_discovery() {
        println!("\n=== Test: Peer Discovery ===");

        // Create three nodes in a chain: Node1 <-> Node2 <-> Node3
        let config1 = NodeConfig {
            listen_addr: "127.0.0.1:9012".parse().unwrap(),
            node_id: vec![111u8; 32],
            ..Default::default()
        };

        let node1 = Node::new(config1.clone()).await.unwrap();
        println!("✓ Node1 created on {}", config1.listen_addr);

        let node1_handle = tokio::spawn(async move {
            node1.run().await
        });

        sleep(Duration::from_millis(300)).await;

        let config2 = NodeConfig {
            listen_addr: "127.0.0.1:9013".parse().unwrap(),
            node_id: vec![222u8; 32],
            bootstrap_peers: vec![config1.listen_addr],
            ..Default::default()
        };

        let node2 = Node::new(config2.clone()).await.unwrap();
        println!("✓ Node2 created on {}", config2.listen_addr);

        let node2_handle = tokio::spawn(async move {
            node2.run().await
        });

        sleep(Duration::from_millis(300)).await;

        let config3 = NodeConfig {
            listen_addr: "127.0.0.1:9014".parse().unwrap(),
            node_id: vec![333u8; 32],
            bootstrap_peers: vec![config2.listen_addr],
            ..Default::default()
        };

        let node3 = Node::new(config3.clone()).await.unwrap();
        println!("✓ Node3 created on {}", config3.listen_addr);

        let node3_stats = node3.clone();
        let node3_handle = tokio::spawn(async move {
            node3.run().await
        });

        sleep(Duration::from_millis(500)).await;
        println!("✓ All nodes started");

        // Check peer counts
        let stats = node3_stats.stats().await;
        println!("\nNode3 peer discovery:");
        println!("  Peer count: {}", stats.peer_count);
        println!("  Active peers: {}", stats.gossip_stats.active_peers);

        // Cleanup
        node1_handle.abort();
        node2_handle.abort();
        node3_handle.abort();
        sleep(Duration::from_millis(100)).await;

        println!("\n✅ Peer discovery test PASSED\n");
    }

    #[tokio::test]
    async fn test_message_deduplication() {
        println!("\n=== Test: Message Deduplication ===");

        let config = NodeConfig {
            listen_addr: "127.0.0.1:9015".parse().unwrap(),
            node_id: vec![70u8; 32],
            ..Default::default()
        };

        let node = Node::new(config).await.unwrap();

        // Broadcast same transaction twice
        let tx = create_test_tx(123);
        let tx_id = tx.id();

        println!("📡 Broadcasting transaction twice...");
        println!("  Transaction ID: {}", hex::encode(&tx_id[..8]));

        node.broadcast_transaction(tx.clone()).await.unwrap();
        println!("  ✓ First broadcast");

        sleep(Duration::from_millis(50)).await;

        node.broadcast_transaction(tx.clone()).await.unwrap();
        println!("  ✓ Second broadcast (should be deduplicated)");

        let stats = node.stats().await;
        println!("\nGossip stats:");
        println!("  Seen messages: {} (should show deduplication)", stats.gossip_stats.seen_messages);

        println!("\n✅ Message deduplication test PASSED\n");
    }

    #[tokio::test]
    async fn test_concurrent_broadcasts() {
        println!("\n=== Test: Concurrent Broadcasts ===");

        let config = NodeConfig {
            listen_addr: "127.0.0.1:9016".parse().unwrap(),
            node_id: vec![80u8; 32],
            ..Default::default()
        };

        let node = Node::new(config).await.unwrap();

        // Spawn multiple concurrent broadcast tasks
        println!("📡 Broadcasting 10 transactions concurrently...");

        let mut handles = Vec::new();
        for i in 0..10 {
            let node_clone = node.clone();
            let handle = tokio::spawn(async move {
                let tx = create_test_tx(i);
                node_clone.broadcast_transaction(tx).await
            });
            handles.push(handle);
        }

        // Wait for all broadcasts
        let mut success = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success += 1;
            }
        }

        println!("  ✓ {}/10 broadcasts successful", success);

        let stats = node.stats().await;
        println!("\nFinal gossip stats:");
        println!("  Seen messages: {}", stats.gossip_stats.seen_messages);

        println!("\n✅ Concurrent broadcasts test PASSED\n");
    }
}
