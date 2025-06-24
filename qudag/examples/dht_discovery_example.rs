//! Example demonstrating Kademlia DHT for decentralized peer discovery
//! 
//! This example shows how to use the QuDAG network layer with Kademlia DHT
//! for peer discovery, content routing, and dark addressing support.

use qudag_network::{
    BootstrapConfig, ContentRoutingConfig, KademliaDHT, PeerReputation,
    DiscoveryConfig, DiscoveryMethod, KademliaPeerDiscovery, DiscoveryEvent,
};
use libp2p::PeerId as LibP2PPeerId;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, error, debug};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Kademlia DHT Discovery Example");

    // Create a local peer ID
    let local_peer_id = LibP2PPeerId::random();
    info!("Local peer ID: {}", local_peer_id);

    // Configure DHT settings
    let dht_config = qudag_network::discovery::DHTConfig {
        bucket_size: 20,
        alpha: 3,
        replication_factor: 20,
        key_space_bits: 256,
        bootstrap_timeout: Duration::from_secs(30),
        refresh_interval: Duration::from_secs(3600),
        enable_republishing: true,
    };

    // Configure bootstrap nodes (in a real scenario, these would be known stable nodes)
    let bootstrap_config = BootstrapConfig {
        nodes: vec![
            // Add some example bootstrap nodes
            (LibP2PPeerId::random(), "127.0.0.1:8000".parse()?),
            (LibP2PPeerId::random(), "127.0.0.1:8001".parse()?),
        ],
        timeout: Duration::from_secs(30),
        min_connections: 2,
        periodic_bootstrap: true,
        bootstrap_interval: Duration::from_secs(3600),
    };

    // Configure content routing
    let content_config = ContentRoutingConfig {
        enabled: true,
        provider_ttl: Duration::from_secs(24 * 60 * 60),
        replication_factor: 20,
        auto_republish: true,
        republish_interval: Duration::from_secs(12 * 60 * 60),
        max_content_size: 1024 * 1024, // 1MB
    };

    // Create peer scoring configuration
    let scoring_config = qudag_network::discovery::PeerScoringConfig {
        initial_score: 50.0,
        max_score: 100.0,
        min_score: -50.0,
        score_decay_rate: 0.1,
        connection_success_bonus: 5.0,
        connection_failure_penalty: 10.0,
        uptime_bonus: 1.0,
        latency_penalty_factor: 0.01,
        enable_geographic_scoring: true,
    };

    // Create Kademlia DHT instance
    let mut dht = KademliaDHT::new(
        local_peer_id,
        dht_config,
        bootstrap_config,
        content_config,
        scoring_config.clone(),
    );

    // Set up event channel for discovery events
    let (event_tx, mut event_rx) = mpsc::channel(100);
    dht.set_event_channel(event_tx.clone());

    // Create peer discovery service
    let discovery_config = DiscoveryConfig {
        methods: vec![DiscoveryMethod::Kademlia, DiscoveryMethod::Mdns],
        bootstrap_nodes: vec!["127.0.0.1:8000".parse()?],
        interval: 30,
        max_peers: 50,
        min_peers: 8,
        reputation_threshold: 0.0,
        network_config: qudag_network::discovery::NetworkConfig::Public {
            nat_traversal: true,
            upnp: true,
            stun_turn: true,
        },
        enable_dark_addressing: true,
        dark_resolver_config: Default::default(),
        dht_config: dht_config.clone(),
        max_concurrent_connections: 100,
        scoring_config: scoring_config.clone(),
        load_balancing_config: Default::default(),
        geo_preferences: Default::default(),
    };

    let mut peer_discovery = KademliaPeerDiscovery::new(discovery_config);
    peer_discovery.set_event_channel(event_tx);

    // Start background task for handling discovery events
    let event_handler = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                DiscoveryEvent::PeerDiscovered(peer) => {
                    info!("Discovered new peer: {} via {:?}", 
                          peer.peer_id, peer.discovery_method);
                    debug!("Peer details: reputation={}, addresses={:?}", 
                           peer.reputation, peer.addresses);
                }
                DiscoveryEvent::BootstrapCompleted { 
                    peers_discovered, 
                    duration, 
                    success_rate 
                } => {
                    info!("Bootstrap completed: {} peers discovered in {:?} (success rate: {:.2}%)",
                          peers_discovered, duration, success_rate * 100.0);
                }
                DiscoveryEvent::BootstrapFailed { reason, .. } => {
                    error!("Bootstrap failed: {}", reason);
                }
                DiscoveryEvent::ReputationUpdated { 
                    peer_id, 
                    new_reputation, 
                    reason 
                } => {
                    debug!("Peer {} reputation updated to {:.2}: {}", 
                           peer_id, new_reputation, reason);
                }
                DiscoveryEvent::PeerBlacklisted { peer_id, reason, .. } => {
                    info!("Peer {} blacklisted: {}", peer_id, reason);
                }
                DiscoveryEvent::DarkAddressDiscovered { 
                    peer_id, 
                    dark_address, 
                    resolution_time 
                } => {
                    info!("Dark address discovered for peer {} in {:?}",
                          peer_id, resolution_time);
                    debug!("Dark address details: {:?}", dark_address);
                }
                DiscoveryEvent::TopologyUpdated { 
                    largest_component_size, 
                    avg_clustering, 
                    diameter 
                } => {
                    info!("Network topology updated: {} nodes in largest component, clustering={:.3}",
                          largest_component_size, avg_clustering);
                    if let Some(d) = diameter {
                        debug!("Network diameter: {}", d);
                    }
                }
                DiscoveryEvent::DHTBucketUpdated { 
                    bucket_index, 
                    peer_count, 
                    health_score 
                } => {
                    debug!("DHT bucket {} updated: {} peers, health={:.2}",
                           bucket_index, peer_count, health_score);
                }
                _ => {}
            }
        }
    });

    // Start peer discovery
    peer_discovery.start().await?;
    info!("Peer discovery started");

    // Simulate content storage and retrieval
    tokio::spawn(async move {
        // Wait a bit for bootstrap to complete
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Store some content in the DHT
        let content_key = b"example_content_key".to_vec();
        let content_value = b"This is example content stored in the DHT".to_vec();
        
        if let Err(e) = dht.store_record(content_key.clone(), content_value).await {
            error!("Failed to store content: {:?}", e);
        } else {
            info!("Content stored successfully");
        }

        // Wait a bit then try to retrieve it
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        if let Err(e) = dht.get_record(content_key).await {
            error!("Failed to retrieve content: {:?}", e);
        } else {
            info!("Content retrieval initiated");
        }

        // Announce ourselves as a provider for some content
        let provider_key = b"service_example".to_vec();
        if let Err(e) = dht.provide(provider_key).await {
            error!("Failed to announce as provider: {:?}", e);
        } else {
            info!("Announced as content provider");
        }

        // Periodically show DHT metrics
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            
            let metrics = dht.get_metrics();
            info!("DHT Metrics: {} total queries, {} successful, routing table size: {}",
                  metrics.total_queries, metrics.successful_queries, metrics.routing_table_size);
            
            if let Some(top_peers) = dht.get_top_peers(5).await.get(0..5) {
                info!("Top 5 peers by reputation:");
                for (i, peer) in top_peers.iter().enumerate() {
                    info!("  {}. {} (score: {:.2}, interactions: {})",
                          i + 1, peer.peer_id, peer.score, peer.total_interactions);
                }
            }
            
            // Perform maintenance
            dht.perform_maintenance().await;
        }
    });

    // Demonstrate peer management
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            
            // Get discovery statistics
            let stats = peer_discovery.get_discovery_stats().await;
            info!("Discovery Stats: {} total peers, {} connectable, avg reputation: {:.2}",
                  stats.total_discovered, stats.connectable_peers, stats.average_reputation);
            
            // Show method breakdown
            for (method, count) in &stats.method_counts {
                debug!("  {:?}: {} peers", method, count);
            }
            
            // Clean up old peers
            peer_discovery.cleanup_old_peers().await;
        }
    });

    // Run for demonstration purposes
    info!("DHT discovery example running. Press Ctrl+C to stop.");
    
    // Keep the main task running
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = event_handler => {
            error!("Event handler task completed unexpectedly");
        }
    }

    // Cleanup
    peer_discovery.stop().await?;
    info!("DHT discovery example completed");

    Ok(())
}