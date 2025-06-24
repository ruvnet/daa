# QuDAG Performance Analysis Report

Generated: 2025-06-19 12:45:31

## Executive Summary

- Total benchmarks analyzed: 20
- Categories: QuDAG Dark Addressing
- Critical bottlenecks identified: 4
- Optimization opportunities: 12

## Performance Metrics by Category

### QuDAG Dark Addressing

- Benchmarks: 20
- Average time: 13810.5μs
- Total time: 276209.7μs
- Fastest operation: resolve_cache_hit
- Slowest operation: batch_resolve_10

## Performance Bottlenecks

| Operation | Category | Time (μs) | Impact |
|-----------|----------|-----------|---------|
| batch_resolve_10 | QuDAG Dark Addressing | 89300.0 | high |
| concurrent_resolution_4 | QuDAG Dark Addressing | 67200.0 | high |
| resolve_single_domain | QuDAG Dark Addressing | 52100.0 | high |
| resolve_cache_miss | QuDAG Dark Addressing | 48700.0 | high |

## Optimization Recommendations

### 1. High Variance

- **Target**: concurrent_generation_10
- **Recommendation**: Investigate source of variance - possible caching or GC issues
- **Potential Improvement**: 95000.0μs

### 2. Batching Opportunity

- **Target**: lookup_existing_domain
- **Recommendation**: Implement batch version of lookup_existing_domain
- **Potential Improvement**: 50-80% for multiple operations

### 3. Batching Opportunity

- **Target**: resolve_with_decryption
- **Recommendation**: Implement batch version of resolve_with_decryption
- **Potential Improvement**: 50-80% for multiple operations

### 4. Batching Opportunity

- **Target**: lookup_with_1000_domains
- **Recommendation**: Implement batch version of lookup_with_1000_domains
- **Potential Improvement**: 50-80% for multiple operations

### 5. Batching Opportunity

- **Target**: verify_fingerprint
- **Recommendation**: Implement batch version of verify_fingerprint
- **Potential Improvement**: 50-80% for multiple operations

### 6. Batching Opportunity

- **Target**: concurrent_generation_10
- **Recommendation**: Implement batch version of concurrent_generation_10
- **Potential Improvement**: 50-80% for multiple operations

### 7. Batching Opportunity

- **Target**: resolve_single_domain
- **Recommendation**: Implement batch version of resolve_single_domain
- **Potential Improvement**: 50-80% for multiple operations

### 8. Batching Opportunity

- **Target**: resolve_cache_hit
- **Recommendation**: Implement batch version of resolve_cache_hit
- **Potential Improvement**: 50-80% for multiple operations

### 9. Batching Opportunity

- **Target**: resolve_cache_miss
- **Recommendation**: Implement batch version of resolve_cache_miss
- **Potential Improvement**: 50-80% for multiple operations

### 10. Caching Opportunity

- **Target**: resolve_single_domain
- **Recommendation**: Implement or improve caching layer
- **Potential Improvement**: 90-95% cache hit rate

### 11. Caching Opportunity

- **Target**: resolve_cache_miss
- **Recommendation**: Implement or improve caching layer
- **Potential Improvement**: 90-95% cache hit rate

### 12. Caching Opportunity

- **Target**: batch_resolve_10
- **Recommendation**: Implement or improve caching layer
- **Potential Improvement**: 90-95% cache hit rate

## Tool Performance Metrics

- Cache hit rate: 100.0%
- Total tool runtime: 0.01s
- Operations profiled: 2

