use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_protocol::{Coordinator, ProtocolConfig};
use tokio::runtime::Runtime;

fn bench_message_propagation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("message_propagation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = ProtocolConfig::default();
                let mut coordinator = Coordinator::new(config).await.unwrap();
                coordinator.start().await.unwrap();

                let message = black_box(vec![1, 2, 3, 4]);
                coordinator.broadcast_message(message).await.unwrap();

                coordinator.stop().await.unwrap();
            });
        })
    });
}

fn bench_node_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("node_initialization", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = ProtocolConfig::default();
                let coordinator = Coordinator::new(config).await.unwrap();
                assert!(coordinator.is_initialized());
            });
        })
    });
}

fn bench_multi_node_broadcast(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("multi_node_broadcast", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create 5 nodes
                let mut nodes = Vec::with_capacity(5);
                for i in 0..5 {
                    let mut config = ProtocolConfig::default();
                    config.network_port = 8000 + i as u16;
                    let node = Coordinator::new(config).await.unwrap();
                    nodes.push(node);
                }

                // Start all nodes
                for node in nodes.iter_mut() {
                    node.start().await.unwrap();
                }

                // Broadcast message from first node
                let message = black_box(vec![1, 2, 3, 4]);
                nodes[0].broadcast_message(message).await.unwrap();

                // Stop all nodes
                for node in nodes.iter_mut() {
                    node.stop().await.unwrap();
                }
            });
        })
    });
}

criterion_group!(
    benches,
    bench_message_propagation,
    bench_node_initialization,
    bench_multi_node_broadcast
);
criterion_main!(benches);
