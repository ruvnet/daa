//! P2P Network Latency and Throughput Benchmarks
//!
//! Every network benchmark runs real `P2PNetwork`s on loopback TCP. Gossipsub
//! refuses to publish while no peer is subscribed to the gradient topic, so
//! the nodes are connected (via `bootstrap`) before anything is measured, and
//! a broadcast only counts as done once the receiving node has decoded it.

use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId,
    Criterion, SamplingMode, Throughput,
};
use daa_compute::p2p::gradient::{AllReduceAlgorithm, GradientManager};
use daa_compute::{P2PNetwork, PeerId, SwarmConfig};
use libp2p::gossipsub::ConfigBuilder;
use rand::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

/// Upper bound for any network condition a benchmark waits on.
const DEADLINE: Duration = Duration::from_secs(30);

/// Largest serialized gradient message a benchmark sends (1M int8 values plus
/// framing); gossipsub's default limit is 64 KiB.
const MAX_MESSAGE_BYTES: usize = 4 * 1024 * 1024;

/// Network benchmarks take milliseconds to seconds per iteration, so a few
/// flat samples give a stable mean without spending minutes per point.
fn network_group<'a>(c: &'a mut Criterion, name: &str) -> BenchmarkGroup<'a, WallTime> {
    let mut group = c.benchmark_group(name);
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(3))
        .sampling_mode(SamplingMode::Flat);
    group
}

/// A loopback-only node with room for large gradient messages.
fn node_config(compression_level: u32, nat_and_relay: bool) -> SwarmConfig {
    SwarmConfig {
        listen_addresses: vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()],
        enable_mdns: false,
        enable_nat_traversal: nat_and_relay,
        enable_relay: nat_and_relay,
        gossipsub_config: ConfigBuilder::default()
            .max_transmit_size(MAX_MESSAGE_BYTES)
            .build()
            .expect("valid gossipsub config"),
        compression_level,
        ..SwarmConfig::default()
    }
}

/// Wait for the next event on any of `networks`; returns the index handled.
async fn step(networks: &mut [&mut P2PNetwork]) -> usize {
    let events = networks
        .iter_mut()
        .map(|network| Box::pin(network.process_next_event()));
    // The other networks are still waiting on their swarm, which is
    // cancellation-safe.
    let (result, index, _) = futures::future::select_all(events).await;
    result.expect("network event loop failed");
    index
}

/// Pump `networks` until `done` holds, panicking after [`DEADLINE`].
async fn pump_until(
    networks: &mut [&mut P2PNetwork],
    what: &str,
    mut done: impl FnMut(&[&mut P2PNetwork]) -> bool,
) {
    let start = Instant::now();
    while !done(networks) {
        assert!(start.elapsed() < DEADLINE, "timed out waiting for {what}");
        let _ = tokio::time::timeout(Duration::from_millis(50), step(networks)).await;
    }
}

/// Pump until the node's listener is bound and return its address.
async fn listen_address(network: &mut P2PNetwork) -> libp2p::Multiaddr {
    pump_until(&mut [&mut *network], "a listen address", |n| {
        !n[0].listen_addresses().is_empty()
    })
    .await;
    network.listen_addresses()[0].clone()
}

/// Bring up `spokes` nodes that dial `hub`, and wait until all are connected.
///
/// The TCP transport dials from its listen port (port reuse, for hole
/// punching). When nodes discover and dial each other at the same moment the
/// dials can collide on one socket pair, which then refuses redials for a while,
/// so a spoke that has not connected within a second is replaced by a fresh
/// node, as a real deployment would move on to another peer.
async fn connect_spokes(hub: &mut P2PNetwork, spokes: usize, config: impl Fn() -> SwarmConfig) {
    let hub_id = hub.local_peer_id();
    let hub_addr = listen_address(hub).await;
    let mut nodes = Vec::with_capacity(spokes);
    for _ in 0..spokes {
        let mut node = P2PNetwork::new(config()).await.expect("spoke network");
        node.connect_to_peer(hub_id, hub_addr.clone())
            .expect("dial hub");
        nodes.push(node);
    }
    let start = Instant::now();
    let mut attempt = Instant::now();
    loop {
        let connected = |node: &P2PNetwork| node.connected_peers().contains(&hub_id);
        let mut all: Vec<&mut P2PNetwork> = vec![&mut *hub];
        all.extend(nodes.iter_mut());
        let _ = tokio::time::timeout(Duration::from_millis(250), async {
            while !all[1..].iter().all(|node| connected(node)) {
                step(&mut all).await;
            }
        })
        .await;
        if nodes.iter().all(connected) {
            return;
        }
        assert!(
            start.elapsed() < DEADLINE,
            "timed out waiting for spokes to connect"
        );
        if attempt.elapsed() >= Duration::from_secs(1) {
            attempt = Instant::now();
            for node in nodes.iter_mut().filter(|node| !connected(node)) {
                *node = P2PNetwork::new(config()).await.expect("spoke network");
                node.connect_to_peer(hub_id, hub_addr.clone())
                    .expect("dial hub");
            }
        }
    }
}

