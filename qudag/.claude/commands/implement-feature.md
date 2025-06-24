# Implement Feature Command

Implement features to pass specified tests following Test-Driven Development workflow with multi-component integration support.

## Usage
`/implement-feature <test-path> [feature-type] [integration-points] [tdd-phase]`

## Parameters
- **test-path** (required): Path to test file or test module (e.g., `tests/crypto/ml_kem_test.rs`)
- **feature-type** (optional): Type of feature - crypto, network, dag, protocol, integration
- **integration-points** (optional): Components that need integration - crypto, dag, network, protocol (comma-separated)
- **tdd-phase** (optional): TDD phase to execute - red, green, refactor, all (default: all)

## Instructions

When this command is invoked:

1. **Analyze test requirements and integration needs**:
   - Read the specified test file to understand requirements
   - Identify expected behavior and API contracts
   - Map integration points if multi-component test
   - Determine implementation scope and dependencies

2. **Execute TDD workflow phases**:

   ### RED Phase - Ensure tests fail appropriately
   ```bash
   # Run tests to verify they fail as expected
   cargo test --test $(basename $test_path .rs) -- --nocapture
   
   # Analyze failure patterns
   # Document expected behavior
   # Identify missing implementations
   ```

   ### GREEN Phase - Implement minimal code to pass tests
   ```bash
   # Create/modify implementation files
   # Add core functionality
   # Handle edge cases from tests
   # Ensure all tests pass
   cargo test --test $(basename $test_path .rs)
   ```

   ### REFACTOR Phase - Optimize and clean up
   ```bash
   # Improve code structure
   # Optimize performance
   # Enhance documentation
   # Verify tests still pass
   cargo test --workspace
   ```

3. **Implement feature based on test type**:

   ### Unit Test Implementation
   ```rust
   // Implement in appropriate module
   pub struct FeatureName {
       // Internal state
   }
   
   impl FeatureName {
       pub fn new() -> Self {
           // Constructor implementation
       }
       
       pub fn operation(&self) -> Result<Output, Error> {
           // Core functionality matching test expectations
       }
   }
   ```

   ### Integration Test Implementation
   ```rust
   // Integration layer implementation
   pub struct IntegratedFeature {
       crypto: Arc<CryptoComponent>,
       network: Arc<NetworkComponent>,
       dag: Arc<DagComponent>,
   }
   
   impl IntegratedFeature {
       pub async fn execute(&self) -> Result<()> {
           // Coordinate components
           let crypto_result = self.crypto.process().await?;
           let network_result = self.network.send(crypto_result).await?;
           let dag_result = self.dag.add_transaction(network_result).await?;
           Ok(())
       }
   }
   ```

4. **Coordinate multi-agent implementation**:
   - Integration agent coordinates overall implementation
   - Delegate component-specific work to specialist agents:
     - crypto_agent: Cryptographic feature implementation
     - network_agent: Network feature implementation
     - consensus_agent: DAG/consensus feature implementation
   - Ensure consistent interfaces across components
   - Validate integration points

5. **Validate implementation quality**:
   ```bash
   # Run all tests including new implementation
   cargo test --workspace
   
   # Check code coverage
   cargo tarpaulin --out Html
   
   # Run security checks
   cargo audit
   
   # Verify performance
   cargo bench --bench relevant_benchmark
   ```

6. **Update documentation and integration guides**:
   - Add inline documentation for public APIs
   - Update module-level documentation
   - Create integration examples
   - Document any breaking changes

## Implementation Workflow

### 1. Test Analysis Phase
- Parse test file to extract requirements
- Identify test patterns (unit, integration, property-based)
- Map dependencies and integration points
- Create implementation plan

### 2. Component Implementation
- Create module structure if needed
- Implement core types and traits
- Add error handling with proper types
- Implement business logic to satisfy tests

### 3. Integration Implementation
- Design component interfaces
- Implement integration glue code
- Add proper synchronization for async operations
- Ensure error propagation across boundaries

### 4. Validation Phase
- Run unit tests for component
- Execute integration tests
- Verify no regression in existing tests
- Check performance impact

