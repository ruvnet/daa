# /performance-benchmark

## Purpose
Execute comprehensive performance benchmarks across QuDAG modules and generate detailed performance reports with regression detection and optimization recommendations.

## Parameters
- `[module]`: Module to benchmark specifically - optional, defaults to "all"
  - Values: `crypto`, `dag`, `network`, `protocol`, `all`
- `[criteria]`: Performance criteria to focus on - optional
  - Values: `throughput`, `latency`, `memory`, `cpu`, `scalability`
- `[--baseline]`: Compare against baseline benchmarks - optional, defaults to true
- `[--profile]`: Generate detailed profiling data - optional, defaults to false

## Prerequisites
- [ ] Rust toolchain with cargo-bench installed
- [ ] Baseline benchmarks exist in `.claude/performance_history.json`
- [ ] No other benchmarks currently running
- [ ] System resources available for benchmarking

## Execution Steps

### 1. Validation Phase
- Verify benchmark environment is clean
- Check system resources (CPU, memory, disk)
- Validate module parameter if provided
- Ensure no conflicting processes running

### 2. Planning Phase
- Analyze requested benchmark scenarios
- Load baseline metrics for comparison
- Calculate expected runtime based on criteria
- Prepare benchmark configuration

### 3. Implementation Phase
- Step 3.1: Run core benchmarks
  ```bash
  # Run module-specific benchmarks
  cargo bench -p qudag-${module} --features bench
  
  # Example for crypto module
  cargo bench -p qudag-crypto --bench ml_kem_bench
  cargo bench -p qudag-crypto --bench ml_dsa_bench
  ```

- Step 3.2: Collect performance metrics
  - Throughput: messages/second, operations/second
  - Latency: p50, p95, p99, max response times
  - Memory: heap usage, peak memory, allocations/sec
  - Scalability: linear factor, saturation point

- Step 3.3: Generate profiling data (if --profile enabled)
  ```bash
  # CPU profiling with flamegraph
  cargo flamegraph --bench ${benchmark_name} -o .claude/flamegraphs/${module}_flame.svg
  
  # Memory profiling
  heaptrack cargo bench -p qudag-${module}
  ```

### 4. Verification Phase
- Compare results against baseline (±5% threshold)
- Detect performance regressions
- Identify optimization opportunities
- Validate benchmark consistency

### 5. Documentation Phase
- Generate performance report at `/workspaces/QuDAG/PERFORMANCE_REPORT.md`
- Update regression tracking in `.claude/performance_history.json`
- Create detailed analysis in `.claude/reports/performance_detailed.json`
- Log optimization recommendations

## Success Criteria
- [ ] All benchmarks complete without errors
- [ ] No critical regressions detected (>10% degradation)
- [ ] Performance targets met:
  - Throughput: >10,000 messages/second baseline
  - Latency: p99 <1s, max <5s
  - Memory: Base node <100MB, Full node <500MB
  - Scalability: Linear factor >0.8 up to 1000 nodes
- [ ] Reports generated in all specified formats
- [ ] Baseline comparison data updated

## Error Handling
- **Benchmark Failure**: Capture error logs, check resource constraints, retry with reduced load
- **Resource Exhaustion**: Monitor system metrics, terminate gracefully, report partial results
- **Invalid Criteria**: Display valid options: throughput, latency, memory, cpu, scalability
- **Regression Detected**: Generate detailed regression report, suggest rollback commits

## Output
- **Success**: 
  ```
  ✅ Performance benchmarks completed successfully
  
  Summary:
  - Throughput: 45,232 msg/s (+12.3% vs baseline)
  - Latency p99: 0.823s (-5.2% vs baseline)
  - Memory usage: 87MB (within target)
  - No regressions detected
  
  Full report: /workspaces/QuDAG/PERFORMANCE_REPORT.md
  ```

- **Failure**: Error details with failed benchmarks and recovery suggestions
- **Reports**: 
  - Summary: `/workspaces/QuDAG/PERFORMANCE_REPORT.md`
  - Detailed: `.claude/reports/performance_detailed.json`
  - Regression: `.claude/reports/regression_analysis.md`

## Example Usage
```
/performance-benchmark
/performance-benchmark crypto
/performance-benchmark network --criteria throughput
/performance-benchmark dag --criteria scalability --profile
/performance-benchmark --baseline --profile
```

### Example Scenario
Running crypto module benchmarks with profiling:
```
/performance-benchmark crypto --profile

Expected output:
- ML-KEM operations: 125,000 ops/s
- ML-DSA signatures: 85,000 ops/s
- Memory usage: 42MB average
- Flamegraph generated at .claude/flamegraphs/crypto_flame.svg
```

## Related Commands
- `/debug-performance`: Deep dive into specific bottlenecks
- `/refactor-optimize`: Apply optimizations based on benchmark results
- `/security-audit`: Ensure optimizations don't compromise security

## Workflow Integration
This command is part of the Performance Optimization workflow and:
- Follows: `/tdd-cycle` (after feature implementation)
- Precedes: `/refactor-optimize` (provides data for optimization)
- Can be run in parallel with: `/integration-test`

## Agent Coordination
- **Primary Agent**: Performance Agent
  - Executes benchmarks and analyzes results
- **Supporting Agents**: 
  - Crypto Agent: Validates cryptographic performance
  - Network Agent: Tests network throughput scenarios
  - Integration Agent: Ensures benchmark isolation

## Notes
- Benchmarks should run on isolated system for consistency
- Results may vary based on system load and configuration
- Always compare against baseline for regression detection
- Profile data generation significantly increases runtime
- Consider running overnight for comprehensive analysis

---

# Benchmark Scenarios

## Throughput Benchmarks
- Message processing rate under load
- Transaction validation speed
- Cryptographic operation throughput
- Network packet handling capacity

## Latency Benchmarks
- End-to-end message latency
- Consensus finalization time
- Cryptographic operation response time
- Network round-trip measurements

## Memory Benchmarks
- Steady-state memory usage
- Memory growth under load
- Allocation/deallocation patterns
- Cache efficiency metrics

## Scalability Benchmarks
- Performance with increasing node count
- Message propagation with network size
- Consensus scalability limits
- Resource usage growth patterns