/// Two connected nodes: `sender` broadcasts, `receiver` decodes.
struct Pair {
    sender: P2PNetwork,
    receiver: P2PNetwork,
    /// Sign of the marker in the next delivery, alternated so each delivery is
    /// distinguishable from the previous one.
    marker: f32,
}

/// Value written to `gradient[0]` of the message a delivery waits for. Every
/// other value stays within ±0.01, and int8 quantisation keeps the min/max
/// exactly, so the marker survives transit.
const MARKER: f32 = 0.02;

impl Pair {
    async fn connect(config: impl Fn() -> SwarmConfig) -> Self {
        let mut receiver = P2PNetwork::new(config()).await.expect("receiver network");
        let receiver_id = receiver.local_peer_id();
        let receiver_addr = listen_address(&mut receiver).await;
        let mut sender = P2PNetwork::new(SwarmConfig {
            bootstrap_nodes: vec![(receiver_id, receiver_addr)],
            ..config()
        })
        .await
        .expect("sender network");
        sender.bootstrap().await.expect("bootstrap from receiver");

        let mut pair = Self {
            sender,
            receiver,
            marker: MARKER,
        };
        // The first delivery also waits for the gossipsub subscription exchange.
        pair.deliver(vec![vec![0.0; 16]]).await;
        pair
    }

    /// Broadcast `gradients` in order and wait until the receiver has decoded
    /// the last one. Returns the time from first publish to receipt.
    async fn deliver(&mut self, mut gradients: Vec<Vec<f32>>) -> Duration {
        let marker = self.marker;
        self.marker = -self.marker;
        let count = gradients.len();
        for (i, gradient) in gradients.iter_mut().enumerate() {
            gradient[0] = if i + 1 == count { marker } else { 0.0 };
        }
        let sender_id = self.sender.local_peer_id();

        let start = Instant::now();
        for gradient in gradients {
            // Publishing fails until a peer is subscribed or while send queues
            // are full; both clear by driving the connection.
            loop {
                match self.sender.broadcast_gradient(gradient.clone()).await {
                    Ok(()) => break,
                    Err(e) => {
                        assert!(start.elapsed() < DEADLINE, "broadcast failed: {e}");
                        let mut both = [&mut self.sender, &mut self.receiver];
                        let _ =
                            tokio::time::timeout(Duration::from_millis(10), step(&mut both)).await;
                    }
                }
            }
        }
        loop {
            let latest = self
                .receiver
                .received_gradients()
                .await
                .into_iter()
                .find(|(peer, _)| *peer == sender_id);
            if let Some((_, values)) = latest {
                if (values[0] - marker).abs() < MARKER / 2.0 {
                    return start.elapsed();
                }
            }
            assert!(start.elapsed() < DEADLINE, "gradient was not delivered");
            // Only re-check after the receiver has handled an event.
            let mut both = [&mut self.sender, &mut self.receiver];
            let _ = tokio::time::timeout(Duration::from_millis(50), async {
                while step(&mut both).await != 1 {}
            })
            .await;
        }
    }
}

/// Gradient of `len` small random values (`gradient[0]` is the delivery marker).
fn random_gradient(len: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..len.max(1))
        .map(|_| rng.gen_range(-0.01..0.01))
        .collect()
}

/// Benchmark `gradients` deliveries over a pair connected with `config`.
fn bench_delivery(
    b: &mut criterion::Bencher<'_, WallTime>,
    rt: &Runtime,
    config: impl Fn() -> SwarmConfig,
    gradients: Vec<Vec<f32>>,
) {
    let pair = Arc::new(Mutex::new(rt.block_on(Pair::connect(config))));
    b.to_async(rt).iter_custom(|iters| {
        let pair = pair.clone();
        let gradients = gradients.clone();
        async move {
            let mut pair = pair.lock().await;
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                total += pair.deliver(gradients.clone()).await;
            }
            total
        }
    });
}

