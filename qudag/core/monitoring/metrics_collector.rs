use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec,
    CounterVec, GaugeVec, HistogramVec, Registry,
};
use std::sync::Arc;
use std::time::Instant;

pub struct MetricsCollector {
    // Message chunking metrics
    pub chunks_processed: CounterVec,
    pub compression_ratio: GaugeVec,
    pub chunking_duration: HistogramVec,
    
    // Connection pool metrics
    pub connection_hits: CounterVec,
    pub connection_misses: CounterVec,
    pub active_connections: GaugeVec,
    pub connection_wait_time: HistogramVec,
    
    // Validation cache metrics
    pub validation_cache_hits: CounterVec,
    pub validation_cache_misses: CounterVec,
    pub cache_memory_usage: GaugeVec,
    pub cache_evictions: CounterVec,
    
    // Swarm coordination metrics
    pub tasks_created: CounterVec,
    pub tasks_completed: CounterVec,
    pub task_latency: HistogramVec,
    pub agent_utilization: GaugeVec,
    pub swarm_queue_depth: GaugeVec,
    
    // System-wide metrics
    pub message_throughput: CounterVec,
    pub error_rate: CounterVec,
    pub system_memory_usage: GaugeVec,
    pub cpu_usage: GaugeVec,
}

impl MetricsCollector {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let chunks_processed = register_counter_vec!(
            "qudag_chunks_processed_total",
            "Total number of message chunks processed",
            &["node_id", "message_type"]
        )?;
        registry.register(Box::new(chunks_processed.clone()))?;
        
        let compression_ratio = register_gauge_vec!(
            "qudag_compression_ratio",
            "Average compression ratio for message chunks",
            &["node_id", "compression_type"]
        )?;
        registry.register(Box::new(compression_ratio.clone()))?;
        
        let chunking_duration = register_histogram_vec!(
            "qudag_chunking_duration_seconds",
            "Time taken to chunk messages",
            &["node_id", "message_size_bucket"]
        )?;
        registry.register(Box::new(chunking_duration.clone()))?;
        
        let connection_hits = register_counter_vec!(
            "qudag_connection_pool_hits_total",
            "Number of connection pool hits",
            &["node_id", "pool_type"]
        )?;
        registry.register(Box::new(connection_hits.clone()))?;
        
        let connection_misses = register_counter_vec!(
            "qudag_connection_pool_misses_total",
            "Number of connection pool misses",
            &["node_id", "pool_type"]
        )?;
        registry.register(Box::new(connection_misses.clone()))?;
        
        let active_connections = register_gauge_vec!(
            "qudag_active_connections",
            "Number of active connections in pool",
            &["node_id", "pool_type", "state"]
        )?;
        registry.register(Box::new(active_connections.clone()))?;
        
        let connection_wait_time = register_histogram_vec!(
            "qudag_connection_wait_time_seconds",
            "Time spent waiting for connection from pool",
            &["node_id", "pool_type"]
        )?;
        registry.register(Box::new(connection_wait_time.clone()))?;
        
        let validation_cache_hits = register_counter_vec!(
            "qudag_validation_cache_hits_total",
            "Number of validation cache hits",
            &["node_id", "validation_type"]
        )?;
        registry.register(Box::new(validation_cache_hits.clone()))?;
        
        let validation_cache_misses = register_counter_vec!(
            "qudag_validation_cache_misses_total",
            "Number of validation cache misses",
            &["node_id", "validation_type"]
        )?;
        registry.register(Box::new(validation_cache_misses.clone()))?;
        
        let cache_memory_usage = register_gauge_vec!(
            "qudag_cache_memory_bytes",
            "Memory usage by caches in bytes",
            &["node_id", "cache_type"]
        )?;
        registry.register(Box::new(cache_memory_usage.clone()))?;
        
        let cache_evictions = register_counter_vec!(
            "qudag_cache_evictions_total",
            "Number of cache evictions",
            &["node_id", "cache_type", "reason"]
        )?;
        registry.register(Box::new(cache_evictions.clone()))?;
        
