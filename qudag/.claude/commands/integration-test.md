# Integration Test Command

Execute comprehensive integration test suite with multi-component coordination and validation.

## Usage
`/integration-test [scope] [components] [scenario]`

## Parameters
- **scope** (optional): Test scope - full, quick, crypto-network, dag-protocol, network-protocol, end-to-end (default: full)
- **components** (optional): Specific components to test together - crypto, dag, network, protocol (comma-separated)
- **scenario** (optional): Integration test scenario - basic, stress, adversarial, performance (default: basic)

## Instructions

When this command is invoked:

1. **Load integration context and test history**:
   - Read `.claude/contexts/integration_context.md` for current integration state
   - Review `.claude/contexts/test_status.md` for recent test results
   - Identify component dependencies and integration points

2. **Coordinate multi-agent test execution**:
   - Activate integration_agent as primary coordinator
   - Delegate component-specific tests to specialist agents:
     - crypto_agent: Cryptographic integration validation
     - network_agent: Network layer integration testing
     - consensus_agent: DAG consensus integration
     - security_agent: Security validation across components
     - performance_agent: Integration performance analysis

3. **Execute integration test scenarios**:

   ### Basic Scenario
   ```bash
   # Component initialization and setup
   cargo test --test integration_basic --features integration
   
   # Inter-component communication tests
   cargo test --test component_communication --features multi-component
   
   # State synchronization verification
   cargo test --test state_sync --features integration
   ```

   ### Stress Scenario
   ```bash
   # High-load multi-component stress testing
   cargo test --test integration_stress --features stress-test
   
   # Concurrent operation testing
   cargo test --test concurrent_ops --features multi-threaded
   
   # Resource exhaustion tests
   cargo test --test resource_limits --features integration
   ```

   ### Adversarial Scenario
   ```bash
   # Byzantine behavior simulation
   cargo test --test byzantine_tests --features adversarial
   
   # Attack scenario validation
   cargo test --test attack_scenarios --features security
   
   # Recovery mechanism testing
   cargo test --test failure_recovery --features resilience
   ```

   ### Performance Scenario
   ```bash
   # Integration performance benchmarks
   cargo bench --bench integration_bench
   
   # End-to-end latency measurement
   cargo bench --bench e2e_latency
   
   # Throughput under integration load
   cargo bench --bench integration_throughput
   ```

4. **Aggregate results from all components**:
   - Collect test results from each component
   - Build component interaction matrix
   - Analyze integration point failures
   - Generate coverage metrics

5. **Analyze inter-component dependencies**:
   - Map data flow between components
   - Verify state consistency across boundaries
   - Identify integration bottlenecks
   - Validate error propagation

6. **Generate comprehensive integration report**:
   - Create detailed test summary
   - Document component interactions
   - Highlight any failures or issues
   - Provide optimization recommendations

## Test Coordination Workflow

### Multi-Component Test Flow
1. **Initialize all protocol components**
   ```rust
   let crypto = Arc::new(CryptoComponent::new());
   let network = Arc::new(NetworkComponent::new());
   let dag = Arc::new(DagComponent::new());
   let protocol = Protocol::new(crypto, network, dag);
   ```

2. **Simulate real-world message flow**
   - Generate test transactions
   - Route through network layer
   - Process in DAG consensus
   - Verify cryptographic signatures

3. **Verify consensus achievement**
   - Check DAG state consistency
   - Validate transaction ordering
   - Confirm finality properties

4. **Test recovery from failures**
   - Inject component failures
   - Verify graceful degradation
   - Test recovery mechanisms

5. **Validate performance metrics**
   - Measure end-to-end latency
   - Check throughput limits
   - Monitor resource usage

## Expected Output

### Integration Test Summary
```
=== QuDAG Integration Test Report ===

Test Scope: full
Components: crypto, dag, network, protocol
Scenario: basic

1. Integration Test Summary
   - Total tests run: 156
   - Passed: 152 (97.4%)
   - Failed: 4 (2.6%)
   - Skipped: 0

2. Component Interaction Matrix
   ┌─────────┬────────┬─────┬─────────┬──────────┐
   │         │ Crypto │ DAG │ Network │ Protocol │
   ├─────────┼────────┼─────┼─────────┼──────────┤
   │ Crypto  │   ✓    │  ✓  │    ✓    │    ✓     │
   │ DAG     │   ✓    │  ✓  │    ✓    │    ✓     │
   │ Network │   ✓    │  ✓  │    ✓    │    ✓     │
   │ Protocol│   ✓    │  ✓  │    ✓    │    ✓     │
   └─────────┴────────┴─────┴─────────┴──────────┘

3. Test Scenario Results
   - Basic: All tests passed
   - Message flow: 1000 msg/s achieved
   - State sync: Consistent across nodes
   - Error handling: Proper propagation

4. Error Analysis
   - Failed: network_partition_recovery_test
     Reason: Timeout during partition healing
     Impact: Minor - affects edge case
   
5. Coverage Report
   - Integration paths: 89% covered
   - Component boundaries: 95% tested
   - Error paths: 87% validated

6. Recommendations
   - Optimize network partition recovery
   - Add more stress test scenarios
   - Enhance monitoring capabilities
```

## Error Handling

- **Test Failure**: 
  - Generate detailed failure report with component trace
  - Identify failing integration point
  - Suggest debugging steps
  
- **Invalid Scope**: 
  - Display valid scopes: full, quick, crypto-network, dag-protocol, network-protocol, end-to-end
  - Recommend appropriate scope based on context
  
- **Component Error**: 
  - Isolate component-specific failures
  - Provide recovery suggestions
  - Generate minimal reproduction case
  
- **Timeout**: 
  - Return partial results with timeout analysis
  - Identify slow integration points
  - Suggest optimization strategies
  
- **Coordination Failure**: 
  - Report multi-agent coordination issues
  - Activate fallback sequential testing
  - Log communication failures

## Success Criteria

- [ ] All integration tests pass for specified scope
- [ ] Component interaction matrix shows full connectivity
- [ ] No performance regressions > 5%
- [ ] Security properties maintained across boundaries
- [ ] Error propagation works correctly
- [ ] State consistency verified
- [ ] Documentation updated with results

## Examples

```bash
# Run full integration test suite
/integration-test

# Quick integration test
/integration-test quick

# Test crypto-network integration under stress
/integration-test crypto-network stress

# Test specific components with adversarial scenario
/integration-test --components crypto,dag,network --scenario adversarial

# End-to-end performance testing
/integration-test end-to-end performance
```

## Related Commands
- `/create-test`: Create new integration test cases
- `/implement-feature`: Implement features to pass integration tests
- `/performance-benchmark`: Detailed performance analysis
- `/security-audit`: Security-focused integration testing

## Workflow Integration
This command is part of the Integration Testing workflow and:
- Follows: `/implement-feature` (after implementation)
- Precedes: `/deploy-validate` (before deployment)
- Can run in parallel with: `/security-audit`, `/performance-benchmark`

## Agent Coordination
- **Primary Agent**: integration_agent (test coordination)
- **Supporting Agents**:
  - crypto_agent: Cryptographic integration validation
  - network_agent: Network layer integration testing
  - consensus_agent: DAG consensus integration
  - security_agent: Cross-component security validation
  - performance_agent: Integration performance analysis

## Notes
- Integration tests may take longer than unit tests (5-30 minutes)
- Ensure all components are built before running
- Some tests require network simulation environment
- Results are cached in `.claude/test-results/` for analysis
- Use `--nocapture` flag for detailed output during debugging