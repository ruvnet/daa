# Performance Context

## Purpose
Track performance metrics and benchmarks

## Schema
```json
{
  "performance_metrics": {
    "throughput": {
      "messages_per_second": 0,
      "target": 10000,
      "last_benchmark": null
    },
    "latency": {
      "consensus_finality_ms": 0,
      "target": 1000,
      "percentile_99": 0
    },
    "resources": {
      "memory_mb": 0,
      "target": 100,
      "cpu_usage": 0
    },
    "scalability": {
      "node_count": 0,
      "throughput_ratio": 0,
      "latency_increase": 0
    }
  }
}
```

## Update Protocol
1. Run performance benchmarks
2. Record metric values
3. Compare against targets
4. Flag regressions

## Access Patterns
- Pre-optimization baseline
- Post-optimization comparison
- Regression detection
- Scalability analysis