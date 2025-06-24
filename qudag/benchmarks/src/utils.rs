use metrics::{counter, gauge, histogram};
use std::time::{Duration, Instant};

pub struct BenchmarkMetrics {
    start_time: Instant,
    message_count: u64,
    node_count: u32,
}

impl BenchmarkMetrics {
    pub fn new(node_count: u32) -> Self {
        Self {
            start_time: Instant::now(),
            message_count: 0,
            node_count,
        }
    }

    pub fn record_message(&mut self) {
        self.message_count += 1;
        counter!("qudag.messages.total", 1);
    }

    pub fn record_latency(&self, duration: Duration) {
        histogram!("qudag.message.latency", duration.as_secs_f64());
    }

    pub fn record_memory_usage(&self, bytes: u64) {
        gauge!("qudag.memory.usage", bytes as f64);
    }

    pub fn calculate_throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed();
        self.message_count as f64 / elapsed.as_secs_f64()
    }
}

pub struct ResourceMonitor {
    peak_memory: u64,
    start_time: Instant,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            peak_memory: 0,
            start_time: Instant::now(),
        }
    }

    pub fn update_memory_usage(&mut self, current_memory: u64) {
        self.peak_memory = self.peak_memory.max(current_memory);
        gauge!("qudag.memory.peak", self.peak_memory as f64);
    }

    pub fn get_runtime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

pub fn calculate_node_scalability(
    base_throughput: f64,
    scaled_throughput: f64,
    node_ratio: f64,
) -> f64 {
    // Calculate scalability factor (1.0 = linear scaling)
    scaled_throughput / (base_throughput * node_ratio)
}
