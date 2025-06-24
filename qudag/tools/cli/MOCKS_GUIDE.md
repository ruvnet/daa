# CLI Mock Implementation Guide

This module provides comprehensive mock implementations for testing CLI commands without requiring actual network connections or running node instances.

## Overview

The mock system includes:

- **MockNode**: Simulates a complete QuDAG node with state management
- **MockPeerManager**: Handles peer connection simulation and tracking
- **MockNetworkStats**: Generates realistic network statistics
- **MockRpcClient**: Simulates RPC communication with configurable behaviors
- **TestScenarioBuilder**: Creates complex test scenarios with multiple nodes

## Basic Usage

### Simple Node Testing

```rust
use qudag_cli::mocks::*;

#[tokio::test]
async fn test_basic_node_operations() {
    let node = MockNode::new("test-node".to_string());
    
    // Test node lifecycle
    assert!(node.start().await.is_ok());
    assert!(node.add_peer("192.168.1.10:8080".to_string()).await.is_ok());
    
    let status = node.get_status().await;
    assert_eq!(status.peers.len(), 1);
    
    assert!(node.stop().await.is_ok());
}
```

### RPC Client Testing

```rust
use qudag_cli::mocks::*;
use std::sync::Arc;

#[tokio::test]
async fn test_rpc_operations() {
    let node = Arc::new(MockNode::new("rpc-node".to_string()));
    let rpc = MockRpcClient::new(node);
    
    // Configure custom behavior
    rpc.set_behavior("custom_operation", MockBehavior {
        should_succeed: true,
        latency_ms: 50,
        error_message: String::new(),
        custom_response: Some(serde_json::json!({
            "result": "success",
            "data": "custom response"
        })),
    }).await;
    
    // Test the operation
    let request = RpcRequest {
        id: uuid::Uuid::new_v4(),
        method: "custom_operation".to_string(),
        params: serde_json::Value::Null,
    };
    
    let response = rpc.process_request(request).await;
    assert!(response.result.is_some());
}
```

## Advanced Features

### Test Scenarios

Create complex multi-node scenarios with different network topologies:

```rust
#[tokio::test]
async fn test_complex_scenario() {
    let scenario = TestScenarioBuilder::new()
        .add_node("gateway".to_string())
        .add_node("worker-1".to_string())
        .add_node("worker-2".to_string())
        .with_topology(NetworkTopology::Star { 
            center: "gateway".to_string() 
        })
        .with_global_behavior("heartbeat".to_string(), MockBehavior {
            should_succeed: true,
            latency_ms: 10,
            custom_response: Some(serde_json::json!({
                "status": "healthy"
            })),
            ..Default::default()
        })
        .build()
        .await;
    
    scenario.start_all_nodes().await.unwrap();
    scenario.simulate_activity(Duration::from_secs(1)).await;
    
    let stats = scenario.get_aggregate_stats().await;
    assert!(stats.messages_sent > 0);
}
```

### Network Topologies

Available topology patterns:

- **FullMesh**: All nodes connected to all other nodes
- **Ring**: Nodes connected in a circular pattern
- **Star**: Central hub with peripheral nodes
- **Custom**: Define specific connections

### Behavior Configuration

Configure mock behaviors for different scenarios:

```rust
// Successful operations with latency
let success_behavior = MockBehavior {
    should_succeed: true,
    latency_ms: 25,
    error_message: String::new(),
    custom_response: None,
};

// Failing operations
let error_behavior = MockBehavior {
    should_succeed: false,
    latency_ms: 0,
    error_message: "Network timeout".to_string(),
    custom_response: None,
};

// Custom responses
let custom_behavior = MockBehavior {
    should_succeed: true,
    latency_ms: 0,
    error_message: String::new(),
    custom_response: Some(serde_json::json!({
        "custom": "data",
        "timestamp": chrono::Utc::now().timestamp()
    })),
};
```

## Testing CLI Commands

### Command Output Verification

```rust
#[tokio::test]
async fn test_status_command_format() {
    let node = MockNode::new("status-test".to_string());
    node.start().await.unwrap();
    node.add_peer("peer1.example.com:8080".to_string()).await.unwrap();
    
    let status = node.get_status().await;
    
    // Verify output format
    println!("Node Status:");
    println!("  ID: {}", status.node_id);
    println!("  State: {}", status.state);
    println!("  Peers: {}", status.peers.len());
    
    assert_eq!(status.state, "Running");
    assert_eq!(status.peers.len(), 1);
}
```

