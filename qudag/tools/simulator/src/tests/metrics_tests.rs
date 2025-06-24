use super::super::metrics::{ConsensusMetrics, LatencyMetrics, NetworkMetrics, ThroughputMetrics};
use std::time::Duration;

#[test]
fn test_network_metrics_creation() {
    let metrics = NetworkMetrics::new();

    // Verify all metrics are initialized with default values
    assert_eq!(metrics.latency.avg_latency, Duration::from_secs(0));
    assert_eq!(metrics.latency.p95_latency, Duration::from_secs(0));
    assert_eq!(metrics.latency.p99_latency, Duration::from_secs(0));
    assert_eq!(metrics.latency.max_latency, Duration::from_secs(0));

    assert_eq!(metrics.throughput.msgs_per_sec, 0.0);
    assert_eq!(metrics.throughput.bytes_per_sec, 0.0);
    assert_eq!(metrics.throughput.drop_rate, 0.0);

    assert_eq!(metrics.consensus.avg_finality_time, Duration::from_secs(0));
    assert_eq!(metrics.consensus.finalized_tx_count, 0);
    assert_eq!(metrics.consensus.pending_tx_count, 0);
}

#[test]
fn test_latency_metrics_serialization() {
    let latency = LatencyMetrics {
        avg_latency: Duration::from_millis(100),
        p95_latency: Duration::from_millis(200),
        p99_latency: Duration::from_millis(500),
        max_latency: Duration::from_millis(1000),
    };

    let serialized = serde_json::to_string(&latency).unwrap();
    let deserialized: LatencyMetrics = serde_json::from_str(&serialized).unwrap();

    assert_eq!(latency.avg_latency, deserialized.avg_latency);
    assert_eq!(latency.p95_latency, deserialized.p95_latency);
    assert_eq!(latency.p99_latency, deserialized.p99_latency);
    assert_eq!(latency.max_latency, deserialized.max_latency);
}

#[test]
fn test_throughput_metrics_serialization() {
    let throughput = ThroughputMetrics {
        msgs_per_sec: 1000.5,
        bytes_per_sec: 1024000.75,
        drop_rate: 0.01,
    };

    let serialized = serde_json::to_string(&throughput).unwrap();
    let deserialized: ThroughputMetrics = serde_json::from_str(&serialized).unwrap();

    assert_eq!(throughput.msgs_per_sec, deserialized.msgs_per_sec);
    assert_eq!(throughput.bytes_per_sec, deserialized.bytes_per_sec);
    assert_eq!(throughput.drop_rate, deserialized.drop_rate);
}

#[test]
fn test_consensus_metrics_serialization() {
    let consensus = ConsensusMetrics {
        avg_finality_time: Duration::from_millis(800),
        finalized_tx_count: 12345,
        pending_tx_count: 67,
    };

    let serialized = serde_json::to_string(&consensus).unwrap();
    let deserialized: ConsensusMetrics = serde_json::from_str(&serialized).unwrap();

    assert_eq!(consensus.avg_finality_time, deserialized.avg_finality_time);
    assert_eq!(
        consensus.finalized_tx_count,
        deserialized.finalized_tx_count
    );
    assert_eq!(consensus.pending_tx_count, deserialized.pending_tx_count);
}

#[test]
fn test_network_metrics_serialization() {
    let metrics = NetworkMetrics {
        latency: LatencyMetrics {
            avg_latency: Duration::from_millis(50),
            p95_latency: Duration::from_millis(100),
            p99_latency: Duration::from_millis(200),
            max_latency: Duration::from_millis(500),
        },
        throughput: ThroughputMetrics {
            msgs_per_sec: 5000.0,
            bytes_per_sec: 2048000.0,
            drop_rate: 0.02,
        },
        consensus: ConsensusMetrics {
            avg_finality_time: Duration::from_millis(600),
            finalized_tx_count: 9999,
            pending_tx_count: 42,
        },
    };

    let serialized = serde_json::to_string(&metrics).unwrap();
    let deserialized: NetworkMetrics = serde_json::from_str(&serialized).unwrap();

    // Verify latency metrics
    assert_eq!(
        metrics.latency.avg_latency,
        deserialized.latency.avg_latency
    );
    assert_eq!(
        metrics.latency.p95_latency,
        deserialized.latency.p95_latency
    );
    assert_eq!(
        metrics.latency.p99_latency,
        deserialized.latency.p99_latency
    );
    assert_eq!(
        metrics.latency.max_latency,
        deserialized.latency.max_latency
    );

    // Verify throughput metrics
    assert_eq!(
        metrics.throughput.msgs_per_sec,
        deserialized.throughput.msgs_per_sec
    );
    assert_eq!(
        metrics.throughput.bytes_per_sec,
        deserialized.throughput.bytes_per_sec
    );
    assert_eq!(
        metrics.throughput.drop_rate,
        deserialized.throughput.drop_rate
    );

    // Verify consensus metrics
    assert_eq!(
        metrics.consensus.avg_finality_time,
        deserialized.consensus.avg_finality_time
    );
    assert_eq!(
        metrics.consensus.finalized_tx_count,
        deserialized.consensus.finalized_tx_count
    );
    assert_eq!(
        metrics.consensus.pending_tx_count,
        deserialized.consensus.pending_tx_count
    );
}

