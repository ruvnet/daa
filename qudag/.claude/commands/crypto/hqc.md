# HQC (Hamming Quasi-Cyclic) Alternative Scheme

HQC post-quantum cryptography alternative for QuDAG diversified quantum resistance.

## Key Generation Commands

### Generate HQC Key Pairs
```bash
# Generate standard HQC key pair
cargo run --bin qudag -- crypto hqc keygen --output-dir ./keys/hqc/

# Generate specific security levels
cargo run --bin qudag -- crypto hqc keygen --level 128 --output ./keys/hqc-128.key
cargo run --bin qudag -- crypto hqc keygen --level 192 --output ./keys/hqc-192.key
cargo run --bin qudag -- crypto hqc keygen --level 256 --output ./keys/hqc-256.key

# Generate with custom parameters
cargo run --bin qudag -- crypto hqc keygen \
  --security-level 128 \
  --code-dimension 17669 \
  --error-weight 75 \
  --output ./keys/hqc-custom.key
```

### Key Management
```bash
# Display HQC key information
cargo run --bin qudag -- crypto hqc info --key ./keys/hqc-128.key

# Validate key structure
cargo run --bin qudag -- crypto hqc validate --key ./keys/hqc-128.key

# Export public key only
cargo run --bin qudag -- crypto hqc export-public \
  --key ./keys/hqc-128.key \
  --output ./keys/hqc-128.pub

# Convert key formats
cargo run --bin qudag -- crypto hqc convert \
  --input ./keys/hqc-128.key \
  --format pem \
  --output ./keys/hqc-128.pem
```

## Key Encapsulation Operations

### Encapsulation (KEM)
```bash
# Basic encapsulation
cargo run --bin qudag -- crypto hqc encapsulate \
  --public-key ./keys/recipient-hqc.pub \
  --output-ciphertext ./exchange/hqc-ciphertext.bin \
  --output-secret ./exchange/hqc-shared-secret.bin

# Encapsulation with error injection testing
cargo run --bin qudag -- crypto hqc encapsulate \
  --public-key ./keys/recipient-hqc.pub \
  --inject-errors 5 \
  --output-ciphertext ./test/error-ct.bin \
  --output-secret ./test/error-secret.bin

# Batch encapsulation
cargo run --bin qudag -- crypto hqc encapsulate-batch \
  --public-keys ./keys/recipients/ \
  --output-dir ./exchange/hqc-batch/ \
  --parallel 4
```

### Decapsulation
```bash
# Basic decapsulation
cargo run --bin qudag -- crypto hqc decapsulate \
  --private-key ./keys/hqc-128.key \
  --ciphertext ./exchange/hqc-ciphertext.bin \
  --output-secret ./exchange/hqc-recovered-secret.bin

# Decapsulation with error correction analysis
cargo run --bin qudag -- crypto hqc decapsulate \
  --private-key ./keys/hqc-128.key \
  --ciphertext ./exchange/hqc-ciphertext.bin \
  --output-secret ./exchange/hqc-recovered-secret.bin \
  --error-analysis \
  --output-report ./reports/hqc-error-correction.json

# Batch decapsulation
cargo run --bin qudag -- crypto hqc decapsulate-batch \
  --private-key ./keys/hqc-128.key \
  --ciphertexts ./exchange/hqc-batch/ \
  --output-dir ./secrets/hqc-batch/
```

## Testing Commands

### Unit Tests
```bash
# Run HQC specific tests
cargo test hqc --lib --features hqc

# Test error correction capabilities
cargo test hqc::error_correction --lib -- --nocapture

# Test different security parameters
cargo test hqc_parameters --lib

# Test key encapsulation mechanism
cargo test hqc_kem --lib
```

### Integration Tests
```bash
# Run HQC integration tests
cargo test --test hqc_integration

# Test interoperability with other schemes
cargo test --test crypto_interop -- hqc

# Test error handling scenarios
cargo test --test hqc_error_scenarios
```

