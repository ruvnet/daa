# Crypto Performance Benchmarks

Comprehensive performance testing and analysis for QuDAG quantum-resistant cryptography.

## Individual Algorithm Benchmarks

### ML-DSA Performance
```bash
# ML-DSA key generation benchmarks
cargo bench --bench ml_dsa_keygen

# ML-DSA signing benchmarks
cargo bench --bench ml_dsa_sign -- --warm-up-time 5 --measurement-time 30

# ML-DSA verification benchmarks
cargo bench --bench ml_dsa_verify -- --sample-size 1000

# ML-DSA batch operations
cargo bench --bench ml_dsa_batch_operations

# ML-DSA security level comparison
cargo bench --bench ml_dsa_security_levels -- --output-format json > ./benchmarks/ml-dsa-levels.json
```

### ML-KEM Performance
```bash
# ML-KEM key generation benchmarks
cargo bench --bench ml_kem_keygen

# ML-KEM encapsulation benchmarks
cargo bench --bench ml_kem_encapsulate -- --warm-up-time 5

# ML-KEM decapsulation benchmarks
cargo bench --bench ml_kem_decapsulate -- --sample-size 1000

# ML-KEM key exchange protocol
cargo bench --bench ml_kem_key_exchange

# ML-KEM parameter comparison
cargo bench --bench ml_kem_parameters -- --output-format json > ./benchmarks/ml-kem-params.json
```

### HQC Performance
```bash
# HQC key generation benchmarks
cargo bench --bench hqc_keygen

# HQC encapsulation benchmarks
cargo bench --bench hqc_encapsulate

# HQC decapsulation benchmarks
cargo bench --bench hqc_decapsulate

# HQC error correction performance
cargo bench --bench hqc_error_correction

# HQC security level analysis
cargo bench --bench hqc_security_analysis -- --output-format json > ./benchmarks/hqc-analysis.json
```

### Quantum Fingerprints Performance
```bash
# Fingerprint generation benchmarks
cargo bench --bench fingerprint_generation

# Fingerprint verification benchmarks
cargo bench --bench fingerprint_verification

# Fingerprint comparison benchmarks
cargo bench --bench fingerprint_comparison

# Large data fingerprinting
cargo bench --bench fingerprint_large_data

# Quantum property analysis performance
cargo bench --bench quantum_analysis_performance
```

## Comparative Analysis

### Algorithm Comparison
```bash
# Compare all signature schemes
cargo bench --bench signature_comparison -- --output-format json > ./benchmarks/signature-comparison.json

# Compare all KEM schemes
cargo bench --bench kem_comparison -- --output-format json > ./benchmarks/kem-comparison.json

# Compare with classical cryptography (if available)
cargo bench --bench classical_vs_postquantum

# Performance vs security trade-off analysis
cargo bench --bench performance_security_tradeoff -- --output-format json > ./benchmarks/tradeoff-analysis.json
```

### Cross-Platform Benchmarks
```bash
# x86_64 benchmarks
cargo bench --target x86_64-unknown-linux-gnu -- --output-format json > ./benchmarks/x86_64-results.json

# ARM64 benchmarks
cargo bench --target aarch64-unknown-linux-gnu -- --output-format json > ./benchmarks/arm64-results.json

# WebAssembly benchmarks
cargo bench --target wasm32-unknown-unknown -- --output-format json > ./benchmarks/wasm-results.json

# Compare cross-platform performance
cargo run --bin compare-platforms -- \
  --results ./benchmarks/x86_64-results.json,./benchmarks/arm64-results.json \
  --output ./benchmarks/platform-comparison.json
```

### Compiler Optimization Analysis
```bash
# Debug build performance
cargo bench --profile dev -- --output-format json > ./benchmarks/debug-performance.json

# Release build performance
cargo bench --profile release -- --output-format json > ./benchmarks/release-performance.json

# Optimized build performance
RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo bench -- --output-format json > ./benchmarks/optimized-performance.json

# LTO (Link Time Optimization) analysis
cargo bench --config 'profile.release.lto=true' -- --output-format json > ./benchmarks/lto-performance.json
```

## Memory Usage Profiling

### Memory Allocation Analysis
```bash
# Heap allocation profiling
cargo bench --bench crypto_memory -- --features heap-profiling

# Stack usage analysis
cargo run --bin stack-analyzer -- \
  --crypto-operations ml-dsa,ml-kem,hqc \
  --output ./profiles/stack-usage.json

# Memory fragmentation analysis
cargo run --bin memory-fragmentation -- \
  --test-duration 3600s \
  --operations keygen,sign,verify,encapsulate,decapsulate \
  --output ./profiles/fragmentation-analysis.json
```

### Memory Performance Benchmarks
```bash
# Memory bandwidth usage
cargo bench --bench memory_bandwidth

# Cache performance analysis
cargo bench --bench cache_performance -- --features cache-profiling

# Memory latency benchmarks
cargo bench --bench memory_latency

# NUMA (Non-Uniform Memory Access) analysis
cargo run --bin numa-analyzer -- \
  --crypto-operations all \
  --output ./profiles/numa-analysis.json
```

