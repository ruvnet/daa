//! Comprehensive tests for traffic obfuscation functionality

use qudag_network::{
    message::{MessageQueue, NetworkMessage},
    onion::{MixMessage, MixMessageType, MixNode},
    traffic_obfuscation::{
        ObfuscationPattern, TrafficObfuscationConfig, TrafficObfuscator, DEFAULT_MESSAGE_SIZE,
        STANDARD_MESSAGE_SIZES,
    },
    types::MessagePriority,
};
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn test_message_size_normalization() {
    let config = TrafficObfuscationConfig {
        enable_size_normalization: true,
        standard_message_size: DEFAULT_MESSAGE_SIZE,
        ..Default::default()
    };

    let obfuscator = TrafficObfuscator::new(config);

    // Test small message padding
    let small_msg = NetworkMessage {
        id: "test1".to_string(),
        source: vec![1, 2, 3],
        destination: vec![4, 5, 6],
        payload: vec![0u8; 100], // 100 bytes
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = obfuscator.obfuscate_message(small_msg.clone()).await;
    assert!(result.is_ok());

    // Check statistics
    let stats = obfuscator.get_stats().await;
    assert_eq!(stats.normalized_messages, 1);
    assert!(stats.total_padding_bytes > 0);
}

#[tokio::test]
async fn test_dummy_traffic_generation() {
    let config = TrafficObfuscationConfig {
        enable_dummy_traffic: true,
        dummy_traffic_ratio: 0.5, // 50% dummy traffic
        enable_mix_batching: true,
        mix_batch_size: 10,
        ..Default::default()
    };

    let obfuscator = TrafficObfuscator::new(config);
    obfuscator.start().await;

    // Wait for dummy traffic generation
    sleep(Duration::from_millis(500)).await;

    let stats = obfuscator.get_stats().await;
    assert!(stats.dummy_messages > 0);
}

#[tokio::test]
async fn test_traffic_shaping_delays() {
    let config = TrafficObfuscationConfig {
        enable_traffic_shaping: true,
        traffic_delay_range: (50, 100), // 50-100ms delays
        ..Default::default()
    };

    let obfuscator = TrafficObfuscator::new(config);

    let msg = NetworkMessage {
        id: "delay_test".to_string(),
        source: vec![1],
        destination: vec![2],
        payload: vec![0u8; 1000],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let start = tokio::time::Instant::now();
    let _ = obfuscator.obfuscate_message(msg).await.unwrap();
    let elapsed = start.elapsed();

    // Should have at least minimum delay
    assert!(elapsed >= Duration::from_millis(50));
    // Should not exceed maximum delay by too much
    assert!(elapsed < Duration::from_millis(150));
}

#[tokio::test]
async fn test_mix_network_batching() {
    let config = TrafficObfuscationConfig {
        enable_mix_batching: true,
        mix_batch_size: 5,
        mix_batch_timeout: Duration::from_millis(200),
        ..Default::default()
    };

    let obfuscator = TrafficObfuscator::new(config);
    obfuscator.start().await;

    // Send messages to fill a batch
    for i in 0..5 {
        let msg = NetworkMessage {
            id: format!("batch_{}", i),
            source: vec![1],
            destination: vec![2],
            payload: vec![i as u8; 100],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let result = obfuscator.obfuscate_message(msg).await.unwrap();
        // Should return empty (batching)
        assert!(result.is_empty());
    }

    // Process the batch
    let batch = obfuscator.process_batch().await.unwrap();
    assert!(!batch.is_empty());

    let stats = obfuscator.get_stats().await;
    assert_eq!(stats.batches_processed, 1);
}

#[tokio::test]
async fn test_protocol_obfuscation_patterns() {
    let patterns = vec![
        ObfuscationPattern::Http,
        ObfuscationPattern::Https,
        ObfuscationPattern::WebSocket,
        ObfuscationPattern::Dns,
        ObfuscationPattern::Custom(vec![0xDE, 0xAD, 0xBE, 0xEF]),
    ];

    for pattern in patterns {
        let config = TrafficObfuscationConfig {
            enable_protocol_obfuscation: true,
            obfuscation_patterns: vec![pattern.clone()],
            enable_mix_batching: false, // Disable batching for this test
            ..Default::default()
        };

        let obfuscator = TrafficObfuscator::new(config);

        let msg = NetworkMessage {
            id: "proto_test".to_string(),
            source: vec![1],
            destination: vec![2],
            payload: b"test data".to_vec(),
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let obfuscated = obfuscator.obfuscate_message(msg).await.unwrap();
        assert!(!obfuscated.is_empty());

        // Verify pattern-specific characteristics
        match pattern {
            ObfuscationPattern::Http => {
                let data_str = String::from_utf8_lossy(&obfuscated);
                assert!(data_str.contains("HTTP/1.1"));
                assert!(data_str.contains("POST"));
            }
            ObfuscationPattern::Https => {
                assert_eq!(obfuscated[0], 0x17); // TLS application data
                assert_eq!(obfuscated[1], 0x03); // TLS 1.2
                assert_eq!(obfuscated[2], 0x03);
            }
            ObfuscationPattern::WebSocket => {
                assert_eq!(obfuscated[0], 0x82); // Binary frame
            }
            ObfuscationPattern::Dns => {
                // DNS has specific structure
                assert!(obfuscated.len() > 12); // At least header size
            }
            ObfuscationPattern::Custom(ref header) => {
                assert!(obfuscated.starts_with(header));
            }
        }
    }
}

#[tokio::test]
async fn test_message_queue_with_obfuscation() {
    let config = TrafficObfuscationConfig {
        enable_size_normalization: true,
        standard_message_size: 2048,
        enable_dummy_traffic: false, // Disable for predictable test
        ..Default::default()
    };

    let (queue, mut rx) = MessageQueue::with_obfuscation(config);

    // Spawn receiver
    let receiver_handle = tokio::spawn(async move {
        let mut count = 0;
        while let Some(_) = rx.recv().await {
            count += 1;
            if count >= 3 {
                break;
            }
        }
        count
    });

    // Send messages
    for i in 0..3 {
        let msg = NetworkMessage {
            id: format!("queue_msg_{}", i),
            source: vec![1],
            destination: vec![2],
            payload: vec![i as u8; 500], // Small payload
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        queue.enqueue(msg).await.unwrap();
    }

    // Wait for processing
    let count = timeout(Duration::from_secs(5), receiver_handle)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(count, 3);

    // Check obfuscation stats
    let stats = queue.get_obfuscation_stats().await.unwrap();
    assert_eq!(stats.total_messages, 3);
    assert!(stats.total_padding_bytes > 0);
}

#[tokio::test]
async fn test_burst_prevention() {
    let config = TrafficObfuscationConfig {
        enable_burst_prevention: true,
        max_burst_size: 5,
        burst_prevention_delay: 100,
        enable_traffic_shaping: true,
        traffic_delay_range: (10, 20),
        ..Default::default()
    };

    let obfuscator = TrafficObfuscator::new(config);

    let start = tokio::time::Instant::now();

    // Send burst of messages
    for i in 0..10 {
        let msg = NetworkMessage {
            id: format!("burst_{}", i),
            source: vec![1],
            destination: vec![2],
            payload: vec![0u8; 100],
            priority: MessagePriority::High,
            ttl: Duration::from_secs(60),
        };

        let _ = obfuscator.obfuscate_message(msg).await.unwrap();
    }

    let elapsed = start.elapsed();

    // Should have applied burst prevention delays
    assert!(elapsed > Duration::from_millis(100));
}

#[tokio::test]
async fn test_size_normalization_standard_sizes() {
    for &target_size in &STANDARD_MESSAGE_SIZES {
        let config = TrafficObfuscationConfig {
            enable_size_normalization: true,
            standard_message_size: target_size,
            ..Default::default()
        };

        let obfuscator = TrafficObfuscator::new(config);

        // Test message smaller than target
        let small_msg = NetworkMessage {
            id: format!("size_test_{}", target_size),
            source: vec![1],
            destination: vec![2],
            payload: vec![0u8; target_size / 2],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let _ = obfuscator.obfuscate_message(small_msg).await.unwrap();

        let stats = obfuscator.get_stats().await;
        assert!(stats.normalized_messages > 0);
        assert_eq!(stats.total_padding_bytes as usize, target_size / 2);
    }
}

#[tokio::test]
async fn test_mix_node_integration() {
    let mut mix_node = MixNode::new(vec![1, 2, 3, 4]);

    // Add messages
    for i in 0..5 {
        let msg = MixMessage {
            content: vec![i; 100],
            priority: 1,
            timestamp: 0,
            message_type: MixMessageType::Real,
            normalized_size: 0,
        };

        mix_node.add_message(msg).await.unwrap();
    }

    // Flush batch
    let batch = mix_node.flush_batch().await.unwrap();
    assert!(!batch.is_empty());

    // Check for dummy messages
    let dummy_count = batch
        .iter()
        .filter(|msg| matches!(msg.message_type, MixMessageType::Dummy))
        .count();
    assert!(dummy_count > 0);
}

#[tokio::test]
async fn test_traffic_analysis_resistance() {
    let config = TrafficObfuscationConfig {
        enable_size_normalization: true,
        enable_dummy_traffic: true,
        enable_traffic_shaping: true,
        enable_mix_batching: true,
        enable_protocol_obfuscation: true,
        ..Default::default()
    };

    let obfuscator = TrafficObfuscator::new(config);
    obfuscator.start().await;

    // Send variety of messages
    let priorities = [
        MessagePriority::High,
        MessagePriority::Normal,
        MessagePriority::Low,
    ];
    let sizes = [100, 500, 1000, 2000, 5000];

    for (i, (&priority, &size)) in priorities.iter().zip(sizes.iter()).enumerate() {
        let msg = NetworkMessage {
            id: format!("resist_{}", i),
            source: vec![i as u8],
            destination: vec![(i + 1) as u8],
            payload: vec![0u8; size],
            priority,
            ttl: Duration::from_secs(300),
        };

        let _ = obfuscator.obfuscate_message(msg).await.unwrap();
    }

    // Wait for processing
    sleep(Duration::from_millis(200)).await;

    let stats = obfuscator.get_stats().await;
    assert!(stats.total_messages >= 5);
    assert!(stats.normalized_messages > 0);
    assert!(stats.dummy_messages > 0);
    assert!(stats.protocol_obfuscations > 0);
}