### Security Tests
```bash
# Security validation tests
cargo test --test security_tests -- hqc

# Test against known attacks
cargo test hqc_attack_resistance --lib --features security-tests

# Error correction security tests
cargo test hqc_error_correction_security --lib

# Constant-time implementation tests
cargo test hqc_timing --lib --features timing-tests
```

## Performance Benchmarks

### Core Benchmarks
```bash
# Benchmark key generation
cargo bench --bench hqc_keygen

# Benchmark encapsulation
cargo bench --bench hqc_encapsulate

# Benchmark decapsulation
cargo bench --bench hqc_decapsulate

# Error correction performance
cargo bench --bench hqc_error_correction
```

### Comparative Analysis
```bash
# Compare HQC with ML-KEM
cargo bench --bench kem_comparison -- hqc ml_kem

# Security level performance comparison
cargo bench --bench hqc_security_levels

# Memory usage analysis
cargo bench --bench hqc_memory -- --features heap-profiling
```

### Error Correction Benchmarks
```bash
# Benchmark error correction capacity
cargo bench --bench hqc_error_capacity

# Test performance under various error rates
cargo run --bin error-rate-bench -- hqc \
  --error-rates 0.01,0.02,0.05,0.1 \
  --iterations 1000
```

## Compatibility Testing

### Cross-Platform Testing
```bash
# Test on different architectures
cargo test hqc --lib --target x86_64-unknown-linux-gnu
cargo test hqc --lib --target aarch64-unknown-linux-gnu
cargo test hqc --lib --target wasm32-unknown-unknown

# Test with different compiler optimizations
RUSTFLAGS="-C opt-level=0" cargo test hqc --lib
RUSTFLAGS="-C opt-level=3" cargo test hqc --lib
```

### Reference Implementation Testing
```bash
# Test against reference vectors
cargo test hqc_reference_vectors --lib -- --nocapture

# Validate against NIST submission
cargo test --test nist_hqc_validation

# Cross-implementation compatibility
cargo test --test hqc_compatibility
```

### Error Rate Testing
```bash
# Test error correction limits
cargo test hqc_error_limits --lib

# Simulate channel errors
cargo run --bin channel-simulator -- hqc \
  --error-probability 0.05 \
  --iterations 10000 \
  --output ./reports/hqc-channel-test.json

# Burst error testing
cargo test hqc_burst_errors --lib
```

## Development Workflows

### HQC Validation Workflow
```bash
# Complete HQC validation
./scripts/validate-hqc.sh

# Manual validation steps:
cargo test hqc --lib
cargo test --test hqc_integration
cargo bench --bench hqc_keygen -- --quick
cargo run --example hqc_demo
```

### Error Correction Analysis
```bash
# Analyze error correction performance
cargo run --bin error-analysis -- hqc \
  --test-patterns ./test-data/error-patterns/ \
  --output ./analysis/hqc-error-analysis.json

# Generate error correction reports
cargo run --bin generate-report -- hqc-error-correction \
  --data ./analysis/ \
  --format html \
  --output ./reports/hqc-error-correction.html
```

### Security Analysis Workflow
```bash
# Security parameter analysis
cargo test hqc_security_analysis --lib --features security-tests

# Cryptanalysis resistance testing
cargo test hqc_cryptanalysis --lib

# Side-channel analysis
cargo test hqc_side_channel --lib --features timing-tests
```

## Configuration Examples

### HQC Parameters Configuration
```json
{
  "hqc_config": {
    "default_security_level": 128,
    "code_parameters": {
      "128": {
        "n": 17669,
        "k": 17_664,
        "w": 75,
        "wr": 75,
        "we": 75
      },
      "192": {
        "n": 35_851,
        "k": 35_844,
        "w": 114,
        "wr": 114,
        "we": 114
      },
      "256": {
        "n": 57_637,
        "k": 57_628,
        "w": 149,
        "wr": 149,
        "we": 149
      }
    },
    "error_correction_enabled": true,
    "constant_time_ops": true
  }
}
```

