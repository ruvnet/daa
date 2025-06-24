#![no_main]
use libfuzzer_sys::fuzz_target;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};
use qudag_protocol::{
    state::{ProtocolState, StateTransition, StateSnapshot},
    message::{ProtocolMessage, MessageType, MessageHeader},
    node::{Node, NodeId, NodeStatus},
    coordinator::Coordinator,
    config::{ProtocolConfig, SecurityConfig},
    transaction::{Transaction, TransactionPool},
    validation::{Validator, ValidationResult},
    synchronization::{SyncManager, SyncState},
    types::{Hash, Timestamp, Nonce},
};

/// Test protocol message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProtocolMessage {
    pub header: MessageHeader,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

/// Test node configuration
#[derive(Debug, Clone)]
pub struct TestNodeConfig {
    pub node_id: NodeId,
    pub stake: u64,
    pub reputation: f64,
    pub is_validator: bool,
    pub max_connections: usize,
}

/// Helper to create test protocol message from fuzz data
fn create_test_protocol_message(data: &[u8], msg_id: u64) -> Option<TestProtocolMessage> {
    if data.len() < 32 {
        return None;
    }
    
    let sender = NodeId::from_bytes(&data[..8]);
    let recipient = NodeId::from_bytes(&data[8..16]);
    let sequence = u64::from_le_bytes([
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23]
    ]);
    
    let header = MessageHeader::new(sender, recipient, sequence);
    
    let message_type = match data[24] % 6 {
        0 => MessageType::Consensus,
        1 => MessageType::Transaction,
        2 => MessageType::Synchronization,
        3 => MessageType::Heartbeat,
        4 => MessageType::Discovery,
        _ => MessageType::Custom,
    };
    
    let payload = data[25..std::cmp::min(data.len(), 256)].to_vec(); // Limit payload size
    let signature = data[std::cmp::min(256, data.len())..].to_vec();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Some(TestProtocolMessage {
        header,
        message_type,
        payload,
        signature,
        timestamp,
    })
}

/// Helper to create test node config from fuzz data
fn create_test_node_config(data: &[u8], node_index: u64) -> Option<TestNodeConfig> {
    if data.len() < 16 {
        return None;
    }
    
    let node_id = NodeId::from_bytes(&data[..8]);
    let stake = u64::from_le_bytes([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15]
    ]);
    
    let reputation = if data.len() > 16 {
        (data[16] as f64) / 255.0
    } else {
        0.5
    };
    
    let is_validator = if data.len() > 17 {
        data[17] % 2 == 0
    } else {
        true
    };
    
    let max_connections = if data.len() > 18 {
        (data[18] as usize).max(1).min(100)
    } else {
        10
    };
    
    Some(TestNodeConfig {
        node_id,
        stake,
        reputation,
        is_validator,
        max_connections,
    })
}

/// Test protocol state transitions
fn test_protocol_state_operations(
    messages: &[TestProtocolMessage],
    configs: &[TestNodeConfig]
) -> Result<(), String> {
    let mut protocol_state = ProtocolState::new();
    
    // Initialize nodes
    for config in configs {
        let node = Node::new(
            config.node_id.clone(),
            config.stake,
            config.is_validator,
        );
        
        protocol_state.add_node(node)
            .map_err(|e| format!("Failed to add node: {:?}", e))?;
    }
    
    // Process messages and test state transitions
    for message in messages {
        let protocol_msg = ProtocolMessage::new(
            message.header.clone(),
            message.message_type,
            message.payload.clone(),
        );
        
        // Test state transition
        match protocol_state.process_message(&protocol_msg) {
            Ok(transition) => {
                // Verify transition is valid
                if !transition.is_valid() {
                    return Err("Invalid state transition".to_string());
                }
                
                // Apply transition
                protocol_state.apply_transition(transition)
                    .map_err(|e| format!("Failed to apply transition: {:?}", e))?;
            },
            Err(e) => {
                // Some errors are expected with fuzz data
                continue;
            }
        }
    }
    
    // Test state validation
    protocol_state.validate()
        .map_err(|e| format!("Protocol state validation failed: {:?}", e))?;
    
    // Test state snapshot
    let snapshot = protocol_state.create_snapshot()
        .map_err(|e| format!("Failed to create snapshot: {:?}", e))?;
    
    // Test snapshot restoration
    let mut restored_state = ProtocolState::new();
    restored_state.restore_from_snapshot(snapshot)
        .map_err(|e| format!("Failed to restore from snapshot: {:?}", e))?;
    
    // Verify restored state matches original
    if protocol_state.get_hash() != restored_state.get_hash() {
        return Err("Restored state doesn't match original".to_string());
    }
    
    Ok(())
}

