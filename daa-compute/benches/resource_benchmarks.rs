//! Resource Utilization Benchmarks - Bandwidth, CPU, and GPU profiling

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio::runtime::Runtime;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use rand::prelude::*;

/// Resource utilization metrics
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub bandwidth_mbps: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub gpu_usage_percent: Option<f64>,
    pub gpu_memory_mb: Option<f64>,
    pub network_latency_ms: f64,
    pub disk_io_mbps: f64,
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            bandwidth_mbps: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            gpu_usage_percent: None,
            gpu_memory_mb: None,
            network_latency_ms: 0.0,
            disk_io_mbps: 0.0,
        }
    }
}

/// Simulated resource monitor
pub struct ResourceMonitor {
    bandwidth_counter: Arc<AtomicU64>,
    cpu_load_counter: Arc<AtomicU64>,
    memory_usage: Arc<AtomicU64>,
    gpu_usage: Arc<AtomicU64>,
    start_time: Instant,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            bandwidth_counter: Arc::new(AtomicU64::new(0)),
            cpu_load_counter: Arc::new(AtomicU64::new(0)),
            memory_usage: Arc::new(AtomicU64::new(0)),
            gpu_usage: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }
    
    pub fn record_bandwidth(&self, bytes: u64) {
        self.bandwidth_counter.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn record_cpu_work(&self, operations: u64) {
        self.cpu_load_counter.fetch_add(operations, Ordering::Relaxed);
    }
    
    pub fn record_memory_usage(&self, bytes: u64) {
        self.memory_usage.store(bytes, Ordering::Relaxed);
    }
    
    pub fn record_gpu_work(&self, operations: u64) {
        self.gpu_usage.fetch_add(operations, Ordering::Relaxed);
    }
    
    pub fn get_metrics(&self) -> ResourceMetrics {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let bytes_transferred = self.bandwidth_counter.load(Ordering::Relaxed);
        let cpu_operations = self.cpu_load_counter.load(Ordering::Relaxed);
        let memory_bytes = self.memory_usage.load(Ordering::Relaxed);
        let gpu_operations = self.gpu_usage.load(Ordering::Relaxed);
        
        ResourceMetrics {
            bandwidth_mbps: (bytes_transferred as f64 / elapsed) / (1024.0 * 1024.0),
            cpu_usage_percent: (cpu_operations as f64 / elapsed) / 10000.0, // Normalize
            memory_usage_mb: memory_bytes as f64 / (1024.0 * 1024.0),
            gpu_usage_percent: if gpu_operations > 0 { Some((gpu_operations as f64 / elapsed) / 10000.0) } else { None },
            gpu_memory_mb: if gpu_operations > 0 { Some(memory_bytes as f64 / (1024.0 * 1024.0) * 0.5) } else { None },
            network_latency_ms: rand::thread_rng().gen_range(1.0..100.0), // Simulated
            disk_io_mbps: (bytes_transferred as f64 / elapsed) / (1024.0 * 1024.0) * 0.1, // Simulated
        }
    }
}

