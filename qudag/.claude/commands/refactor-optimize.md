# /refactor-optimize

## Purpose
Refactor and optimize QuDAG modules for improved performance, security, or maintainability while preserving test coverage and API compatibility.

## Parameters
- `<module>`: Module to refactor - required
  - Values: `crypto`, `dag`, `network`, `protocol`
- `[--focus]`: Optimization focus area - optional
  - Values: `performance`, `security`, `readability`, `maintainability`
- `[--strategies]`: Specific optimization strategies - optional, comma-separated
  - Values: `async_optimization`, `memory_reduction`, `lock_elimination`, `batch_processing`, `cache_locality`, `algorithmic_improvement`
- `[--preserve-api]`: Maintain existing public API - optional, defaults to true

## Prerequisites
- [ ] All tests passing for target module
- [ ] Current benchmarks recorded as baseline
- [ ] No uncommitted changes in module
- [ ] Performance profile data available (from `/debug-performance`)

## Execution Steps

### 1. Validation Phase
- Verify module exists and builds successfully
- Run current test suite and record coverage
- Capture baseline performance metrics
- Check API compatibility requirements

### 2. Planning Phase
- Analyze performance profile data
- Identify optimization opportunities
- Plan refactoring strategy based on focus
- Create rollback checkpoint

### 3. Implementation Phase
- Step 3.1: Create feature branch and baseline
  ```bash
  cd /workspaces/QuDAG
  git checkout -b refactor/${module}-optimization
  
  # Run baseline benchmarks
  cargo bench -p qudag-${module} > .claude/baselines/${module}_bench_before.txt
  
  # Check current test coverage
  cargo tarpaulin -p qudag-${module} --out Json > .claude/baselines/${module}_coverage_before.json
  ```

- Step 3.2: Apply optimization strategies

  **Async Optimization**:
  ```rust
  // Before: Blocking operations
  let result = expensive_computation();
  
  // After: Async with proper task spawning
  let result = tokio::spawn(async move {
      expensive_computation()
  }).await?;
  ```

  **Memory Reduction**:
  ```rust
  // Before: Frequent allocations
  let mut results = Vec::new();
  for item in items {
      results.push(process(item));
  }
  
  // After: Pre-allocated with capacity
  let mut results = Vec::with_capacity(items.len());
  for item in items {
      results.push(process(item));
  }
  ```

  **Lock Elimination**:
  ```rust
  // Before: Mutex-protected shared state
  let data = Arc::new(Mutex::new(HashMap::new()));
  
  // After: Lock-free with channels
  let (tx, rx) = mpsc::channel();
  // Use message passing instead
  ```

- Step 3.3: Maintain test coverage
  ```bash
  # Run tests after each significant change
  cargo test -p qudag-${module}
  
  # Ensure coverage doesn't drop
  cargo tarpaulin -p qudag-${module} --out Json > .claude/baselines/${module}_coverage_after.json
  ```

- Step 3.4: Update documentation and APIs
  - Update module documentation
  - Add optimization notes
  - Document any API changes
  - Update performance characteristics

### 4. Verification Phase
- Run complete test suite
- Execute performance benchmarks
- Compare against baseline metrics
- Validate API compatibility
- Run security audit if applicable

### 5. Documentation Phase
- Generate optimization report
- Document performance improvements
- Update module README
- Create migration guide if API changed

## Success Criteria
- [ ] All tests pass with ≥95% coverage maintained
- [ ] Performance improvements measurable:
  - Throughput: >10% improvement OR
  - Latency: >10% reduction OR
  - Memory: >20% reduction
- [ ] No security vulnerabilities introduced
- [ ] API compatibility maintained (if --preserve-api)
- [ ] Code complexity not significantly increased

## Error Handling
- **Invalid Module**: List valid modules: crypto, dag, network, protocol
- **Test Regression**: Rollback changes, analyze failing tests, fix incrementally
- **Performance Regression**: Compare profiles, identify regression cause, try alternative approach
- **API Breaking Change**: Use feature flags for gradual migration, provide compatibility layer

## Output
- **Success**: 
  ```
  ✅ Refactoring completed successfully
  
  Module: crypto
  Focus: performance
  
  Results:
  - Throughput: +23% (125k → 154k ops/s)
  - Memory usage: -18% (42MB → 34MB)
  - Test coverage: 96.2% (maintained)
  - API compatibility: Preserved
  
  Key optimizations:
  1. Async key generation with pooling
  2. Zero-copy signature verification
  3. SIMD-accelerated hashing
  
  Full report: .claude/reports/refactor_crypto_20240315.md
  ```

- **Failure**: Error details with rollback instructions
- **Reports**:
  - Optimization summary
  - Before/after metrics
  - API changes (if any)
  - Migration guide

## Example Usage
```
/refactor-optimize crypto
/refactor-optimize network --focus performance
/refactor-optimize dag --focus security
/refactor-optimize protocol --strategies async_optimization,memory_reduction
/refactor-optimize crypto --focus performance --preserve-api false
```

### Example Scenario
Optimizing the network module for better throughput:
```
/refactor-optimize network --focus performance --strategies async_optimization,batch_processing

Expected changes:
- Batch message processing for 3x throughput
- Async I/O with tokio for better concurrency
- Connection pooling to reduce overhead
- Zero-copy deserialization where possible
```

## Related Commands
- `/performance-benchmark`: Measure optimization impact
- `/debug-performance`: Identify optimization targets
- `/security-audit`: Verify security after refactoring
- `/integration-test`: Ensure system-wide compatibility

## Workflow Integration
This command is part of the Performance Optimization workflow and:
- Follows: `/debug-performance` (provides optimization targets)
- Precedes: `/performance-benchmark` (validates improvements)
- Can be run in parallel with: Documentation updates

## Agent Coordination
- **Primary Agent**: Performance Agent
  - Leads refactoring and optimization
- **Supporting Agents**: 
  - Code Quality Agent: Ensures maintainability
  - Security Agent: Validates security implications
  - Test Agent: Maintains coverage and correctness
  - API Agent: Tracks compatibility changes

## Notes
- Always create a feature branch before refactoring
- Incremental changes are safer than massive rewrites
- Profile-guided optimization yields best results
- Consider trade-offs between different optimization goals
- Document why specific optimizations were chosen

---

# Optimization Patterns

## Performance Patterns

### Async Optimization
- Use `tokio::spawn` for CPU-bound tasks
- Implement buffered channels for backpressure
- Apply `select!` for concurrent operations
- Utilize `FuturesUnordered` for parallel execution

### Memory Reduction
- Implement arena allocators for temporary data
- Use object pooling for frequent allocations
- Design compact data structures
- Apply lazy initialization patterns

### Lock Elimination
- Prefer lock-free data structures
- Implement RCU (Read-Copy-Update) patterns
- Use message passing over shared state
- Apply fine-grained locking when needed

## Quality Metrics

### Performance Metrics
- Throughput change (percentage)
- Latency change (percentiles)
- Memory usage change
- CPU utilization

### Code Quality Metrics
- Cyclomatic complexity (before/after)
- Code coverage percentage
- Count of unsafe blocks
- API surface changes