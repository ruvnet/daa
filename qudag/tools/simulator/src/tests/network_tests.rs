use super::super::metrics::NetworkMetrics;
use super::super::network::{NetworkSimulator, SimulatorConfig, SimulatorEvent};
use anyhow::Result;
use qudag_protocol::config::Config as ProtocolConfig;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_network_simulator_creation() {
    let config = SimulatorConfig {
        node_count: 4,
        latency_ms: 100,
        drop_rate: 0.1,
        partition_prob: 0.2,
    };

    let (simulator, mut events_rx) = NetworkSimulator::new(config.clone());

    assert_eq!(simulator.config.node_count, 4);
    assert_eq!(simulator.config.latency_ms, 100);
    assert_eq!(simulator.config.drop_rate, 0.1);
    assert_eq!(simulator.config.partition_prob, 0.2);
    assert_eq!(simulator.nodes.len(), 0);
}

#[tokio::test]
async fn test_add_node() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 2,
        latency_ms: 50,
        drop_rate: 0.0,
        partition_prob: 0.0,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Add a node
    simulator.add_node(ProtocolConfig::default()).await?;

    assert_eq!(simulator.nodes.len(), 1);
    assert_eq!(simulator.nodes[0].id, "node-0");

    // Verify event was sent
    let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
    match event {
        Some(SimulatorEvent::NodeJoined(id)) => assert_eq!(id, "node-0"),
        _ => panic!("Expected NodeJoined event"),
    }

    Ok(())
}

#[tokio::test]
async fn test_add_multiple_nodes() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 3,
        latency_ms: 50,
        drop_rate: 0.0,
        partition_prob: 0.0,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Add multiple nodes
    for i in 0..3 {
        simulator.add_node(ProtocolConfig::default()).await?;
        assert_eq!(simulator.nodes.len(), i + 1);
        assert_eq!(simulator.nodes[i].id, format!("node-{}", i));

        // Verify event was sent
        let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
        match event {
            Some(SimulatorEvent::NodeJoined(id)) => assert_eq!(id, format!("node-{}", i)),
            _ => panic!("Expected NodeJoined event for node-{}", i),
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_remove_node() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 2,
        latency_ms: 50,
        drop_rate: 0.0,
        partition_prob: 0.0,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Add nodes first
    simulator.add_node(ProtocolConfig::default()).await?;
    simulator.add_node(ProtocolConfig::default()).await?;

    // Consume NodeJoined events
    timeout(Duration::from_millis(100), events_rx.recv()).await?;
    timeout(Duration::from_millis(100), events_rx.recv()).await?;

    assert_eq!(simulator.nodes.len(), 2);

    // Remove a node
    simulator.remove_node("node-1").await?;
    assert_eq!(simulator.nodes.len(), 1);
    assert_eq!(simulator.nodes[0].id, "node-0");

    // Verify event was sent
    let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
    match event {
        Some(SimulatorEvent::NodeLeft(id)) => assert_eq!(id, "node-1"),
        _ => panic!("Expected NodeLeft event"),
    }

    Ok(())
}

#[tokio::test]
async fn test_remove_nonexistent_node() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 1,
        latency_ms: 50,
        drop_rate: 0.0,
        partition_prob: 0.0,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Try to remove a non-existent node
    simulator.remove_node("nonexistent").await?;
    assert_eq!(simulator.nodes.len(), 0);

    // Should not receive any events
    let result = timeout(Duration::from_millis(50), events_rx.recv()).await;
    assert!(result.is_err()); // Timeout expected

    Ok(())
}

#[tokio::test]
async fn test_create_partition() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 4,
        latency_ms: 50,
        drop_rate: 0.0,
        partition_prob: 0.5, // 50% partition probability
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Add nodes
    for _ in 0..4 {
        simulator.add_node(ProtocolConfig::default()).await?;
    }

    // Consume all NodeJoined events
    for _ in 0..4 {
        timeout(Duration::from_millis(100), events_rx.recv()).await?;
    }

    // Create partition
    simulator.create_partition().await?;

    // Verify partition event was sent
    let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
    match event {
        Some(SimulatorEvent::Partition { nodes }) => {
            assert_eq!(nodes.len(), 2); // 50% of 4 nodes = 2
            assert_eq!(nodes[0], "node-0");
            assert_eq!(nodes[1], "node-1");
        }
        _ => panic!("Expected Partition event"),
    }

    Ok(())
}

#[tokio::test]
async fn test_heal_partition() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 4,
        latency_ms: 50,
        drop_rate: 0.0,
        partition_prob: 0.5,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Add nodes
    for _ in 0..4 {
        simulator.add_node(ProtocolConfig::default()).await?;
    }

    // Consume all NodeJoined events
    for _ in 0..4 {
        timeout(Duration::from_millis(100), events_rx.recv()).await?;
    }

    // Create and heal partition
    simulator.create_partition().await?;
    timeout(Duration::from_millis(100), events_rx.recv()).await?; // Consume partition event

    simulator.heal_partition().await?;

    // Verify heal event was sent
    let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
    match event {
        Some(SimulatorEvent::Heal) => {}
        _ => panic!("Expected Heal event"),
    }

    Ok(())
}

#[tokio::test]
async fn test_simulator_config_serialization() {
    let config = SimulatorConfig {
        node_count: 10,
        latency_ms: 200,
        drop_rate: 0.15,
        partition_prob: 0.3,
    };

    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: SimulatorConfig = serde_json::from_str(&serialized).unwrap();

    assert_eq!(config.node_count, deserialized.node_count);
    assert_eq!(config.latency_ms, deserialized.latency_ms);
    assert_eq!(config.drop_rate, deserialized.drop_rate);
    assert_eq!(config.partition_prob, deserialized.partition_prob);
}

#[tokio::test]
async fn test_edge_case_zero_nodes() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 0,
        latency_ms: 50,
        drop_rate: 0.0,
        partition_prob: 0.5,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Try to create partition with zero nodes
    simulator.create_partition().await?;

    // Should still send partition event, but with empty nodes
    let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
    match event {
        Some(SimulatorEvent::Partition { nodes }) => {
            assert_eq!(nodes.len(), 0);
        }
        _ => panic!("Expected Partition event with empty nodes"),
    }

    Ok(())
}

#[tokio::test]
async fn test_edge_case_single_node_partition() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 1,
        latency_ms: 50,
        drop_rate: 0.0,
        partition_prob: 0.5,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Add single node
    simulator.add_node(ProtocolConfig::default()).await?;
    timeout(Duration::from_millis(100), events_rx.recv()).await?; // Consume join event

    // Create partition with single node
    simulator.create_partition().await?;

    let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
    match event {
        Some(SimulatorEvent::Partition { nodes }) => {
            assert_eq!(nodes.len(), 0); // 50% of 1 = 0
        }
        _ => panic!("Expected Partition event"),
    }

    Ok(())
}

#[tokio::test]
async fn test_high_drop_rate_config() {
    let config = SimulatorConfig {
        node_count: 5,
        latency_ms: 1000,
        drop_rate: 0.99, // Very high drop rate
        partition_prob: 0.8,
    };

    let (simulator, _events_rx) = NetworkSimulator::new(config.clone());

    // Verify configuration is preserved even with extreme values
    assert_eq!(simulator.config.drop_rate, 0.99);
    assert_eq!(simulator.config.latency_ms, 1000);
    assert_eq!(simulator.config.partition_prob, 0.8);
}
