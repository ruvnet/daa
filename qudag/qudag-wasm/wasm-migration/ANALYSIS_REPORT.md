# QuDAG WASM Crypto Analysis Report

## Executive Summary

The QuDAG WASM build is failing due to C-based cryptographic dependencies that require system headers (stdlib.h, string.h) not available in WebAssembly environments. I've identified pure Rust alternatives for all critical components except HQC.

## Root Cause Analysis

### Failing Dependencies
1. **pqcrypto-dilithium v0.5** - Used in `core/crypto/src/signatures/ml_dsa.rs`
2. **pqcrypto-kyber v0.5** - Listed as dependency but not directly used (placeholder implementation)
3. **pqcrypto-hqc v0.2** - Used in `core/crypto/src/hqc.rs`

All three crates use C implementations from PQClean that compile native code requiring system headers.

## WASM-Compatible Alternatives

### 1. ML-KEM (Kyber) Replacements

**Recommended: `ml-kem` crate**
- Pure Rust implementation of FIPS 203
- No unsafe code
- Actively maintained
- Requires Rust 1.81+

**Alternative: `pqc_kyber` crate**
- Explicit WASM support with npm package
- no_std compatible
- Pure Rust, no allocator needed
- Well-tested in WASM environments

### 2. ML-DSA (Dilithium) Replacements

**Recommended: `fips204` crate**
- Pure Rust implementation of FIPS 204
- Includes WASM examples
- no_std, no heap allocations
- Supports all security levels (ML-DSA-44, 65, 87)
- Requires Rust 1.70+

**Alternative: `ml-dsa` crate**
- Pure Rust but not independently audited
- Requires Rust 1.81+

### 3. HQC Status

**No pure Rust implementation available**

Options:
- Conditional compilation to exclude HQC in WASM builds
- Implement stub that returns "UnsupportedInWasm" errors
- Use additional ML-KEM rounds as cryptographic fallback
- Wait for community to develop pure Rust HQC

## Implementation Strategy

### Phase 1: Update Dependencies
```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
pqcrypto-dilithium = "0.5"
pqcrypto-kyber = "0.5"
pqcrypto-hqc = "0.2"

[target.'cfg(target_arch = "wasm32")'.dependencies]
ml-kem = "0.2.0"
fips204 = "0.3.0"
```

### Phase 2: Create Abstraction Layer
- Use `cfg` attributes to conditionally compile native vs WASM implementations
- Define common traits for ML-KEM and ML-DSA operations
- Implement wrappers for each platform

### Phase 3: Handle HQC
- Create HqcStub for WASM that returns appropriate errors
- Use feature flags to enable/disable HQC support
- Document WASM limitations

## Risk Assessment

1. **API Compatibility**: Different crates have different APIs requiring wrapper implementations
2. **Performance**: Pure Rust may be slower than optimized C (acceptable for WASM use case)
3. **Security Audit**: Some alternatives lack independent security audits
4. **Feature Parity**: HQC unavailable in WASM reduces cryptographic diversity

## Recommendations

1. **Immediate**: Implement ML-KEM and ML-DSA replacements using recommended crates
2. **Short-term**: Create comprehensive test suite to verify cryptographic compatibility
3. **Medium-term**: Benchmark performance differences and optimize if needed
4. **Long-term**: Monitor for pure Rust HQC implementations or contribute one

## Files Created

1. `/workspaces/QuDAG/wasm-migration/crypto-alternatives.md` - Detailed technical analysis
2. `/workspaces/QuDAG/wasm-migration/sample-implementation.rs` - Example implementation code
3. Memory entry: `swarm-wasm-crypto-1750601234/crypto-analyst/alternatives` - Structured findings

## Next Steps

The WASM Engineer should use these findings to:
1. Update Cargo.toml with conditional dependencies
2. Implement the abstraction layer following the sample code
3. Create WASM-specific test cases
4. Update documentation about WASM limitations