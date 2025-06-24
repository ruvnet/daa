#![deny(unsafe_code)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_network::{
    connection::ConnectionManager, message::MessageQueue, prelude::*, routing::Router,
};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

fn benchmark_message_queue(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.benchmark_group("message_queue")
        .throughput(criterion::Throughput::Elements(10000))
        .bench_function("enqueue_dequeue", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let (queue, _rx) = MessageQueue::new();

                    // Enqueue messages
                    for i in 0..10000 {
                        let msg = NetworkMessage {
                            id: i.to_string(),
                            source: vec![1],
                            destination: vec![2],
                            payload: vec![0; 1000],
                            priority: MessagePriority::Normal,
                            ttl: Duration::from_secs(60),
                        };
                        queue.enqueue(msg).await.unwrap();
                    }

                    // Dequeue messages
                    for _ in 0..10000 {
                        black_box(queue.dequeue().await);
                    }
                });
            });
        });
}

fn benchmark_routing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.benchmark_group("routing")
        .throughput(criterion::Throughput::Elements(1000))
        .bench_function("anonymous_routing", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let router = Router::new();

                    // Add test peers
                    for _ in 0..100 {
                        router.add_peer(libp2p::PeerId::random()).await;
                    }

                    let msg = NetworkMessage {
                        id: "test".into(),
                        source: vec![1],
                        destination: vec![2],
                        payload: vec![0; 1000],
                        priority: MessagePriority::Normal,
                        ttl: Duration::from_secs(60),
                    };

                    // Route messages
                    for _ in 0..1000 {
                        black_box(
                            router
                                .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
                                .await,
                        );
                    }
                });
            });
        });
}

fn benchmark_connection_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.benchmark_group("connection_manager")
        .throughput(criterion::Throughput::Elements(1000))
        .bench_function("connection_operations", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let manager = ConnectionManager::new(1000);
                    let peers: Vec<_> = (0..1000).map(|_| libp2p::PeerId::random()).collect();

                    // Benchmark connection operations
                    for peer in peers.iter() {
                        black_box(manager.connect(*peer).await);
                        black_box(
                            manager
                                .update_status(*peer, ConnectionStatus::Connected)
                                .await,
                        );
                        black_box(manager.get_status(peer).await);
                    }

                    for peer in peers.iter() {
                        black_box(manager.disconnect(peer).await);
                    }
                });
            });
        });
}

criterion_group!(
    benches,
    benchmark_message_queue,
    benchmark_routing,
    benchmark_connection_manager
);
criterion_main!(benches);
