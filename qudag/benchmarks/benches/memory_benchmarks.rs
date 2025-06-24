use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Memory tracking allocator for benchmarking
struct MemoryTracker {
    allocated: AtomicUsize,
    peak_usage: AtomicUsize,
}

impl MemoryTracker {
    const fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
        }
    }

    fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
    }

    fn current_usage(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    fn peak_usage(&self) -> usize {
        self.peak_usage.load(Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for MemoryTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let current =
                self.allocated.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            let mut peak = self.peak_usage.load(Ordering::Relaxed);
            while current > peak {
                match self.peak_usage.compare_exchange_weak(
                    peak,
                    current,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(p) => peak = p,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.allocated.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static MEMORY_TRACKER: MemoryTracker = MemoryTracker::new();

/// Simulated QuDAG node components for memory testing
struct MockNode {
    id: u64,
    connections: Vec<MockConnection>,
    dag_vertices: Vec<MockVertex>,
    crypto_keys: Vec<MockKey>,
    message_queue: Vec<MockMessage>,
}

struct MockConnection {
    peer_id: u64,
    buffer: Vec<u8>,
    state: ConnectionState,
}

#[derive(Clone)]
enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
}

struct MockVertex {
    id: u64,
    data: Vec<u8>,
    references: Vec<u64>,
    signature: Vec<u8>,
}

struct MockKey {
    public_key: Vec<u8>,
    private_key: Vec<u8>,
    key_type: KeyType,
}

#[derive(Clone)]
enum KeyType {
    MLKem,
    MLDSA,
    HQC,
}

struct MockMessage {
    id: u64,
    payload: Vec<u8>,
    routing_info: Vec<u8>,
    timestamp: u64,
}

impl MockNode {
    fn new(id: u64) -> Self {
        Self {
            id,
            connections: Vec::new(),
            dag_vertices: Vec::new(),
            crypto_keys: Vec::new(),
            message_queue: Vec::new(),
        }
    }

    fn add_connection(&mut self, peer_id: u64, buffer_size: usize) {
        self.connections.push(MockConnection {
            peer_id,
            buffer: vec![0u8; buffer_size],
            state: ConnectionState::Connected,
        });
    }

    fn add_vertex(&mut self, vertex_id: u64, data_size: usize, ref_count: usize) {
        self.dag_vertices.push(MockVertex {
            id: vertex_id,
            data: vec![0u8; data_size],
            references: (0..ref_count).map(|i| i as u64).collect(),
            signature: vec![0u8; 64], // Standard signature size
        });
    }

    fn add_crypto_key(&mut self, key_type: KeyType) {
        let (pub_size, priv_size) = match key_type {
            KeyType::MLKem => (1184, 2400), // ML-KEM-768 sizes
            KeyType::MLDSA => (1952, 4000), // ML-DSA sizes
            KeyType::HQC => (2249, 40),     // HQC sizes
        };

        self.crypto_keys.push(MockKey {
            public_key: vec![0u8; pub_size],
            private_key: vec![0u8; priv_size],
            key_type,
        });
    }

    fn add_message(&mut self, msg_id: u64, payload_size: usize) {
        self.message_queue.push(MockMessage {
            id: msg_id,
            payload: vec![0u8; payload_size],
            routing_info: vec![0u8; 256], // Onion routing overhead
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
    }

    fn process_messages(&mut self, count: usize) {
        // Simulate message processing
        for _ in 0..count {
            if let Some(msg) = self.message_queue.pop() {
                // Simulate processing overhead
                let _processed = msg.payload.len() + msg.routing_info.len();
            }
        }
    }

    fn estimate_memory_usage(&self) -> usize {
        let connections_memory = self
            .connections
            .iter()
            .map(|c| c.buffer.len() + std::mem::size_of::<MockConnection>())
            .sum::<usize>();

        let vertices_memory = self
            .dag_vertices
            .iter()
            .map(|v| {
                v.data.len()
                    + v.references.len() * 8
                    + v.signature.len()
                    + std::mem::size_of::<MockVertex>()
            })
            .sum::<usize>();

        let keys_memory = self
            .crypto_keys
            .iter()
            .map(|k| k.public_key.len() + k.private_key.len() + std::mem::size_of::<MockKey>())
            .sum::<usize>();

        let messages_memory = self
            .message_queue
            .iter()
            .map(|m| m.payload.len() + m.routing_info.len() + std::mem::size_of::<MockMessage>())
            .sum::<usize>();

        connections_memory
            + vertices_memory
            + keys_memory
            + messages_memory
            + std::mem::size_of::<MockNode>()
    }
}

fn benchmark_base_node_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("base_node_memory");

    // Test base node memory usage under different loads
    let connection_counts = [10, 50, 100, 200];

    for &conn_count in &connection_counts {
        group.bench_with_input(
            BenchmarkId::new("base_node_connections", conn_count),
            &conn_count,
            |b, &conn_count| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start_mem = MEMORY_TRACKER.current_usage();

                        let start = Instant::now();

                        let mut node = MockNode::new(1);

                        // Add connections
                        for i in 0..conn_count {
                            node.add_connection(i as u64, 8192); // 8KB buffer per connection
                        }

                        // Add some crypto keys
                        node.add_crypto_key(KeyType::MLKem);
                        node.add_crypto_key(KeyType::MLDSA);
                        node.add_crypto_key(KeyType::HQC);

                        let duration = start.elapsed();
                        let peak_mem = MEMORY_TRACKER.peak_usage();
                        let estimated_mem = node.estimate_memory_usage();

                        println!(
                            "Connections: {}, Peak memory: {} MB, Estimated: {} MB",
                            conn_count,
                            peak_mem / (1024 * 1024),
                            estimated_mem / (1024 * 1024)
                        );

                        // Verify memory usage is under target (100MB)
                        if peak_mem > 100 * 1024 * 1024 {
                            println!(
                                "WARNING: Peak memory usage {} MB exceeds 100MB target",
                                peak_mem / (1024 * 1024)
                            );
                        }

                        black_box(node);
                        total_duration += duration;
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

fn benchmark_dag_memory_growth(c: &mut Criterion) {
    let mut group = c.benchmark_group("dag_memory_growth");

    // Test DAG memory usage as it grows
    let vertex_counts = [1000, 5000, 10000, 50000];

    for &vertex_count in &vertex_counts {
        group.bench_with_input(
            BenchmarkId::new("dag_vertices", vertex_count),
            &vertex_count,
            |b, &vertex_count| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();

                        let start = Instant::now();

                        let mut node = MockNode::new(1);

                        // Add vertices to simulate DAG growth
                        for i in 0..vertex_count {
                            let data_size = 256; // Average transaction size
                            let ref_count = std::cmp::min(3, i); // Each vertex references up to 3 previous
                            node.add_vertex(i as u64, data_size, ref_count);
                        }

                        let duration = start.elapsed();
                        let peak_mem = MEMORY_TRACKER.peak_usage();
                        let estimated_mem = node.estimate_memory_usage();

                        println!(
                            "Vertices: {}, Peak memory: {} MB, Estimated: {} MB",
                            vertex_count,
                            peak_mem / (1024 * 1024),
                            estimated_mem / (1024 * 1024)
                        );

                        // Check memory scaling
                        let mem_per_vertex = peak_mem / vertex_count;
                        println!("Memory per vertex: {} bytes", mem_per_vertex);

                        black_box(node);
                        total_duration += duration;
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

fn benchmark_message_queue_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_queue_memory");

    // Test message queue memory usage
    let message_counts = [1000, 5000, 10000, 20000];
    let message_sizes = [256, 1024, 4096];

    for &msg_count in &message_counts {
        for &msg_size in &message_sizes {
            group.bench_with_input(
                BenchmarkId::new("messages", format!("{}x{}", msg_count, msg_size)),
                &(msg_count, msg_size),
                |b, &(msg_count, msg_size)| {
                    b.iter_custom(|iters| {
                        let mut total_duration = Duration::new(0, 0);

                        for _ in 0..iters {
                            MEMORY_TRACKER.reset();

                            let start = Instant::now();

                            let mut node = MockNode::new(1);

                            // Add messages to queue
                            for i in 0..msg_count {
                                node.add_message(i as u64, msg_size);
                            }

                            // Process some messages
                            node.process_messages(msg_count / 2);

                            let duration = start.elapsed();
                            let peak_mem = MEMORY_TRACKER.peak_usage();

                            println!(
                                "Messages: {}x{} bytes, Peak memory: {} MB",
                                msg_count,
                                msg_size,
                                peak_mem / (1024 * 1024)
                            );

                            // Calculate throughput
                            let total_data = msg_count * msg_size;
                            let throughput_mbps =
                                (total_data as f64 / (1024.0 * 1024.0)) / duration.as_secs_f64();
                            println!("Processing throughput: {:.2} MB/s", throughput_mbps);

                            black_box(node);
                            total_duration += duration;
                        }

                        total_duration
                    });
                },
            );
        }
    }

    group.finish();
}

fn benchmark_crypto_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("crypto_memory_overhead");

    // Test memory overhead of crypto operations
    let key_counts = [1, 10, 50, 100];

    for &key_count in &key_counts {
        group.bench_with_input(
            BenchmarkId::new("crypto_keys", key_count),
            &key_count,
            |b, &key_count| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();

                        let start = Instant::now();

                        let mut node = MockNode::new(1);

                        // Add different types of crypto keys
                        for _ in 0..key_count {
                            node.add_crypto_key(KeyType::MLKem);
                            node.add_crypto_key(KeyType::MLDSA);
                            node.add_crypto_key(KeyType::HQC);
                        }

                        let duration = start.elapsed();
                        let peak_mem = MEMORY_TRACKER.peak_usage();

                        println!(
                            "Key sets: {}, Peak memory: {} MB",
                            key_count,
                            peak_mem / (1024 * 1024)
                        );

                        // Calculate memory per key set
                        let mem_per_key_set = peak_mem / key_count;
                        println!(
                            "Memory per key set (ML-KEM + ML-DSA + HQC): {} KB",
                            mem_per_key_set / 1024
                        );

                        black_box(node);
                        total_duration += duration;
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

fn benchmark_full_node_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_node_simulation");

    // Simulate a full node with realistic workload
    group.bench_function("realistic_node_workload", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                MEMORY_TRACKER.reset();

                let start = Instant::now();

                let mut node = MockNode::new(1);

                // Realistic node setup
                // 50 peer connections
                for i in 0..50 {
                    node.add_connection(i as u64, 16384); // 16KB buffer per connection
                }

                // Crypto keys for various purposes
                for _ in 0..5 {
                    node.add_crypto_key(KeyType::MLKem); // For key exchange
                    node.add_crypto_key(KeyType::MLDSA); // For signatures
                    node.add_crypto_key(KeyType::HQC); // For encryption
                }

                // DAG with reasonable size (simulating some history)
                for i in 0..5000 {
                    node.add_vertex(i as u64, 512, std::cmp::min(3, i)); // 512-byte transactions
                }

                // Message queue with active messages
                for i in 0..1000 {
                    node.add_message(i as u64, 1024); // 1KB messages
                }

                // Process some messages
                node.process_messages(500);

                let duration = start.elapsed();
                let peak_mem = MEMORY_TRACKER.peak_usage();
                let estimated_mem = node.estimate_memory_usage();

                println!("Full node simulation:");
                println!("  Peak memory: {} MB", peak_mem / (1024 * 1024));
                println!("  Estimated memory: {} MB", estimated_mem / (1024 * 1024));
                println!("  Setup time: {}ms", duration.as_millis());

                // Critical check: verify we're under 100MB target
                if peak_mem > 100 * 1024 * 1024 {
                    println!(
                        "ERROR: Node memory usage {} MB exceeds 100MB target!",
                        peak_mem / (1024 * 1024)
                    );
                } else {
                    println!("✓ Node memory usage within 100MB target");
                }

                black_box(node);
                total_duration += duration;
            }

            total_duration
        });
    });

    group.finish();
}

fn benchmark_memory_leak_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_leak_detection");

    // Test for memory leaks during repeated operations
    group.bench_function("repeated_operations", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut initial_memory = 0;
            let mut final_memory = 0;

            for i in 0..iters {
                if i == 0 {
                    MEMORY_TRACKER.reset();
                    initial_memory = MEMORY_TRACKER.current_usage();
                }

                let start = Instant::now();

                // Perform operations that should not leak memory
                {
                    let mut node = MockNode::new(i as u64);
                    node.add_connection(1, 1024);
                    node.add_vertex(1, 256, 1);
                    node.add_crypto_key(KeyType::MLKem);
                    node.add_message(1, 512);
                    node.process_messages(1);

                    black_box(node);
                } // node should be dropped here

                let duration = start.elapsed();
                total_duration += duration;

                if i == iters - 1 {
                    final_memory = MEMORY_TRACKER.current_usage();
                }
            }

            let memory_growth = final_memory.saturating_sub(initial_memory);
            println!("Memory leak test:");
            println!("  Initial memory: {} KB", initial_memory / 1024);
            println!("  Final memory: {} KB", final_memory / 1024);
            println!("  Memory growth: {} KB", memory_growth / 1024);

            if memory_growth > 1024 * 1024 {
                // 1MB threshold
                println!(
                    "WARNING: Potential memory leak detected! Growth: {} MB",
                    memory_growth / (1024 * 1024)
                );
            } else {
                println!("✓ No significant memory leaks detected");
            }

            total_duration
        });
    });

    group.finish();
}

criterion_group!(
    name = memory_benches;
    config = Criterion::default()
        .sample_size(20)
        .measurement_time(Duration::from_secs(30));
    targets =
        benchmark_base_node_memory,
        benchmark_dag_memory_growth,
        benchmark_message_queue_memory,
        benchmark_crypto_memory_overhead,
        benchmark_full_node_simulation,
        benchmark_memory_leak_detection
);
criterion_main!(memory_benches);
