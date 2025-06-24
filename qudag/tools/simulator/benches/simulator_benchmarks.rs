use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use qudag_simulator::{
    metrics::NetworkMetrics,
    network::{NetworkSimulator, SimulatorConfig},
    scenarios::{
        test_basic_connectivity, test_byzantine_tolerance, test_network_partition,
        NetworkConditions, ScenarioConfig,
    },
};
use std::time::Duration;
use tokio::runtime::Runtime;

pub fn benchmark_simulator(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulator");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    // Network setup benchmark with varying node counts
    for node_count in [5, 10, 25, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("network_setup", node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    Runtime::new().unwrap().block_on(async {
                        let config = SimulatorConfig {
                            node_count,
                            latency_ms: 50,
                            drop_rate: 0.01,
                            partition_prob: 0.0,
                        };

                        let (mut sim, _) = NetworkSimulator::new(config);

                        // Add nodes
                        for _ in 0..node_count {
                            sim.add_node(Default::default()).await.unwrap();
                        }

                        sim
                    })
                })
            },
        );
    }

    // Message routing benchmark with varying network sizes
    for node_count in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("basic_connectivity", node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    Runtime::new().unwrap().block_on(async {
                        let config = ScenarioConfig {
                            node_count,
                            duration: Duration::from_millis(100), // Shorter for benchmarking
                            msg_rate: 100.0,
                            network: NetworkConditions {
                                latency: Duration::from_millis(10),
                                loss_rate: 0.01,
                                partition_prob: 0.0,
                            },
                        };

                        test_basic_connectivity(config).await.unwrap()
                    })
                })
            },
        );
    }

    // Byzantine tolerance benchmark
    for node_count in [6, 12, 21].iter() {
        group.bench_with_input(
            BenchmarkId::new("byzantine_tolerance", node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    Runtime::new().unwrap().block_on(async {
                        let config = ScenarioConfig {
                            node_count,
                            duration: Duration::from_millis(100),
                            msg_rate: 50.0,
                            network: NetworkConditions {
                                latency: Duration::from_millis(20),
                                loss_rate: 0.02,
                                partition_prob: 0.0,
                            },
                        };

                        test_byzantine_tolerance(config).await.unwrap()
                    })
                })
            },
        );
    }

    // Network partition benchmark
    for partition_prob in [0.1, 0.3, 0.5].iter() {
        group.bench_with_input(
            BenchmarkId::new("network_partition", (partition_prob * 100.0) as u32),
            partition_prob,
            |b, &partition_prob| {
                b.iter(|| {
                    Runtime::new().unwrap().block_on(async {
                        let config = ScenarioConfig {
                            node_count: 10,
                            duration: Duration::from_millis(150),
                            msg_rate: 20.0,
                            network: NetworkConditions {
                                latency: Duration::from_millis(30),
                                loss_rate: 0.01,
                                partition_prob,
                            },
                        };

                        test_network_partition(config).await.unwrap()
                    })
                })
            },
        );
    }

    group.finish();
}

pub fn benchmark_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics");
    group.sample_size(100);

    // Metrics creation benchmark
    group.bench_function("metrics_creation", |b| b.iter(|| NetworkMetrics::new()));

    // Metrics serialization benchmark
    group.bench_function("metrics_serialization", |b| {
        let metrics = NetworkMetrics::new();
        b.iter(|| serde_json::to_string(&metrics).unwrap())
    });

    // Metrics deserialization benchmark
    group.bench_function("metrics_deserialization", |b| {
        let metrics = NetworkMetrics::new();
        let serialized = serde_json::to_string(&metrics).unwrap();
        b.iter(|| {
            let _: NetworkMetrics = serde_json::from_str(&serialized).unwrap();
        })
    });

    group.finish();
}

pub fn benchmark_node_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_operations");
    group.sample_size(50);

    // Node addition benchmark
    group.bench_function("node_addition", |b| {
        b.iter(|| {
            Runtime::new().unwrap().block_on(async {
                let config = SimulatorConfig {
                    node_count: 1,
                    latency_ms: 10,
                    drop_rate: 0.0,
                    partition_prob: 0.0,
                };

                let (mut sim, _) = NetworkSimulator::new(config);
                sim.add_node(Default::default()).await.unwrap();
                sim
            })
        })
    });

    // Node removal benchmark
    group.bench_function("node_removal", |b| {
        b.iter(|| {
            Runtime::new().unwrap().block_on(async {
                let config = SimulatorConfig {
                    node_count: 1,
                    latency_ms: 10,
                    drop_rate: 0.0,
                    partition_prob: 0.0,
                };

                let (mut sim, _) = NetworkSimulator::new(config);
                sim.add_node(Default::default()).await.unwrap();
                sim.remove_node("node-0").await.unwrap();
                sim
            })
        })
    });

    // Partition creation benchmark
    group.bench_function("partition_creation", |b| {
        b.iter(|| {
            Runtime::new().unwrap().block_on(async {
                let config = SimulatorConfig {
                    node_count: 10,
                    latency_ms: 10,
                    drop_rate: 0.0,
                    partition_prob: 0.5,
                };

                let (mut sim, _) = NetworkSimulator::new(config);

                // Add nodes
                for _ in 0..10 {
                    sim.add_node(Default::default()).await.unwrap();
                }

                sim.create_partition().await.unwrap();
                sim
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_simulator,
    benchmark_metrics,
    benchmark_node_operations
);
criterion_main!(benches);
