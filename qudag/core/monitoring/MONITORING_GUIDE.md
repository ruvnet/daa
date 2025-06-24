# QuDAG Performance Monitoring Guide

## Overview

The QuDAG monitoring system provides comprehensive performance tracking for all optimized components, including message chunking, connection pooling, validation caching, and swarm coordination.

## Architecture

```
┌─────────────────────┐
│   Application       │
│  (with monitored    │
│   components)       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Metrics Collector  │
│   (Prometheus)      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Prometheus Server   │
│  (scrapes metrics)  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Grafana Dashboard  │
│  (visualization)    │
└─────────────────────┘
```

## Quick Start

### 1. Start Monitoring System

```rust
use qudag::monitoring::{MonitoringSystem, MetricsCollector, StructuredLogger};

// Initialize monitoring on port 9090
let monitoring = MonitoringSystem::new(9090)?;
monitoring.start().await?;

// Get references for components
let metrics = monitoring.metrics.clone();
let logger = monitoring.logger.clone();
```

### 2. Integrate with Components

```rust
// Example: Monitor chunked message processing
let processor = MonitoredChunkedProcessor::new(metrics.clone(), logger.clone());

// Process message with automatic metric collection
let chunks = processor.process_message("node1", &message).await?;
```

### 3. Configure Prometheus

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'qudag'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
```

### 4. Import Grafana Dashboard

1. Open Grafana (http://localhost:3000)
2. Go to Dashboards → Import
3. Upload `/core/monitoring/dashboard_config.json`
4. Select Prometheus data source

## Metrics Reference

### Message Chunking Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `qudag_chunks_processed_total` | Counter | Total chunks processed | node_id, message_type |
| `qudag_compression_ratio` | Gauge | Current compression ratio | node_id, compression_type |
| `qudag_chunking_duration_seconds` | Histogram | Time to chunk messages | node_id, message_size_bucket |

### Connection Pool Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `qudag_connection_pool_hits_total` | Counter | Pool hit count | node_id, pool_type |
| `qudag_connection_pool_misses_total` | Counter | Pool miss count | node_id, pool_type |
| `qudag_active_connections` | Gauge | Active/idle connections | node_id, pool_type, state |
| `qudag_connection_wait_time_seconds` | Histogram | Wait time for connection | node_id, pool_type |

### Validation Cache Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `qudag_validation_cache_hits_total` | Counter | Cache hits | node_id, validation_type |
| `qudag_validation_cache_misses_total` | Counter | Cache misses | node_id, validation_type |
| `qudag_cache_memory_bytes` | Gauge | Cache memory usage | node_id, cache_type |
| `qudag_cache_evictions_total` | Counter | Cache evictions | node_id, cache_type, reason |

### Swarm Coordination Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `qudag_swarm_tasks_created_total` | Counter | Tasks created | swarm_id, task_type, priority |
| `qudag_swarm_tasks_completed_total` | Counter | Tasks completed | swarm_id, task_type, status |
| `qudag_swarm_task_latency_seconds` | Histogram | Task execution time | swarm_id, task_type |
| `qudag_agent_utilization_ratio` | Gauge | Agent utilization (0-1) | swarm_id, agent_id, agent_type |
| `qudag_swarm_queue_depth` | Gauge | Queue depth | swarm_id, queue_type, priority |

### System Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `qudag_message_throughput_total` | Counter | Messages processed | node_id, message_type, direction |
| `qudag_errors_total` | Counter | Error count | node_id, component, error_type |
| `qudag_system_memory_bytes` | Gauge | Memory usage | node_id, memory_type |
| `qudag_cpu_usage_percent` | Gauge | CPU usage | node_id, cpu_core |

## Performance Baselines

### Expected Performance Ranges

| Component | Metric | Normal Range | Warning | Critical |
|-----------|--------|--------------|---------|----------|
| Message Throughput | msgs/sec | 100-1000 | <10 | 0 |
| Chunk Compression | ratio | 0.6-0.9 | <0.5 | - |
| Connection Pool | hit rate | 0.8-0.95 | <0.8 | - |
| Validation Cache | hit rate | 0.7-0.95 | <0.7 | - |
| Task Latency | P95 seconds | 5-30 | >30 | >60 |
| CPU Usage | percent | 20-70 | >80 | >90 |
| Memory Usage | percent | 30-80 | >85 | >90 |

## Alert Configuration

### Critical Alerts

1. **Message Processing Stalled**: No messages processed for 2+ minutes
2. **Connection Pool Exhausted**: No idle connections available
3. **High Error Rate**: >0.1 errors/second
4. **High Memory Usage**: >90% memory utilization

### Warning Alerts

1. **Low Throughput**: <10 messages/second for 5+ minutes
2. **High Latency**: P95 latency >30s for tasks
3. **Low Cache Hit Rate**: <70% for 10+ minutes
4. **Performance Regression**: >20% degradation vs baseline

## Troubleshooting

### High Task Latency

1. Check `qudag_swarm_queue_depth` - high values indicate backlog
2. Review `qudag_agent_utilization_ratio` - low values suggest scaling needed
3. Examine `qudag_errors_total` for correlated errors

### Low Cache Hit Rate

1. Monitor `qudag_cache_evictions_total` for memory pressure
2. Check `qudag_cache_memory_bytes` usage patterns
3. Review cache configuration and TTL settings

### Connection Pool Issues

1. Track `qudag_connection_wait_time_seconds` for delays
2. Monitor `qudag_active_connections` distribution
3. Consider increasing pool size if consistently exhausted

### Memory Issues

1. Check `qudag_cache_memory_bytes` for cache bloat
2. Review `qudag_system_memory_bytes` trends
3. Analyze `qudag_chunks_processed_total` vs memory growth

## Custom Queries

### Average Compression Efficiency
```promql
avg(qudag_compression_ratio) by (node_id)
```

### Task Success Rate
```promql
rate(qudag_swarm_tasks_completed_total{status="success"}[5m]) / 
rate(qudag_swarm_tasks_completed_total[5m])
```

### Connection Pool Efficiency
```promql
sum(rate(qudag_connection_pool_hits_total[5m])) by (node_id) /
(sum(rate(qudag_connection_pool_hits_total[5m])) by (node_id) + 
 sum(rate(qudag_connection_pool_misses_total[5m])) by (node_id))