## Scalability Testing

### Throughput Analysis
```bash
# Single-threaded throughput
cargo bench --bench single_thread_throughput

# Multi-threaded throughput
cargo bench --bench multi_thread_throughput -- --threads 1,2,4,8,16

# Async/await performance
cargo bench --bench async_crypto_performance

# Batch operation scaling
cargo run --bin batch-scaling -- \
  --batch-sizes 1,10,100,1000,10000 \
  --operations sign,verify,encapsulate,decapsulate \
  --output ./benchmarks/batch-scaling.json
```

### Concurrent Performance
```bash
# Concurrent crypto operations
cargo bench --bench concurrent_crypto

# Lock contention analysis
cargo run --bin lock-contention -- \
  --concurrent-operations 100 \
  --thread-counts 1,2,4,8,16 \
  --output ./profiles/lock-contention.json

# Thread pool optimization
cargo bench --bench thread_pool_optimization
```

### Network Performance Impact
```bash
# Network latency simulation
cargo run --bin network-crypto-bench -- \
  --latencies 1ms,10ms,50ms,100ms,500ms \
  --bandwidths 1Mbps,10Mbps,100Mbps,1Gbps \
  --output ./benchmarks/network-impact.json

# Protocol overhead analysis
cargo run --bin protocol-overhead -- \
  --crypto-protocols ml-dsa,ml-kem,hqc \
  --message-sizes 64,256,1024,4096,16384 \
  --output ./benchmarks/protocol-overhead.json
```

## Real-World Performance Testing

### Application-Level Benchmarks
```bash
# Transaction signing performance
cargo run --bin transaction-bench -- \
  --transaction-types simple,complex,batch \
  --signing-algorithms ml-dsa-65,ml-dsa-87 \
  --transactions-per-second 1000 \
  --duration 300s \
  --output ./benchmarks/transaction-performance.json

# Node authentication performance
cargo run --bin auth-bench -- \
  --authentication-methods ml-kem,hqc \
  --concurrent-authentications 100 \
  --output ./benchmarks/auth-performance.json

# Key exchange performance in network protocols
cargo run --bin network-key-exchange-bench -- \
  --protocols tcp,udp,quic \
  --key-exchange-methods ml-kem-768,hqc-128 \
  --output ./benchmarks/network-key-exchange.json
```

### Load Testing
```bash
# Sustained load testing
cargo run --bin sustained-load-test -- \
  --crypto-operations all \
  --load-percentage 80 \
  --duration 1h \
  --output ./benchmarks/sustained-load.json

# Burst load testing
cargo run --bin burst-load-test -- \
  --burst-sizes 10,100,1000 \
  --burst-intervals 1s,5s,10s \
  --crypto-operations sign,verify \
  --output ./benchmarks/burst-load.json

# Stress testing
cargo run --bin stress-test -- \
  --max-operations-per-second 10000 \
  --ramp-up-time 60s \
  --steady-state-time 300s \
  --output ./benchmarks/stress-test.json
```

## Performance Optimization

### Optimization Analysis
```bash
# Identify performance bottlenecks
cargo run --bin perf-bottleneck-analyzer -- \
  --crypto-module qudag-crypto \
  --profile-time 300s \
  --output ./analysis/bottlenecks.json

# Optimization opportunity identification
cargo run --bin optimization-finder -- \
  --baseline ./benchmarks/current-performance.json \
  --target-improvement 20% \
  --output ./analysis/optimization-opportunities.json

# Performance regression detection
cargo run --bin regression-detector -- \
  --baseline ./benchmarks/baseline.json \
  --current ./benchmarks/current.json \
  --threshold 5% \
  --output ./analysis/regressions.json
```

### Tuning Parameters
```bash
# Automatic parameter tuning
cargo run --bin auto-tune -- \
  --target-metric throughput \
  --constraints memory<1GB,latency<100ms \
  --output ./config/optimized-params.json

# Parameter sensitivity analysis
cargo run --bin param-sensitivity -- \
  --parameters quantum-bits,entanglement-depth,batch-size \
  --ranges 128-512,4-16,10-1000 \
  --output ./analysis/param-sensitivity.json

# Performance parameter optimization
cargo run --bin param-optimizer -- \
  --algorithm genetic \
  --generations 100 \
  --population-size 50 \
  --output ./config/optimal-params.json
```

## Benchmarking Infrastructure

### Benchmark Automation
```bash
# Automated benchmark suite
./scripts/run-all-benchmarks.sh

# Continuous benchmarking setup
cargo run --bin setup-continuous-benchmarking -- \
  --schedule "0 3 * * *" \
  --benchmarks crypto-full-suite \
  --baseline-update weekly \
  --report-webhook https://perf-webhook.example.com

# Performance CI/CD integration
cargo run --bin benchmark-ci -- \
  --ci-system github-actions \
  --performance-gate 95% \
  --output ./ci/benchmark-config.yml
```

