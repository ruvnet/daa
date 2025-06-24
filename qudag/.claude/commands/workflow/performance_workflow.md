# Performance Workflow

## Steps

### 1. Baseline
- Run benchmarks
- Record metrics
- Set targets
- Profile hotspots

### 2. Optimization
- Analyze bottlenecks
- Implement changes
- Verify correctness
- Measure impact

### 3. Validation
- Compare metrics
- Check regressions
- Verify scaling
- Document gains

### 4. Monitoring
- Track metrics
- Alert regressions
- Monitor resources
- Log anomalies

## Decision Gates
- Performance improved
- No regressions
- Tests still pass
- Resources in budget

## Success Criteria
- 10k+ msg/s throughput
- Sub-second consensus
- <100MB memory
- Linear scaling

## Example
```rust
// Performance optimization
#[cfg(test)]
mod benchmarks {
    use criterion::{criterion_group, criterion_main, Criterion};

    pub fn consensus_benchmark(c: &mut Criterion) {
        c.bench_function("consensus_finality", |b| {
            b.iter(|| {
                // Measure consensus latency
                let start = Instant::now();
                consensus.finalize()?;
                let elapsed = start.elapsed();
                
                // Target: <1s finality
                assert!(elapsed < Duration::from_secs(1));
            })
        });
    }
}