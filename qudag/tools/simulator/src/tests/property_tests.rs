use super::super::{
    metrics::NetworkMetrics,
    network::{NetworkSimulator, SimulatorConfig},
    scenarios::{NetworkConditions, ScenarioConfig},
};
use proptest::prelude::*;
use qudag_protocol::config::Config as ProtocolConfig;
use std::time::Duration;
use tokio_test;

// Property-based test strategies
prop_compose! {
    fn arb_simulator_config()(
        node_count in 0usize..100,
        latency_ms in 1u64..5000,
        drop_rate in 0.0f64..1.0,
        partition_prob in 0.0f64..1.0,
    ) -> SimulatorConfig {
        SimulatorConfig {
            node_count,
            latency_ms,
            drop_rate,
            partition_prob,
        }
    }
}

prop_compose! {
    fn arb_network_conditions()(
        latency_ms in 1u64..10000,
        loss_rate in 0.0f64..1.0,
        partition_prob in 0.0f64..1.0,
    ) -> NetworkConditions {
        NetworkConditions {
            latency: Duration::from_millis(latency_ms),
            loss_rate,
            partition_prob,
        }
    }
}

prop_compose! {
    fn arb_scenario_config()(
        node_count in 0usize..50,
        duration_ms in 1u64..1000,
        msg_rate in 0.1f64..1000.0,
        network in arb_network_conditions(),
    ) -> ScenarioConfig {
        ScenarioConfig {
            node_count,
            duration: Duration::from_millis(duration_ms),
            msg_rate,
            network,
        }
    }
}