/// Benchmark bandwidth usage during gradient sharing
fn benchmark_bandwidth_usage_gradient_sharing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("bandwidth_usage_gradient_sharing");
    group.measurement_time(Duration::from_secs(30));
    
    let gradient_sizes = vec![
        ("1M", 1_000_000),
        ("10M", 10_000_000),
        ("100M", 100_000_000),
        ("1B", 1_000_000_000),
    ];
    
    let peer_counts = vec![4, 8, 16, 32];
    
    for (size_name, gradient_size) in gradient_sizes {
        for peer_count in &peer_counts {
            let data_size = gradient_size * 4; // 4 bytes per f32
            group.throughput(Throughput::Bytes((data_size * peer_count) as u64));
            
            group.bench_with_input(
                BenchmarkId::new(format!("size_{}_peers", size_name), peer_count),
                &(gradient_size, *peer_count),
                |b, &(gradient_size, peer_count)| {
                    b.to_async(&rt).iter(|| async move {
                        let monitor = ResourceMonitor::new();
                        let gradient = create_gradient(gradient_size);
                        
                        // Simulate gradient sharing across peers
                        for _ in 0..peer_count {
                            simulate_gradient_transfer(&monitor, &gradient).await;
                        }
                        
                        let metrics = monitor.get_metrics();
                        black_box(metrics.bandwidth_mbps)
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark CPU utilization during model training
fn benchmark_cpu_utilization_training(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("cpu_utilization_training");
    group.measurement_time(Duration::from_secs(25));
    
    let model_sizes = vec![
        ("small", vec![784, 128, 10]),
        ("medium", vec![784, 512, 256, 10]),
        ("large", vec![784, 1024, 512, 256, 10]),
    ];
    
    let batch_sizes = vec![32, 128, 512];
    
    for (model_name, layers) in model_sizes {
        for batch_size in &batch_sizes {
            group.bench_with_input(
                BenchmarkId::new(format!("model_{}_batch", model_name), batch_size),
                &(layers.clone(), *batch_size),
                |b, (layers, batch_size)| {
                    b.to_async(&rt).iter(|| async move {
                        let monitor = ResourceMonitor::new();
                        
                        // Simulate training workload
                        simulate_training_workload(&monitor, layers, *batch_size, 100).await;
                        
                        let metrics = monitor.get_metrics();
                        black_box(metrics.cpu_usage_percent)
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark memory usage during distributed training
fn benchmark_memory_usage_distributed_training(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_usage_distributed_training");
    group.measurement_time(Duration::from_secs(20));
    
    let scenarios = vec![
        ("single_node", 1, 1_000_000),
        ("small_cluster", 4, 1_000_000),
        ("medium_cluster", 8, 5_000_000),
        ("large_cluster", 16, 10_000_000),
    ];
    
    for (scenario_name, node_count, model_params) in scenarios {
        group.throughput(Throughput::Bytes((model_params * 4 * node_count) as u64));
        
        group.bench_with_input(
            BenchmarkId::new("scenario", scenario_name),
            &(node_count, model_params),
            |b, &(node_count, model_params)| {
                b.to_async(&rt).iter(|| async move {
                    let monitor = ResourceMonitor::new();
                    
                    // Simulate distributed training memory usage
                    simulate_distributed_memory_usage(&monitor, node_count, model_params).await;
                    
                    let metrics = monitor.get_metrics();
                    black_box(metrics.memory_usage_mb)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark GPU utilization during training
fn benchmark_gpu_utilization_training(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("gpu_utilization_training");
    group.measurement_time(Duration::from_secs(25));
    
    let gpu_scenarios = vec![
        ("single_gpu", 1, vec![784, 2048, 1024, 10]),
        ("multi_gpu", 4, vec![784, 4096, 2048, 10]),
        ("gpu_cluster", 8, vec![784, 8192, 4096, 10]),
    ];
    
    for (scenario_name, gpu_count, layers) in gpu_scenarios {
        let param_count: usize = layers.windows(2).map(|pair| pair[0] * pair[1]).sum::<usize>() + layers[1..].iter().sum::<usize>();
        group.throughput(Throughput::Elements((param_count * gpu_count) as u64));
        
        group.bench_with_input(
            BenchmarkId::new("scenario", scenario_name),
            &(gpu_count, layers),
            |b, (gpu_count, layers)| {
                b.to_async(&rt).iter(|| async move {
                    let monitor = ResourceMonitor::new();
                    
                    // Simulate GPU training workload
                    simulate_gpu_training_workload(&monitor, *gpu_count, layers, 64).await;
                    
                    let metrics = monitor.get_metrics();
                    black_box(metrics.gpu_usage_percent.unwrap_or(0.0))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark network latency impact on consensus
fn benchmark_network_latency_consensus(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("network_latency_consensus");
    group.measurement_time(Duration::from_secs(30));
    
    let latency_scenarios = vec![
        ("lan", 1),           // 1ms LAN
        ("wan_good", 50),     // 50ms good WAN
        ("wan_poor", 200),    // 200ms poor WAN
        ("satellite", 500),   // 500ms satellite
    ];
    
    let validator_counts = vec![7, 13, 25];
    
    for (latency_name, latency_ms) in latency_scenarios {
        for validator_count in &validator_counts {
            group.bench_with_input(
                BenchmarkId::new(format!("latency_{}_validators", latency_name), validator_count),
                &(latency_ms, *validator_count),
                |b, &(latency_ms, validator_count)| {
                    b.to_async(&rt).iter(|| async move {
                        let monitor = ResourceMonitor::new();
                        
                        // Simulate consensus with network latency
                        simulate_consensus_with_latency(&monitor, validator_count, latency_ms).await;
                        
                        let metrics = monitor.get_metrics();
                        black_box(metrics.network_latency_ms)
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark bandwidth efficiency of different compression levels
fn benchmark_bandwidth_compression_efficiency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("bandwidth_compression_efficiency");
    group.measurement_time(Duration::from_secs(20));
    
    let compression_levels = vec![
        ("none", 0, 1.0),
        ("low", 3, 0.7),
        ("medium", 6, 0.5),
        ("high", 9, 0.3),
    ];
    
    let data_sizes = vec![1_000_000, 10_000_000, 100_000_000]; // 1MB, 10MB, 100MB
    
    for (comp_name, comp_level, compression_ratio) in compression_levels {
        for data_size in &data_sizes {
            let compressed_size = (*data_size as f64 * compression_ratio) as u64;
            group.throughput(Throughput::Bytes(compressed_size));
            
            group.bench_with_input(
                BenchmarkId::new(format!("compression_{}_size", comp_name), data_size),
                &(*data_size, compression_ratio),
                |b, &(data_size, compression_ratio)| {
                    b.to_async(&rt).iter(|| async move {
                        let monitor = ResourceMonitor::new();
                        
                        // Simulate compressed data transfer
                        simulate_compressed_transfer(&monitor, data_size, compression_ratio).await;
                        
                        let metrics = monitor.get_metrics();
                        black_box(metrics.bandwidth_mbps)
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark resource usage during P2P discovery
fn benchmark_resource_usage_p2p_discovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("resource_usage_p2p_discovery");
    group.measurement_time(Duration::from_secs(25));
    
    let network_sizes = vec![10, 50, 100, 500, 1000];
    
    for network_size in network_sizes {
        group.throughput(Throughput::Elements(network_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("network_size", network_size),
            &network_size,
            |b, &network_size| {
                b.to_async(&rt).iter(|| async move {
                    let monitor = ResourceMonitor::new();
                    
                    // Simulate P2P discovery process
                    simulate_p2p_discovery(&monitor, network_size).await;
                    
                    let metrics = monitor.get_metrics();
                    black_box((metrics.bandwidth_mbps, metrics.cpu_usage_percent))
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions for resource simulation

fn create_gradient(size: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen_range(-0.01..0.01)).collect()
}

async fn simulate_gradient_transfer(monitor: &ResourceMonitor, gradient: &[f32]) {
    let bytes_transferred = gradient.len() * 4; // 4 bytes per f32
    monitor.record_bandwidth(bytes_transferred as u64);
    
    // Simulate network transfer time
    let transfer_time = Duration::from_micros((bytes_transferred / 1000) as u64);
    tokio::time::sleep(transfer_time).await;
    
    // Simulate CPU work for serialization/compression
    monitor.record_cpu_work(gradient.len() as u64);
}

async fn simulate_training_workload(
    monitor: &ResourceMonitor,
    layers: &[usize],
    batch_size: usize,
    iterations: usize,
) {
    let param_count: usize = layers.windows(2).map(|pair| pair[0] * pair[1]).sum::<usize>() + layers[1..].iter().sum::<usize>();
    
    // Simulate memory usage for model and batch
    let model_memory = param_count * 4; // 4 bytes per f32
    let batch_memory = batch_size * layers[0] * 4; // Input batch memory
    monitor.record_memory_usage((model_memory + batch_memory) as u64);
    
    for _ in 0..iterations {
        // Simulate forward pass
        let forward_ops = param_count * batch_size;
        monitor.record_cpu_work(forward_ops as u64);
        
        // Simulate backward pass
        let backward_ops = forward_ops * 2; // Backward pass is ~2x forward pass
        monitor.record_cpu_work(backward_ops as u64);
        
        // Simulate computation time
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
}

async fn simulate_distributed_memory_usage(
    monitor: &ResourceMonitor,
    node_count: usize,
    model_params: usize,
) {
    // Each node stores the full model plus gradients
    let memory_per_node = model_params * 4 * 2; // Model + gradients
    let total_memory = memory_per_node * node_count;
    
    // Additional memory for communication buffers
    let comm_buffer_size = model_params * 4; // One gradient buffer per node
    let total_with_buffers = total_memory + comm_buffer_size * node_count;
    
    monitor.record_memory_usage(total_with_buffers as u64);
    
    // Simulate memory allocation time
    tokio::time::sleep(Duration::from_millis(10)).await;
}

async fn simulate_gpu_training_workload(
    monitor: &ResourceMonitor,
    gpu_count: usize,
    layers: &[usize],
    batch_size: usize,
) {
    let param_count: usize = layers.windows(2).map(|pair| pair[0] * pair[1]).sum::<usize>() + layers[1..].iter().sum::<usize>();
    
    // GPU memory usage
    let gpu_memory_per_device = (param_count * 4) + (batch_size * layers[0] * 4); // Model + batch
    let total_gpu_memory = gpu_memory_per_device * gpu_count;
    monitor.record_memory_usage(total_gpu_memory as u64);
    
    // Simulate GPU operations
    for _ in 0..100 {
        let gpu_ops = param_count * batch_size / gpu_count; // Work distributed across GPUs
        monitor.record_gpu_work(gpu_ops as u64);
        
        // GPU computation time
        tokio::time::sleep(Duration::from_micros(50)).await;
    }
}

async fn simulate_consensus_with_latency(
    monitor: &ResourceMonitor,
    validator_count: usize,
    latency_ms: u64,
) {
    // Simulate consensus messages
    let message_count = validator_count * 3; // Propose, prevote, precommit
    let message_size = 1024; // 1KB per message
    
    for _ in 0..message_count {
        monitor.record_bandwidth(message_size);
        
        // Network latency
        tokio::time::sleep(Duration::from_millis(latency_ms)).await;
        
        // CPU work for message verification
        monitor.record_cpu_work(1000);
    }
}

async fn simulate_compressed_transfer(
    monitor: &ResourceMonitor,
    original_size: usize,
    compression_ratio: f64,
) {
    let compressed_size = (original_size as f64 * compression_ratio) as u64;
    
    // CPU work for compression
    monitor.record_cpu_work(original_size as u64);
    
    // Network transfer of compressed data
    monitor.record_bandwidth(compressed_size);
    
    // Compression time
    let compression_time = Duration::from_micros((original_size / 1000) as u64);
    tokio::time::sleep(compression_time).await;
    
    // Transfer time
    let transfer_time = Duration::from_micros((compressed_size / 1000) as u64);
    tokio::time::sleep(transfer_time).await;
}

async fn simulate_p2p_discovery(monitor: &ResourceMonitor, network_size: usize) {
    // Simulate DHT operations
    let dht_queries = (network_size as f64).log2().ceil() as usize;
    
    for _ in 0..dht_queries {
        // Each query involves network communication
        monitor.record_bandwidth(512); // 512 bytes per query
        
        // CPU work for routing table updates
        monitor.record_cpu_work(100);
        
        // Network round trip
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Memory usage for peer tables
    let peer_table_size = network_size * 64; // 64 bytes per peer entry
    monitor.record_memory_usage(peer_table_size as u64);
}

criterion_group!(
    benches,
    benchmark_bandwidth_usage_gradient_sharing,
    benchmark_cpu_utilization_training,
    benchmark_memory_usage_distributed_training,
    benchmark_gpu_utilization_training,
    benchmark_network_latency_consensus,
    benchmark_bandwidth_compression_efficiency,
    benchmark_resource_usage_p2p_discovery
);
criterion_main!(benches);