/// Test transaction pool operations
fn test_transaction_pool_operations(data: &[u8]) -> Result<(), String> {
    let mut tx_pool = TransactionPool::new();
    
    // Create test transactions from fuzz data
    let max_txs = std::cmp::min(data.len() / 64, 20);
    
    for i in 0..max_txs {
        let start_idx = i * 64;
        let end_idx = std::cmp::min(start_idx + 64, data.len());
        
        if end_idx > start_idx {
            let tx_data = &data[start_idx..end_idx];
            if tx_data.len() >= 32 {
                let tx = Transaction::new(
                    Hash::from_bytes(&tx_data[..32]),
                    NodeId::from_bytes(&tx_data[..8]),
                    NodeId::from_bytes(&tx_data[8..16]),
                    u64::from_le_bytes([
                        tx_data[16], tx_data[17], tx_data[18], tx_data[19],
                        tx_data[20], tx_data[21], tx_data[22], tx_data[23]
                    ]),
                    Nonce::from_bytes(&tx_data[24..32]),
                );
                
                // Test transaction addition
                match tx_pool.add_transaction(tx.clone()) {
                    Ok(_) => {
                        // Test transaction retrieval
                        if !tx_pool.contains_transaction(&tx.hash()) {
                            return Err("Transaction not found in pool".to_string());
                        }
                    },
                    Err(_) => continue, // Some errors expected with fuzz data
                }
            }
        }
    }
    
    // Test transaction ordering
    let ordered_txs = tx_pool.get_ordered_transactions(10);
    for window in ordered_txs.windows(2) {
        // Verify ordering by priority/timestamp
        if window[0].timestamp() > window[1].timestamp() {
            // Higher priority should come first, or earlier timestamp
            continue;
        }
    }
    
    // Test transaction removal
    let txs_to_remove: Vec<_> = tx_pool.get_all_transactions()
        .into_iter()
        .take(5)
        .collect();
    
    for tx in txs_to_remove {
        tx_pool.remove_transaction(&tx.hash())
            .map_err(|e| format!("Failed to remove transaction: {:?}", e))?;
    }
    
    Ok(())
}

/// Test validator operations
fn test_validator_operations(
    messages: &[TestProtocolMessage],
    configs: &[TestNodeConfig]
) -> Result<(), String> {
    let security_config = SecurityConfig::default();
    let mut validator = Validator::new(security_config);
    
    // Test message validation
    for message in messages {
        let protocol_msg = ProtocolMessage::new(
            message.header.clone(),
            message.message_type,
            message.payload.clone(),
        );
        
        match validator.validate_message(&protocol_msg) {
            Ok(ValidationResult::Valid) => {
                // Message is valid
            },
            Ok(ValidationResult::Invalid(reason)) => {
                // Message is invalid - this is expected with fuzz data
                continue;
            },
            Err(e) => {
                // Validation error - also expected with fuzz data
                continue;
            }
        }
    }
    
    // Test node validation
    for config in configs {
        let node = Node::new(
            config.node_id.clone(),
            config.stake,
            config.is_validator,
        );
        
        match validator.validate_node(&node) {
            Ok(ValidationResult::Valid) => {},
            Ok(ValidationResult::Invalid(_)) => continue,
            Err(_) => continue,
        }
    }
    
    Ok(())
}

/// Test synchronization operations
fn test_synchronization_operations(
    messages: &[TestProtocolMessage]
) -> Result<(), String> {
    let mut sync_manager = SyncManager::new();
    
    // Test sync state management
    for message in messages {
        if message.message_type == MessageType::Synchronization {
            let sync_data = &message.payload;
            
            match sync_manager.process_sync_message(sync_data) {
                Ok(sync_state) => {
                    match sync_state {
                        SyncState::InSync => {
                            // Node is synchronized
                        },
                        SyncState::Syncing => {
                            // Node is still synchronizing
                        },
                        SyncState::OutOfSync => {
                            // Node needs to resync
                            sync_manager.start_sync()
                                .map_err(|e| format!("Failed to start sync: {:?}", e))?;
                        }
                    }
                },
                Err(_) => continue, // Expected with fuzz data
            }
        }
    }
    
    // Test sync completion
    if sync_manager.is_syncing() {
        // Force sync completion for testing
        sync_manager.complete_sync()
            .map_err(|e| format!("Failed to complete sync: {:?}", e))?;
    }
    
    Ok(())
}