#[test]
fn test_metrics_debug_format() {
    let metrics = NetworkMetrics::new();
    let debug_str = format!("{:?}", metrics);

    // Verify debug output contains key fields
    assert!(debug_str.contains("NetworkMetrics"));
    assert!(debug_str.contains("latency"));
    assert!(debug_str.contains("throughput"));
    assert!(debug_str.contains("consensus"));
}

#[test]
fn test_latency_metrics_edge_cases() {
    // Test with very small durations
    let latency = LatencyMetrics {
        avg_latency: Duration::from_nanos(1),
        p95_latency: Duration::from_nanos(10),
        p99_latency: Duration::from_nanos(100),
        max_latency: Duration::from_nanos(1000),
    };

    let serialized = serde_json::to_string(&latency).unwrap();
    let deserialized: LatencyMetrics = serde_json::from_str(&serialized).unwrap();

    assert_eq!(latency.avg_latency, deserialized.avg_latency);
    assert_eq!(latency.p95_latency, deserialized.p95_latency);
    assert_eq!(latency.p99_latency, deserialized.p99_latency);
    assert_eq!(latency.max_latency, deserialized.max_latency);
}

#[test]
fn test_throughput_metrics_edge_cases() {
    // Test with very high throughput
    let throughput = ThroughputMetrics {
        msgs_per_sec: 1_000_000.0,
        bytes_per_sec: 1_000_000_000.0,
        drop_rate: 1.0, // 100% drop rate
    };

    let serialized = serde_json::to_string(&throughput).unwrap();
    let deserialized: ThroughputMetrics = serde_json::from_str(&serialized).unwrap();

    assert_eq!(throughput.msgs_per_sec, deserialized.msgs_per_sec);
    assert_eq!(throughput.bytes_per_sec, deserialized.bytes_per_sec);
    assert_eq!(throughput.drop_rate, deserialized.drop_rate);
}

#[test]
fn test_consensus_metrics_edge_cases() {
    // Test with very large numbers
    let consensus = ConsensusMetrics {
        avg_finality_time: Duration::from_secs(3600), // 1 hour
        finalized_tx_count: usize::MAX,
        pending_tx_count: 0,
    };

    let serialized = serde_json::to_string(&consensus).unwrap();
    let deserialized: ConsensusMetrics = serde_json::from_str(&serialized).unwrap();

    assert_eq!(consensus.avg_finality_time, deserialized.avg_finality_time);
    assert_eq!(
        consensus.finalized_tx_count,
        deserialized.finalized_tx_count
    );
    assert_eq!(consensus.pending_tx_count, deserialized.pending_tx_count);
}

#[test]
fn test_metrics_clone() {
    let original = NetworkMetrics::new();
    let cloned = original.clone();

    // Verify that cloned values match original
    assert_eq!(original.latency.avg_latency, cloned.latency.avg_latency);
    assert_eq!(
        original.throughput.msgs_per_sec,
        cloned.throughput.msgs_per_sec
    );
    assert_eq!(
        original.consensus.finalized_tx_count,
        cloned.consensus.finalized_tx_count
    );
}

#[test]
fn test_metrics_with_realistic_values() {
    let metrics = NetworkMetrics {
        latency: LatencyMetrics {
            avg_latency: Duration::from_millis(45),
            p95_latency: Duration::from_millis(95),
            p99_latency: Duration::from_millis(180),
            max_latency: Duration::from_millis(350),
        },
        throughput: ThroughputMetrics {
            msgs_per_sec: 8_500.0,
            bytes_per_sec: 12_800_000.0, // ~12.8 MB/s
            drop_rate: 0.001,            // 0.1% drop rate
        },
        consensus: ConsensusMetrics {
            avg_finality_time: Duration::from_millis(950), // Sub-second finality
            finalized_tx_count: 1_234_567,
            pending_tx_count: 89,
        },
    };

    // Verify realistic constraints
    assert!(metrics.latency.avg_latency <= metrics.latency.p95_latency);
    assert!(metrics.latency.p95_latency <= metrics.latency.p99_latency);
    assert!(metrics.latency.p99_latency <= metrics.latency.max_latency);
    assert!(metrics.throughput.drop_rate >= 0.0 && metrics.throughput.drop_rate <= 1.0);
    assert!(metrics.consensus.avg_finality_time < Duration::from_secs(1)); // Sub-second requirement
    assert!(metrics.throughput.msgs_per_sec > 10_000.0); // Performance target exceeded
}
