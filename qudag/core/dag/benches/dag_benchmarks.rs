use blake3::Hash;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_dag::{Edge, Graph, Node};

fn create_test_node(data: &[u8], parents: Vec<Hash>) -> Node {
    Node::new(data.to_vec(), parents)
}

fn bench_node_creation(c: &mut Criterion) {
    c.bench_function("node_creation", |b| {
        b.iter(|| create_test_node(black_box(&[1, 2, 3]), vec![]));
    });
}

fn bench_node_addition(c: &mut Criterion) {
    let graph = Graph::new();
    let node = create_test_node(&[1], vec![]);
    let node_hash = *node.hash();
    graph.add_node(node).unwrap();

    c.bench_function("node_addition", |b| {
        b.iter(|| {
            let child = create_test_node(black_box(&[2]), vec![node_hash]);
            graph.add_node(child.clone()).unwrap_or(());
        });
    });
}

fn bench_cycle_detection(c: &mut Criterion) {
    let graph = Graph::new();

    // Create a chain of 100 nodes
    let mut prev_hash = None;
    for i in 0..100 {
        let parents = prev_hash.map(|h| vec![h]).unwrap_or_default();
        let node = create_test_node(&[i as u8], parents);
        let hash = *node.hash();
        graph.add_node(node).unwrap();
        prev_hash = Some(hash);
    }

    c.bench_function("cycle_detection", |b| {
        b.iter(|| {
            if let Some(hash) = prev_hash {
                let node = create_test_node(black_box(&[255]), vec![hash]);
                graph.add_node(node.clone()).unwrap_or(());
            }
        });
    });
}

fn bench_large_dag_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_dag");
    group.sample_size(10);

    let graph = Graph::new();
    let mut nodes = Vec::new();

    // Create initial DAG with 1000 nodes
    for i in 0..1000 {
        let parents = if i > 0 {
            vec![*nodes[i - 1].hash()]
        } else {
            vec![]
        };
        let node = create_test_node(&[i as u8], parents);
        nodes.push(node.clone());
        graph.add_node(node).unwrap();
    }

    // Benchmark state updates
    group.bench_function("state_updates", |b| {
        b.iter(|| {
            for node in nodes.iter().take(100) {
                graph
                    .update_node_state(node.hash(), crate::node::NodeState::Verified)
                    .unwrap_or(());
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_node_creation,
    bench_node_addition,
    bench_cycle_detection,
    bench_large_dag_operations
);
criterion_main!(benches);