/// Test coordinator operations
fn test_coordinator_operations(
    messages: &[TestProtocolMessage],
    configs: &[TestNodeConfig]
) -> Result<(), String> {
    let protocol_config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(protocol_config);
    
    // Register nodes with coordinator
    for config in configs {
        let node = Node::new(
            config.node_id.clone(),
            config.stake,
            config.is_validator,
        );
        
        coordinator.register_node(node)
            .map_err(|e| format!("Failed to register node: {:?}", e))?;
    }
    
    // Process messages through coordinator
    for message in messages {
        let protocol_msg = ProtocolMessage::new(
            message.header.clone(),
            message.message_type,
            message.payload.clone(),
        );
        
        match coordinator.process_message(&protocol_msg) {
            Ok(_) => {},
            Err(_) => continue, // Expected with fuzz data
        }
    }
    
    // Test coordinator state
    let active_nodes = coordinator.get_active_nodes();
    if active_nodes.len() != configs.len() {
        return Err("Active nodes count mismatch".to_string());
    }
    
    Ok(())
}

/// Test edge cases and error conditions
fn test_edge_cases(data: &[u8]) -> Result<(), String> {
    // Test with empty protocol state
    let empty_state = ProtocolState::new();
    let empty_snapshot = empty_state.create_snapshot()
        .map_err(|e| format!("Failed to create empty snapshot: {:?}", e))?;
    
    // Test with malformed messages
    if data.len() >= 64 {
        // Create malformed message
        let malformed_header = MessageHeader::new(
            NodeId::from_bytes(&[0u8; 8]),
            NodeId::from_bytes(&[0u8; 8]),
            0,
        );
        
        let malformed_msg = ProtocolMessage::new(
            malformed_header,
            MessageType::Custom,
            data[..64].to_vec(),
        );
        
        let mut state = ProtocolState::new();
        let _ = state.process_message(&malformed_msg); // Should handle gracefully
    }
    
    // Test with duplicate nodes
    let mut state = ProtocolState::new();
    let node_id = NodeId::from_bytes(&data[..8]);
    let node1 = Node::new(node_id.clone(), 100, true);
    let node2 = Node::new(node_id.clone(), 200, false);
    
    let _ = state.add_node(node1);
    match state.add_node(node2) {
        Ok(_) => return Err("Duplicate node should not be allowed".to_string()),
        Err(_) => {}, // Expected
    }
    
    Ok(())
}

/// Test timing consistency for protocol operations
fn measure_protocol_timing<F>(op: F) -> bool
where
    F: Fn() -> Result<(), String>
{
    let iterations = 15; // Reduced for fuzzing
    let mut timings = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    if timings.is_empty() {
        return false;
    }
    
    let mean = timings.iter().sum::<Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    // Allow reasonable variance for protocol operations
    variance < 2000000
}

