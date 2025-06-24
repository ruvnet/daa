use criterion::{criterion_group, criterion_main, Criterion};
use qudag_simulator::{
    metrics::NetworkMetrics,
    scenarios::{NetworkConditions, ScenarioConfig},
};
use std::time::Duration;

/// Run system-wide benchmarks
pub fn benchmark_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("system");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    // Basic throughput benchmark
    group.bench_function("throughput", |b| {
        b.iter(|| {
            let config = ScenarioConfig {
                node_count: 10,
                duration: Duration::from_secs(60),
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
        })
    });

    // Consensus latency benchmark
    group.bench_function("consensus_latency", |b| {
        b.iter(|| {
            let config = ScenarioConfig {
                node_count: 10,
                duration: Duration::from_secs(60),
                msg_rate: 100.0,
                network: NetworkConditions {
                    latency: Duration::from_millis(50),
                    loss_rate: 0.01,
                    partition_prob: 0.0,
                },
            };
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    qudag_simulator::scenarios::test_byzantine_tolerance(config).await
                })
                .unwrap()
        })
    });

    // Network partition recovery benchmark
    group.bench_function("partition_recovery", |b| {
        b.iter(|| {
            let config = ScenarioConfig {
                node_count: 10,
                duration: Duration::from_secs(60),
                msg_rate: 100.0,
                network: NetworkConditions {
                    latency: Duration::from_millis(50),
                    loss_rate: 0.01,
                    partition_prob: 0.3,
                },
            };
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    qudag_simulator::scenarios::test_network_partition(config).await
                })
                .unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_system);
criterion_main!(benches);
