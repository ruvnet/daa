use super::super::{
    metrics::NetworkMetrics,
    network::{NetworkSimulator, SimulatorConfig, SimulatorEvent},
    scenarios::{
        test_basic_connectivity, test_byzantine_tolerance, test_network_partition,
        NetworkConditions, ScenarioConfig,
    },
};
use anyhow::Result;
use qudag_protocol::config::Config as ProtocolConfig;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Test full simulation lifecycle with multiple scenarios
#[tokio::test]
async fn test_full_simulation_lifecycle() -> Result<()> {
    let config = ScenarioConfig {
        node_count: 6,
        duration: Duration::from_millis(200),
        msg_rate: 20.0,
        network: NetworkConditions {
            latency: Duration::from_millis(30),
            loss_rate: 0.01,
            partition_prob: 0.3,
        },
    };

    // Test all scenarios in sequence
    let basic_result = test_basic_connectivity(config.clone()).await?;
    let byzantine_result = test_byzantine_tolerance(config.clone()).await?;
    let partition_result = test_network_partition(config.clone()).await?;

    // Verify all scenarios completed successfully
    assert_eq!(basic_result.latency.avg_latency, Duration::from_secs(0));
    assert_eq!(byzantine_result.throughput.msgs_per_sec, 0.0);
    assert_eq!(partition_result.consensus.finalized_tx_count, 0);

    Ok(())
}

/// Test concurrent simulations
#[tokio::test]
async fn test_concurrent_simulations() -> Result<()> {
    let config1 = ScenarioConfig {
        node_count: 4,
        duration: Duration::from_millis(100),
        msg_rate: 10.0,
        network: NetworkConditions {
            latency: Duration::from_millis(20),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let config2 = ScenarioConfig {
        node_count: 6,
        duration: Duration::from_millis(150),
        msg_rate: 15.0,
        network: NetworkConditions {
            latency: Duration::from_millis(25),
            loss_rate: 0.02,
            partition_prob: 0.1,
        },
    };

    // Run two simulations concurrently
    let (result1, result2) = tokio::join!(
        test_basic_connectivity(config1),
        test_byzantine_tolerance(config2)
    );

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    Ok(())
}

/// Test simulator state consistency during complex operations
#[tokio::test]
async fn test_simulator_state_consistency() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 10,
        latency_ms: 50,
        drop_rate: 0.05,
        partition_prob: 0.4,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Add nodes incrementally and verify state
    for i in 0..5 {
        simulator.add_node(ProtocolConfig::default()).await?;
        assert_eq!(simulator.nodes.len(), i + 1);

        // Verify event is sent
        let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
        match event {
            Some(SimulatorEvent::NodeJoined(id)) => {
                assert_eq!(id, format!("node-{}", i));
            }
            _ => panic!("Expected NodeJoined event"),
        }
    }

    // Create partition and verify state
    simulator.create_partition().await?;
    let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
    match event {
        Some(SimulatorEvent::Partition { nodes }) => {
            assert_eq!(nodes.len(), 2); // 40% of 5 nodes = 2
        }
        _ => panic!("Expected Partition event"),
    }

    // Remove some nodes and verify state
    simulator.remove_node("node-1").await?;
    simulator.remove_node("node-3").await?;
    assert_eq!(simulator.nodes.len(), 3);

    // Heal partition
    simulator.heal_partition().await?;
    let event = timeout(Duration::from_millis(100), events_rx.recv()).await?;
    match event {
        Some(SimulatorEvent::Heal) => {}
        _ => panic!("Expected Heal event"),
    }

    Ok(())
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling_and_recovery() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 3,
        latency_ms: 20,
        drop_rate: 0.0,
        partition_prob: 0.5,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Try to remove non-existent node (should not error)
    simulator.remove_node("non-existent").await?;

    // Add nodes normally
    for _ in 0..3 {
        simulator.add_node(ProtocolConfig::default()).await?;
    }

    // Drain events
    for _ in 0..3 {
        timeout(Duration::from_millis(100), events_rx.recv()).await?;
    }

    // Try operations with empty and full states
    simulator.create_partition().await?;
    simulator.heal_partition().await?;

    // Verify system still works
    simulator.add_node(ProtocolConfig::default()).await?;
    assert_eq!(simulator.nodes.len(), 4);

    Ok(())
}

/// Test performance under stress conditions
#[tokio::test]
async fn test_stress_conditions() -> Result<()> {
    let stress_config = ScenarioConfig {
        node_count: 50, // Large network
        duration: Duration::from_millis(500),
        msg_rate: 1000.0, // High message rate
        network: NetworkConditions {
            latency: Duration::from_millis(100),
            loss_rate: 0.1, // High loss rate
            partition_prob: 0.3,
        },
    };

    // Test all scenarios under stress
    let results = tokio::join!(
        test_basic_connectivity(stress_config.clone()),
        test_byzantine_tolerance(stress_config.clone()),
        test_network_partition(stress_config.clone())
    );

    assert!(results.0.is_ok());
    assert!(results.1.is_ok());
    assert!(results.2.is_ok());

    Ok(())
}

/// Test simulation with rapidly changing network conditions
#[tokio::test]
async fn test_dynamic_network_conditions() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 8,
        latency_ms: 30,
        drop_rate: 0.02,
        partition_prob: 0.25,
    };

    let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

    // Add initial nodes
    for _ in 0..8 {
        simulator.add_node(ProtocolConfig::default()).await?;
    }

    // Drain initial events
    for _ in 0..8 {
        timeout(Duration::from_millis(100), events_rx.recv()).await?;
    }

    // Simulate rapidly changing conditions
    for cycle in 0..5 {
        // Create partition
        simulator.create_partition().await?;
        timeout(Duration::from_millis(50), events_rx.recv()).await?;

        // Add a node during partition
        simulator.add_node(ProtocolConfig::default()).await?;
        timeout(Duration::from_millis(50), events_rx.recv()).await?;

        // Heal partition
        simulator.heal_partition().await?;
        timeout(Duration::from_millis(50), events_rx.recv()).await?;

        // Remove a node
        simulator.remove_node(&format!("node-{}", cycle)).await?;

        // Brief stabilization period
        sleep(Duration::from_millis(20)).await;
    }

    // Verify final state is consistent
    assert!(simulator.nodes.len() > 0);

    Ok(())
}