/// Benchmark P2P message latency
fn benchmark_p2p_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = network_group(c, "p2p_latency");

    // Test different message sizes
    let message_sizes = vec![1_024, 10_240, 102_400, 1_048_576]; // 1KB, 10KB, 100KB, 1MB

    for size in message_sizes {
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("broadcast", size), &size, |b, &size| {
            // Create gradient of specified size and measure delivery latency
            let gradient = random_gradient(size / 4);
            bench_delivery(b, &rt, || node_config(3, true), vec![gradient]);
        });
    }

    group.finish();
}

/// Benchmark P2P network throughput
fn benchmark_p2p_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = network_group(c, "p2p_throughput");

    // Test sustained throughput with different batch sizes
    let batch_sizes = vec![10, 50, 100];
    let gradient_size = 100_000; // 100K parameters

    for batch_size in batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("batch_broadcast", batch_size),
            &batch_size,
            |b, &batch_size| {
                // Broadcast multiple gradients to measure throughput
                let batch: Vec<Vec<f32>> = (0..batch_size)
                    .map(|_| random_gradient(gradient_size))
                    .collect();
                bench_delivery(b, &rt, || node_config(3, true), batch);
            },
        );
    }

    group.finish();
}

/// Benchmark gradient compression impact on latency
fn benchmark_compression_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = network_group(c, "compression_impact");

    let gradient_size = 1_000_000; // 1M parameters
    let compression_levels = vec![0, 3, 6, 9];

    // Create realistic gradient (sparse, with many small values):
    // 10% of values non-zero (typical sparsity)
    let mut rng = rand::thread_rng();
    let mut gradient = vec![0.0f32; gradient_size];
    for value in gradient.iter_mut().step_by(10) {
        *value = rng.gen_range(-0.01..0.01);
    }

    for level in compression_levels {
        group.bench_with_input(
            BenchmarkId::new("compression_level", level),
            &level,
            |b, &level| {
                bench_delivery(b, &rt, || node_config(level, true), vec![gradient.clone()]);
            },
        );
    }

    group.finish();
}

/// Benchmark NAT traversal overhead
fn benchmark_nat_traversal(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = network_group(c, "nat_traversal");

    for (name, nat_and_relay) in [("with_nat", true), ("without_nat", false)] {
        group.bench_function(name, |b| {
            // Send test gradient over a pair connected via bootstrap
            let gradient = random_gradient(10_000);
            bench_delivery(b, &rt, || node_config(3, nat_and_relay), vec![gradient]);
        });
    }

    group.finish();
}

/// Benchmark peer discovery time
fn benchmark_peer_discovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = network_group(c, "peer_discovery");

    // Test discovery with different network sizes
    let network_sizes = vec![4, 8, 16];

    for size in network_sizes {
        group.bench_with_input(BenchmarkId::new("network_size", size), &size, |b, &size| {
            b.to_async(&rt).iter_custom(|iters| async move {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let mut hub = P2PNetwork::new(node_config(3, true))
                        .await
                        .expect("hub network");
                    listen_address(&mut hub).await;

                    // Measure time for `size` peers to dial and connect
                    let start = Instant::now();
                    connect_spokes(&mut hub, size, || node_config(3, true)).await;
                    total += start.elapsed();
                }
                total
            });
        });
    }

    group.finish();
}

/// Benchmark gradient aggregation algorithms
fn benchmark_gradient_aggregation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = network_group(c, "gradient_aggregation");

    let gradient_sizes = vec![10_000, 100_000, 1_000_000];
    let peer_counts = vec![4, 16];

    for gradient_size in gradient_sizes {
        for &peer_count in &peer_counts {
            group.throughput(Throughput::Elements((gradient_size * peer_count) as u64));

            // One gradient per peer, including our own
            let local = PeerId::random();
            let manager = GradientManager::new(local, 3);
            let gradients: HashMap<PeerId, Vec<f32>> = (0..peer_count)
                .map(|i| {
                    let peer = if i == 0 { local } else { PeerId::random() };
                    (peer, random_gradient(gradient_size))
                })
                .collect();

            group.bench_with_input(
                BenchmarkId::new(format!("size_{}_peers", gradient_size), peer_count),
                &gradients,
                |b, gradients| {
                    b.to_async(&rt).iter(|| async {
                        // All-reduce operation (averages the gradients)
                        let aggregated = manager
                            .all_reduce(AllReduceAlgorithm::Ring, gradients)
                            .await
                            .expect("all-reduce");
                        black_box(aggregated)
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_p2p_latency,
    benchmark_p2p_throughput,
    benchmark_compression_impact,
    benchmark_nat_traversal,
    benchmark_peer_discovery,
    benchmark_gradient_aggregation
);
criterion_main!(benches);
