//! Example of using NAT traversal functionality in QuDAG network
//! 
//! This example demonstrates:
//! - Setting up NAT traversal configuration
//! - Detecting NAT type and public address
//! - Creating port mappings
//! - Establishing connections through NAT

use qudag_network::{
    NetworkManager, NetworkConfig, NatTraversalConfig, NatTraversalManager,
    NatType, PortMappingProtocol, StunServer, TurnServer, ConnectionManager
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();
    
    info!("NAT Traversal Example Starting");
    
    // Create NAT traversal configuration with STUN servers
    let nat_config = NatTraversalConfig {
        enable_stun: true,
        enable_turn: false, // Disable TURN for this example
        enable_upnp: true,
        enable_nat_pmp: true,
        enable_hole_punching: true,
        enable_relay: true,
        enable_ipv6: true,
        stun_servers: vec![
            StunServer::new("stun1.l.google.com:19302".parse().unwrap(), 1),
            StunServer::new("stun2.l.google.com:19302".parse().unwrap(), 2),
        ],
        turn_servers: vec![],
        max_relay_connections: 10,
        hole_punch_timeout: Duration::from_secs(30),
        detection_interval: Duration::from_secs(300),
        upgrade_interval: Duration::from_secs(60),
        port_mapping_lifetime: Duration::from_secs(3600),
    };
    
    // Create network configuration with NAT traversal enabled
    let network_config = NetworkConfig {
        max_connections: 50,
        enable_nat_traversal: true,
        nat_traversal_config: Some(nat_config.clone()),
        ..Default::default()
    };
    
    // Create and initialize network manager
    let mut network_manager = NetworkManager::with_config(network_config);
    
    match network_manager.initialize().await {
        Ok(()) => info!("Network manager initialized successfully"),
        Err(e) => {
            error!("Failed to initialize network manager: {}", e);
            return Err(e.into());
        }
    }
    
    // Give some time for NAT detection to complete
    sleep(Duration::from_secs(2)).await;
    
    // Get NAT information
    if let Some(nat_info) = network_manager.get_nat_info() {
        info!("NAT Detection Results:");
        info!("  NAT Type: {:?}", nat_info.nat_type);
        info!("  Public IP: {:?}", nat_info.public_ip);
        info!("  Public Port: {:?}", nat_info.public_port);
        info!("  Local IP: {}", nat_info.local_ip);
        info!("  Local Port: {}", nat_info.local_port);
        info!("  Hairpinning: {}", nat_info.hairpinning);
        info!("  Confidence: {:.2}", nat_info.confidence);
        
        // Provide recommendations based on NAT type
        match nat_info.nat_type {
            NatType::None => {
                info!("✅ No NAT detected - direct connections should work");
            }
            NatType::FullCone => {
                info!("✅ Full cone NAT - good connectivity expected");
            }
            NatType::RestrictedCone => {
                info!("⚠️  Restricted cone NAT - hole punching may be needed");
            }
            NatType::PortRestrictedCone => {
                info!("⚠️  Port restricted cone NAT - hole punching recommended");
            }
            NatType::Symmetric => {
                info!("❌ Symmetric NAT - relay connections may be required");
            }
            NatType::Unknown => {
                warn!("❓ Unknown NAT type - mixed connectivity expected");
            }
        }
    } else {
        warn!("NAT detection not available or failed");
    }
    
    // Try to create port mappings
    info!("\nAttempting to create port mappings...");
    
    // Create TCP port mapping
    match network_manager.create_port_mapping(
        8080, // local port
        8080, // external port  
        PortMappingProtocol::TCP,
    ).await {
        Ok(mapping) => {
            info!("✅ Successfully created TCP port mapping:");
            info!("  Local Port: {}", mapping.local_port);
            info!("  External Port: {}", mapping.external_port);
            info!("  Method: {:?}", mapping.method);
            info!("  Created At: {:?}", mapping.created_at);
        }
        Err(e) => {
            warn!("❌ Failed to create TCP port mapping: {}", e);
        }
    }
    
    // Create UDP port mapping
    match network_manager.create_port_mapping(
        8081, // local port
        8081, // external port
        PortMappingProtocol::UDP,
    ).await {
        Ok(mapping) => {
            info!("✅ Successfully created UDP port mapping:");
            info!("  Local Port: {}", mapping.local_port);
            info!("  External Port: {}", mapping.external_port);
            info!("  Method: {:?}", mapping.method);
        }
        Err(e) => {
            warn!("❌ Failed to create UDP port mapping: {}", e);
        }
    }
    
    // Get NAT traversal statistics
    if let Some(stats) = network_manager.get_nat_traversal_stats() {
        info!("\nNAT Traversal Statistics:");
        info!("  STUN Success: {}", stats.stun_success.load(std::sync::atomic::Ordering::Relaxed));
        info!("  STUN Failures: {}", stats.stun_failures.load(std::sync::atomic::Ordering::Relaxed));
        info!("  Hole Punch Success: {}", stats.hole_punch_success.load(std::sync::atomic::Ordering::Relaxed));
        info!("  Hole Punch Failures: {}", stats.hole_punch_failures.load(std::sync::atomic::Ordering::Relaxed));
        info!("  Active Relay Connections: {}", stats.relay_connections.load(std::sync::atomic::Ordering::Relaxed));
        info!("  Upgraded Connections: {}", stats.upgraded_connections.load(std::sync::atomic::Ordering::Relaxed));
        info!("  Port Mappings Created: {}", stats.port_mappings_created.load(std::sync::atomic::Ordering::Relaxed));
        info!("  Port Mappings Failed: {}", stats.port_mappings_failed.load(std::sync::atomic::Ordering::Relaxed));
    }
    
    // Demonstrate connecting to a peer (this would normally involve actual peer discovery)
    info!("\nDemonstrating peer connection with NAT traversal...");
    
    // In a real scenario, you would have discovered peer addresses
    let peer_address = "example.peer.address:8080";
    
    match network_manager.connect_peer(peer_address).await {
        Ok(peer_id) => {
            info!("✅ Successfully connected to peer: {:?}", peer_id);
            
            // Get peer metadata
            if let Some(metadata) = network_manager.get_peer_metadata(&peer_id).await {
                info!("Peer metadata:");
                info!("  Address: {}", metadata.address);
                info!("  Protocol Version: {}", metadata.protocol_version);
                info!("  Latency: {} ms", metadata.latency_ms);
                info!("  Reputation: {:.2}", metadata.reputation);
            }
        }
        Err(e) => {
            info!("Connection attempt completed (expected in demo): {}", e);
        }
    }
    
    // Show network statistics
    let network_stats = network_manager.get_network_stats().await;
    info!("\nNetwork Statistics:");
    info!("  Connected Peers: {}", network_stats.connected_peers);
    info!("  Average Reputation: {:.2}", network_stats.average_reputation);
    info!("  Blacklisted Peers: {}", network_stats.blacklisted_peers);
    info!("  Trusted Peers: {}", network_stats.trusted_peers);
    
    // Demonstrate NAT traversal manager directly
    info!("\nDemonstrating direct NAT traversal manager usage...");
    
    let connection_manager = Arc::new(ConnectionManager::new(50));
    let nat_manager = Arc::new(NatTraversalManager::new(nat_config, connection_manager));
    
    match nat_manager.initialize().await {
        Ok(()) => {
            info!("✅ NAT traversal manager initialized directly");
            
            // Get current NAT info
            if let Some(nat_info) = nat_manager.get_nat_info() {
                info!("Direct NAT info - Type: {:?}", nat_info.nat_type);
            }
            
            // Get statistics
            let stats = nat_manager.get_stats();
            info!("Direct stats - STUN success: {}", 
                  stats.stun_success.load(std::sync::atomic::Ordering::Relaxed));
        }
        Err(e) => {
            warn!("Direct NAT manager initialization failed: {}", e);
        }
    }
    
    // Cleanup
    info!("\nShutting down...");
    
    match network_manager.shutdown().await {
        Ok(()) => info!("✅ Network manager shut down successfully"),
        Err(e) => error!("❌ Error during shutdown: {}", e),
    }
    
    info!("NAT Traversal Example Completed");
    Ok(())
}

/// Helper function to analyze NAT type and provide recommendations
fn analyze_nat_type(nat_type: NatType) -> &'static str {
    match nat_type {
        NatType::None => "Direct connection possible",
        NatType::FullCone => "Good connectivity - all techniques available", 
        NatType::RestrictedCone => "Moderate connectivity - hole punching recommended",
        NatType::PortRestrictedCone => "Limited connectivity - UPnP/hole punching needed",
        NatType::Symmetric => "Poor connectivity - relay required",
        NatType::Unknown => "Unknown connectivity - try all techniques",
    }
}

/// Helper function to recommend connection strategies
fn recommend_strategies(nat_type: NatType) -> Vec<&'static str> {
    match nat_type {
        NatType::None => vec!["Direct connection"],
        NatType::FullCone => vec!["Direct connection", "UPnP", "Hole punching"],
        NatType::RestrictedCone => vec!["UPnP", "Hole punching", "STUN"],
        NatType::PortRestrictedCone => vec!["UPnP", "NAT-PMP", "Hole punching"],
        NatType::Symmetric => vec!["TURN relay", "Custom relay", "UPnP (if available)"],
        NatType::Unknown => vec!["Try all available methods"],
    }
}