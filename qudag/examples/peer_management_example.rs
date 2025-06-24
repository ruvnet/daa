/// Example demonstrating peer management functionality in QuDAG CLI
///
/// This example shows how to:
/// - Initialize a peer manager
/// - Add and remove peers
/// - Import/export peer lists
/// - Test connectivity
/// - Manage peer metadata

use qudag_cli::peer_manager::{PeerManager, PeerManagerConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("QuDAG Peer Management Example");
    println!("=============================\n");

    // Configure peer manager
    let config = PeerManagerConfig {
        data_path: PathBuf::from("./peers.json"),
        max_peers: 100,
        auto_save_interval: 300,
        connection_timeout: 30,
        auto_discovery: true,
    };

    // Create peer manager
    let manager = PeerManager::new(config).await?;
    println!("✓ Peer manager initialized\n");

    // Example 1: Add a peer
    println!("1. Adding a peer");
    match manager.add_peer("192.168.1.100:8000".to_string(), Some("Alice's Node".to_string())).await {
        Ok(peer_id) => {
            println!("✓ Successfully added peer: {}", peer_id);
            println!("  Address: 192.168.1.100:8000");
            println!("  Nickname: Alice's Node\n");
        }
        Err(e) => {
            println!("✗ Failed to add peer: {}\n", e);
        }
    }

    // Example 2: List all peers
    println!("2. Listing all peers");
    let peers = manager.list_peers().await?;
    println!("Found {} peers:", peers.len());
    for peer in &peers {
        println!("  - {} ({})", 
            peer.nickname.as_ref().unwrap_or(&"<unnamed>".to_string()),
            peer.address
        );
        println!("    Trust level: {}", peer.trust_level);
        println!("    Tags: {:?}", peer.tags);
    }
    println!();

    // Example 3: Update peer metadata
    if let Some(peer) = peers.first() {
        println!("3. Updating peer metadata");
        manager.update_peer_metadata(
            peer.id.clone(),
            None,
            Some(85), // Increase trust level
            Some(vec!["trusted".to_string(), "fast".to_string()])
        ).await?;
        println!("✓ Updated peer metadata\n");
    }

    // Example 4: Export peers
    println!("4. Exporting peers");
    let export_path = PathBuf::from("./peers_backup.json");
    let count = manager.export_peers(export_path.clone(), None).await?;
    println!("✓ Exported {} peers to {:?}\n", count, export_path);

    // Example 5: Test connectivity
    println!("5. Testing connectivity to all peers");
    let results = manager.test_all_peers(|current, total| {
        print!("\rTesting peer {}/{}...", current, total);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }).await?;
    
    println!("\n\nConnectivity Test Results:");
    let success_count = results.iter().filter(|(_, success, _)| *success).count();
    println!("✓ Successful: {}/{}", success_count, results.len());
    println!("✗ Failed: {}/{}", results.len() - success_count, results.len());
    
    if !results.is_empty() {
        let total_latency: f64 = results.iter()
            .filter_map(|(_, _, latency)| *latency)
            .sum();
        let latency_count = results.iter()
            .filter(|(_, _, latency)| latency.is_some())
            .count();
        
        if latency_count > 0 {
            println!("Average latency: {:.2}ms", total_latency / latency_count as f64);
        }
    }
    println!();

    // Example 6: Network statistics
    println!("6. Network Statistics");
    let stats = manager.get_network_stats().await?;
    println!("Total known peers: {}", stats.total_known_peers);
    println!("Connected peers: {}", stats.connected_peers);
    println!("Average reputation: {:.2}", stats.average_reputation);
    println!("Blacklisted peers: {}", stats.blacklisted_peers);
    println!("Trusted peers: {}", stats.trusted_peers);
    println!();

    // Save before shutdown
    println!("7. Saving peer data");
    manager.save_peers().await?;
    println!("✓ Peer data saved\n");

    // Shutdown
    println!("8. Shutting down");
    manager.shutdown().await?;
    println!("✓ Peer manager shutdown complete");

    Ok(())
}

// Example peer list JSON format:
/*
[
  {
    "id": "12D3KooWExample123",
    "address": "192.168.1.100:8000",
    "nickname": "Alice's Node",
    "trust_level": 85,
    "first_seen": 1700000000,
    "last_seen": 1700001000,
    "total_messages": 1523,
    "success_rate": 0.98,
    "avg_latency_ms": 23.5,
    "tags": ["trusted", "fast"],
    "persistent": true
  },
  {
    "id": "12D3KooWExample456",
    "address": "10.0.0.50:8080",
    "nickname": "Bob's Node",
    "trust_level": 70,
    "first_seen": 1700000500,
    "last_seen": 1700001500,
    "total_messages": 892,
    "success_rate": 0.95,
    "avg_latency_ms": 45.2,
    "tags": ["relay"],
    "persistent": true
  }
]
*/