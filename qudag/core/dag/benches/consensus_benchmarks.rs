use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_dag::consensus::DagConsensus;
use qudag_dag::graph::Graph;
use rand::Rng;

fn consensus_round_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus");

    // Test different network sizes
    for size in [10, 50, 100].iter() {
        group.bench_function(format!("consensus_round_{}_nodes", size), |b| {
            let mut rng = rand::thread_rng();
            let mut graph = Graph::new();
            let mut consensus = DagConsensus::new();

            // Add random vertices and edges
            for i in 0..*size {
                graph.add_vertex(i);
                if i > 0 {
                    graph.add_edge(rng.gen_range(0..i), i);
                }
            }

            b.iter(|| {
                consensus.process_round(black_box(&mut graph));
            });
        });
    }
    group.finish();
}

criterion_group!(benches, consensus_round_benchmark);
criterion_main!(benches);
