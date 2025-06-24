use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qudag_network::connection::{ConnectionManager, SecureConfig, SecureConnection, TransportKeys};
use qudag_network::types::{ConnectionStatus, MessagePriority, NetworkMessage, PeerId};
use quinn::{Endpoint, ServerConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

fn benchmark_optimized_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Optimized test configuration with higher throughput targets
    const MSG_COUNT: usize = 1_000_000; // Test with 1M messages for stress testing
    const BATCH_SIZE: usize = 4096; // Larger batches for better throughput
    const CONCURRENT_SENDERS: usize = 16; // More concurrent senders
    const MSG_SIZE: usize = 512; // Optimal message size for network MTU

    // Benchmark ultra-high-throughput message processing
    c.bench_function("ultra_high_throughput_1M", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = Arc::new(ConnectionManager::new(10000)); // Higher connection limit
                let mut handles = vec![];

                // Pre-generate test data
                let test_data = Bytes::from(vec![0u8; MSG_SIZE]);

                // Spawn concurrent senders with optimized batching
                for i in 0..CONCURRENT_SENDERS {
                    let manager = manager.clone();
                    let data = test_data.clone();

                    let handle = tokio::spawn(async move {
                        let messages_per_sender = MSG_COUNT / CONCURRENT_SENDERS;
                        let peer_base = i * 1000; // Unique peer space per sender

                        for j in 0..messages_per_sender {
                            let peer = PeerId::from_bytes({
                                let mut id = [0u8; 32];
                                id[0..4].copy_from_slice(&(peer_base + j).to_le_bytes());
                                id
                            });

                            // Batch connection operations
                            if j % BATCH_SIZE == 0 {
                                let mut batch_peers = Vec::with_capacity(BATCH_SIZE);
                                for k in 0..BATCH_SIZE.min(messages_per_sender - j) {
                                    let batch_peer = PeerId::from_bytes({
                                        let mut id = [0u8; 32];
                                        id[0..4]
                                            .copy_from_slice(&(peer_base + j + k).to_le_bytes());
                                        id
                                    });
                                    batch_peers.push(batch_peer);
                                }

                                // Process batch
                                for batch_peer in batch_peers {
                                    black_box(manager.connect(batch_peer).await.unwrap());
                                    black_box(
                                        manager
                                            .update_status(batch_peer, ConnectionStatus::Connected),
                                    );
                                }
                            }
                        }
                    });
                    handles.push(handle);
                }

                // Wait for all sends to complete
                for handle in handles {
                    handle.await.unwrap();
                }
            })
        })
    });

    // Benchmark optimized connection pooling
    c.bench_function("optimized_connection_pooling", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = Arc::new(ConnectionManager::with_pool_timeout(
                    10000,
                    Duration::from_secs(300),
                ));

                let test_peers: Vec<PeerId> = (0..1000)
                    .map(|i| {
                        PeerId::from_bytes({
                            let mut id = [0u8; 32];
                            id[0..4].copy_from_slice(&i.to_le_bytes());
                            id
                        })
                    })
                    .collect();

                // Setup initial connections in parallel
                let setup_handles: Vec<_> = test_peers
                    .chunks(100)
                    .map(|chunk| {
                        let manager = manager.clone();
                        let peers = chunk.to_vec();
                        tokio::spawn(async move {
                            for peer in peers {
                                manager.connect(peer).await.unwrap();
                            }
                        })
                    })
                    .collect();

                for handle in setup_handles {
                    handle.await.unwrap();
                }

                // Test rapid connection reuse
                let mut reuse_times = Vec::new();
                for chunk in test_peers.chunks(100) {
                    let start = Instant::now();
                    for peer in chunk {
                        black_box(manager.connect(*peer).await.unwrap());
                    }
                    reuse_times.push(start.elapsed());
                }

                let avg_reuse = reuse_times.iter().sum::<Duration>() / reuse_times.len() as u32;
                black_box(avg_reuse);
            })
        })
    });

    // Benchmark zero-copy message encryption
    c.bench_function("zero_copy_encryption_benchmark", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = SecureConfig {
                    transport_keys: TransportKeys::generate(),
                    timeout: std::time::Duration::from_secs(5),
                    keepalive: std::time::Duration::from_secs(10),
                };
                let test_addr = "127.0.0.1:0".parse().unwrap();
                let server_config = ServerConfig::default();
                let (endpoint, _) =
                    Endpoint::server(server_config, "127.0.0.1:0".parse().unwrap()).unwrap();

                let connection = SecureConnection::new(&endpoint, test_addr, config)
                    .await
                    .unwrap();

                // Pre-allocate message buffers for zero-copy operations
                let mut message_buffers: Vec<Bytes> = Vec::with_capacity(10000);
                for i in 0..10000 {
                    let mut data = vec![0u8; MSG_SIZE];
                    data[0..4].copy_from_slice(&i.to_le_bytes());
                    message_buffers.push(Bytes::from(data));
                }

                let start = Instant::now();
                for data in message_buffers {
                    black_box(connection.send(data).await.unwrap());
                }
                let elapsed = start.elapsed();

                let throughput = 10000.0 / elapsed.as_secs_f64();
                let mb_per_sec = (throughput * MSG_SIZE as f64) / (1024.0 * 1024.0);
                black_box((throughput, mb_per_sec));
            })
        })
    });

    // Benchmark concurrent connection management under extreme load
    c.bench_function("extreme_load_connection_management", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = Arc::new(ConnectionManager::new(50000)); // Very high limit
                let mut handles = vec![];

                // Simulate extreme load with many concurrent workers
                for worker_id in 0..32 {
                    let manager = manager.clone();
                    let handle = tokio::spawn(async move {
                        for i in 0..1000 {
                            let peer = PeerId::from_bytes({
                                let mut id = [0u8; 32];
                                id[0..4].copy_from_slice(&(worker_id * 1000 + i).to_le_bytes());
                                id
                            });

                            // Rapid connect/update/query/disconnect cycle
                            black_box(manager.connect(peer).await.unwrap());
                            black_box(manager.update_status(peer, ConnectionStatus::Connected));
                            black_box(manager.get_status(&peer));
                            black_box(manager.update_metrics(5000.0, 10));
                            black_box(manager.disconnect(&peer));
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            })
        })
    });

    // Benchmark metrics collection performance
    c.bench_function("high_frequency_metrics_collection", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = ConnectionManager::new(1000);

                // Rapid metrics updates
                for i in 0..10000 {
                    let msgs_per_sec = 1000.0 + (i as f64 * 0.1);
                    let latency_ms = 10 + (i % 100);

                    black_box(manager.update_metrics(msgs_per_sec, latency_ms as u64));

                    if i % 100 == 0 {
                        black_box(manager.get_metrics());
                        black_box(manager.get_queue_metrics());
                        black_box(manager.get_latency_metrics());
                        black_box(manager.get_throughput_metrics());
                    }
                }
            })
        })
    });

    // Benchmark auto-recovery mechanisms
    c.bench_function("connection_auto_recovery", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = Arc::new(ConnectionManager::new(1000));

                // Setup some connections
                for i in 0..100 {
                    let peer = PeerId::from_bytes({
                        let mut id = [0u8; 32];
                        id[0..4].copy_from_slice(&i.to_le_bytes());
                        id
                    });
                    manager.connect(peer).await.unwrap();
                }

                // Simulate some connection failures
                for i in 0..50 {
                    let peer = PeerId::from_bytes({
                        let mut id = [0u8; 32];
                        id[0..4].copy_from_slice(&i.to_le_bytes());
                        id
                    });
                    manager
                        .update_status(peer, ConnectionStatus::Failed("Simulated failure".into()));
                }

                // Test recovery
                let start = Instant::now();
                let recovered = black_box(manager.auto_recover().await.unwrap());
                let recovery_time = start.elapsed();

                black_box((recovered, recovery_time));
            })
        })
    });
}

criterion_group!(
    name = optimized_benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark_optimized_throughput
);
criterion_main!(optimized_benches);
