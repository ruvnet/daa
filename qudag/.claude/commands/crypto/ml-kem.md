# ML-KEM (Post-Quantum Key Encapsulation)

ML-KEM-768 key encapsulation mechanism for QuDAG quantum-resistant key exchange.

## Key Generation Commands

### Generate ML-KEM Key Pair
```bash
# Generate standard ML-KEM-768 key pair
cargo run --bin qudag -- crypto ml-kem keygen --output-dir ./keys/ml-kem/

# Generate specific security level
cargo run --bin qudag -- crypto ml-kem keygen --level 512 --output ./keys/ml-kem-512.key
cargo run --bin qudag -- crypto ml-kem keygen --level 768 --output ./keys/ml-kem-768.key
cargo run --bin qudag -- crypto ml-kem keygen --level 1024 --output ./keys/ml-kem-1024.key

# Generate ephemeral keys for session
cargo run --bin qudag -- crypto ml-kem keygen --ephemeral --output ./temp/session.key
```

### Key Management
```bash
# Display key information
cargo run --bin qudag -- crypto ml-kem info --key ./keys/ml-kem-768.key

# Validate key pair
cargo run --bin qudag -- crypto ml-kem validate --key ./keys/ml-kem-768.key

# Export public key
cargo run --bin qudag -- crypto ml-kem export-public \
  --key ./keys/ml-kem-768.key \
  --output ./keys/ml-kem-768.pub
```

## Key Exchange Operations

### Encapsulation (Sender Side)
```bash
# Encapsulate shared secret
cargo run --bin qudag -- crypto ml-kem encapsulate \
  --public-key ./keys/recipient.pub \
  --output-ciphertext ./exchange/ciphertext.bin \
  --output-secret ./exchange/shared-secret.bin

# Encapsulate with additional entropy
cargo run --bin qudag -- crypto ml-kem encapsulate \
  --public-key ./keys/recipient.pub \
  --entropy-file ./random/entropy.bin \
  --output-ciphertext ./exchange/ciphertext.bin \
  --output-secret ./exchange/shared-secret.bin

# Batch encapsulation for multiple recipients
cargo run --bin qudag -- crypto ml-kem encapsulate-batch \
  --public-keys ./keys/recipients/ \
  --output-dir ./exchange/batch/
```

### Decapsulation (Recipient Side)
```bash
# Decapsulate shared secret
cargo run --bin qudag -- crypto ml-kem decapsulate \
  --private-key ./keys/ml-kem-768.key \
  --ciphertext ./exchange/ciphertext.bin \
  --output-secret ./exchange/recovered-secret.bin

# Verify decapsulation success
cargo run --bin qudag -- crypto ml-kem verify-decap \
  --secret1 ./exchange/shared-secret.bin \
  --secret2 ./exchange/recovered-secret.bin

# Batch decapsulation
cargo run --bin qudag -- crypto ml-kem decapsulate-batch \
  --private-key ./keys/ml-kem-768.key \
  --ciphertexts ./exchange/batch/ \
  --output-dir ./secrets/batch/
```

## Testing Commands

### Unit Tests
```bash
# Run ML-KEM specific tests
cargo test ml_kem --lib --features ml-kem

# Test key encapsulation mechanism
cargo test ml_kem::kem --lib -- --nocapture

# Test different security parameters
cargo test ml_kem_parameters --lib

# Test error handling and edge cases
cargo test ml_kem_edge_cases --lib
```

### Integration Tests
```bash
# Run ML-KEM integration tests
cargo test --test ml_kem_integration

# Test key exchange protocols
cargo test --test key_exchange_protocols -- ml_kem

# Test compatibility with other systems
cargo test --test ml_kem_compatibility
```

### Security Tests
```bash
# Security validation tests
cargo test --test security_tests -- ml_kem

# Test against known attack vectors
cargo test ml_kem_security --lib --features security-tests

# Constant-time operation validation
cargo test ml_kem_timing --lib --features timing-tests

# Memory safety validation
cargo test ml_kem_memory_safety --lib
```

## Performance Benchmarks

### Core Operation Benchmarks
```bash
# Benchmark key generation
cargo bench --bench ml_kem_keygen

# Benchmark encapsulation
cargo bench --bench ml_kem_encapsulate

# Benchmark decapsulation
cargo bench --bench ml_kem_decapsulate

# Complete ML-KEM benchmark suite
cargo bench --bench crypto_ml_kem
```

### Security Level Comparisons
```bash
# Compare different ML-KEM security levels
cargo bench --bench ml_kem_levels_comparison

# Memory usage benchmarks
cargo bench --bench ml_kem_memory -- --features heap-profiling

# Throughput benchmarks
cargo bench --bench ml_kem_throughput
```

### Network Performance
```bash
# Network overhead analysis
cargo run --bin network-bench -- ml-kem \
  --simulate-latency 50ms \
  --simulate-bandwidth 1Gbps

# Protocol efficiency metrics
cargo run --bin protocol-bench -- ml-kem-exchange \
  --iterations 1000 \
  --output ./reports/ml-kem-protocol.json
```

## Key Exchange Protocols