        let tasks_created = register_counter_vec!(
            "qudag_swarm_tasks_created_total",
            "Total number of swarm tasks created",
            &["swarm_id", "task_type", "priority"]
        )?;
        registry.register(Box::new(tasks_created.clone()))?;
        
        let tasks_completed = register_counter_vec!(
            "qudag_swarm_tasks_completed_total",
            "Total number of swarm tasks completed",
            &["swarm_id", "task_type", "status"]
        )?;
        registry.register(Box::new(tasks_completed.clone()))?;
        
        let task_latency = register_histogram_vec!(
            "qudag_swarm_task_latency_seconds",
            "Latency of swarm task execution",
            &["swarm_id", "task_type"]
        )?;
        registry.register(Box::new(task_latency.clone()))?;
        
        let agent_utilization = register_gauge_vec!(
            "qudag_agent_utilization_ratio",
            "Agent utilization ratio (0-1)",
            &["swarm_id", "agent_id", "agent_type"]
        )?;
        registry.register(Box::new(agent_utilization.clone()))?;
        
        let swarm_queue_depth = register_gauge_vec!(
            "qudag_swarm_queue_depth",
            "Number of tasks in swarm queue",
            &["swarm_id", "queue_type", "priority"]
        )?;
        registry.register(Box::new(swarm_queue_depth.clone()))?;
        
        let message_throughput = register_counter_vec!(
            "qudag_message_throughput_total",
            "Total messages processed",
            &["node_id", "message_type", "direction"]
        )?;
        registry.register(Box::new(message_throughput.clone()))?;
        
        let error_rate = register_counter_vec!(
            "qudag_errors_total",
            "Total number of errors",
            &["node_id", "component", "error_type"]
        )?;
        registry.register(Box::new(error_rate.clone()))?;
        
        let system_memory_usage = register_gauge_vec!(
            "qudag_system_memory_bytes",
            "System memory usage in bytes",
            &["node_id", "memory_type"]
        )?;
        registry.register(Box::new(system_memory_usage.clone()))?;
        
        let cpu_usage = register_gauge_vec!(
            "qudag_cpu_usage_percent",
            "CPU usage percentage",
            &["node_id", "cpu_core"]
        )?;
        registry.register(Box::new(cpu_usage.clone()))?;
        
