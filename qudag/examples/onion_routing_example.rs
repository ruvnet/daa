//! Example demonstrating ML-KEM based onion routing
//!
//! This example shows how to:
//! - Create an onion router with ML-KEM encryption
//! - Build anonymous circuits with 3+ hops
//! - Apply traffic analysis resistance
//! - Manage circuit lifecycles

use qudag_network::onion::{MLKEMOnionRouter, CircuitManager, DirectoryClient};
use qudag_network::router::Router;
use qudag_network::types::{NetworkMessage, MessagePriority, PeerId, RoutingStrategy};
use std::time::Duration;
use tokio::time::{sleep, interval};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting ML-KEM Onion Routing Example");

    // Create onion router
    let onion_router = MLKEMOnionRouter::new().await?;
    info!("Created ML-KEM onion router");

    // Create circuit manager
    let mut circuit_manager = CircuitManager::new();
    let directory_client = DirectoryClient::new();

    // Build a 3-hop circuit
    info!("Building anonymous circuit with 3 hops...");
    let circuit_id = circuit_manager.build_circuit(3, &directory_client).await?;
    circuit_manager.activate_circuit(circuit_id)?;
    info!("Circuit {} activated", circuit_id);

    // Create router for message routing
    let router = Router::new().await?;

    // Example message to send anonymously
    let message = NetworkMessage {
        id: "anon-msg-001".to_string(),
        source: vec![0u8; 32], // Anonymous source
        destination: vec![255u8; 32], // Target destination
        payload: b"Secret message through onion routing".to_vec(),
        priority: MessagePriority::High,
        ttl: Duration::from_secs(60),
    };

    // Route message anonymously
    info!("Routing message anonymously...");
    let route = router.route(
        &message,
        RoutingStrategy::Anonymous { hops: 3 }
    ).await?;

    info!("Message routed through {} hops", route.len());

    // Simulate periodic circuit rotation
    let mut rotation_interval = interval(Duration::from_secs(300)); // 5 minutes
    
    // Circuit monitoring task
    let monitor_task = tokio::spawn(async move {
        loop {
            rotation_interval.tick().await;
            
            if circuit_manager.needs_rotation() {
                info!("Circuit rotation needed");
                
                // Build new circuit
                match circuit_manager.build_circuit(3, &directory_client).await {
                    Ok(new_circuit_id) => {
                        if let Err(e) = circuit_manager.activate_circuit(new_circuit_id) {
                            error!("Failed to activate new circuit: {}", e);
                        } else {
                            info!("Rotated to new circuit {}", new_circuit_id);
                        }
                    }
                    Err(e) => error!("Failed to build new circuit: {}", e),
                }
                
                // Clean up old circuits
                circuit_manager.cleanup_inactive_circuits();
            }
            
            // Get circuit statistics
            let stats = circuit_manager.get_stats();
            info!(
                "Circuit stats - Total: {}, Active: {}, Bandwidth: {} bytes, Quality: {:.2}",
                stats.total_circuits,
                stats.active_circuits,
                stats.total_bandwidth,
                stats.average_quality
            );
        }
    });

    // Simulate sending messages through circuits
    for i in 0..10 {
        let test_message = NetworkMessage {
            id: format!("test-msg-{}", i),
            source: vec![0u8; 32],
            destination: vec![255u8; 32],
            payload: format!("Test message {}", i).into_bytes(),
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(30),
        };

        match router.route(&test_message, RoutingStrategy::Anonymous { hops: 3 }).await {
            Ok(route) => {
                info!("Message {} routed through {} hops", i, route.len());
                
                // Update circuit metrics
                if let Some(circuit) = circuit_manager.get_active_circuit() {
                    circuit_manager.update_circuit_metrics(
                        circuit.id,
                        test_message.payload.len() as u64,
                        true
                    );
                }
            }
            Err(e) => {
                warn!("Failed to route message {}: {}", i, e);
                
                // Update circuit metrics for failure
                if let Some(circuit) = circuit_manager.get_active_circuit() {
                    circuit_manager.update_circuit_metrics(
                        circuit.id,
                        0,
                        false
                    );
                }
            }
        }

        // Random delay between messages
        sleep(Duration::from_millis(rand::random::<u64>() % 1000 + 500)).await;
    }

    // Demonstrate traffic analysis resistance
    info!("Demonstrating traffic analysis resistance...");
    
    // Create a mix node for batch processing
    use qudag_network::onion::{MixNode, MixMessage, MixMessageType};
    
    let mut mix_node = MixNode::new(vec![1, 2, 3, 4]);
    
    // Add real messages
    for i in 0..5 {
        let msg = MixMessage {
            content: format!("Real message {}", i).into_bytes(),
            priority: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: MixMessageType::Real,
            normalized_size: 0,
        };
        
        mix_node.add_message(msg).await?;
    }
    
    // Flush batch (will add dummy messages automatically)
    let batch = mix_node.flush_batch().await?;
    info!("Flushed batch with {} messages (including dummies)", batch.len());
    
    // Get mix node statistics
    let mix_stats = mix_node.get_stats();
    info!(
        "Mix node stats - Buffer: {}, Dummy ratio: {:.2}, Target rate: {} msg/s",
        mix_stats.buffer_size,
        mix_stats.dummy_ratio,
        mix_stats.target_rate
    );

    // Demonstrate metadata protection
    use qudag_network::onion::MetadataProtector;
    
    let protector = MetadataProtector::new();
    let original_metadata = b"sensitive routing information";
    let protected = protector.protect_metadata(original_metadata)?;
    
    info!(
        "Protected metadata - Size: {} -> {}, Headers: {}",
        original_metadata.len(),
        protected.normalized_size,
        protected.random_headers.len()
    );

    // Clean up
    monitor_task.abort();
    info!("Example completed successfully");

    Ok(())
}