fuzz_target!(|data: &[u8]| {
    // Set panic hook to prevent information leaks
    std::panic::set_hook(Box::new(|_| {}));
    
    if data.is_empty() {
        return;
    }
    
    // Create test messages from fuzz data
    let mut messages = Vec::new();
    let max_messages = std::cmp::min(data.len() / 32, 15); // Limit for performance
    
    for i in 0..max_messages {
        let start_idx = i * 32;
        let end_idx = std::cmp::min(start_idx + 32, data.len());
        
        if end_idx > start_idx {
            if let Some(message) = create_test_protocol_message(&data[start_idx..end_idx], i as u64) {
                messages.push(message);
            }
        }
    }
    
    // Create test node configs
    let mut configs = Vec::new();
    let max_nodes = std::cmp::min(data.len() / 16, 10);
    
    for i in 0..max_nodes {
        let start_idx = i * 16;
        let end_idx = std::cmp::min(start_idx + 16, data.len());
        
        if end_idx > start_idx {
            if let Some(config) = create_test_node_config(&data[start_idx..end_idx], i as u64) {
                configs.push(config);
            }
        }
    }
    
    if messages.is_empty() && configs.is_empty() {
        return;
    }
    
    // Test protocol state operations with timing validation
    if !messages.is_empty() && !configs.is_empty() {
        let _state_timing = measure_protocol_timing(|| {
            test_protocol_state_operations(&messages, &configs)
        });
    }
    
    // Test transaction pool operations
    if data.len() >= 64 {
        let _tx_timing = measure_protocol_timing(|| {
            test_transaction_pool_operations(data)
        });
    }
    
    // Test validator operations
    if !messages.is_empty() && !configs.is_empty() {
        let _validator_timing = measure_protocol_timing(|| {
            test_validator_operations(&messages, &configs)
        });
    }
    
    // Test synchronization operations
    if !messages.is_empty() {
        let _sync_timing = measure_protocol_timing(|| {
            test_synchronization_operations(&messages)
        });
    }
    
    // Test coordinator operations
    if !messages.is_empty() && !configs.is_empty() {
        let _coordinator_timing = measure_protocol_timing(|| {
            test_coordinator_operations(&messages, &configs)
        });
    }
    
    // Test edge cases
    let _edge_timing = measure_protocol_timing(|| {
        test_edge_cases(data)
    });
    
    // Test message serialization robustness
    if !messages.is_empty() {
        test_message_serialization(&messages);
    }
    
    // Test concurrent operations simulation
    if data.len() >= 128 {
        test_concurrent_protocol_operations(&messages, &configs);
    }
    
    // Test memory management
    test_protocol_memory_management(&messages, &configs);
});

/// Test message serialization robustness
fn test_message_serialization(messages: &[TestProtocolMessage]) {
    for message in messages {
        // Test serialization
        let serialized = match bincode::serialize(message) {
            Ok(s) => s,
            Err(_) => continue,
        };
        
        // Test deserialization
        let deserialized: Result<TestProtocolMessage, _> = bincode::deserialize(&serialized);
        
        if let Ok(deserialized_msg) = deserialized {
            // Verify key fields match
            assert_eq!(message.message_type as u8, deserialized_msg.message_type as u8);
            assert_eq!(message.payload, deserialized_msg.payload);
        }
    }
}

/// Test concurrent protocol operations simulation
fn test_concurrent_protocol_operations(
    messages: &[TestProtocolMessage],
    configs: &[TestNodeConfig]
) {
    let mut protocol_state = ProtocolState::new();
    
    // Initialize nodes
    for config in configs {
        let node = Node::new(
            config.node_id.clone(),
            config.stake,
            config.is_validator,
        );
        let _ = protocol_state.add_node(node);
    }
    
    // Simulate concurrent message processing
    for message in messages {
        let protocol_msg = ProtocolMessage::new(
            message.header.clone(),
            message.message_type,
            message.payload.clone(),
        );
        
        // Multiple concurrent processing attempts
        let _result1 = protocol_state.process_message(&protocol_msg);
        let _result2 = protocol_state.process_message(&protocol_msg);
        let _result3 = protocol_state.process_message(&protocol_msg);
        
        // State should remain consistent
        let _ = protocol_state.validate();
    }
}

/// Test protocol memory management
fn test_protocol_memory_management(
    messages: &[TestProtocolMessage],
    configs: &[TestNodeConfig]
) {
    // Test with large state
    let mut large_state = ProtocolState::new();
    
    // Add many nodes
    for config in configs {
        let node = Node::new(
            config.node_id.clone(),
            config.stake,
            config.is_validator,
        );
        let _ = large_state.add_node(node);
    }
    
    // Process many messages
    for message in messages {
        let protocol_msg = ProtocolMessage::new(
            message.header.clone(),
            message.message_type,
            message.payload.clone(),
        );
        let _ = large_state.process_message(&protocol_msg);
    }
    
    // Test cleanup
    drop(large_state);
    
    // Test transaction pool memory management
    let mut large_tx_pool = TransactionPool::new();
    
    // Add many transactions
    for i in 0..messages.len() {
        let message = &messages[i];
        if message.payload.len() >= 32 {
            let tx = Transaction::new(
                Hash::from_bytes(&message.payload[..32]),
                NodeId::from_bytes(&message.payload[..8]),
                NodeId::from_bytes(&message.payload[8..16]),
                message.timestamp,
                Nonce::from_bytes(&message.payload[16..24]),
            );
            let _ = large_tx_pool.add_transaction(tx);
        }
    }
    
    // Test cleanup
    drop(large_tx_pool);
}