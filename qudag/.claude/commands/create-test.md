# Create Test Command

When the user runs `/create-test [path] [description] [test_type] [components] [tdd_phase]`, execute the following:

## Parameters
- **path**: Path to test file (e.g., `tests/crypto/ml_kem_test.rs`) - REQUIRED
- **description**: Description of the feature to test - REQUIRED  
- **test_type**: Type of test (unit, integration, security, performance, multi-component) - Optional, default: unit
- **components**: Components for integration tests (crypto, dag, network, protocol) - Optional
- **tdd_phase**: TDD phase (red, green, refactor) - Optional, default: red

## Execution Steps

### Step 1: Validate Parameters
1. Verify the path is valid and follows project structure
2. Check if file already exists and warn if it does
3. Parse test type and validate against allowed values
4. If integration or multi-component test, validate component list

### Step 2: Generate Test Based on Type

#### For Unit Tests:
```rust
use super::*;
use proptest::prelude::*;
use test_case::test_case;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_${feature_name}() {
        // TDD RED Phase: This test should fail initially
        // TODO: Implement the actual functionality to make this pass
        
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert!(result.is_ok());
        // TODO: Add more specific assertions based on requirements
    }
    
    // Property-based test example
    proptest! {
        #[test]
        fn test_${feature_name}_properties(input: ValidInput) {
            // Test invariants and properties
            let result = function_under_test(input);
            prop_assert!(result.is_ok());
        }
    }
    
    // Helper function
    fn setup_test_data() -> TestData {
        // TODO: Create test data
        unimplemented!("Create test data for ${description}")
    }
}
```

#### For Integration Tests:
```rust
use qudag_protocol::*;
use qudag_crypto::*;
use qudag_dag::*;
use qudag_network::*;
use tokio::test;
use std::sync::Arc;
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn test_${feature_name}_integration() {
    // TDD RED Phase: Integration test should fail initially
    // Testing: ${description}
    
    // Setup multi-component test environment
    let test_env = TestEnvironment::new().await;
    
    // Initialize components
    let crypto = Arc::new(CryptoComponent::new());
    let network = Arc::new(NetworkComponent::new());
    let dag = Arc::new(DagComponent::new());
    let protocol = Protocol::new(crypto.clone(), network.clone(), dag.clone());

    // Configure component interactions
    test_env.configure_integration(&protocol).await;

    // Execute multi-component test scenario
    let result = protocol.execute_integrated_operation().await;

    // Verify component state consistency
    assert!(verify_component_states(&crypto, &network, &dag).await);
    
    // Verify integration results
    assert!(result.is_success());
    assert_eq!(result.integration_points_tested(), expected_points());
}

// Helper function for state verification
async fn verify_component_states(
    crypto: &Arc<CryptoComponent>,
    network: &Arc<NetworkComponent>,
    dag: &Arc<DagComponent>
) -> bool {
    // TODO: Implement cross-component consistency checks
    unimplemented!("Verify component states for ${description}")
}
```

#### For Security Tests:
```rust
use qudag_crypto::security::*;
use constant_time_eq::constant_time_eq;
use zeroize::Zeroize;

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_${feature_name}_constant_time() {
        // TDD RED Phase: Security test should verify constant-time operations
        // Testing: ${description}
        
        // Test constant-time operations
        let secret1 = generate_secret();
        let secret2 = generate_secret();
        
        // Measure timing for equal secrets
        let start = std::time::Instant::now();
        let equal = constant_time_eq(&secret1, &secret1);
        let equal_time = start.elapsed();
        
        // Measure timing for different secrets
        let start = std::time::Instant::now();
        let not_equal = constant_time_eq(&secret1, &secret2);
        let not_equal_time = start.elapsed();
        
        // Times should be statistically similar
        assert!(equal);
        assert!(!not_equal);
        // TODO: Add statistical timing analysis
    }

    #[test]
    fn test_${feature_name}_zeroization() {
        // Test proper cleanup of sensitive data
        let mut sensitive_data = SensitiveData::new();
        let ptr = sensitive_data.as_ptr();
        
        sensitive_data.zeroize();
        
        // Verify memory is cleared
        unsafe {
            assert_eq!(*ptr, 0);
        }
    }
    
    #[test]
    fn test_${feature_name}_side_channel_resistance() {
        // TODO: Implement side-channel resistance tests
        unimplemented!("Side-channel resistance test for ${description}")
    }
}
```

#### For Performance Tests:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use qudag_benchmarks::*;

fn bench_${feature_name}(c: &mut Criterion) {
    // TDD RED Phase: Performance benchmark baseline
    // Benchmarking: ${description}
    
    let mut group = c.benchmark_group("${feature_name}");
    
    // Test different input sizes
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("baseline", size), size, |b, &size| {
            let input = generate_input(size);
            b.iter(|| {
                black_box(function_under_test(&input));
            });
        });
    }
    
    // Throughput benchmark
    group.throughput(criterion::Throughput::Elements(1000));
    group.bench_function("throughput", |b| {
        let input = generate_bulk_input(1000);
        b.iter(|| {
            black_box(process_bulk(&input));
        });
    });
    
    group.finish();
}