### Basic Key Exchange
```bash
# Alice generates key pair
cargo run --bin qudag -- crypto ml-kem keygen --output ./alice/ml-kem.key

# Alice exports public key
cargo run --bin qudag -- crypto ml-kem export-public \
  --key ./alice/ml-kem.key \
  --output ./alice/ml-kem.pub

# Bob encapsulates with Alice's public key
cargo run --bin qudag -- crypto ml-kem encapsulate \
  --public-key ./alice/ml-kem.pub \
  --output-ciphertext ./bob/ciphertext.bin \
  --output-secret ./bob/shared-secret.bin

# Alice decapsulates Bob's ciphertext
cargo run --bin qudag -- crypto ml-kem decapsulate \
  --private-key ./alice/ml-kem.key \
  --ciphertext ./bob/ciphertext.bin \
  --output-secret ./alice/shared-secret.bin
```

### Authenticated Key Exchange
```bash
# Generate long-term identity keys
cargo run --bin qudag -- crypto ml-kem keygen \
  --output ./identity/alice-long-term.key

# Generate ephemeral keys
cargo run --bin qudag -- crypto ml-kem keygen \
  --ephemeral \
  --output ./ephemeral/alice-session.key

# Perform authenticated key exchange
cargo run --bin qudag -- crypto ml-kem auth-exchange \
  --identity-key ./identity/alice-long-term.key \
  --ephemeral-key ./ephemeral/alice-session.key \
  --peer-identity ./identity/bob-long-term.pub \
  --peer-ephemeral ./ephemeral/bob-session.pub
```

## Development Workflows

### Key Exchange Testing
```bash
# Test complete key exchange flow
./scripts/test-ml-kem-exchange.sh

# Manual testing steps:
cargo test ml_kem_key_exchange --lib
cargo test --test ml_kem_protocol
cargo bench --bench ml_kem_exchange -- --quick
```

### Security Validation Workflow
```bash
# Complete security validation
cargo test --test security_tests -- ml_kem --features security-tests
cargo test ml_kem_timing --lib --features timing-tests
cargo test ml_kem_memory_safety --lib

# Fuzz testing
cargo install cargo-fuzz
cargo fuzz run ml_kem_encapsulate -- -max_total_time=300
cargo fuzz run ml_kem_decapsulate -- -max_total_time=300
```

### Performance Validation
```bash
# Performance regression testing
cargo bench --bench ml_kem_regression

# Generate performance baseline
cargo run --bin perf-baseline -- ml-kem \
  --output ./baselines/ml-kem-baseline.json

# Compare against baseline
cargo run --bin perf-compare -- \
  --baseline ./baselines/ml-kem-baseline.json \
  --current-results ./results/ml-kem-current.json
```

## Configuration Examples

### Protocol Configuration
```json
{
  "ml_kem_config": {
    "default_security_level": 768,
    "key_reuse_policy": "ephemeral_only",
    "batch_size": 100,
    "timeout_seconds": 30,
    "validate_on_decapsulation": true
  }
}
```

### Test Configuration
```json
{
  "ml_kem_test_config": {
    "security_levels": [512, 768, 1024],
    "test_vectors": "./test-vectors/ml-kem/",
    "performance_iterations": 10000,
    "memory_limit_mb": 512,
    "timing_threshold_ns": 1000000
  }
}
```

## Common Use Cases

### Secure Channel Establishment
```bash
# Node-to-node secure channel
cargo run --bin qudag -- network establish-channel \
  --local-key ./keys/node-ml-kem.key \
  --peer-public-key ./keys/peer-ml-kem.pub \
  --protocol ml-kem-768

# Client authentication
cargo run --bin qudag -- client auth \
  --client-key ./keys/client-ml-kem.key \
  --server-public-key ./keys/server-ml-kem.pub
```

### Session Key Generation
```bash
# Generate session keys for encryption
cargo run --bin qudag -- crypto derive-session-key \
  --ml-kem-secret ./secrets/shared-secret.bin \
  --context "qudag.session.encryption.v1" \
  --output ./keys/session-aes.key

# Generate authentication keys
cargo run --bin qudag -- crypto derive-auth-key \
  --ml-kem-secret ./secrets/shared-secret.bin \
  --context "qudag.session.auth.v1" \
  --output ./keys/session-hmac.key
```

### Hybrid Key Exchange
```bash
# Combine ML-KEM with classical ECDH for hybrid security
cargo run --bin qudag -- crypto hybrid-exchange \
  --ml-kem-key ./keys/ml-kem.key \
  --ecdh-key ./keys/ecdh.key \
  --peer-ml-kem-pub ./keys/peer-ml-kem.pub \
  --peer-ecdh-pub ./keys/peer-ecdh.pub \
  --output ./secrets/hybrid-shared-secret.bin
```

## Debugging and Troubleshooting

### Debug Key Exchange
```bash
# Debug encapsulation process
RUST_LOG=debug cargo run --bin qudag -- crypto ml-kem encapsulate \
  --debug \
  --public-key ./keys/debug.pub \
  --output-ciphertext ./debug/ct.bin \
  --output-secret ./debug/secret.bin

# Verify key exchange step by step
cargo run --bin qudag -- crypto ml-kem debug-exchange \
  --key-pair ./keys/ml-kem.key \
  --verbose
```

### Validate Implementations
```bash
# Cross-reference with test vectors
cargo test ml_kem_test_vectors --lib -- --nocapture

# Validate against NIST reference
cargo test --test nist_ml_kem_validation

# Check interoperability
cargo test --test ml_kem_interop
```