proptest! {
    #[test]
    fn test_simulator_creation_with_any_valid_config(config in arb_simulator_config()) {
        tokio_test::block_on(async move {
            let (simulator, _events_rx) = NetworkSimulator::new(config.clone());

            // Verify simulator is created with correct configuration
            prop_assert_eq!(simulator.config.node_count, config.node_count);
            prop_assert_eq!(simulator.config.latency_ms, config.latency_ms);
            prop_assert_eq!(simulator.config.drop_rate, config.drop_rate);
            prop_assert_eq!(simulator.config.partition_prob, config.partition_prob);
            prop_assert_eq!(simulator.nodes.len(), 0);
        });
    }

    #[test]
    fn test_node_addition_maintains_invariants(
        config in arb_simulator_config(),
        node_count in 0usize..20
    ) {
        tokio_test::block_on(async move {
            let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

            for i in 0..node_count {
                let result = simulator.add_node(ProtocolConfig::default()).await;
                prop_assert!(result.is_ok());
                prop_assert_eq!(simulator.nodes.len(), i + 1);
                prop_assert_eq!(simulator.nodes[i].id, format!("node-{}", i));

                // Verify event is sent
                let event_result = tokio::time::timeout(
                    Duration::from_millis(100),
                    events_rx.recv()
                ).await;
                prop_assert!(event_result.is_ok());
            }
        });
    }

    #[test]
    fn test_partition_probability_affects_partition_size(
        node_count in 1usize..50,
        partition_prob in 0.0f64..1.0
    ) {
        tokio_test::block_on(async move {
            let config = SimulatorConfig {
                node_count,
                latency_ms: 50,
                drop_rate: 0.0,
                partition_prob,
            };

            let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

            // Add nodes
            for _ in 0..node_count {
                simulator.add_node(ProtocolConfig::default()).await.unwrap();
            }

            // Drain join events
            for _ in 0..node_count {
                tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await.unwrap();
            }

            // Create partition
            simulator.create_partition().await.unwrap();

            // Verify partition event
            let event = tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await.unwrap().unwrap();
            if let super::super::network::SimulatorEvent::Partition { nodes } = event {
                let expected_size = (node_count as f64 * partition_prob) as usize;
                prop_assert_eq!(nodes.len(), expected_size);
            } else {
                return Err(proptest::test_runner::TestCaseError::fail("Expected Partition event"));
            }
        });
    }

    #[test]
    fn test_metrics_serialization_roundtrip(
        avg_latency_ms in 0u64..10000,
        p95_latency_ms in 0u64..20000,
        p99_latency_ms in 0u64..30000,
        max_latency_ms in 0u64..60000,
        msgs_per_sec in 0.0f64..100000.0,
        bytes_per_sec in 0.0f64..1000000000.0,
        drop_rate in 0.0f64..1.0,
        finality_time_ms in 0u64..10000,
        finalized_count in 0usize..1000000,
        pending_count in 0usize..10000,
    ) {
        let metrics = NetworkMetrics {
            latency: super::super::metrics::LatencyMetrics {
                avg_latency: Duration::from_millis(avg_latency_ms),
                p95_latency: Duration::from_millis(p95_latency_ms),
                p99_latency: Duration::from_millis(p99_latency_ms),
                max_latency: Duration::from_millis(max_latency_ms),
            },
            throughput: super::super::metrics::ThroughputMetrics {
                msgs_per_sec,
                bytes_per_sec,
                drop_rate,
            },
            consensus: super::super::metrics::ConsensusMetrics {
                avg_finality_time: Duration::from_millis(finality_time_ms),
                finalized_tx_count: finalized_count,
                pending_tx_count: pending_count,
            },
        };

        let serialized = serde_json::to_string(&metrics).unwrap();
        let deserialized: NetworkMetrics = serde_json::from_str(&serialized).unwrap();

        prop_assert_eq!(metrics.latency.avg_latency, deserialized.latency.avg_latency);
        prop_assert_eq!(metrics.latency.p95_latency, deserialized.latency.p95_latency);
        prop_assert_eq!(metrics.latency.p99_latency, deserialized.latency.p99_latency);
        prop_assert_eq!(metrics.latency.max_latency, deserialized.latency.max_latency);
        prop_assert_eq!(metrics.throughput.msgs_per_sec, deserialized.throughput.msgs_per_sec);
        prop_assert_eq!(metrics.throughput.bytes_per_sec, deserialized.throughput.bytes_per_sec);
        prop_assert_eq!(metrics.throughput.drop_rate, deserialized.throughput.drop_rate);
        prop_assert_eq!(metrics.consensus.avg_finality_time, deserialized.consensus.avg_finality_time);
        prop_assert_eq!(metrics.consensus.finalized_tx_count, deserialized.consensus.finalized_tx_count);
        prop_assert_eq!(metrics.consensus.pending_tx_count, deserialized.consensus.pending_tx_count);
    }

    #[test]
    fn test_scenario_execution_always_succeeds(config in arb_scenario_config()) {
        // Limit test duration to prevent timeouts
        let limited_config = ScenarioConfig {
            duration: Duration::from_millis(std::cmp::min(config.duration.as_millis() as u64, 200)),
            ..config
        };

        tokio_test::block_on(async move {
            let connectivity_result = super::super::scenarios::test_basic_connectivity(limited_config.clone()).await;
            prop_assert!(connectivity_result.is_ok());

            let byzantine_result = super::super::scenarios::test_byzantine_tolerance(limited_config.clone()).await;
            prop_assert!(byzantine_result.is_ok());

            let partition_result = super::super::scenarios::test_network_partition(limited_config).await;
            prop_assert!(partition_result.is_ok());
        });
    }

    #[test]
    fn test_node_removal_maintains_consistency(
        initial_nodes in 1usize..20,
        nodes_to_remove in prop::collection::vec(0usize..19, 0..10)
    ) {
        tokio_test::block_on(async move {
            let config = SimulatorConfig {
                node_count: initial_nodes,
                latency_ms: 50,
                drop_rate: 0.0,
                partition_prob: 0.0,
            };

            let (mut simulator, mut events_rx) = NetworkSimulator::new(config);

            // Add initial nodes
            for _ in 0..initial_nodes {
                simulator.add_node(ProtocolConfig::default()).await.unwrap();
            }

            // Drain join events
            for _ in 0..initial_nodes {
                tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await.unwrap();
            }

            let mut expected_count = initial_nodes;

            // Remove nodes (some may not exist)
            for &node_idx in &nodes_to_remove {
                if node_idx < expected_count {
                    let node_id = format!("node-{}", node_idx);
                    simulator.remove_node(&node_id).await.unwrap();
                    expected_count -= 1;

                    // Should receive NodeLeft event
                    let event = tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await.unwrap().unwrap();
                    if let super::super::network::SimulatorEvent::NodeLeft(id) = event {
                        prop_assert_eq!(id, node_id);
                    } else {
                        return Err(proptest::test_runner::TestCaseError::fail("Expected NodeLeft event"));
                    }
                } else {
                    // Trying to remove non-existent node should not change count
                    simulator.remove_node(&format!("node-{}", node_idx)).await.unwrap();
                }
            }

            // Verify final state is consistent
            prop_assert!(simulator.nodes.len() <= initial_nodes);

            // All remaining nodes should have valid IDs
            for node in &simulator.nodes {
                prop_assert!(node.id.starts_with("node-"));
            }
        });
    }

    #[test]
    fn test_simulator_config_bounds(
        node_count in 0usize..1000,
        latency_ms in 0u64..60000,
        drop_rate in -1.0f64..2.0,
        partition_prob in -1.0f64..2.0,
    ) {
        // Test that simulator handles edge cases gracefully
        let config = SimulatorConfig {
            node_count,
            latency_ms,
            drop_rate: drop_rate.max(0.0).min(1.0), // Clamp to valid range
            partition_prob: partition_prob.max(0.0).min(1.0), // Clamp to valid range
        };

        tokio_test::block_on(async move {
            let (simulator, _events_rx) = NetworkSimulator::new(config.clone());

            // Should always create successfully with clamped values
            prop_assert!(simulator.config.drop_rate >= 0.0 && simulator.config.drop_rate <= 1.0);
            prop_assert!(simulator.config.partition_prob >= 0.0 && simulator.config.partition_prob <= 1.0);
        });
    }
}
