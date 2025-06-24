use crate::monitoring::{MetricsCollector, MetricTimer, LogContext, StructuredLogger};
use std::sync::Arc;

/// Integration trait for components to report metrics
pub trait MonitoredComponent {
    fn register_metrics(&self, collector: Arc<MetricsCollector>);
    fn set_logger(&mut self, logger: Arc<StructuredLogger>);
}

/// Example integration for ChunkedMessageProcessor
pub struct MonitoredChunkedProcessor {
    metrics: Arc<MetricsCollector>,
    logger: Arc<StructuredLogger>,
}

impl MonitoredChunkedProcessor {
    pub fn process_message(&self, node_id: &str, message: &[u8]) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let timer = MetricTimer::new();
        let message_size = message.len();
        
        // Process message (simulated)
        let chunks = self.chunk_message(message)?;
        let compression_ratio = self.calculate_compression_ratio(&chunks, message_size);
        
        // Record metrics
        self.metrics.record_chunking_time(node_id, message_size, timer.elapsed_seconds());
        self.metrics.record_chunk_processed(node_id, "data", compression_ratio);
        
        // Log performance
        self.logger.log_optimization(
            "message_chunking",
            message_size as f64,
            chunks.iter().map(|c| c.len()).sum::<usize>() as f64,
            LogContext::new()
                .with_node(node_id.to_string())
                .with_component("chunking".to_string())
        );
        
        Ok(chunks)
    }
    
    fn chunk_message(&self, message: &[u8]) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        // Chunking implementation
        Ok(vec![message.to_vec()])
    }
    
    fn calculate_compression_ratio(&self, chunks: &[Vec<u8>], original_size: usize) -> f64 {
        let compressed_size: usize = chunks.iter().map(|c| c.len()).sum();
        1.0 - (compressed_size as f64 / original_size as f64)
    }
}

/// Example integration for ConnectionPool
pub struct MonitoredConnectionPool {
    metrics: Arc<MetricsCollector>,
    logger: Arc<StructuredLogger>,
}

impl MonitoredConnectionPool {
    pub async fn get_connection(&self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let timer = MetricTimer::new();
        
        // Try to get from pool (simulated)
        let hit = self.try_get_from_pool().await;
        
        if hit {
            self.metrics.record_connection_pool_hit(node_id, "tcp");
        } else {
            let wait_time = timer.elapsed_seconds();
            self.metrics.record_connection_pool_miss(node_id, "tcp", wait_time);
            
            if wait_time > 1.0 {
                self.logger.warn(
                    &format!("Long connection wait time: {:.2}s", wait_time),
                    LogContext::new()
                        .with_node(node_id.to_string())
                        .with_component("connection_pool".to_string())
                );
            }
        }
        
        // Update pool stats
        self.update_pool_stats(node_id).await;
        
        Ok(())
    }
    
    async fn try_get_from_pool(&self) -> bool {
        // Simulated pool check
        true
    }
    
    async fn update_pool_stats(&self, node_id: &str) {
        // Get current pool stats (simulated)
        let active = 10.0;
        let idle = 5.0;
        
        self.metrics.update_active_connections(node_id, "tcp", active, idle);
    }
}

/// Example integration for ValidationCache  
pub struct MonitoredValidationCache {
    metrics: Arc<MetricsCollector>,
    logger: Arc<StructuredLogger>,
}

impl MonitoredValidationCache {
    pub fn get(&self, node_id: &str, key: &str) -> Option<bool> {
        // Try cache lookup (simulated)
        let hit = self.cache_lookup(key);
        
        if hit.is_some() {
            self.metrics.record_cache_hit(node_id, "signature");
        } else {
            self.metrics.record_cache_miss(node_id, "signature");
        }
        
        // Periodically update memory stats
        if rand::random::<f64>() < 0.01 {
            self.update_memory_stats(node_id);
        }
        
        hit
    }
    
    pub fn put(&self, node_id: &str, key: &str, value: bool) {
        // Add to cache (simulated)
        self.cache_insert(key, value);
        
        // Check if eviction needed
        if self.needs_eviction() {
            self.metrics.record_cache_eviction(node_id, "signature", "memory_pressure");
            self.evict_oldest();
        }
    }
    
    fn cache_lookup(&self, _key: &str) -> Option<bool> {
        // Simulated lookup
        Some(true)
    }
    
    fn cache_insert(&self, _key: &str, _value: bool) {
        // Simulated insert
    }
    
    fn needs_eviction(&self) -> bool {
        // Simulated check
        false
    }
    
    fn evict_oldest(&self) {
        // Simulated eviction
    }
    
    fn update_memory_stats(&self, node_id: &str) {
        // Get current memory usage (simulated)
        let memory_bytes = 1024.0 * 1024.0 * 50.0; // 50MB
        
        self.metrics.update_cache_memory(node_id, "signature", memory_bytes);
        
        // Log cache performance
        let hit_rate = 0.85;
        self.logger.log_cache_performance(
            "signature", 
            hit_rate, 
            memory_bytes / 1024.0 / 1024.0,
            LogContext::new()
                .with_node(node_id.to_string())
                .with_component("cache".to_string())
        );
    }
}

