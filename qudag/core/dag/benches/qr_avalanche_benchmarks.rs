//! Performance benchmarks for QR-Avalanche consensus algorithm.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qudag_dag::{ConsensusStatus, QRAvalanche, QRAvalancheConfig, VertexId};
use std::time::Duration;
use tokio::runtime::Runtime;

/// Create a test vertex ID
fn create_vertex_id(id: usize) -> VertexId {
    VertexId::from_bytes(format!("vertex_{}", id).into_bytes())
}

/// Setup a consensus instance with participants
fn setup_consensus_with_participants(participant_count: usize) -> QRAvalanche {
    let mut consensus = QRAvalanche::new();

    // Add participants
    for i in 0..participant_count {
        let participant_id = VertexId::from_bytes(format!("participant_{}", i).into_bytes());
        consensus.add_participant(participant_id);
    }

    consensus
}

/// Benchmark vertex processing throughput
fn bench_vertex_processing_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("vertex_processing_throughput");

    for vertex_count in [100, 1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("process_vertices", vertex_count),
            vertex_count,
            |b, &vertex_count| {
                b.iter(|| {
                    let mut consensus = setup_consensus_with_participants(50);

                    for i in 0..vertex_count {
                        let vertex_id = create_vertex_id(i);
                        black_box(consensus.process_vertex(vertex_id).unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark consensus finality latency
fn bench_consensus_finality_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("consensus_finality_latency");

    for participant_count in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("finality_latency", participant_count),
            participant_count,
            |b, &participant_count| {
                b.to_async(&rt).iter(|| async {
                    let mut consensus = setup_consensus_with_participants(participant_count);
                    let vertex_id = create_vertex_id(0);

                    // Process vertex
                    consensus.process_vertex(vertex_id.clone()).unwrap();

                    // Run consensus round to achieve finality
                    let result = consensus.run_consensus_round(&vertex_id).await;
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark vote recording performance
fn bench_vote_recording(c: &mut Criterion) {
    let mut group = c.benchmark_group("vote_recording");

    for vote_count in [100, 1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("record_votes", vote_count),
            vote_count,
            |b, &vote_count| {
                b.iter(|| {
                    let mut consensus = setup_consensus_with_participants(100);
                    let vertex_id = create_vertex_id(0);
                    consensus.process_vertex(vertex_id.clone()).unwrap();

                    for i in 0..*vote_count {
                        let voter_id = create_vertex_id(i + 1000);
                        let vote = i % 2 == 0; // Alternate between true/false
                        black_box(
                            consensus
                                .record_vote(vertex_id.clone(), voter_id, vote)
                                .unwrap(),
                        );
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark fork detection and resolution
fn bench_fork_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("fork_resolution");

    for conflict_count in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("resolve_forks", conflict_count),
            conflict_count,
            |b, &conflict_count| {
                b.iter(|| {
                    let mut consensus = setup_consensus_with_participants(100);

                    // Create conflicting vertices
                    for i in 0..*conflict_count {
                        let vertex_id = create_vertex_id(i);
                        consensus.process_vertex(vertex_id).unwrap();
                    }

                    // Detect and resolve forks
                    let resolved = consensus.detect_and_resolve_forks().unwrap();
                    black_box(resolved);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Byzantine behavior detection
fn bench_byzantine_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("byzantine_detection");

    for byzantine_count in [5, 10, 20, 30].iter() {
        group.bench_with_input(
            BenchmarkId::new("detect_byzantine", byzantine_count),
            byzantine_count,
            |b, &byzantine_count| {
                b.iter(|| {
                    let mut consensus = setup_consensus_with_participants(100);

                    // Simulate some Byzantine behavior
                    for i in 0..*byzantine_count {
                        let vertex_id = create_vertex_id(i);
                        let voter_id = create_vertex_id(i + 500);
                        consensus.process_vertex(vertex_id.clone()).unwrap();

                        // Record conflicting votes to trigger Byzantine detection
                        consensus
                            .record_vote(vertex_id.clone(), voter_id.clone(), true)
                            .unwrap();
                        // This should trigger Byzantine behavior detection
                        let _ = consensus.record_vote(vertex_id, voter_id, false);
                    }

                    // Detect Byzantine patterns
                    let detected = consensus.detect_byzantine_patterns();
                    black_box(detected);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent consensus performance
fn bench_concurrent_consensus(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_consensus");

    for concurrent_vertices in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_processing", concurrent_vertices),
            concurrent_vertices,
            |b, &concurrent_vertices| {
                b.to_async(&rt).iter(|| async {
                    let mut consensus = setup_consensus_with_participants(50);

                    // Process multiple vertices concurrently
                    let mut handles = Vec::new();

                    for i in 0..concurrent_vertices {
                        let vertex_id = create_vertex_id(i);
                        consensus.process_vertex(vertex_id.clone()).unwrap();

                        let mut consensus_clone = setup_consensus_with_participants(50);
                        let handle = tokio::spawn(async move {
                            consensus_clone.run_consensus_round(&vertex_id).await
                        });
                        handles.push(handle);
                    }

                    // Wait for all to complete
                    for handle in handles {
                        let result = handle.await.unwrap();
                        black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different configuration parameters
fn bench_config_variations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("config_variations");

    let configs = vec![
        (
            "conservative",
            QRAvalancheConfig {
                beta: 0.9,
                alpha: 0.7,
                query_sample_size: 30,
                max_rounds: 150,
                finality_threshold: 0.95,
                round_timeout: Duration::from_millis(200),
            },
        ),
        ("balanced", QRAvalancheConfig::default()),
        (
            "aggressive",
            QRAvalancheConfig {
                beta: 0.7,
                alpha: 0.5,
                query_sample_size: 15,
                max_rounds: 50,
                finality_threshold: 0.8,
                round_timeout: Duration::from_millis(50),
            },
        ),
    ];

    for (config_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("config", config_name),
            &config,
            |b, config| {
                b.to_async(&rt).iter(|| async {
                    let mut consensus = QRAvalanche::with_config(config.clone());

                    // Add participants
                    for i in 0..50 {
                        let participant_id = create_vertex_id(i + 1000);
                        consensus.add_participant(participant_id);
                    }

                    let vertex_id = create_vertex_id(0);
                    consensus.process_vertex(vertex_id.clone()).unwrap();

                    let result = consensus.run_consensus_round(&vertex_id).await;
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage and scalability
fn bench_memory_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_scalability");

    for vertex_count in [1000, 5000, 10000, 20000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", vertex_count),
            vertex_count,
            |b, &vertex_count| {
                b.iter(|| {
                    let mut consensus = setup_consensus_with_participants(100);

                    // Add many vertices to test memory usage
                    for i in 0..vertex_count {
                        let vertex_id = create_vertex_id(i);
                        consensus.process_vertex(vertex_id.clone()).unwrap();

                        // Add some votes
                        for j in 0..5 {
                            let voter_id = create_vertex_id(vertex_count + j);
                            let vote = (i + j) % 2 == 0;
                            let _ = consensus.record_vote(vertex_id.clone(), voter_id, vote);
                        }
                    }

                    // Test fork resolution with many vertices
                    let resolved = consensus.detect_and_resolve_forks().unwrap();
                    black_box(resolved);

                    // Get metrics
                    let metrics = consensus.get_metrics();
                    black_box(metrics);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark performance under Byzantine conditions
fn bench_byzantine_resilience(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("byzantine_resilience");

    for byzantine_ratio in [0.1, 0.2, 0.3].iter() {
        group.bench_with_input(
            BenchmarkId::new("byzantine_ratio", format!("{:.1}", byzantine_ratio)),
            byzantine_ratio,
            |b, &byzantine_ratio| {
                b.to_async(&rt).iter(|| async {
                    let participant_count = 100;
                    let byzantine_count = (participant_count as f64 * byzantine_ratio) as usize;

                    let mut consensus = setup_consensus_with_participants(participant_count);

                    // Mark some participants as Byzantine
                    for i in 0..byzantine_count {
                        let byzantine_id = create_vertex_id(i);
                        consensus
                            .voting_record
                            .byzantine_voters
                            .insert(byzantine_id);
                    }

                    let vertex_id = create_vertex_id(1000);
                    consensus.process_vertex(vertex_id.clone()).unwrap();

                    let result = consensus.run_consensus_round(&vertex_id).await;
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_vertex_processing_throughput,
    bench_consensus_finality_latency,
    bench_vote_recording,
    bench_fork_resolution,
    bench_byzantine_detection,
    bench_concurrent_consensus,
    bench_config_variations,
    bench_memory_scalability,
    bench_byzantine_resilience
);

criterion_main!(benches);
