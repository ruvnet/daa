# /tdd-cycle

## Purpose
Execute a complete Test-Driven Development (TDD) cycle for implementing a new feature in the QuDAG protocol, following the RED-GREEN-REFACTOR methodology to ensure high-quality, well-tested code.

## Parameters
- `<module>`: Target module name (crypto|dag|network|protocol|consensus|security|performance) - REQUIRED
- `<feature>`: Feature name to implement using TDD methodology - REQUIRED

## Prerequisites
- [ ] Rust development environment is set up
- [ ] Module exists in the workspace structure
- [ ] No existing implementation of the feature
- [ ] Cargo workspace builds successfully

## Execution Steps

### 1. Validation Phase
- Validate module name matches available modules
- Check feature name follows naming conventions
- Verify module directory exists at `/workspaces/QuDAG/core/<module>/`
- Ensure no conflicting implementations exist

### 2. Planning Phase
- Load the appropriate agent context based on module mapping
- Review existing module structure and test organization
- Identify test file location and naming convention
- Plan test scenarios for the feature

### 3. Implementation Phase

#### 3.1 RED Phase - Write Failing Test First
- Create test file at `/workspaces/QuDAG/core/<module>/tests/<feature>_test.rs`
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn test_<feature>_basic_functionality() {
          // Test implementation that should fail
          assert!(false, "Not yet implemented");
      }
  }
  ```
- Add integration tests if needed in `/workspaces/QuDAG/tests/integration/`
- Run tests to verify compilation and initial failure:
  ```bash
  cargo test -p qudag-<module> <feature>
  ```
- Document expected behavior in test comments

#### 3.2 GREEN Phase - Write Minimal Code to Pass Test
- Implement minimal feature code in `/workspaces/QuDAG/core/<module>/src/<feature>.rs`
- Add module declaration to `lib.rs` if new file created
- Focus on making tests pass, not optimization
- Run tests to verify they now pass:
  ```bash
  cargo test -p qudag-<module> <feature>
  ```

#### 3.3 REFACTOR Phase - Improve Code While Keeping Tests Green
- Apply Rust idioms and best practices
- Implement proper error handling with `thiserror`
- Add logging with `tracing` where appropriate
- Ensure constant-time operations for crypto code
- Run all module tests to verify no regressions:
  ```bash
  cargo test -p qudag-<module>
  ```
- Check code quality:
  ```bash
  cargo clippy -p qudag-<module> -- -D warnings
  cargo fmt --check -p qudag-<module>
  ```

### 4. Verification Phase
- Run full test suite to ensure no regressions
- Check test coverage meets >90% threshold
- Verify security requirements for crypto modules
- Benchmark performance if applicable
- Update test status in contexts/test_status.md

### 5. Documentation Phase
- Add inline documentation to new code
- Update module README if exists
- Document API changes in CHANGELOG
- Create examples if public API added

## Success Criteria
- [ ] All new tests pass successfully
- [ ] Test coverage exceeds 90% for new code
- [ ] No existing tests broken (no regressions)
- [ ] Code passes `cargo clippy` with no warnings
- [ ] Code properly formatted with `cargo fmt`
- [ ] Security audit passes for crypto code
- [ ] Performance benchmarks meet targets

## Error Handling
- **Invalid Module**: Display valid modules: crypto, dag, network, protocol, consensus, security, performance
- **Test Compilation Failure**: Show full compiler error and suggest fixes
- **Test Runtime Failure**: Display test output and failure reason
- **Coverage Below Threshold**: List uncovered lines and suggest additional tests
- **Clippy Warnings**: Show warnings and apply suggested fixes
- **Security Audit Failure**: Detail vulnerabilities and required fixes

## Output
- **Success**: 
  ```
  ✅ TDD Cycle Complete for <module>::<feature>
  - Tests written: X
  - Tests passing: X/X
  - Coverage: XX%
  - Implementation: /workspaces/QuDAG/core/<module>/src/<feature>.rs
  - Tests: /workspaces/QuDAG/core/<module>/tests/<feature>_test.rs
  ```
- **Failure**: Detailed error message with recovery steps
- **Reports**: Test results saved to `.claude/test-results/<module>-<feature>.log`

## Example Usage
```
/tdd-cycle crypto ml_kem_keygen
```

### Example Scenario
Implementing ML-KEM key generation in the crypto module:
1. Creates test file: `/workspaces/QuDAG/core/crypto/tests/ml_kem_keygen_test.rs`
2. Writes failing tests for key generation, validation, and edge cases
3. Implements minimal keygen in `/workspaces/QuDAG/core/crypto/src/kem/ml_kem.rs`
4. Refactors to use constant-time operations and proper zeroization
5. Verifies all crypto tests pass with >95% coverage

## Related Commands
- `/create-test`: Create individual test files
- `/implement-feature`: Implement feature for existing tests
- `/refactor-optimize`: Refactor existing implementations
- `/security-audit`: Run security analysis on implementation

## Workflow Integration
This command is part of the TDD Development workflow and:
- Follows: Project setup and module creation
- Precedes: `/security-audit` for crypto features
- Can be run in parallel with: Documentation updates
- References: workflow/tdd_workflow.md

## Agent Coordination
- **Primary Agent**: Determined by module parameter
  - crypto → Crypto Agent (quantum-resistant implementations)
  - network → Network Agent (P2P and routing)
  - dag → Consensus Agent (DAG operations)
  - consensus → Consensus Agent (QR-Avalanche)
  - protocol → Integration Agent (protocol coordination)
  - security → Security Agent (security features)
  - performance → Performance Agent (optimizations)
- **Supporting Agents**: 
  - Security Agent: Reviews all crypto implementations
  - Performance Agent: Validates performance targets
  - Integration Agent: Ensures module compatibility

## Notes
- Always run security audit after implementing crypto features
- Performance benchmarks required for consensus-critical code
- Integration tests needed for cross-module features
- Use property-based testing for complex algorithms
- Ensure all crypto operations are constant-time
- Memory must be securely cleared for sensitive data

## Module Mapping Reference
- `crypto`: Quantum-resistant cryptographic primitives (ML-KEM, ML-DSA, HQC)
- `dag`: DAG structure and operations
- `network`: P2P networking and anonymous routing
- `protocol`: Main protocol coordination
- `consensus`: QR-Avalanche consensus algorithm
- `security`: Security utilities and primitives
- `performance`: Performance-critical optimizations