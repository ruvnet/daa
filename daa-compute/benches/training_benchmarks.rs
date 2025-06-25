//! Training Convergence Speed Benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio::runtime::Runtime;
use std::time::Duration;
use std::sync::Arc;
use rand::prelude::*;

/// Represents a simple neural network model for benchmarking
#[derive(Clone)]
pub struct BenchmarkModel {
    pub weights: Vec<f32>,
    pub biases: Vec<f32>,
    pub layers: Vec<usize>,
    pub learning_rate: f32,
}

impl BenchmarkModel {
    pub fn new(layers: Vec<usize>, learning_rate: f32) -> Self {
        let mut rng = rand::thread_rng();
        let total_weights: usize = layers.windows(2).map(|pair| pair[0] * pair[1]).sum();
        let total_biases: usize = layers[1..].iter().sum();
        
        Self {
            weights: (0..total_weights).map(|_| rng.gen_range(-0.1..0.1)).collect(),
            biases: (0..total_biases).map(|_| rng.gen_range(-0.1..0.1)).collect(),
            layers,
            learning_rate,
        }
    }
    
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut activations = input.to_vec();
        let mut weight_idx = 0;
        let mut bias_idx = 0;
        
        for layer_idx in 0..self.layers.len() - 1 {
            let input_size = self.layers[layer_idx];
            let output_size = self.layers[layer_idx + 1];
            let mut next_activations = vec![0.0; output_size];
            
            // Matrix multiplication
            for o in 0..output_size {
                for i in 0..input_size {
                    next_activations[o] += self.weights[weight_idx] * activations[i];
                    weight_idx += 1;
                }
                next_activations[o] += self.biases[bias_idx];
                bias_idx += 1;
                
                // ReLU activation
                next_activations[o] = next_activations[o].max(0.0);
            }
            
            activations = next_activations;
        }
        
        activations
    }
    
    pub fn compute_gradients(&self, input: &[f32], target: &[f32]) -> (Vec<f32>, Vec<f32>) {
        // Simplified gradient computation for benchmarking
        let output = self.forward(input);
        let loss: f32 = output.iter().zip(target).map(|(o, t)| (o - t).powi(2)).sum();
        
        // Mock gradient computation
        let weight_gradients: Vec<f32> = self.weights.iter()
            .map(|w| loss * 0.001 * w.signum())
            .collect();
        let bias_gradients: Vec<f32> = self.biases.iter()
            .map(|b| loss * 0.001 * b.signum())
            .collect();
        
        (weight_gradients, bias_gradients)
    }
    
    pub fn apply_gradients(&mut self, weight_grads: &[f32], bias_grads: &[f32]) {
        for (w, g) in self.weights.iter_mut().zip(weight_grads) {
            *w -= self.learning_rate * g;
        }
        for (b, g) in self.biases.iter_mut().zip(bias_grads) {
            *b -= self.learning_rate * g;
        }
    }
}