### 5. Documentation Phase
- Add comprehensive doc comments
- Create usage examples
- Update API documentation
- Document integration patterns

## Expected Output

### Implementation Summary
```
=== Feature Implementation Report ===

Feature: ML-KEM Key Generation
Test Path: tests/crypto/ml_kem_test.rs
Components Modified: crypto
Integration Points: dag, network

1. Implementation Summary
   - Feature: ML-KEM post-quantum key generation
   - Files Modified: 
     - core/crypto/src/kem/ml_kem.rs (new)
     - core/crypto/src/kem/mod.rs (updated)
     - core/crypto/src/lib.rs (updated)
   - Lines Added: 342
   - Lines Modified: 28

2. Test Results
   - Unit Tests: 15/15 passing ✓
   - Integration Tests: 8/8 passing ✓
   - Property Tests: 1000 cases passed ✓
   - Performance Tests: 
     - Key generation: 2.3ms (target: <5ms) ✓
     - Encapsulation: 1.8ms (target: <3ms) ✓

3. Code Quality
   - Coverage: 94.2% (target: >90%) ✓
   - Clippy: No warnings ✓
   - Security Audit: Passed ✓
   - Documentation: Complete ✓

4. Integration Validation
   - Crypto ↔ DAG: Transaction signing works ✓
   - Crypto ↔ Network: Key exchange functional ✓
   - Error propagation: Properly handled ✓

5. Performance Impact
   - CPU: +2% during key operations
   - Memory: +512KB for key storage
   - Throughput: No regression detected

6. Next Steps
   - Run integration tests with full protocol
   - Deploy to test environment
   - Monitor performance metrics
```

## Error Handling

- **Test Not Found**: 
  - Verify test file path exists
  - Suggest similar test files
  - Show proper path format
  
- **Implementation Error**: 
  - Display compilation errors with context
  - Suggest fixes based on error type
  - Provide rollback instructions
  
- **Test Failure**: 
  - Show detailed test output
  - Identify failing assertions
  - Suggest debugging approach
  
- **Integration Failure**: 
  - Isolate component causing issue
  - Show integration trace
  - Provide fix suggestions
  
- **Performance Regression**: 
  - Display benchmark comparison
  - Identify slow operations
  - Suggest optimization strategies

## Success Criteria

- [ ] All specified tests pass (100%)
- [ ] No regression in existing tests
- [ ] Code coverage ≥ 90% for new code
- [ ] Performance targets met
- [ ] Security audit passes
- [ ] Documentation complete
- [ ] Integration points tested
- [ ] No clippy warnings

## Examples

```bash
# Implement crypto feature from test
/implement-feature tests/crypto/ml_kem_test.rs

# Implement network feature with type hint
/implement-feature tests/network/peer_test.rs network

# Implement with specific integration points
/implement-feature tests/integration/crypto_dag_test.rs --integration-points crypto,dag

# Execute only GREEN phase for existing RED test
/implement-feature tests/protocol/handshake_test.rs --tdd-phase green

# Full integration implementation
/implement-feature tests/integration/end_to_end_test.rs integration crypto,dag,network,protocol
```

## Related Commands
- `/create-test`: Create test before implementation
- `/integration-test`: Run integration tests after implementation
- `/refactor-optimize`: Refactor after tests pass
- `/security-audit`: Validate security properties
- `/performance-benchmark`: Verify performance targets

## Workflow Integration
This command is part of the TDD workflow and:
- Follows: `/create-test` (test creation)
- Precedes: `/integration-test` (validation)
- Can run in parallel with: Documentation updates

## Agent Coordination
- **Primary Agent**: integration_agent (coordination)
- **Supporting Agents**:
  - crypto_agent: Crypto feature implementation
  - network_agent: Network feature implementation
  - consensus_agent: DAG feature implementation
  - security_agent: Security review during implementation
  - performance_agent: Performance validation

## Notes
- Always run tests before starting implementation (RED phase)
- Keep implementations minimal in GREEN phase
- Only optimize during REFACTOR phase
- Commit after each successful phase
- Use feature flags for incomplete implementations
- Integration tests may require multiple components to be implemented