### Error Handling Testing

```rust
#[tokio::test]
async fn test_error_scenarios() {
    let node = Arc::new(MockNode::new("error-test".to_string()));
    let rpc = MockRpcClient::new(node);
    
    // Configure error behavior
    rpc.set_behavior("failing_operation", MockBehavior {
        should_succeed: false,
        error_message: "Service unavailable".to_string(),
        ..Default::default()
    }).await;
    
    let request = RpcRequest {
        id: uuid::Uuid::new_v4(),
        method: "failing_operation".to_string(),
        params: serde_json::Value::Null,
    };
    
    let response = rpc.process_request(request).await;
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().message, "Service unavailable");
}
```

## Mock Components Reference

### MockNode

Simulates a complete QuDAG node with:
- State management (Stopped, Starting, Running, Stopping, Error)
- Peer management with connection tracking
- Network statistics simulation
- DAG statistics tracking
- Memory usage monitoring

Key methods:
- `start()` / `stop()`: Control node lifecycle
- `add_peer()` / `remove_peer()`: Manage peer connections  
- `get_status()`: Retrieve current node status
- `simulate_activity()`: Generate network activity
- `set_behavior()`: Configure method behaviors

### MockPeerManager

Handles peer connections with:
- Connection attempt tracking
- Success/failure simulation
- Latency simulation
- Error message customization

### MockNetworkStats

Provides network statistics with:
- Real-time stats updates
- Historical data tracking
- Configurable activity patterns
- Statistics reset functionality

### MockRpcClient

Simulates RPC communication with:
- Request/response handling
- Behavior configuration per method
- Request history tracking
- Latency simulation
- Error injection

## Best Practices

### Test Organization

1. **Unit Tests**: Test individual mock components
2. **Integration Tests**: Test CLI command interactions
3. **Scenario Tests**: Test complex multi-node scenarios
4. **Error Tests**: Test error handling and edge cases

### Behavior Configuration

1. **Use realistic latencies**: Based on actual network conditions
2. **Test both success and failure paths**
3. **Configure appropriate error messages**
4. **Use custom responses for complex data validation**

### Debugging

1. **Check request history**: Use `get_request_history()` to debug RPC calls
2. **Monitor connection attempts**: Track peer connection patterns
3. **Verify network statistics**: Ensure realistic activity simulation
4. **Use detailed assertions**: Check specific response fields

## Example Test Patterns

### CLI Command Testing

```rust
#[tokio::test]
async fn test_peer_list_command() {
    let node = MockNode::new("peer-test".to_string());
    node.start().await.unwrap();
    
    // Add test peers
    node.add_peer("peer1:8080".to_string()).await.unwrap();
    node.add_peer("peer2:8080".to_string()).await.unwrap();
    
    // Simulate CLI peer list command
    let status = node.get_status().await;
    assert_eq!(status.peers.len(), 2);
    
    // Verify peer information
    let peer1 = &status.peers[0];
    assert!(peer1.address.contains(":8080"));
    assert!(peer1.connected_duration > 0);
}
```

### Performance Testing

```rust
#[tokio::test]
async fn test_concurrent_operations() {
    let node = Arc::new(MockNode::new("perf-test".to_string()));
    let rpc = Arc::new(MockRpcClient::new(node));
    
    // Test concurrent RPC requests
    let mut handles = Vec::new();
    for i in 0..100 {
        let rpc_clone = rpc.clone();
        let handle = tokio::spawn(async move {
            let request = RpcRequest {
                id: uuid::Uuid::new_v4(),
                method: "get_status".to_string(),
                params: serde_json::Value::Null,
            };
            rpc_clone.process_request(request).await
        });
        handles.push(handle);
    }
    
    // Wait for all requests
    let results = futures::future::join_all(handles).await;
    
    // Verify all succeeded
    for result in results {
        let response = result.unwrap();
        assert!(response.result.is_some());
    }
}
```

This mock system provides comprehensive testing capabilities for CLI commands while maintaining realistic behavior and performance characteristics.