/// Benchmark training convergence with different model sizes
fn benchmark_training_convergence_model_size(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("training_convergence_model_size");
    group.measurement_time(Duration::from_secs(30));
    
    let model_configs = vec![
        ("small", vec![784, 128, 10]),          // Small model
        ("medium", vec![784, 512, 256, 10]),    // Medium model
        ("large", vec![784, 1024, 512, 256, 10]), // Large model
        ("xlarge", vec![784, 2048, 1024, 512, 10]), // Extra large model
    ];
    
    for (name, layers) in model_configs {
        let param_count: usize = layers.windows(2).map(|pair| pair[0] * pair[1]).sum::<usize>() + layers[1..].iter().sum::<usize>();
        group.throughput(Throughput::Elements(param_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("model_size", name),
            &layers,
            |b, layers| {
                b.to_async(&rt).iter(|| async move {
                    let mut model = BenchmarkModel::new(layers.clone(), 0.001);
                    let dataset = generate_training_data(1000, layers[0], layers.last().unwrap().clone());
                    
                    let start = std::time::Instant::now();
                    let convergence_time = train_to_convergence(&mut model, &dataset, 0.01).await;
                    black_box(convergence_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark distributed training vs single node training
fn benchmark_distributed_vs_single_training(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("distributed_vs_single_training");
    group.measurement_time(Duration::from_secs(45));
    
    let node_counts = vec![1, 2, 4, 8, 16];
    let model_layers = vec![784, 512, 256, 10];
    
    for node_count in node_counts {
        group.throughput(Throughput::Elements(node_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("nodes", node_count),
            &node_count,
            |b, &node_count| {
                b.to_async(&rt).iter(|| async move {
                    let dataset = generate_training_data(5000, 784, 10);
                    
                    let start = std::time::Instant::now();
                    let convergence_time = if node_count == 1 {
                        train_single_node(&model_layers, &dataset).await
                    } else {
                        train_distributed(&model_layers, &dataset, node_count).await
                    };
                    black_box(convergence_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark training with different batch sizes
fn benchmark_training_batch_sizes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("training_batch_sizes");
    group.measurement_time(Duration::from_secs(25));
    
    let batch_sizes = vec![32, 64, 128, 256, 512];
    let model_layers = vec![784, 256, 10];
    
    for batch_size in batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("batch_size", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async move {
                    let mut model = BenchmarkModel::new(model_layers.clone(), 0.001);
                    let dataset = generate_training_data(2000, 784, 10);
                    
                    let start = std::time::Instant::now();
                    let convergence_time = train_with_batch_size(&mut model, &dataset, batch_size).await;
                    black_box(convergence_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark gradient synchronization frequency impact
fn benchmark_gradient_sync_frequency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("gradient_sync_frequency");
    group.measurement_time(Duration::from_secs(30));
    
    // Sync frequencies: every N batches
    let sync_frequencies = vec![1, 5, 10, 50, 100];
    let model_layers = vec![784, 256, 10];
    let node_count = 8;
    
    for sync_freq in sync_frequencies {
        group.bench_with_input(
            BenchmarkId::new("sync_every_n_batches", sync_freq),
            &sync_freq,
            |b, &sync_freq| {
                b.to_async(&rt).iter(|| async move {
                    let dataset = generate_training_data(2000, 784, 10);
                    
                    let start = std::time::Instant::now();
                    let convergence_time = train_with_sync_frequency(
                        &model_layers, &dataset, node_count, sync_freq
                    ).await;
                    black_box(convergence_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark training with different learning rates
fn benchmark_learning_rate_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("learning_rate_impact");
    group.measurement_time(Duration::from_secs(20));
    
    let learning_rates = vec![0.0001, 0.001, 0.01, 0.1, 0.5];
    let model_layers = vec![784, 256, 10];
    
    for lr in learning_rates {
        group.bench_with_input(
            BenchmarkId::new("learning_rate", (lr * 10000.0) as u32),
            &lr,
            |b, &lr| {
                b.to_async(&rt).iter(|| async move {
                    let mut model = BenchmarkModel::new(model_layers.clone(), lr);
                    let dataset = generate_training_data(1000, 784, 10);
                    
                    let start = std::time::Instant::now();
                    let convergence_time = train_to_convergence(&mut model, &dataset, 0.01).await;
                    black_box(convergence_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark communication overhead in distributed training
fn benchmark_communication_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("communication_overhead");
    group.measurement_time(Duration::from_secs(35));
    
    let model_layers = vec![784, 512, 256, 10];
    let gradient_size: usize = model_layers.windows(2).map(|pair| pair[0] * pair[1]).sum::<usize>() + model_layers[1..].iter().sum::<usize>();
    
    // Different communication patterns
    let comm_patterns = vec![
        ("all_reduce", 8),
        ("parameter_server", 8),
        ("ring_all_reduce", 8),
        ("hierarchical", 16),
    ];
    
    for (pattern, node_count) in comm_patterns {
        group.throughput(Throughput::Bytes((gradient_size * 4 * node_count) as u64)); // 4 bytes per f32
        
        group.bench_with_input(
            BenchmarkId::new("pattern", pattern),
            &(pattern, node_count),
            |b, &(pattern, node_count)| {
                b.to_async(&rt).iter(|| async move {
                    let gradients = vec![0.1f32; gradient_size];
                    
                    let start = std::time::Instant::now();
                    let comm_time = simulate_gradient_communication(pattern, &gradients, node_count).await;
                    black_box(comm_time)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark federated learning convergence
fn benchmark_federated_learning_convergence(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("federated_learning_convergence");
    group.measurement_time(Duration::from_secs(40));
    group.sample_size(10);
    
    let model_layers = vec![784, 128, 10];
    let client_counts = vec![10, 50, 100, 500];
    
    for client_count in client_counts {
        group.throughput(Throughput::Elements(client_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("clients", client_count),
            &client_count,
            |b, &client_count| {
                b.to_async(&rt).iter(|| async move {
                    // Each client has a smaller local dataset
                    let local_dataset_size = 100;
                    
                    let start = std::time::Instant::now();
                    let convergence_time = simulate_federated_learning(
                        &model_layers, client_count, local_dataset_size
                    ).await;
                    black_box(convergence_time)
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions for training simulation

fn generate_training_data(samples: usize, input_size: usize, output_size: usize) -> Vec<(Vec<f32>, Vec<f32>)> {
    let mut rng = rand::thread_rng();
    let mut dataset = Vec::new();
    
    for _ in 0..samples {
        let input: Vec<f32> = (0..input_size).map(|_| rng.gen_range(0.0..1.0)).collect();
        let mut target = vec![0.0; output_size];
        target[rng.gen_range(0..output_size)] = 1.0; // One-hot encoding
        dataset.push((input, target));
    }
    
    dataset
}

async fn train_to_convergence(
    model: &mut BenchmarkModel,
    dataset: &[(Vec<f32>, Vec<f32>)],
    target_loss: f32,
) -> Duration {
    let start = std::time::Instant::now();
    let mut epoch = 0;
    let max_epochs = 1000;
    
    while epoch < max_epochs {
        let mut total_loss = 0.0;
        
        for (input, target) in dataset {
            let output = model.forward(input);
            let loss: f32 = output.iter().zip(target).map(|(o, t)| (o - t).powi(2)).sum();
            total_loss += loss;
            
            let (weight_grads, bias_grads) = model.compute_gradients(input, target);
            model.apply_gradients(&weight_grads, &bias_grads);
        }
        
        let avg_loss = total_loss / dataset.len() as f32;
        if avg_loss < target_loss {
            break;
        }
        
        epoch += 1;
        
        // Simulate some computation time
        if epoch % 10 == 0 {
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
    }
    
    start.elapsed()
}

async fn train_single_node(layers: &[usize], dataset: &[(Vec<f32>, Vec<f32>)]) -> Duration {
    let mut model = BenchmarkModel::new(layers.to_vec(), 0.001);
    train_to_convergence(&mut model, dataset, 0.01).await
}

async fn train_distributed(layers: &[usize], dataset: &[(Vec<f32>, Vec<f32>)], node_count: usize) -> Duration {
    let start = std::time::Instant::now();
    
    // Split dataset among nodes
    let chunk_size = dataset.len() / node_count;
    let chunks: Vec<_> = dataset.chunks(chunk_size).collect();
    
    // Simulate distributed training
    let mut handles = Vec::new();
    
    for chunk in chunks {
        let chunk = chunk.to_vec();
        let layers = layers.to_vec();
        
        let handle = tokio::spawn(async move {
            let mut model = BenchmarkModel::new(layers, 0.001);
            // Train on local data
            for _ in 0..10 { // 10 local epochs
                for (input, target) in &chunk {
                    let (weight_grads, bias_grads) = model.compute_gradients(input, target);
                    model.apply_gradients(&weight_grads, &bias_grads);
                }
            }
            model
        });
        handles.push(handle);
    }
    
    // Wait for all nodes and aggregate
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Simulate gradient aggregation time
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    start.elapsed()
}

async fn train_with_batch_size(
    model: &mut BenchmarkModel,
    dataset: &[(Vec<f32>, Vec<f32>)],
    batch_size: usize,
) -> Duration {
    let start = std::time::Instant::now();
    
    for epoch in 0..20 {
        for batch in dataset.chunks(batch_size) {
            // Accumulate gradients over batch
            let mut weight_grads = vec![0.0; model.weights.len()];
            let mut bias_grads = vec![0.0; model.biases.len()];
            
            for (input, target) in batch {
                let (w_grads, b_grads) = model.compute_gradients(input, target);
                for (acc, grad) in weight_grads.iter_mut().zip(&w_grads) {
                    *acc += grad;
                }
                for (acc, grad) in bias_grads.iter_mut().zip(&b_grads) {
                    *acc += grad;
                }
            }
            
            // Average and apply gradients
            let batch_size_f = batch.len() as f32;
            for grad in &mut weight_grads {
                *grad /= batch_size_f;
            }
            for grad in &mut bias_grads {
                *grad /= batch_size_f;
            }
            
            model.apply_gradients(&weight_grads, &bias_grads);
        }
        
        // Early stopping simulation
        if epoch > 10 {
            break;
        }
    }
    
    start.elapsed()
}

async fn train_with_sync_frequency(
    layers: &[usize],
    dataset: &[(Vec<f32>, Vec<f32>)],
    node_count: usize,
    sync_frequency: usize,
) -> Duration {
    let start = std::time::Instant::now();
    
    // Simulate training with periodic synchronization
    let batch_size = 32;
    let batches: Vec<_> = dataset.chunks(batch_size).collect();
    
    for (batch_idx, _batch) in batches.iter().enumerate() {
        // Local training step
        tokio::time::sleep(Duration::from_micros(100)).await;
        
        // Synchronize every sync_frequency batches
        if batch_idx % sync_frequency == 0 {
            // Simulate gradient synchronization
            let param_count: usize = layers.windows(2).map(|pair| pair[0] * pair[1]).sum::<usize>() + layers[1..].iter().sum::<usize>();
            let sync_time = Duration::from_millis((param_count * node_count) as u64 / 1000);
            tokio::time::sleep(sync_time).await;
        }
    }
    
    start.elapsed()
}

async fn simulate_gradient_communication(pattern: &str, gradients: &[f32], node_count: usize) -> Duration {
    let start = std::time::Instant::now();
    let gradient_size = gradients.len();
    
    match pattern {
        "all_reduce" => {
            // All-reduce: log(nodes) communication rounds
            let rounds = (node_count as f32).log2().ceil() as usize;
            for _ in 0..rounds {
                tokio::time::sleep(Duration::from_micros((gradient_size / 1000) as u64)).await;
            }
        },
        "parameter_server" => {
            // Parameter server: 2 communication rounds (send + receive)
            tokio::time::sleep(Duration::from_micros((gradient_size * 2 / 1000) as u64)).await;
        },
        "ring_all_reduce" => {
            // Ring all-reduce: 2*(nodes-1) communication steps
            let steps = 2 * (node_count - 1);
            for _ in 0..steps {
                tokio::time::sleep(Duration::from_micros((gradient_size / node_count / 1000) as u64)).await;
            }
        },
        "hierarchical" => {
            // Hierarchical: local reduce + global reduce
            tokio::time::sleep(Duration::from_micros((gradient_size / 2000) as u64)).await; // Local
            tokio::time::sleep(Duration::from_micros((gradient_size / 4000) as u64)).await; // Global
        },
        _ => {
            tokio::time::sleep(Duration::from_micros((gradient_size / 1000) as u64)).await;
        }
    }
    
    start.elapsed()
}

async fn simulate_federated_learning(
    layers: &[usize],
    client_count: usize,
    local_dataset_size: usize,
) -> Duration {
    let start = std::time::Instant::now();
    
    // Simulate federated learning rounds
    let rounds = 10;
    
    for round in 0..rounds {
        // Client selection (subset of clients participate)
        let participating_clients = (client_count as f32 * 0.1).max(1.0) as usize; // 10% participation
        
        // Parallel client training
        let client_handles: Vec<_> = (0..participating_clients).map(|_| {
            let layers = layers.to_vec();
            tokio::spawn(async move {
                // Local training
                let local_dataset = generate_training_data(local_dataset_size, layers[0], *layers.last().unwrap());
                let mut model = BenchmarkModel::new(layers, 0.01);
                
                // Local epochs
                for _ in 0..5 {
                    for (input, target) in &local_dataset {
                        let (weight_grads, bias_grads) = model.compute_gradients(input, target);
                        model.apply_gradients(&weight_grads, &bias_grads);
                    }
                }
                
                model
            })
        }).collect();
        
        // Wait for client training
        for handle in client_handles {
            handle.await.unwrap();
        }
        
        // Simulate server aggregation
        let param_count: usize = layers.windows(2).map(|pair| pair[0] * pair[1]).sum::<usize>() + layers[1..].iter().sum::<usize>();
        let aggregation_time = Duration::from_millis((param_count * participating_clients) as u64 / 10000);
        tokio::time::sleep(aggregation_time).await;
    }
    
    start.elapsed()
}

criterion_group!(
    benches,
    benchmark_training_convergence_model_size,
    benchmark_distributed_vs_single_training,
    benchmark_training_batch_sizes,
    benchmark_gradient_sync_frequency,
    benchmark_learning_rate_impact,
    benchmark_communication_overhead,
    benchmark_federated_learning_convergence
);
criterion_main!(benches);