# QuDAG Exchange Test Suite

## Overview

This test suite follows Test-Driven Development (TDD) methodology, where tests are written before implementation. The comprehensive test coverage ensures the QuDAG Exchange system is robust, secure, and performs as expected.

## Test Organization

### Core Tests (`qudag-exchange-core/tests/`)

1. **`test_ledger.rs`** - Core data structures
   - Account management and rUv balances
   - Ledger operations (credit, debit, transfer)
   - Atomic transaction guarantees
   - Property-based testing for invariants

2. **`test_vault_integration.rs`** - Quantum-resistant cryptography
   - ML-DSA key generation and signing
   - ML-KEM encryption/decryption
   - HQC hybrid encryption
   - Vault security and persistence
   - Timing attack resistance

3. **`test_transactions.rs`** - Transaction processing
   - Transaction creation and validation
   - Serialization (JSON, bincode, canonical)
   - BLAKE3 hashing
   - Quantum-resistant signatures
   - Multi-signature support

4. **`test_resource_metering.rs`** - Resource management
   - rUv consumption tracking
   - Resource type definitions and pricing
   - Quota management
   - Usage reports
   - Special operations (quantum, vault, consensus)

5. **`test_consensus_integration.rs`** - QR-Avalanche consensus
   - DAG vertex creation and validation
   - Conflict detection (double-spend prevention)
   - Byzantine fault tolerance
   - Quantum-resistant finality proofs
   - Performance benchmarks

6. **`test_integration.rs`** - End-to-end workflows
   - Complete transaction flow
   - Multi-agent exchanges
   - Resource exhaustion handling
   - Dark domain registration
   - Fault recovery scenarios

## Running Tests

### Basic Test Commands

```bash
# Run all tests
cd qudag-exchange
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test test_ledger

# Run specific test
cargo test test_new_account_creation

# Run tests in parallel (default)
cargo test

# Run tests sequentially
cargo test -- --test-threads=1
```

### Property-Based Tests

```bash
# Run property tests with more iterations
PROPTEST_CASES=1000 cargo test prop_

# Run with specific seed for reproducibility
PROPTEST_SEED=42 cargo test prop_
```

### Performance Tests

```bash
# Run performance benchmarks (ignored by default)
cargo test -- --ignored

# Run specific benchmark
cargo test bench_ledger_operations -- --ignored
```

### Security Tests

```bash
# Run security-focused tests
cargo test security

# Test with sanitizers (requires nightly)
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test
```

## Test Categories

### Unit Tests
Test individual components in isolation:
- Data structure operations
- Cryptographic primitives
- Serialization/deserialization
- Basic validation logic

### Integration Tests
Test interactions between components:
- Vault + Ledger integration
- Transaction + Consensus flow
- Metering + Resource tracking
- Multi-component workflows

### Property-Based Tests
Use `proptest` to verify invariants:
- Balance conservation in transfers
- Deterministic serialization
- Monotonic properties
- Edge case discovery

### Performance Tests
Benchmark critical operations:
- Transaction throughput
- Consensus finalization speed
- Concurrent operation handling
- Large-scale stress tests

### Security Tests
Validate security properties:
- Timing attack resistance
- Memory zeroization
- Quantum resistance
- Byzantine fault tolerance

## Test Data and Fixtures

The test suite includes utilities and fixtures in `mod.rs`:
- `test_utils` - Helper functions for creating test data
- `fixtures` - Pre-configured test environments

## Coverage Goals

- **Line Coverage**: >90%
- **Branch Coverage**: >85%
- **Security Critical Code**: 100%
- **Error Handling Paths**: 100%

## Running Coverage Reports

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/tarpaulin-report.html
```

## Continuous Integration

Tests are automatically run on:
- Every commit
- Pull requests
- Nightly builds (including performance tests)
- Security audit schedule

## Writing New Tests

When adding new features:

1. **Write tests first** following TDD
2. **Use descriptive test names** that explain the scenario
3. **Test both success and failure cases**
4. **Include property-based tests** for complex logic
5. **Add performance tests** for critical paths
6. **Document assumptions** in test comments

## Test Environment Variables

```bash
# Enable debug logging in tests
RUST_LOG=debug cargo test

# Set test timeout
TEST_TIMEOUT=300 cargo test

# Enable backtrace on test failure
RUST_BACKTRACE=1 cargo test
```

## Troubleshooting

### Tests Hanging
- Check for deadlocks in concurrent tests
- Use `--test-threads=1` to isolate
- Enable debug logging

### Flaky Tests
- Look for timing dependencies
- Check for shared state between tests
- Use deterministic test data

### Performance Regressions
- Compare benchmark results over time
- Profile with `cargo flamegraph`
- Check for unnecessary allocations

## Security Testing Notes

- All cryptographic operations use constant-time implementations
- Sensitive data is zeroized after use
- Timing tests verify constant-time properties
- Fuzzing targets are available in `fuzz/`