### Benchmark Data Management
```bash
# Historical performance tracking
cargo run --bin perf-history -- \
  --add-results ./benchmarks/current-results.json \
  --database ./benchmarks/performance-history.db

# Performance trend analysis
cargo run --bin trend-analyzer -- \
  --history-database ./benchmarks/performance-history.db \
  --period 6months \
  --output ./analysis/performance-trends.json

# Benchmark result comparison
cargo run --bin compare-benchmarks -- \
  --results ./benchmarks/v1.0.json,./benchmarks/v1.1.json,./benchmarks/v1.2.json \
  --output ./comparison/version-comparison.html
```

## Configuration Examples

### Benchmark Configuration
```json
{
  "benchmark_config": {
    "default_settings": {
      "warm_up_time": 5,
      "measurement_time": 30,
      "sample_size": 1000,
      "confidence_level": 0.95,
      "noise_threshold": 0.05
    },
    "crypto_operations": {
      "ml_dsa": {
        "security_levels": [65, 87],
        "message_sizes": [32, 256, 1024, 4096],
        "batch_sizes": [1, 10, 100, 1000]
      },
      "ml_kem": {
        "security_levels": [512, 768, 1024],
        "key_sizes": [800, 1184, 1568]
      },
      "hqc": {
        "security_levels": [128, 192, 256],
        "error_rates": [0.01, 0.02, 0.05]
      }
    },
    "platforms": ["x86_64", "aarch64", "wasm32"],
    "optimization_levels": ["debug", "release", "release-lto"]
  }
}
```

### Performance Targets
```json
{
  "performance_targets": {
    "ml_dsa_65": {
      "keygen_ops_per_sec": 1000,
      "sign_ops_per_sec": 5000,
      "verify_ops_per_sec": 10000,
      "memory_usage_mb": 10,
      "latency_ms": 1
    },
    "ml_kem_768": {
      "keygen_ops_per_sec": 2000,
      "encapsulate_ops_per_sec": 8000,
      "decapsulate_ops_per_sec": 8000,
      "memory_usage_mb": 5,
      "latency_ms": 0.5
    },
    "hqc_128": {
      "keygen_ops_per_sec": 500,
      "encapsulate_ops_per_sec": 1000,
      "decapsulate_ops_per_sec": 1000,
      "memory_usage_mb": 15,
      "latency_ms": 2
    }
  }
}
```

## Reporting and Visualization

### Performance Reports
```bash
# Generate comprehensive performance report
cargo run --bin generate-perf-report -- \
  --benchmark-results ./benchmarks/ \
  --format html,pdf,json \
  --output ./reports/performance-report

# Executive summary report
cargo run --bin executive-summary -- \
  --performance-data ./benchmarks/current-results.json \
  --baseline ./benchmarks/baseline.json \
  --output ./reports/executive-summary.pdf

# Technical deep-dive report
cargo run --bin technical-report -- \
  --benchmark-data ./benchmarks/ \
  --analysis-data ./analysis/ \
  --output ./reports/technical-deep-dive.html
```

### Performance Visualization
```bash
# Generate performance charts
cargo run --bin perf-charts -- \
  --data ./benchmarks/results.json \
  --chart-types line,bar,scatter,heatmap \
  --output ./charts/

# Interactive performance dashboard
cargo run --bin perf-dashboard -- \
  --data-source ./benchmarks/performance-history.db \
  --port 8080 \
  --real-time-updates

# Performance comparison visualization
cargo run --bin compare-visualization -- \
  --before ./benchmarks/v1.0-results.json \
  --after ./benchmarks/v1.1-results.json \
  --output ./visualizations/v1.0-vs-v1.1.html
```

## Development Workflows

### Performance Development Workflow
```bash
# Pre-development baseline
cargo bench -- --save-baseline before-changes

# Post-development comparison
cargo bench -- --baseline before-changes

# Performance validation workflow
./scripts/validate-performance.sh

# Manual performance workflow:
cargo bench --bench crypto_full_suite
cargo run --bin perf-analyzer -- --compare-with-baseline
cargo test performance_regression_tests --lib
```

### Optimization Workflow
```bash
# Performance profiling
cargo run --bin perf-profiler -- \
  --target crypto-operations \
  --duration 300s \
  --output ./profiles/performance-profile.json

# Optimization implementation
# ... make optimizations ...

# Optimization validation
cargo bench -- --baseline pre-optimization
cargo run --bin validate-optimization -- \
  --before ./profiles/pre-optimization.json \
  --after ./profiles/post-optimization.json
```

### Release Performance Validation
```bash
# Pre-release performance validation
cargo run --bin release-perf-validation -- \
  --release-candidate v1.2.0-rc1 \
  --baseline v1.1.0 \
  --acceptance-criteria ./config/perf-acceptance.json

# Performance sign-off
cargo run --bin perf-signoff -- \
  --validation-results ./validation/perf-validation.json \
  --sign-off-criteria ./config/signoff-criteria.json
```