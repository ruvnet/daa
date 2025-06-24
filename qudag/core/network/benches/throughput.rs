use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qudag_network::connection::{ConnectionManager, SecureConfig, SecureConnection, TransportKeys};
use qudag_network::types::{ConnectionStatus, MessagePriority, NetworkMessage, PeerId};
use quinn::{Endpoint, ServerConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

fn benchmark_message_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Test configuration
    const MSG_COUNT: usize = 100_000; // Test with 100K messages
    const BATCH_SIZE: usize = 1024;
    const CONCURRENT_SENDERS: usize = 8;
    const MSG_SIZE: usize = 1024; // 1KB messages

    // Benchmark high-throughput message processing
    c.bench_function("message_throughput_100k", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = ConnectionManager::new(1000);
                let mut handles = vec![];

                // Pre-generate test data
                let test_data = Bytes::from(vec![0u8; MSG_SIZE]);

                // Spawn concurrent senders
                for i in 0..CONCURRENT_SENDERS {
                    let manager = Arc::new(manager.clone());
                    let data = test_data.clone();

                    let handle = tokio::spawn(async move {
                        let messages_per_sender = MSG_COUNT / CONCURRENT_SENDERS;

                        for j in 0..messages_per_sender {
                            let peer = PeerId::random();
                            black_box(manager.connect(peer).await.unwrap());
                            black_box(manager.update_status(peer, ConnectionStatus::Connected));
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

    // Benchmark batched anonymous routing
    c.bench_function("message_routing_anonymous_batched", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = ConnectionManager::new(100);

                let peer = PeerId::random();
                black_box(manager.connect(peer).await.unwrap());
                black_box(manager.update_status(peer, ConnectionStatus::Connected));
                black_box(manager.get_status(&peer));
            })
        })
    });
}

fn benchmark_connection_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Benchmark connection management under load
    c.bench_function("connection_management_high_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = Arc::new(ConnectionManager::new(1000));
                let mut handles = vec![];

                // Simulate high-load connection management
                for _ in 0..100 {
                    let manager = manager.clone();
                    let handle = tokio::spawn(async move {
                        for _ in 0..100 {
                            let peer = PeerId::random();
                            black_box(manager.connect(peer).await.unwrap());
                            black_box(manager.update_status(peer, ConnectionStatus::Connected));
                            black_box(manager.get_status(&peer));
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
}

fn benchmark_encryption_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    const MSG_SIZE: usize = 1024;

    // Benchmark encryption performance
    c.bench_function("message_encryption_throughput", |b| {
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
                let data = Bytes::from(vec![0u8; MSG_SIZE]);

                for _ in 0..1000 {
                    black_box(connection.send(data.clone()).await.unwrap());
                }
            })
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark_message_throughput, benchmark_connection_management, benchmark_encryption_performance
);
criterion_main!(benches);
