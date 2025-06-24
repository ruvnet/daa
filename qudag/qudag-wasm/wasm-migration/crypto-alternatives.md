# WASM-Compatible Crypto Alternatives for QuDAG

## Problem Summary
The current QuDAG crypto dependencies use C-based implementations that fail in WASM due to missing stdlib.h, string.h, and other system headers:
- `pqcrypto-dilithium` (0.5) - ML-DSA implementation
- `pqcrypto-kyber` (0.5) - ML-KEM implementation  
- `pqcrypto-hqc` (0.2) - HQC implementation

## Recommended Pure Rust Alternatives

### 1. ML-KEM (Kyber) Replacement

**Primary Choice: `ml-kem` crate**
- Pure Rust implementation of FIPS 203 ML-KEM standard
- No unsafe code, WASM compatible
- Requires Rust 1.81+
- Implements all security levels (ML-KEM-512, ML-KEM-768, ML-KEM-1024)

```toml
[dependencies]
ml-kem = "0.2.0"  # Check for latest version
```

**Alternative: `pqc_kyber` crate**
- Explicitly tested with WASM, has npm package
- no_std compatible, no allocator needed
- Pure Rust, no unsafe code
- Supports Kyber512, Kyber768, Kyber1024

```toml
[dependencies]
pqc_kyber = { version = "0.8.1", features = ["wasm"] }
```

### 2. ML-DSA (Dilithium) Replacement

**Primary Choice: `fips204` crate**
- Pure Rust implementation of FIPS 204 ML-DSA standard
- Explicitly tested with WASM examples
- no_std, no heap allocations
- Requires Rust 1.70+
- Implements ML-DSA-44, ML-DSA-65, ML-DSA-87

```toml
[dependencies]
fips204 = "0.3.0"  # Check for latest version
```

**Alternative: `ml-dsa` crate**
- Pure Rust, likely WASM compatible
- Requires Rust 1.81+
- Warning: Not independently audited

```toml
[dependencies]
ml-dsa = "0.2.0"  # Check for latest version
```

### 3. HQC Replacement

**Challenge**: No pure Rust implementation found for HQC.

**Options**:
1. **Conditional Compilation**: Exclude HQC for WASM builds
2. **Stub Implementation**: Create WASM stub that returns errors
3. **Alternative Algorithm**: Use additional ML-KEM/ML-DSA rounds
4. **Future Work**: Wait for pure Rust HQC implementation

## Migration Strategy

### Phase 1: Update Cargo.toml Dependencies

```toml
[dependencies]
# Remove these:
# pqcrypto-dilithium = "0.5"
# pqcrypto-kyber = "0.5"
# pqcrypto-hqc = "0.2"

# Add these with feature flags:
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
pqcrypto-dilithium = "0.5"
pqcrypto-kyber = "0.5"
pqcrypto-hqc = "0.2"

[target.'cfg(target_arch = "wasm32")'.dependencies]
ml-kem = "0.2.0"
fips204 = "0.3.0"
# or alternatives:
# pqc_kyber = { version = "0.8.1", features = ["wasm"] }
# ml-dsa = "0.2.0"
```

### Phase 2: Create Abstraction Layer

```rust
// crypto/src/quantum/mod.rs
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
```

### Phase 3: Implement WASM-specific Modules

```rust
// crypto/src/quantum/wasm/ml_kem.rs
use ml_kem::{MlKem768, Encapsulate, Decapsulate};

pub struct MlKemWrapper {
    // Implementation using ml-kem crate
}

// crypto/src/quantum/wasm/ml_dsa.rs
use fips204::{ml_dsa_65, traits::{Signer, Verifier}};

pub struct MlDsaWrapper {
    // Implementation using fips204 crate
}
```

### Phase 4: HQC Handling for WASM

```rust
// crypto/src/quantum/wasm/hqc.rs
#[cfg(target_arch = "wasm32")]
pub struct HqcStub;

#[cfg(target_arch = "wasm32")]
impl HqcStub {
    pub fn new() -> Result<Self, Error> {
        Err(Error::UnsupportedInWasm("HQC not available in WASM"))
    }
}
```

## Feature Parity Considerations

1. **API Differences**: The pure Rust crates have different APIs than pqcrypto variants
2. **Performance**: Pure Rust may be slower than optimized C implementations
3. **Security Levels**: Ensure matching security parameter sets (768-bit for ML-KEM, 65 for ML-DSA)
4. **Serialization**: Different crates may use different serialization formats

## Testing Strategy

1. Create feature flags for WASM builds
2. Implement compatibility tests comparing outputs
3. Add WASM-specific test suite
4. Benchmark performance differences

## Web Crypto API Integration

For additional performance, consider using Web Crypto API for:
- AES-GCM (already using aes-gcm crate)
- SHA-256/SHA-3 (for hashing)
- ECDH with P-256 (for key exchange)

```rust
#[cfg(target_arch = "wasm32")]
use web_sys::crypto;
```

## Recommendations

1. **Immediate Action**: Implement ML-KEM and ML-DSA replacements using recommended crates
2. **HQC Strategy**: Use conditional compilation to disable HQC in WASM builds
3. **Testing**: Extensive testing to ensure cryptographic compatibility
4. **Documentation**: Update docs to note WASM limitations
5. **Future**: Monitor for pure Rust HQC implementations