/// Example integration for SwarmCoordinator
pub struct MonitoredSwarmCoordinator {
    metrics: Arc<MetricsCollector>,
    logger: Arc<StructuredLogger>,
}

impl MonitoredSwarmCoordinator {
    pub fn create_task(&self, swarm_id: &str, task_type: &str, priority: &str) {
        self.metrics.record_task_created(swarm_id, task_type, priority);
        
        // Update queue depth
        self.update_queue_metrics(swarm_id);
    }
    
    pub fn complete_task(&self, swarm_id: &str, task_type: &str, status: &str, start_time: std::time::Instant) {
        let latency = start_time.elapsed().as_secs_f64();
        
        self.metrics.record_task_completed(swarm_id, task_type, status, latency);
        
        // Check for latency anomalies
        if latency > 60.0 {
            self.logger.log_performance_anomaly(
                "task_latency",
                30.0,
                latency,
                LogContext::new()
                    .with_component("swarm".to_string())
                    .with_metadata("swarm_id".to_string(), swarm_id.into())
                    .with_metadata("task_type".to_string(), task_type.into())
            );
        }
        
        // Update queue depth
        self.update_queue_metrics(swarm_id);
    }
    
    pub fn update_agent_metrics(&self, swarm_id: &str, agent_id: &str, agent_type: &str, utilization: f64) {
        self.metrics.update_agent_utilization(swarm_id, agent_id, agent_type, utilization);
    }
    
    fn update_queue_metrics(&self, swarm_id: &str) {
        // Get current queue stats (simulated)
        let high_priority = 5.0;
        let normal_priority = 15.0;
        let low_priority = 30.0;
        
        self.metrics.update_queue_depth(swarm_id, "task", "high", high_priority);
        self.metrics.update_queue_depth(swarm_id, "task", "normal", normal_priority);
        self.metrics.update_queue_depth(swarm_id, "task", "low", low_priority);
        
        // Log coordination status
        self.logger.log_swarm_coordination(
            swarm_id,
            8, // active agents
            50, // queue depth
            250.0, // avg latency ms
            LogContext::new()
                .with_component("swarm".to_string())
        );
    }
}

/// System-wide monitoring updater
pub struct SystemMonitor {
    metrics: Arc<MetricsCollector>,
    logger: Arc<StructuredLogger>,
}

impl SystemMonitor {
    pub fn new(metrics: Arc<MetricsCollector>, logger: Arc<StructuredLogger>) -> Self {
        Self { metrics, logger }
    }
    
    pub async fn update_system_metrics(&self, node_id: &str) {
        // Get system stats (would use sysinfo crate in real implementation)
        let memory_used = 4.0 * 1024.0 * 1024.0 * 1024.0; // 4GB
        let memory_total = 16.0 * 1024.0 * 1024.0 * 1024.0; // 16GB
        let cpu_percent = 45.0;
        
        self.metrics.update_system_metrics(node_id, memory_used, memory_total, cpu_percent);
    }
    
    pub fn record_error(&self, node_id: &str, component: &str, error: &dyn std::error::Error) {
        let error_type = self.classify_error(error);
        
        self.metrics.record_error(node_id, component, &error_type);
        
        self.logger.error(
            &format!("Error in {}: {}", component, error),
            LogContext::new()
                .with_node(node_id.to_string())
                .with_component(component.to_string())
                .with_metadata("error_type".to_string(), error_type.into())
        );
    }
    
    fn classify_error(&self, error: &dyn std::error::Error) -> String {
        // Simple error classification
        if error.to_string().contains("connection") {
            "connection_error".to_string()
        } else if error.to_string().contains("timeout") {
            "timeout_error".to_string()
        } else if error.to_string().contains("validation") {
            "validation_error".to_string()
        } else {
            "unknown_error".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    
    #[tokio::test]
    async fn test_monitoring_integration() {
        let registry = Registry::new();
        let metrics = Arc::new(MetricsCollector::new(&registry).unwrap());
        let logger = Arc::new(StructuredLogger::new());
        
        // Test chunked processor monitoring
        let processor = MonitoredChunkedProcessor {
            metrics: Arc::clone(&metrics),
            logger: Arc::clone(&logger),
        };
        
        let result = processor.process_message("node1", b"test message").unwrap();
        assert!(!result.is_empty());
        
        // Test connection pool monitoring
        let pool = MonitoredConnectionPool {
            metrics: Arc::clone(&metrics),
            logger: Arc::clone(&logger),
        };
        
        pool.get_connection("node1").await.unwrap();
        
        // Test cache monitoring
        let cache = MonitoredValidationCache {
            metrics: Arc::clone(&metrics),
            logger: Arc::clone(&logger),
        };
        
        cache.get("node1", "test_key");
        cache.put("node1", "test_key", true);
        
        // Verify metrics were recorded
        let metric_families = registry.gather();
        assert!(!metric_families.is_empty());
    }
}