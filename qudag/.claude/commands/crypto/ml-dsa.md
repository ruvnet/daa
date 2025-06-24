# ML-DSA (Post-Quantum Digital Signatures)

ML-DSA signature operations for QuDAG quantum-resistant cryptography.

## Key Generation Commands

### Generate ML-DSA Key Pair
```bash
# Generate standard ML-DSA-65 key pair
cargo run --bin qudag -- crypto ml-dsa keygen --output-dir ./keys/ml-dsa/

# Generate specific security level
cargo run --bin qudag -- crypto ml-dsa keygen --level 65 --output ./keys/ml-dsa-65.key
cargo run --bin qudag -- crypto ml-dsa keygen --level 87 --output ./keys/ml-dsa-87.key

# Generate with custom seed for testing
cargo run --bin qudag -- crypto ml-dsa keygen --seed 0x123456789abcdef --output ./test/keys/
```

### Key Information
```bash
# Display key information
cargo run --bin qudag -- crypto ml-dsa info --key ./keys/ml-dsa-65.key

# Validate key format
cargo run --bin qudag -- crypto ml-dsa validate --key ./keys/ml-dsa-65.key
```

## Signing Operations

### Sign Messages
```bash
# Sign a message
cargo run --bin qudag -- crypto ml-dsa sign \
  --key ./keys/ml-dsa-65.key \
  --message "Hello QuDAG" \
  --output ./signatures/message.sig

# Sign file content
cargo run --bin qudag -- crypto ml-dsa sign \
  --key ./keys/ml-dsa-65.key \
  --file ./data/document.txt \
  --output ./signatures/document.sig

# Sign with context (domain separation)
cargo run --bin qudag -- crypto ml-dsa sign \
  --key ./keys/ml-dsa-65.key \
  --message "transaction data" \
  --context "qudag.transaction.v1" \
  --output ./signatures/tx.sig
```

### Verify Signatures
```bash
# Verify signature
cargo run --bin qudag -- crypto ml-dsa verify \
  --public-key ./keys/ml-dsa-65.pub \
  --message "Hello QuDAG" \
  --signature ./signatures/message.sig

# Verify file signature
cargo run --bin qudag -- crypto ml-dsa verify \
  --public-key ./keys/ml-dsa-65.pub \
  --file ./data/document.txt \
  --signature ./signatures/document.sig

# Batch signature verification
cargo run --bin qudag -- crypto ml-dsa verify-batch \
  --config ./signatures/batch-verify.json
```

## Testing Commands

### Unit Tests
```bash
# Run ML-DSA specific tests
cargo test ml_dsa --lib --features ml-dsa

# Run signature tests with verbose output
cargo test ml_dsa::signature --lib -- --nocapture

# Test different security levels
cargo test ml_dsa_levels --lib

# Test edge cases and error handling
cargo test ml_dsa_edge_cases --lib
```

### Integration Tests
```bash
# Run ML-DSA integration tests
cargo test --test ml_dsa_integration

# Test cross-platform compatibility
cargo test --test ml_dsa_cross_platform

# Test interoperability with other crypto
cargo test --test crypto_integration -- ml_dsa
```

### Security Tests
```bash
# Run security-focused ML-DSA tests
cargo test --test security_tests -- ml_dsa

# Memory safety tests
cargo test ml_dsa_memory_safety --lib

# Constant-time operation tests
cargo test ml_dsa_timing --lib --features timing-tests

# Side-channel resistance tests
cargo test ml_dsa_side_channel --lib --features security-tests
```

## Performance Benchmarks

### Basic Benchmarks
```bash
# Benchmark key generation
cargo bench --bench ml_dsa_keygen

# Benchmark signing operations
cargo bench --bench ml_dsa_sign

# Benchmark verification operations
cargo bench --bench ml_dsa_verify

# Comprehensive ML-DSA benchmarks
cargo bench --bench crypto_ml_dsa
```

### Comparative Benchmarks
```bash
# Compare ML-DSA levels
cargo bench --bench ml_dsa_levels_comparison

# Compare with classical signatures (if available)
cargo bench --bench signature_comparison -- ml_dsa

# Memory usage benchmarks
cargo bench --bench ml_dsa_memory -- --features heap-profiling
```

### Performance Analysis
```bash
# Generate performance report
cargo run --bin benchmark -- \
  --crypto ml-dsa \
  --operations keygen,sign,verify \
  --iterations 1000 \
  --output ./reports/ml-dsa-performance.json

# Profile with detailed metrics
cargo run --bin perf-profile -- ml-dsa-full-suite
```

## Development Workflows

### Validation Workflow
```bash
# Complete ML-DSA validation
./scripts/validate-ml-dsa.sh

# Or manual steps:
cargo test ml_dsa --lib
cargo test --test ml_dsa_integration
cargo bench --bench ml_dsa_keygen -- --quick
cargo run --example ml_dsa_demo
```

### Security Validation
```bash
# Security test suite
cargo test --test security_tests -- ml_dsa --features security-tests

# Timing attack resistance
cargo test ml_dsa_timing --lib --features timing-tests -- --nocapture

# Memory corruption tests
cargo test ml_dsa_memory --lib --features memory-tests
```

### Debug and Troubleshooting
```bash
# Debug key generation
RUST_LOG=debug cargo test ml_dsa_keygen -- --nocapture

# Debug signature failures
RUST_LOG=trace cargo run --bin qudag -- crypto ml-dsa sign --debug \
  --key ./keys/test.key --message "debug test"

# Verbose error reporting
cargo run --bin qudag -- crypto ml-dsa verify --verbose \
  --public-key ./keys/test.pub \
  --message "test message" \
  --signature ./sigs/test.sig
```

## Example Configurations

### Batch Operations Config
```json
{
  "ml_dsa_batch_sign": {
    "key_file": "./keys/ml-dsa-65.key",
    "messages": [
      {"data": "message1", "output": "./sigs/msg1.sig"},
      {"data": "message2", "output": "./sigs/msg2.sig"}
    ]
  }
}
```

### Test Configuration
```json
{
  "ml_dsa_test_config": {
    "security_levels": [65, 87],
    "message_sizes": [32, 256, 1024, 4096],
    "iterations": 1000,
    "timing_tests": true,
    "memory_tests": true
  }
}
```

## Common Use Cases

### Transaction Signing
```bash
# Sign QuDAG transaction
cargo run --bin qudag -- crypto ml-dsa sign \
  --key ./keys/node.key \
  --file ./transactions/tx-001.json \
  --context "qudag.transaction.v1" \
  --output ./transactions/tx-001.sig
```

### Node Authentication
```bash
# Generate node identity key
cargo run --bin qudag -- crypto ml-dsa keygen \
  --level 65 \
  --output ./config/node-identity.key

# Sign node certificate
cargo run --bin qudag -- crypto ml-dsa sign \
  --key ./config/node-identity.key \
  --file ./config/node-cert.json \
  --context "qudag.node.identity.v1"
```

### Batch Verification
```bash
# Verify multiple signatures efficiently
cargo run --bin qudag -- crypto ml-dsa verify-batch \
  --public-keys ./keys/ \
  --signatures ./sigs/ \
  --messages ./messages/ \
  --parallel 4
```