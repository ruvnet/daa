//! Consensus Finalization Time Benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio::runtime::Runtime;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use rand::prelude::*;

// Simulated consensus structures (since we don't have access to daa-chain directly)
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub validator_id: String,
    pub public_key: Vec<u8>,
    pub stake: u64,
    pub reputation: f64,
    pub last_seen: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct Vote {
    pub validator_id: String,
    pub epoch: u64,
    pub round: u64,
    pub block_hash: [u8; 32],
    pub vote_type: VoteType,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum VoteType {
    Proposal,
    Prevote,
    Precommit,
    Commit,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub hash: [u8; 32],
    pub height: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: [u8; 32],
    pub data: Vec<u8>,
}

/// Benchmark consensus finalization with different validator set sizes
fn benchmark_consensus_finalization_by_validator_count(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("consensus_finalization_validator_count");
    group.measurement_time(Duration::from_secs(30));
    
    // Test different validator set sizes
    let validator_counts = vec![4, 7, 13, 25, 50, 100];
    
    for validator_count in validator_counts {
        group.throughput(Throughput::Elements(validator_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("validators", validator_count),
            &validator_count,
            |b, &validator_count| {
                b.to_async(&rt).iter(|| async move {
                    let validators = create_validator_set(validator_count);
                    let block = create_test_block(100); // 100 transactions
                    
                    // Measure time to achieve consensus
                    let start = std::time::Instant::now();
                    
                    // Simulate consensus rounds
                    let finalization_time = simulate_consensus_rounds(&validators, &block).await;
                    
                    black_box(finalization_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark consensus finalization with different transaction loads
fn benchmark_consensus_finalization_by_tx_count(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("consensus_finalization_tx_count");
    group.measurement_time(Duration::from_secs(20));
    
    let validator_count = 13; // Fixed validator set
    let tx_counts = vec![10, 50, 100, 500, 1000, 5000];
    
    for tx_count in tx_counts {
        group.throughput(Throughput::Elements(tx_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("transactions", tx_count),
            &tx_count,
            |b, &tx_count| {
                b.to_async(&rt).iter(|| async move {
                    let validators = create_validator_set(validator_count);
                    let block = create_test_block(tx_count);
                    
                    let start = std::time::Instant::now();
                    let finalization_time = simulate_consensus_rounds(&validators, &block).await;
                    black_box(finalization_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark consensus under different network conditions
fn benchmark_consensus_network_conditions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("consensus_network_conditions");
    group.measurement_time(Duration::from_secs(25));
    
    let scenarios = vec![
        ("ideal", 0.0, 5),         // 0% loss, 5ms latency
        ("good", 0.01, 50),        // 1% loss, 50ms latency
        ("moderate", 0.05, 100),   // 5% loss, 100ms latency
        ("poor", 0.1, 500),        // 10% loss, 500ms latency
    ];
    
    for (name, packet_loss, latency_ms) in scenarios {
        group.bench_with_input(
            BenchmarkId::new("network", name),
            &(packet_loss, latency_ms),
            |b, &(packet_loss, latency_ms)| {
                b.to_async(&rt).iter(|| async move {
                    let validators = create_validator_set(13);
                    let block = create_test_block(100);
                    
                    let start = std::time::Instant::now();
                    let finalization_time = simulate_consensus_with_network_conditions(
                        &validators, &block, packet_loss, latency_ms
                    ).await;
                    black_box(finalization_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark consensus with byzantine validators
fn benchmark_consensus_byzantine_tolerance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("consensus_byzantine_tolerance");
    group.measurement_time(Duration::from_secs(30));
    
    let validator_count = 25;
    let byzantine_percentages = vec![0, 10, 20, 33]; // Up to 33% byzantine (theoretical maximum)
    
    for byzantine_pct in byzantine_percentages {
        group.bench_with_input(
            BenchmarkId::new("byzantine_pct", byzantine_pct),
            &byzantine_pct,
            |b, &byzantine_pct| {
                b.to_async(&rt).iter(|| async move {
                    let validators = create_validator_set_with_byzantine(validator_count, byzantine_pct);
                    let block = create_test_block(100);
                    
                    let start = std::time::Instant::now();
                    let finalization_time = simulate_byzantine_consensus(&validators, &block).await;
                    black_box(finalization_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark consensus round duration vs finalization time tradeoff
fn benchmark_consensus_round_timing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("consensus_round_timing");
    group.measurement_time(Duration::from_secs(20));
    
    // Different round durations in milliseconds
    let round_durations = vec![1000, 3000, 5000, 10000, 15000]; // 1s to 15s
    
    for round_duration_ms in round_durations {
        group.bench_with_input(
            BenchmarkId::new("round_duration_ms", round_duration_ms),
            &round_duration_ms,
            |b, &round_duration_ms| {
                b.to_async(&rt).iter(|| async move {
                    let validators = create_validator_set(13);
                    let block = create_test_block(100);
                    
                    let start = std::time::Instant::now();
                    let finalization_time = simulate_consensus_with_round_timing(
                        &validators, &block, Duration::from_millis(round_duration_ms)
                    ).await;
                    black_box(finalization_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark parallel consensus for multiple blocks
fn benchmark_parallel_consensus(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_consensus");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);
    
    let block_counts = vec![1, 2, 4, 8, 16];
    
    for block_count in block_counts {
        group.throughput(Throughput::Elements(block_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("parallel_blocks", block_count),
            &block_count,
            |b, &block_count| {
                b.to_async(&rt).iter(|| async move {
                    let validators = create_validator_set(13);
                    let blocks: Vec<Block> = (0..block_count)
                        .map(|i| create_test_block_with_height(100, i as u64))
                        .collect();
                    
                    let start = std::time::Instant::now();
                    
                    // Run consensus for all blocks in parallel
                    let handles: Vec<_> = blocks.into_iter().map(|block| {
                        let validators = validators.clone();
                        tokio::spawn(async move {
                            simulate_consensus_rounds(&validators, &block).await
                        })
                    }).collect();
                    
                    // Wait for all to complete
                    for handle in handles {
                        handle.await.unwrap();
                    }
                    
                    black_box(start.elapsed())
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions for simulation

fn create_validator_set(count: usize) -> Vec<ValidatorInfo> {
    let mut validators = Vec::new();
    let mut rng = rand::thread_rng();
    
    for i in 0..count {
        validators.push(ValidatorInfo {
            validator_id: format!("validator-{}", i),
            public_key: (0..32).map(|_| rng.gen()).collect(),
            stake: rng.gen_range(1000..10000),
            reputation: rng.gen_range(0.8..1.0),
            last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            is_active: true,
        });
    }
    
    validators
}

fn create_validator_set_with_byzantine(total_count: usize, byzantine_pct: u32) -> Vec<ValidatorInfo> {
    let mut validators = create_validator_set(total_count);
    let byzantine_count = (total_count * byzantine_pct as usize) / 100;
    
    // Mark some validators as byzantine (lower reputation)
    for i in 0..byzantine_count {
        validators[i].reputation = 0.1;
    }
    
    validators
}

fn create_test_block(tx_count: usize) -> Block {
    let mut rng = rand::thread_rng();
    let mut transactions = Vec::new();
    
    for i in 0..tx_count {
        transactions.push(Transaction {
            hash: [rng.gen(); 32],
            data: (0..rng.gen_range(100..1000)).map(|_| rng.gen()).collect(),
        });
    }
    
    Block {
        hash: [rng.gen(); 32],
        height: 1,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        transactions,
    }
}

fn create_test_block_with_height(tx_count: usize, height: u64) -> Block {
    let mut block = create_test_block(tx_count);
    block.height = height;
    block
}

async fn simulate_consensus_rounds(validators: &[ValidatorInfo], block: &Block) -> Duration {
    let start = std::time::Instant::now();
    
    // Simulate the typical consensus phases: Proposal -> Prevote -> Precommit -> Commit
    
    // Phase 1: Proposal (leader proposes block)
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Phase 2: Prevote (validators vote on proposal)
    let prevote_time = simulate_voting_phase(validators, VoteType::Prevote).await;
    
    // Phase 3: Precommit (validators commit to the block)
    let precommit_time = simulate_voting_phase(validators, VoteType::Precommit).await;
    
    // Phase 4: Commit (final commitment)
    let commit_time = simulate_voting_phase(validators, VoteType::Commit).await;
    
    // Total consensus time
    start.elapsed()
}

async fn simulate_voting_phase(validators: &[ValidatorInfo], vote_type: VoteType) -> Duration {
    let start = std::time::Instant::now();
    
    // Simulate network delay for vote collection
    let network_delay = Duration::from_millis(rand::thread_rng().gen_range(50..200));
    tokio::time::sleep(network_delay).await;
    
    // Simulate vote processing time (scales with validator count)
    let processing_time = Duration::from_millis(validators.len() as u64 * 2);
    tokio::time::sleep(processing_time).await;
    
    start.elapsed()
}

async fn simulate_consensus_with_network_conditions(
    validators: &[ValidatorInfo],
    block: &Block,
    packet_loss: f64,
    latency_ms: u64,
) -> Duration {
    let start = std::time::Instant::now();
    
    // Simulate additional delays due to network conditions
    let base_delay = Duration::from_millis(latency_ms);
    
    // Proposal phase with network delay
    tokio::time::sleep(base_delay).await;
    
    // Each voting phase is affected by network conditions
    for vote_type in [VoteType::Prevote, VoteType::Precommit, VoteType::Commit] {
        // Network delay
        tokio::time::sleep(base_delay).await;
        
        // Packet loss simulation (retransmissions)
        if rand::thread_rng().gen::<f64>() < packet_loss {
            // Simulate retransmission delay
            tokio::time::sleep(Duration::from_millis(latency_ms * 2)).await;
        }
        
        simulate_voting_phase(validators, vote_type).await;
    }
    
    start.elapsed()
}

async fn simulate_byzantine_consensus(validators: &[ValidatorInfo], block: &Block) -> Duration {
    let start = std::time::Instant::now();
    
    // Byzantine consensus typically requires more rounds
    let byzantine_count = validators.iter().filter(|v| v.reputation < 0.5).count();
    let extra_rounds = (byzantine_count / 3).max(1);
    
    // Simulate additional consensus rounds due to byzantine behavior
    for _ in 0..extra_rounds {
        simulate_consensus_rounds(validators, block).await;
    }
    
    start.elapsed()
}

async fn simulate_consensus_with_round_timing(
    validators: &[ValidatorInfo],
    block: &Block,
    round_duration: Duration,
) -> Duration {
    let start = std::time::Instant::now();
    
    // Each phase takes a portion of the round duration
    let phase_duration = round_duration / 4;
    
    // Proposal phase
    tokio::time::sleep(phase_duration).await;
    
    // Voting phases
    for _ in 0..3 {
        tokio::time::sleep(phase_duration).await;
    }
    
    start.elapsed()
}

criterion_group!(
    benches,
    benchmark_consensus_finalization_by_validator_count,
    benchmark_consensus_finalization_by_tx_count,
    benchmark_consensus_network_conditions,
    benchmark_consensus_byzantine_tolerance,
    benchmark_consensus_round_timing,
    benchmark_parallel_consensus
);
criterion_main!(benches);