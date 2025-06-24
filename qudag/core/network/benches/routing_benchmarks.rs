use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qudag_network::{MessagePriority, NetworkMessage, PeerId, Router, RoutingStrategy};
use std::time::Duration;

async fn setup_test_network(size: usize) -> Router {
    let router = Router::new();

    // Add test peers
    for _ in 0..size {
        router.add_peer(PeerId::random()).await;
    }

    router
}

fn bench_route_computation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("route_computation");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("anonymous_route", size),
            size,
            |b, &size| {
                let router = rt.block_on(setup_test_network(size));
                let msg = NetworkMessage {
                    id: "test".into(),
                    source: PeerId::random().to_bytes().to_vec(),
                    destination: PeerId::random().to_bytes().to_vec(),
                    payload: vec![0; 32],
                    priority: MessagePriority::Normal,
                    ttl: Duration::from_secs(60),
                };

                b.to_async(&rt).iter(|| async {
                    black_box(
                        router
                            .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
                            .await
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

fn bench_circuit_setup(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("circuit_setup");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("setup", size), size, |b, &size| {
            let router = rt.block_on(setup_test_network(size));
            let source = PeerId::random();
            let dest = PeerId::random();

            b.to_async(&rt).iter(|| async {
                black_box(
                    router
                        .setup_circuit(
                            &source, &dest, 3, // hops
                        )
                        .await
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_message_routing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("message_routing");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("route_message", size), size, |b, &size| {
            let router = rt.block_on(setup_test_network(size));
            let msg = NetworkMessage {
                id: "test".into(),
                source: PeerId::random().to_bytes().to_vec(),
                destination: PeerId::random().to_bytes().to_vec(),
                payload: vec![0; 1024], // 1KB message
                priority: MessagePriority::Normal,
                ttl: Duration::from_secs(60),
            };

            b.to_async(&rt)
                .iter(|| async { black_box(router.send_message(msg.clone()).await.unwrap()) });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_route_computation,
    bench_circuit_setup,
    bench_message_routing
);
criterion_main!(benches);
