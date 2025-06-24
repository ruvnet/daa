use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use qudag_protocol::config::Config as ProtocolConfig;
use qudag_simulator::{
    metrics::NetworkMetrics,
    network::{NetworkSimulator, SimulatorConfig},
    scenarios::{NetworkConditions, ScenarioConfig},
};
use std::time::Duration;

fn throughput_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for node_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("message_throughput", node_count),
            node_count,
            |b, &n| {
                b.iter(|| {
                    let config = ScenarioConfig {
                        node_count: n,
                        duration: Duration::from_secs(10),
                        msg_rate: 1000.0,
                        network: NetworkConditions {
                            latency: Duration::from_millis(50),
                            loss_rate: 0.01,
                            partition_prob: 0.0,
                        },
                    };
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async {
                            qudag_simulator::scenarios::test_basic_connectivity(config).await
                        })
                        .unwrap()
                });
            },
        );
    }
    group.finish();
}

fn latency_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for node_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("message_latency", node_count),
            node_count,
            |b, &n| {
                b.iter(|| {
                    let config = ScenarioConfig {
                        node_count: n,
                        duration: Duration::from_secs(5),
                        msg_rate: 100.0,
                        network: NetworkConditions {
                            latency: Duration::from_millis(10),
                            loss_rate: 0.0,
                            partition_prob: 0.0,
                        },
                    };
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async {
                            qudag_simulator::scenarios::test_byzantine_tolerance(config).await
                        })
                        .unwrap()
                });
            },
        );
    }
    group.finish();
}

fn scalability_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(5);

    for node_count in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("network_scalability", node_count),
            node_count,
            |b, &n| {
                b.iter(|| {
                    let config = ScenarioConfig {
                        node_count: n,
                        duration: Duration::from_secs(20),
                        msg_rate: 500.0,
                        network: NetworkConditions {
                            latency: Duration::from_millis(100),
                            loss_rate: 0.02,
                            partition_prob: 0.1,
                        },
                    };
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async {
                            qudag_simulator::scenarios::test_network_partition(config).await
                        })
                        .unwrap()
                });
            },
        );
    }
    group.finish();
}

fn resource_usage_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_usage");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let node_counts = [10, 50, 100]; // Number of nodes
    for node_count in node_counts.iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", node_count),
            node_count,
            |b, &n| {
                b.iter(|| {
                    let sim_config = SimulatorConfig {
                        node_count: n,
                        latency_ms: 50,
                        drop_rate: 0.01,
                        partition_prob: 0.0,
                    };
                    tokio::runtime::Runtime::new().unwrap().block_on(async {
                        let (mut simulator, _events_rx) = NetworkSimulator::new(sim_config);
                        for _ in 0..n {
                            simulator.add_node(ProtocolConfig::default()).await.unwrap();
                        }
                        NetworkMetrics::new()
                    })
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    system_benches,
    throughput_benchmarks,
    latency_benchmarks,
    scalability_benchmarks,
    resource_usage_benchmarks
);
criterion_main!(system_benches);
