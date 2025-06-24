//! Example demonstrating traffic obfuscation and mix network functionality
//!
//! This example shows how to:
//! - Configure traffic obfuscation
//! - Send messages with size normalization
//! - Generate dummy traffic
//! - Use mix network batching
//! - Apply protocol obfuscation

use qudag_network::{
    message::{MessageEnvelope, MessageQueue},
    traffic_obfuscation::{
        ObfuscationPattern, TrafficObfuscationConfig, TrafficObfuscator,
        DEFAULT_MESSAGE_SIZE, STANDARD_MESSAGE_SIZES,
    },
    transport::{SecureTransport, Transport, TransportConfig},
    types::{MessagePriority, NetworkMessage},
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting traffic obfuscation example");

    // Example 1: Basic traffic obfuscation configuration
    basic_obfuscation_example().await?;

    // Example 2: Mix network batching
    mix_network_example().await?;

    // Example 3: Protocol obfuscation patterns
    protocol_obfuscation_example().await?;

    // Example 4: Integrated transport with obfuscation
    transport_obfuscation_example().await?;

    Ok(())
}

async fn basic_obfuscation_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Basic Traffic Obfuscation Example ===");

    // Configure traffic obfuscation
    let config = TrafficObfuscationConfig {
        enable_size_normalization: true,
        standard_message_size: DEFAULT_MESSAGE_SIZE, // 4KB
        enable_dummy_traffic: true,
        dummy_traffic_ratio: 0.2, // 20% dummy traffic
        enable_traffic_shaping: true,
        traffic_delay_range: (10, 100), // 10-100ms delays
        ..Default::default()
    };

    // Create message queue with obfuscation
    let (mut queue, mut rx) = MessageQueue::with_obfuscation(config.clone());

    // Create a small message
    let small_msg = NetworkMessage {
        id: "msg1".to_string(),
        source: vec![1, 2, 3],
        destination: vec![4, 5, 6],
        payload: vec![0u8; 100], // 100 bytes
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(300),
    };

    info!("Original message size: {} bytes", small_msg.payload.len());

    // Enqueue the message (will be normalized to 4KB)
    queue.enqueue(small_msg).await?;

    // Check obfuscation statistics
    if let Some(stats) = queue.get_obfuscation_stats().await {
        info!("Obfuscation stats:");
        info!("  Total messages: {}", stats.total_messages);
        info!("  Normalized messages: {}", stats.normalized_messages);
        info!("  Total padding bytes: {}", stats.total_padding_bytes);
        info!("  Dummy messages: {}", stats.dummy_messages);
    }

    // Process some messages to see dummy traffic
    tokio::spawn(async move {
        while let Some(_) = rx.recv().await {
            // Message notification received
        }
    });

    // Wait for dummy traffic generation
    sleep(Duration::from_secs(2)).await;

    // Check stats again
    if let Some(stats) = queue.get_obfuscation_stats().await {
        info!("\nAfter 2 seconds:");
        info!("  Dummy messages generated: {}", stats.dummy_messages);
    }

    Ok(())
}

async fn mix_network_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Mix Network Batching Example ===");

    // Configure with mix batching
    let config = TrafficObfuscationConfig {
        enable_mix_batching: true,
        mix_batch_size: 10,
        mix_batch_timeout: Duration::from_millis(500),
        ..Default::default()
    };

    let obfuscator = TrafficObfuscator::new(config);
    obfuscator.start().await;

    // Send multiple messages
    for i in 0..15 {
        let msg = NetworkMessage {
            id: format!("batch_msg_{}", i),
            source: vec![1],
            destination: vec![2],
            payload: vec![i as u8; 1000],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Messages will be batched
        obfuscator.obfuscate_message(msg).await?;
    }

    // Wait for batch processing
    sleep(Duration::from_millis(600)).await;

    // Process the batch
    let batch = obfuscator.process_batch().await?;
    info!("Batch processed with {} messages", batch.len());

    let stats = obfuscator.get_stats().await;
    info!("Mix network stats:");
    info!("  Batches processed: {}", stats.batches_processed);
    info!("  Average batch size: {:.2}", stats.avg_batch_size);

    Ok(())
}

async fn protocol_obfuscation_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Protocol Obfuscation Example ===");

    // Configure with various obfuscation patterns
    let config = TrafficObfuscationConfig {
        enable_protocol_obfuscation: true,
        obfuscation_patterns: vec![
            ObfuscationPattern::Http,
            ObfuscationPattern::Https,
            ObfuscationPattern::WebSocket,
            ObfuscationPattern::Dns,
        ],
        ..Default::default()
    };

    let obfuscator = TrafficObfuscator::new(config);
    obfuscator.start().await;

    // Create a message
    let msg = NetworkMessage {
        id: "proto_test".to_string(),
        source: vec![1],
        destination: vec![2],
        payload: b"Secret data that needs obfuscation".to_vec(),
        priority: MessagePriority::High,
        ttl: Duration::from_secs(300),
    };

    info!("Original payload: {:?}", String::from_utf8_lossy(&msg.payload));

    // Obfuscate the message
    let obfuscated = obfuscator.obfuscate_message(msg).await?;
    
    if !obfuscated.is_empty() {
        info!("Obfuscated data length: {} bytes", obfuscated.len());
        
        // Show first 100 bytes of obfuscated data
        let preview = if obfuscated.len() > 100 {
            &obfuscated[..100]
        } else {
            &obfuscated
        };
        
        // Try to detect protocol pattern
        if preview.starts_with(b"POST") || preview.starts_with(b"GET") {
            info!("Detected HTTP-like obfuscation");
        } else if preview[0] == 0x17 && preview[1] == 0x03 {
            info!("Detected HTTPS-like obfuscation");
        } else if preview[0] == 0x82 {
            info!("Detected WebSocket-like obfuscation");
        } else {
            info!("Custom obfuscation pattern applied");
        }
    }

    Ok(())
}

async fn transport_obfuscation_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Transport with Traffic Obfuscation Example ===");

    // Configure transport with obfuscation
    let mut transport_config = TransportConfig::default();
    transport_config.enable_traffic_obfuscation = true;
    transport_config.traffic_obfuscation_config = TrafficObfuscationConfig {
        enable_size_normalization: true,
        standard_message_size: STANDARD_MESSAGE_SIZES[3], // 4KB
        enable_dummy_traffic: true,
        dummy_traffic_ratio: 0.15,
        enable_traffic_shaping: true,
        traffic_delay_range: (20, 80),
        enable_mix_batching: true,
        mix_batch_size: 25,
        enable_protocol_obfuscation: true,
        ..Default::default()
    };

    // Create secure transport with obfuscation
    let mut transport = SecureTransport::with_config(transport_config.clone());
    transport.init(transport_config).await?;

    info!("Transport initialized with traffic obfuscation");
    
    // Get transport statistics
    let stats = transport.get_stats();
    info!("Transport stats:");
    info!("  Total connections: {}", stats.total_connections);
    info!("  Active connections: {}", stats.active_connections);

    // Demonstrate message size options
    info!("\nAvailable standard message sizes:");
    for (i, &size) in STANDARD_MESSAGE_SIZES.iter().enumerate() {
        info!("  [{}] {} bytes", i, size);
    }

    Ok(())
}

// Example output formatter
fn format_bytes(bytes: &[u8], max_len: usize) -> String {
    let display_len = bytes.len().min(max_len);
    let hex: Vec<String> = bytes[..display_len]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    
    if bytes.len() > max_len {
        format!("{} ... ({} bytes total)", hex.join(" "), bytes.len())
    } else {
        hex.join(" ")
    }
}