        Ok(Self {
            chunks_processed,
            compression_ratio,
            chunking_duration,
            connection_hits,
            connection_misses,
            active_connections,
            connection_wait_time,
            validation_cache_hits,
            validation_cache_misses,
            cache_memory_usage,
            cache_evictions,
            tasks_created,
            tasks_completed,
            task_latency,
            agent_utilization,
            swarm_queue_depth,
            message_throughput,
            error_rate,
            system_memory_usage,
            cpu_usage,
        })
    }
    
    // Helper methods for recording metrics
    
    pub fn record_chunk_processed(&self, node_id: &str, message_type: &str, compression_ratio_val: f64) {
        self.chunks_processed
            .with_label_values(&[node_id, message_type])
            .inc();
        
        self.compression_ratio
            .with_label_values(&[node_id, "zstd"])
            .set(compression_ratio_val);
    }
    
    pub fn record_chunking_time(&self, node_id: &str, message_size: usize, duration: f64) {
        let size_bucket = match message_size {
            0..=1024 => "small",
            1025..=10240 => "medium",
            10241..=102400 => "large",
            _ => "xlarge",
        };
        
        self.chunking_duration
            .with_label_values(&[node_id, size_bucket])
            .observe(duration);
    }
    
    pub fn record_connection_pool_hit(&self, node_id: &str, pool_type: &str) {
        self.connection_hits
            .with_label_values(&[node_id, pool_type])
            .inc();
    }
    
    pub fn record_connection_pool_miss(&self, node_id: &str, pool_type: &str, wait_time: f64) {
        self.connection_misses
            .with_label_values(&[node_id, pool_type])
            .inc();
        
        self.connection_wait_time
            .with_label_values(&[node_id, pool_type])
            .observe(wait_time);
    }
    
    pub fn update_active_connections(&self, node_id: &str, pool_type: &str, active: f64, idle: f64) {
        self.active_connections
            .with_label_values(&[node_id, pool_type, "active"])
            .set(active);
        
        self.active_connections
            .with_label_values(&[node_id, pool_type, "idle"])
            .set(idle);
    }
    
    pub fn record_cache_hit(&self, node_id: &str, cache_type: &str) {
        self.validation_cache_hits
            .with_label_values(&[node_id, cache_type])
            .inc();
    }
    
    pub fn record_cache_miss(&self, node_id: &str, cache_type: &str) {
        self.validation_cache_misses
            .with_label_values(&[node_id, cache_type])
            .inc();
    }
    
    pub fn update_cache_memory(&self, node_id: &str, cache_type: &str, bytes: f64) {
        self.cache_memory_usage
            .with_label_values(&[node_id, cache_type])
            .set(bytes);
    }
    
    pub fn record_cache_eviction(&self, node_id: &str, cache_type: &str, reason: &str) {
        self.cache_evictions
            .with_label_values(&[node_id, cache_type, reason])
            .inc();
    }
    
    pub fn record_task_created(&self, swarm_id: &str, task_type: &str, priority: &str) {
        self.tasks_created
            .with_label_values(&[swarm_id, task_type, priority])
            .inc();
    }
    
    pub fn record_task_completed(&self, swarm_id: &str, task_type: &str, status: &str, latency: f64) {
        self.tasks_completed
            .with_label_values(&[swarm_id, task_type, status])
            .inc();
        
        self.task_latency
            .with_label_values(&[swarm_id, task_type])
            .observe(latency);
    }
    
    pub fn update_agent_utilization(&self, swarm_id: &str, agent_id: &str, agent_type: &str, utilization: f64) {
        self.agent_utilization
            .with_label_values(&[swarm_id, agent_id, agent_type])
            .set(utilization);
    }
    
    pub fn update_queue_depth(&self, swarm_id: &str, queue_type: &str, priority: &str, depth: f64) {
        self.swarm_queue_depth
            .with_label_values(&[swarm_id, queue_type, priority])
            .set(depth);
    }
    
    pub fn record_message(&self, node_id: &str, message_type: &str, direction: &str) {
        self.message_throughput
            .with_label_values(&[node_id, message_type, direction])
            .inc();
    }
    
    pub fn record_error(&self, node_id: &str, component: &str, error_type: &str) {
        self.error_rate
            .with_label_values(&[node_id, component, error_type])
            .inc();
    }
    
    pub fn update_system_metrics(&self, node_id: &str, memory_used: f64, memory_total: f64, cpu_percent: f64) {
        self.system_memory_usage
            .with_label_values(&[node_id, "used"])
            .set(memory_used);
        
        self.system_memory_usage
            .with_label_values(&[node_id, "total"])
            .set(memory_total);
        
        self.cpu_usage
            .with_label_values(&[node_id, "average"])
            .set(cpu_percent);
    }
}

// Performance tracking timer utility
pub struct MetricTimer {
    start: Instant,
}

impl MetricTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
    
    pub fn elapsed_seconds(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    
    #[test]
    fn test_metrics_collector_creation() {
        let registry = Registry::new();
        let collector = MetricsCollector::new(&registry).unwrap();
        
        // Test recording some metrics
        collector.record_chunk_processed("node1", "data", 0.75);
        collector.record_connection_pool_hit("node1", "tcp");
        collector.record_cache_hit("node1", "signature");
        collector.record_task_created("swarm1", "research", "high");
        
        // Verify metrics are recorded
        let metric_families = registry.gather();
        assert!(!metric_families.is_empty());
    }
}