# QuDAG Performance Monitoring System

Production-ready monitoring system for tracking and optimizing QuDAG performance across all components.

## Features

- **Comprehensive Metrics**: 20+ metrics covering all optimized components
- **Real-time Dashboards**: Grafana dashboards with 5-second refresh
- **Smart Alerting**: Multi-level alerts with anomaly detection
- **Performance Baselines**: Automatic baseline tracking and regression detection
- **Integration Ready**: Easy integration with existing components

## Quick Start

### 1. Deploy Monitoring Stack

```bash
cd /workspaces/QuDAG/core/monitoring
./deploy-monitoring.sh
```

### 2. Integrate with QuDAG

```rust
use qudag::monitoring::MonitoringSystem;

// Start monitoring on port 9090
let monitoring = MonitoringSystem::new(9090)?;
monitoring.start().await?;
```

### 3. Access Dashboards

- **Grafana**: http://localhost:3001 (admin/admin)
- **Prometheus**: http://localhost:9091
- **Alertmanager**: http://localhost:9093

## Architecture

```
QuDAG Components
      ↓
Metrics Collector (Port 9090)
      ↓
Prometheus Server → Grafana Dashboards
      ↓
Alertmanager → Notifications
```

## Key Metrics

### Performance Metrics
- Message throughput (msgs/sec)
- Chunk compression ratio
- Connection pool hit rate
- Validation cache efficiency
- Task execution latency

### Resource Metrics
- CPU usage
- Memory consumption
- Active connections
- Queue depths

### Health Metrics
- Error rates
- Component availability
- Performance regressions

## Monitoring Components

| File | Purpose |
|------|---------|
| `metrics_collector.rs` | Core metrics collection |
| `exporter.rs` | Prometheus endpoint |
| `logger.rs` | Structured logging |
| `integration.rs` | Component integration |
| `dashboard_config.json` | Grafana dashboard |
| `alerting_rules.yaml` | Prometheus alerts |

## Integration Guide

### Monitor a Component

```rust
use qudag::monitoring::{MonitoredComponent, MetricsCollector};

impl MonitoredComponent for MyComponent {
    fn register_metrics(&self, collector: Arc<MetricsCollector>) {
        // Register component metrics
    }
}
```

### Record Custom Metrics

```rust
// Time an operation
let timer = MetricTimer::new();
process_data().await?;
metrics.record_operation_time("process", timer.elapsed_seconds());

// Track errors
metrics.record_error("node1", "processor", "timeout");
```

## Performance Baselines

| Metric | Normal | Warning | Critical |
|--------|--------|---------|----------|
| Throughput | 100-1000 msgs/s | <10 | 0 |
| Cache Hit Rate | 70-95% | <70% | - |
| Task Latency P95 | 5-30s | >30s | >60s |
| CPU Usage | 20-70% | >80% | >90% |

## Troubleshooting

See [MONITORING_GUIDE.md](MONITORING_GUIDE.md) for detailed troubleshooting steps.

## Maintenance

- **Daily**: Review critical alerts
- **Weekly**: Analyze performance trends
- **Monthly**: Update baselines and thresholds

## Docker Commands

```bash
# View logs
docker-compose logs -f

# Stop monitoring
docker-compose down

# Clean up data
docker-compose down -v
```