```

### Memory Pressure Score
```promql
(sum(qudag_cache_memory_bytes) + 
 sum(qudag_system_memory_bytes{memory_type="used"})) /
sum(qudag_system_memory_bytes{memory_type="total"})
```

## Best Practices

1. **Set Baselines**: Establish normal operating ranges during initial deployment
2. **Progressive Alerts**: Use multiple severity levels to avoid alert fatigue
3. **Correlate Metrics**: Look for patterns across related metrics
4. **Regular Reviews**: Weekly performance reviews to identify trends
5. **Capacity Planning**: Use historical data for scaling decisions

## Integration Examples

### Adding Monitoring to New Components

```rust
use qudag::monitoring::{MonitoredComponent, MetricsCollector, StructuredLogger};

pub struct MyComponent {
    metrics: Arc<MetricsCollector>,
    logger: Arc<StructuredLogger>,
}

impl MonitoredComponent for MyComponent {
    fn register_metrics(&self, collector: Arc<MetricsCollector>) {
        // Register component-specific metrics
    }
    
    fn set_logger(&mut self, logger: Arc<StructuredLogger>) {
        self.logger = logger;
    }
}
```

### Custom Metric Recording

```rust
// Time an operation
let timer = MetricTimer::new();
let result = expensive_operation().await?;
metrics.record_operation_time("my_operation", timer.elapsed_seconds());

// Track resource usage
metrics.update_resource_usage("my_resource", current_usage, max_capacity);

// Log performance events
logger.log_optimization(
    "query_optimization",
    old_duration,
    new_duration,
    LogContext::new()
        .with_component("database")
        .with_metadata("query_type", "aggregate")
);
```

## Maintenance

### Daily Tasks
- Review critical alerts
- Check system resource trends
- Verify metric collection health

### Weekly Tasks
- Analyze performance trends
- Update alert thresholds if needed
- Review and archive old logs

### Monthly Tasks
- Performance baseline updates
- Dashboard optimization
- Alert rule refinement
- Capacity planning review