//! Example: Distributed gradient sharing using P2P network
//!
//! This example demonstrates how to use the P2P communication layer
//! for distributed AI training with gradient aggregation.

use daa_compute::{P2PNetwork, SwarmConfig, GradientMessage};
use libp2p::{PeerId, Multiaddr};
use tokio;
use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Create swarm configuration
    let mut config = SwarmConfig::default();
    
    // Add bootstrap nodes (these would be well-known nodes in production)
    config.bootstrap_nodes = vec![
        (
            "12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp".parse::<PeerId>()?,
            "/ip4/104.131.131.82/tcp/4001".parse::<Multiaddr>()?,
        ),
    ];
    
    // Enable all features
    config.enable_mdns = true;
    config.enable_nat_traversal = true;
    config.enable_relay = true;
    config.compression_level = 3; // Zstd level 3 for good balance

    // Create P2P network
    let mut network = P2PNetwork::new(config).await?;
    info!("P2P network initialized");

    // Bootstrap the network
    network.bootstrap().await?;
    info!("Connected to bootstrap nodes");

    // Simulate gradient computation
    let gradient = compute_gradient();
    info!("Computed local gradient with {} parameters", gradient.len());

    // Broadcast gradient to network
    network.broadcast_gradient(gradient).await?;
    info!("Broadcasted gradient to network");

    // Run network event loop in background
    let network_handle = tokio::spawn(async move {
        if let Err(e) = network.run().await {
            eprintln!("Network error: {}", e);
        }
    });

    // Wait for aggregated gradient
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // In production, you would wait for enough peers to contribute
    // and then retrieve the aggregated gradient
    
    info!("Gradient sharing example completed");
    
    Ok(())
}

/// Simulate gradient computation for a neural network
fn compute_gradient() -> Vec<f32> {
    // Simulate gradients for a small model (e.g., 1M parameters)
    let size = 1_000_000;
    let mut gradient = Vec::with_capacity(size);
    
    // Generate synthetic gradients (normally these would come from backprop)
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for _ in 0..size {
        // Most gradients are small, some are larger (typical distribution)
        let value = if rng.gen_bool(0.9) {
            rng.gen_range(-0.01..0.01)
        } else {
            rng.gen_range(-0.1..0.1)
        };
        gradient.push(value);
    }
    
    gradient
}

/// Example: Browser node using WASM
#[cfg(target_arch = "wasm32")]
mod browser {
    use wasm_bindgen::prelude::*;
    use daa_compute::{P2PNetwork, SwarmConfig};
    
    #[wasm_bindgen]
    pub async fn start_browser_node() -> Result<(), JsValue> {
        // Configure for browser environment
        let mut config = SwarmConfig::default();
        config.listen_addresses = vec![
            "/ip4/0.0.0.0/tcp/0/ws".parse().unwrap(),
        ];
        
        // Create network with WebRTC support
        let network = P2PNetwork::new(config).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Browser nodes typically have smaller gradients
        let gradient = vec![0.1; 10000]; // 10k parameters
        
        network.broadcast_gradient(gradient).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(())
    }
}

/// Example: Cloud node with GPU
#[cfg(not(target_arch = "wasm32"))]
mod cloud {
    use super::*;
    use daa_compute::gradient::{AllReduceAlgorithm, GradientManager};
    
    pub async fn run_cloud_node() -> Result<()> {
        let config = SwarmConfig {
            listen_addresses: vec![
                "/ip4/0.0.0.0/tcp/9000".parse()?,
                "/ip4/0.0.0.0/tcp/9001/ws".parse()?,
            ],
            compression_level: 5, // Higher compression for cloud
            ..Default::default()
        };
        
        let mut network = P2PNetwork::new(config).await?;
        
        // Cloud nodes can handle larger models
        let gradient = compute_large_gradient();
        
        // Use hierarchical all-reduce for geo-distributed cloud
        // This would be configured in GradientManager
        
        network.broadcast_gradient(gradient).await?;
        
        // Run indefinitely
        network.run().await
    }
    
    fn compute_large_gradient() -> Vec<f32> {
        // Simulate 1B parameter model
        vec![0.001; 1_000_000_000]
    }
}

/// Example: Edge node with limited bandwidth
mod edge {
    use super::*;
    
    pub async fn run_edge_node() -> Result<()> {
        let config = SwarmConfig {
            compression_level: 9, // Maximum compression for edge
            enable_nat_traversal: true, // Essential for home networks
            ..Default::default()
        };
        
        let mut network = P2PNetwork::new(config).await?;
        
        // Edge nodes might train on local data
        let gradient = compute_edge_gradient();
        
        // Compress heavily before sending
        network.broadcast_gradient(gradient).await?;
        
        Ok(())
    }
    
    fn compute_edge_gradient() -> Vec<f32> {
        // Smaller model for edge devices
        vec![0.01; 100_000]
    }
}