### Test Configuration
```json
{
  "hqc_test_config": {
    "security_levels": [128, 192, 256],
    "error_rates": [0.01, 0.02, 0.05, 0.1],
    "test_iterations": 10000,
    "reference_vectors": "./test-vectors/hqc/",
    "performance_baseline": "./baselines/hqc-baseline.json",
    "error_injection_tests": true
  }
}
```

## Common Use Cases

### Alternative Key Exchange
```bash
# Use HQC as backup to ML-KEM
cargo run --bin qudag -- crypto negotiate-kem \
  --preferred ml-kem-768 \
  --fallback hqc-128 \
  --peer-capabilities ./config/peer-crypto-caps.json

# HQC-only key exchange
cargo run --bin qudag -- crypto hqc-exchange \
  --local-key ./keys/hqc-128.key \
  --peer-public-key ./keys/peer-hqc-128.pub \
  --context "qudag.backup.exchange.v1"
```

### Diversified Cryptography
```bash
# Dual-algorithm protection
cargo run --bin qudag -- crypto dual-kem \
  --primary-scheme ml-kem-768 \
  --backup-scheme hqc-128 \
  --primary-key ./keys/ml-kem.key \
  --backup-key ./keys/hqc.key \
  --peer-primary-pub ./keys/peer-ml-kem.pub \
  --peer-backup-pub ./keys/peer-hqc.pub
```

### Error-Tolerant Communications
```bash
# Use HQC for noisy channels
cargo run --bin qudag -- network configure-channel \
  --kem hqc-128 \
  --error-correction-level high \
  --channel-type satellite \
  --noise-threshold 0.05
```

## Debugging and Analysis

### Debug HQC Operations
```bash
# Debug key generation
RUST_LOG=debug cargo test hqc_keygen -- --nocapture

# Debug encapsulation with detailed logging
RUST_LOG=trace cargo run --bin qudag -- crypto hqc encapsulate \
  --debug \
  --public-key ./keys/debug-hqc.pub \
  --output-ciphertext ./debug/hqc-ct.bin \
  --output-secret ./debug/hqc-secret.bin

# Analyze error correction process
cargo run --bin qudag -- crypto hqc debug-error-correction \
  --ciphertext ./debug/hqc-ct.bin \
  --expected-errors 10 \
  --verbose
```

### Performance Analysis
```bash
# Detailed performance profiling
cargo run --bin perf-profile -- hqc-full-suite \
  --security-levels 128,192,256 \
  --operations keygen,encapsulate,decapsulate \
  --output ./profiles/hqc-performance.json

# Error correction performance analysis
cargo run --bin error-perf-analysis -- hqc \
  --error-patterns ./test-data/error-patterns/ \
  --output ./analysis/hqc-error-perf.json
```

### Validation Tools
```bash
# Validate HQC implementation correctness
cargo run --bin validate-hqc -- \
  --test-vectors ./test-vectors/hqc/ \
  --reference-implementation ./reference/hqc/ \
  --output ./validation/hqc-validation-report.json

# Cross-reference with academic papers
cargo test hqc_academic_validation --lib -- --nocapture
```

## Maintenance Commands

### Update HQC Parameters
```bash
# Update security parameters
cargo run --bin update-crypto-params -- hqc \
  --parameter-file ./config/hqc-params-v2.json \
  --validate \
  --backup-old

# Regenerate test vectors
cargo run --bin generate-test-vectors -- hqc \
  --security-levels 128,192,256 \
  --count 1000 \
  --output ./test-vectors/hqc/updated/
```

### Compatibility Maintenance
```bash
# Check compatibility with new versions
cargo test hqc_version_compatibility --lib

# Update reference implementation
./scripts/update-hqc-reference.sh

# Verify backward compatibility
cargo test hqc_backward_compatibility --lib
```