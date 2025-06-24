use criterion::{criterion_group, criterion_main, Criterion};
use qudag_simulator::{
    metrics::NetworkMetrics,
    network::{NetworkSimulator, SimulatorConfig},
};
use std::time::Duration;

/// Run specific scenario benchmarks
pub fn benchmark_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenarios");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    // Large network scenario
    group.bench_function("large_network", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let (mut sim, _) = NetworkSimulator::new(SimulatorConfig {
                        node_count: 100,
                        latency_ms: 50,
                        drop_rate: 0.01,
                        partition_prob: 0.0,
                    });

                    // Add nodes
                    for _ in 0..100 {
                        sim.add_node(Default::default()).await.unwrap();
                    }

                    tokio::time::sleep(Duration::from_secs(60)).await;
                    Ok::<NetworkMetrics, anyhow::Error>(NetworkMetrics::new())
                })
                .unwrap()
        })
    });

    // High message rate scenario
    group.bench_function("high_message_rate", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let (mut sim, _) = NetworkSimulator::new(SimulatorConfig {
                        node_count: 10,
                        latency_ms: 50,
                        drop_rate: 0.01,
                        partition_prob: 0.0,
                    });

                    // Add nodes
                    for _ in 0..10 {
                        sim.add_node(Default::default()).await.unwrap();
                    }

                    // Generate high message load
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    Ok::<NetworkMetrics, anyhow::Error>(NetworkMetrics::new())
                })
                .unwrap()
        })
    });

    // Network churn scenario
    group.bench_function("network_churn", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let (mut sim, _) = NetworkSimulator::new(SimulatorConfig {
                        node_count: 20,
                        latency_ms: 50,
                        drop_rate: 0.01,
                        partition_prob: 0.0,
                    });

                    // Add/remove nodes randomly
                    for _ in 0..20 {
                        if rand::random::<bool>() {
                            sim.add_node(Default::default()).await.unwrap();
                        } else {
                            sim.remove_node("random-id").await.unwrap();
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }

                    Ok::<NetworkMetrics, anyhow::Error>(NetworkMetrics::new())
                })
                .unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_scenarios);
criterion_main!(benches);
