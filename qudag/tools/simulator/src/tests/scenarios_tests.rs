use super::super::scenarios::{
    test_basic_connectivity, test_byzantine_tolerance, test_network_partition, NetworkConditions,
    ScenarioConfig,
};
use std::time::Duration;

#[tokio::test]
async fn test_scenario_config_creation() {
    let config = ScenarioConfig {
        node_count: 10,
        duration: Duration::from_secs(30),
        msg_rate: 100.0,
        network: NetworkConditions {
            latency: Duration::from_millis(50),
            loss_rate: 0.01,
            partition_prob: 0.1,
        },
    };

    assert_eq!(config.node_count, 10);
    assert_eq!(config.duration, Duration::from_secs(30));
    assert_eq!(config.msg_rate, 100.0);
    assert_eq!(config.network.latency, Duration::from_millis(50));
    assert_eq!(config.network.loss_rate, 0.01);
    assert_eq!(config.network.partition_prob, 0.1);
}

#[tokio::test]
async fn test_basic_connectivity_scenario() {
    let config = ScenarioConfig {
        node_count: 4,
        duration: Duration::from_millis(100), // Short duration for test
        msg_rate: 10.0,
        network: NetworkConditions {
            latency: Duration::from_millis(20),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_basic_connectivity(config).await;
    assert!(result.is_ok());

    let metrics = result.unwrap();
    // Verify metrics structure is correct
    assert_eq!(metrics.latency.avg_latency, Duration::from_secs(0));
    assert_eq!(metrics.throughput.msgs_per_sec, 0.0);
    assert_eq!(metrics.consensus.finalized_tx_count, 0);
}

#[tokio::test]
async fn test_basic_connectivity_single_node() {
    let config = ScenarioConfig {
        node_count: 1,
        duration: Duration::from_millis(50),
        msg_rate: 1.0,
        network: NetworkConditions {
            latency: Duration::from_millis(10),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_basic_connectivity(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_basic_connectivity_large_network() {
    let config = ScenarioConfig {
        node_count: 100,
        duration: Duration::from_millis(100),
        msg_rate: 50.0,
        network: NetworkConditions {
            latency: Duration::from_millis(30),
            loss_rate: 0.02,
            partition_prob: 0.1,
        },
    };

    let result = test_basic_connectivity(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_byzantine_tolerance_scenario() {
    let config = ScenarioConfig {
        node_count: 6, // 4 honest + 2 byzantine
        duration: Duration::from_millis(100),
        msg_rate: 10.0,
        network: NetworkConditions {
            latency: Duration::from_millis(20),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_byzantine_tolerance(config).await;
    assert!(result.is_ok());

    let metrics = result.unwrap();
    // Verify metrics structure is correct
    assert_eq!(metrics.latency.avg_latency, Duration::from_secs(0));
    assert_eq!(metrics.throughput.msgs_per_sec, 0.0);
    assert_eq!(metrics.consensus.finalized_tx_count, 0);
}

#[tokio::test]
async fn test_byzantine_tolerance_minimum_nodes() {
    // Test with minimum viable Byzantine tolerance setup
    let config = ScenarioConfig {
        node_count: 3, // 2 honest + 1 byzantine
        duration: Duration::from_millis(50),
        msg_rate: 5.0,
        network: NetworkConditions {
            latency: Duration::from_millis(15),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_byzantine_tolerance(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_byzantine_tolerance_high_latency() {
    let config = ScenarioConfig {
        node_count: 9, // 6 honest + 3 byzantine
        duration: Duration::from_millis(200),
        msg_rate: 20.0,
        network: NetworkConditions {
            latency: Duration::from_millis(500), // High latency
            loss_rate: 0.05,
            partition_prob: 0.1,
        },
    };

    let result = test_byzantine_tolerance(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_partition_scenario() {
    let config = ScenarioConfig {
        node_count: 8,
        duration: Duration::from_millis(100),
        msg_rate: 15.0,
        network: NetworkConditions {
            latency: Duration::from_millis(25),
            loss_rate: 0.01,
            partition_prob: 0.5,
        },
    };

    let result = test_network_partition(config).await;
    assert!(result.is_ok());

    let metrics = result.unwrap();
    // Verify metrics structure is correct
    assert_eq!(metrics.latency.avg_latency, Duration::from_secs(0));
    assert_eq!(metrics.throughput.msgs_per_sec, 0.0);
    assert_eq!(metrics.consensus.finalized_tx_count, 0);
}

#[tokio::test]
async fn test_network_partition_small_network() {
    let config = ScenarioConfig {
        node_count: 2,
        duration: Duration::from_millis(50),
        msg_rate: 5.0,
        network: NetworkConditions {
            latency: Duration::from_millis(10),
            loss_rate: 0.0,
            partition_prob: 0.5,
        },
    };

    let result = test_network_partition(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_partition_high_partition_probability() {
    let config = ScenarioConfig {
        node_count: 10,
        duration: Duration::from_millis(150),
        msg_rate: 25.0,
        network: NetworkConditions {
            latency: Duration::from_millis(40),
            loss_rate: 0.03,
            partition_prob: 0.9, // Very high partition probability
        },
    };

    let result = test_network_partition(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scenario_config_edge_cases() {
    // Test with zero duration
    let config = ScenarioConfig {
        node_count: 5,
        duration: Duration::from_millis(0),
        msg_rate: 10.0,
        network: NetworkConditions {
            latency: Duration::from_millis(20),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_basic_connectivity(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scenario_zero_nodes() {
    let config = ScenarioConfig {
        node_count: 0,
        duration: Duration::from_millis(50),
        msg_rate: 10.0,
        network: NetworkConditions {
            latency: Duration::from_millis(20),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_basic_connectivity(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scenario_extreme_message_rate() {
    let config = ScenarioConfig {
        node_count: 3,
        duration: Duration::from_millis(100),
        msg_rate: 10000.0, // Very high message rate
        network: NetworkConditions {
            latency: Duration::from_millis(10),
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_basic_connectivity(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scenario_extreme_latency() {
    let config = ScenarioConfig {
        node_count: 4,
        duration: Duration::from_millis(200),
        msg_rate: 5.0,
        network: NetworkConditions {
            latency: Duration::from_secs(2), // Very high latency
            loss_rate: 0.0,
            partition_prob: 0.0,
        },
    };

    let result = test_basic_connectivity(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scenario_maximum_loss_rate() {
    let config = ScenarioConfig {
        node_count: 5,
        duration: Duration::from_millis(100),
        msg_rate: 10.0,
        network: NetworkConditions {
            latency: Duration::from_millis(20),
            loss_rate: 1.0, // 100% loss rate
            partition_prob: 0.0,
        },
    };

    let result = test_basic_connectivity(config).await;
    assert!(result.is_ok());
}

#[test]
fn test_network_conditions_debug() {
    let conditions = NetworkConditions {
        latency: Duration::from_millis(50),
        loss_rate: 0.05,
        partition_prob: 0.1,
    };

    let debug_str = format!("{:?}", conditions);
    assert!(debug_str.contains("NetworkConditions"));
    assert!(debug_str.contains("latency"));
    assert!(debug_str.contains("loss_rate"));
    assert!(debug_str.contains("partition_prob"));
}

#[test]
fn test_scenario_config_debug() {
    let config = ScenarioConfig {
        node_count: 7,
        duration: Duration::from_secs(5),
        msg_rate: 50.0,
        network: NetworkConditions {
            latency: Duration::from_millis(30),
            loss_rate: 0.02,
            partition_prob: 0.15,
        },
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ScenarioConfig"));
    assert!(debug_str.contains("node_count"));
    assert!(debug_str.contains("duration"));
    assert!(debug_str.contains("msg_rate"));
    assert!(debug_str.contains("network"));
}

#[test]
fn test_scenario_config_clone() {
    let original = ScenarioConfig {
        node_count: 6,
        duration: Duration::from_secs(10),
        msg_rate: 75.0,
        network: NetworkConditions {
            latency: Duration::from_millis(40),
            loss_rate: 0.03,
            partition_prob: 0.2,
        },
    };

    let cloned = original.clone();

    assert_eq!(original.node_count, cloned.node_count);
    assert_eq!(original.duration, cloned.duration);
    assert_eq!(original.msg_rate, cloned.msg_rate);
    assert_eq!(original.network.latency, cloned.network.latency);
    assert_eq!(original.network.loss_rate, cloned.network.loss_rate);
    assert_eq!(
        original.network.partition_prob,
        cloned.network.partition_prob
    );
}