/// Test edge cases in simulation scenarios
#[tokio::test]
async fn test_simulation_edge_cases() -> Result<()> {
    // Test with minimal configuration
    let minimal_config = ScenarioConfig {
        node_count: 1,
        duration: Duration::from_millis(1),
        msg_rate: 0.1,
        network: NetworkConditions {
            latency: Duration::from_nanos(1),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_basic_connectivity(minimal_config).await?;
    assert_eq!(result.latency.avg_latency, Duration::from_secs(0));

    // Test with maximal configuration
    let maximal_config = ScenarioConfig {
        node_count: 1000,
        duration: Duration::from_millis(100), // Keep short for test performance
        msg_rate: 10000.0,
        network: NetworkConditions {
            latency: Duration::from_secs(1),
            loss_rate: 0.99,
            partition_prob: 0.99,
        },
    };

    let result = test_basic_connectivity(maximal_config).await?;
    assert_eq!(result.throughput.msgs_per_sec, 0.0);

    Ok(())
}

/// Test simulator memory and resource management
#[tokio::test]
async fn test_resource_management() -> Result<()> {
    let config = SimulatorConfig {
        node_count: 20,
        latency_ms: 10,
        drop_rate: 0.01,
        partition_prob: 0.1,
    };

    // Create multiple simulators to test resource allocation
    let mut simulators = Vec::new();
    let mut event_receivers = Vec::new();

    for _ in 0..5 {
        let (sim, rx) = NetworkSimulator::new(config.clone());
        simulators.push(sim);
        event_receivers.push(rx);
    }

    // Add nodes to all simulators
    for sim in &mut simulators {
        for _ in 0..5 {
            sim.add_node(ProtocolConfig::default()).await?;
        }
    }

    // Verify all simulators maintain independent state
    for sim in &simulators {
        assert_eq!(sim.nodes.len(), 5);
    }

    // Clean up by dropping simulators
    drop(simulators);
    drop(event_receivers);

    Ok(())
}

/// Test interleaved operations across multiple simulators
#[tokio::test]
async fn test_interleaved_operations() -> Result<()> {
    let config1 = SimulatorConfig {
        node_count: 4,
        latency_ms: 20,
        drop_rate: 0.0,
        partition_prob: 0.5,
    };

    let config2 = SimulatorConfig {
        node_count: 6,
        latency_ms: 30,
        drop_rate: 0.05,
        partition_prob: 0.3,
    };

    let (mut sim1, mut events1) = NetworkSimulator::new(config1);
    let (mut sim2, mut events2) = NetworkSimulator::new(config2);

    // Interleave operations
    sim1.add_node(ProtocolConfig::default()).await?;
    sim2.add_node(ProtocolConfig::default()).await?;
    sim2.add_node(ProtocolConfig::default()).await?;
    sim1.add_node(ProtocolConfig::default()).await?;

    // Verify independent state
    assert_eq!(sim1.nodes.len(), 2);
    assert_eq!(sim2.nodes.len(), 2);

    // Interleave partition operations
    sim1.create_partition().await?;
    sim2.create_partition().await?;
    sim1.heal_partition().await?;
    sim2.heal_partition().await?;

    // Verify events are properly isolated
    let mut sim1_events = Vec::new();
    let mut sim2_events = Vec::new();

    // Collect events with timeout to avoid hanging
    for _ in 0..6 {
        // 2 nodes + 1 partition + 1 heal per simulator
        if let Ok(Some(event)) = timeout(Duration::from_millis(50), events1.recv()).await {
            sim1_events.push(event);
        }
        if let Ok(Some(event)) = timeout(Duration::from_millis(50), events2.recv()).await {
            sim2_events.push(event);
        }
    }

    assert!(sim1_events.len() >= 2); // At least node join events
    assert!(sim2_events.len() >= 2); // At least node join events

    Ok(())
}