fn generate_input(size: usize) -> TestInput {
    // TODO: Generate input of specified size
    unimplemented!("Generate input for ${description}")
}

criterion_group!(benches, bench_${feature_name});
criterion_main!(benches);
```

#### For Multi-Component Tests:
```rust
use qudag_protocol::*;
use qudag_crypto::*;
use qudag_dag::*;
use qudag_network::*;
use tokio::test;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_${feature_name}_multi_component() {
    // TDD RED Phase: Multi-component integration test
    // Testing: ${description}
    // Components: ${components}
    
    // Initialize multi-component test harness
    let harness = MultiComponentHarness::new()
        .with_crypto()
        .with_network()
        .with_dag()
        .with_protocol()
        .build()
        .await;

    // Scenario: ${description}
    
    // Step 1: Initialize components with test configuration
    harness.initialize_test_scenario().await;
    
    // Step 2: Execute cross-component operations
    let crypto_result = harness.crypto.generate_keys().await?;
    let network_result = harness.network.establish_connection().await?;
    let dag_result = harness.dag.add_transaction(crypto_result.sign()).await?;
    
    // Step 3: Verify component interactions
    assert!(harness.verify_crypto_network_integration().await);
    assert!(harness.verify_network_dag_integration().await);
    assert!(harness.verify_dag_protocol_integration().await);
    
    // Step 4: Test error propagation across components
    harness.inject_network_failure();
    let error_result = harness.protocol.process_message().await;
    assert!(matches!(error_result, Err(ProtocolError::NetworkError(_))));
    
    // Step 5: Verify system recovery
    harness.recover_network();
    assert!(harness.verify_system_consistency().await);
}

struct MultiComponentHarness {
    crypto: Arc<RwLock<CryptoComponent>>,
    network: Arc<RwLock<NetworkComponent>>,
    dag: Arc<RwLock<DagComponent>>,
    protocol: Arc<RwLock<ProtocolComponent>>,
}

impl MultiComponentHarness {
    // TODO: Implement harness builder and verification methods
}
```

### Step 3: Add Documentation and Metadata
1. Add module-level documentation explaining the test purpose
2. Include test metadata for tracking
3. Add TDD phase markers and TODOs

### Step 4: Update Test Tracking
1. Create or update entry in `.claude/contexts/test_status.md`:
   ```markdown
   ## ${module}/${feature}
   - Status: RED
   - Test: ${path}
   - Description: ${description}
   - Type: ${test_type}
   - Created: ${timestamp}
   ```

2. If integration test, update `.claude/contexts/integration_context.md`:
   ```markdown
   ## Integration Test: ${feature}
   - Components: ${components}
   - Test Path: ${path}
   - Status: RED phase
   ```

### Step 5: Provide Next Steps
Display guidance for the user:
```
✅ Created test file: ${path}

Next steps for TDD workflow:
1. Run the test to verify it fails (RED phase):
   cargo test ${test_name}

2. The test should fail with:
   - Compilation errors (missing implementation)
   - Or assertion failures (incorrect behavior)

3. Implement minimal code to make the test pass:
   /implement-feature ${path}

4. Once green, refactor while keeping tests passing:
   /refactor-optimize ${module}

TDD Phase: RED ❌ → GREEN → REFACTOR
```

## Test Templates Reference

### Standard Imports by Type
- **Unit**: `use super::*; use proptest::prelude::*; use test_case::test_case;`
- **Integration**: `use tokio::test; use std::sync::Arc; use wiremock::{Mock, ResponseTemplate};`
- **Security**: `use constant_time_eq::constant_time_eq; use zeroize::Zeroize;`
- **Performance**: `use criterion::{black_box, criterion_group, criterion_main, Criterion};`
- **Multi-Component**: `use tokio::sync::RwLock; use futures::future::join_all;`

### Documentation Standards
Every test file should include:
```rust
//! Test module for ${feature}
//! 
//! This module tests ${description}
//! 
//! Test coverage includes:
//! - Basic functionality
//! - Edge cases
//! - Error conditions
//! - Integration points (if applicable)
```

## Error Handling

If any errors occur during execution:

1. **Invalid Path**: 
   - Suggest correct test directory structure
   - Show example paths for the module

2. **File Already Exists**:
   - Ask if user wants to append tests or create new file
   - Show existing test structure

3. **Invalid Test Type**:
   - Show valid options: unit, integration, security, performance, multi-component
   - Suggest most appropriate type based on description

4. **Missing Components**:
   - For integration tests, require at least 2 components
   - Show valid component options

## Related Commands
- `/implement-feature` - Implement code to pass the test
- `/tdd-cycle` - Execute full TDD cycle
- `/refactor-optimize` - Refactor after tests pass

## Agent Assignment
Based on test type and module:
- **crypto tests** → Use Crypto Agent context
- **network tests** → Use Network Agent context  
- **dag/consensus tests** → Use Consensus Agent context
- **integration tests** → Use Integration Agent context
- **security tests** → Use Security Agent context
- **performance tests** → Use Performance Agent context