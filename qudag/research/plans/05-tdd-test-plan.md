# QuDAG CLI Comprehensive TDD Test Plan

## Overview

This document outlines a comprehensive Test-Driven Development (TDD) test plan for all QuDAG CLI commands. The plan follows the RED-GREEN-REFACTOR cycle and covers unit tests, integration tests, mock strategies, performance tests, and CI/CD integration.

## Test Structure and Organization

### Directory Structure

```
tools/cli/tests/
├── unit/
│   ├── args_test.rs              # Command-line argument parsing
│   ├── commands_test.rs          # Individual command logic
│   ├── validation_test.rs        # Input validation
│   ├── error_handling_test.rs    # Error cases and recovery
│   └── output_formatting_test.rs # Output format consistency
├── integration/
│   ├── node_lifecycle_test.rs    # Node start/stop sequences
│   ├── peer_management_test.rs   # Peer operations with running node
│   ├── network_operations_test.rs # Network commands integration
│   ├── address_system_test.rs    # Dark addressing integration
│   └── rpc_interaction_test.rs   # RPC client-server tests
├── performance/
│   ├── command_latency_test.rs   # Command execution timing
│   ├── resource_usage_test.rs    # Memory and CPU monitoring
│   ├── concurrent_ops_test.rs    # Parallel command execution
│   └── stress_test.rs            # High-load scenarios
├── mocks/
│   ├── mock_node.rs              # Mock node implementation
│   ├── mock_network.rs           # Network layer mocks
│   ├── mock_rpc.rs               # RPC server mocks
│   └── mock_dag.rs               # DAG state mocks
└── fixtures/
    ├── test_configs/             # Test configuration files
    ├── test_data/                # Sample data for tests
    └── expected_outputs/         # Expected command outputs
```

## Unit Tests for Each Command

### 1. Node Management Commands

#### `start` Command Tests

```rust
#[cfg(test)]
mod start_command_tests {
    use super::*;
    
    // RED Phase Tests
    #[test]
    fn test_start_with_default_config() {
        // Should start node with default port 8000
        // Should create default data directory
        // Should use info log level
    }
    
    #[test]
    fn test_start_with_custom_port() {
        // Should validate port range (1-65535)
        // Should reject invalid ports
        // Should handle port conflicts
    }
    
    #[test]
    fn test_start_with_data_directory() {
        // Should create directory if not exists
        // Should validate directory permissions
        // Should handle path errors
    }
    
    #[test]
    fn test_start_with_initial_peers() {
        // Should validate peer addresses
        // Should handle invalid peer formats
        // Should connect to specified peers
    }
    
    #[test]
    fn test_start_idempotency() {
        // Should detect already running node
        // Should provide clear error message
        // Should not corrupt existing state
    }
    
    #[test]
    fn test_start_signal_handling() {
        // Should handle SIGINT gracefully
        // Should handle SIGTERM gracefully
        // Should clean up resources on exit
    }
}
```

#### `stop` Command Tests

```rust
#[cfg(test)]
mod stop_command_tests {
    use super::*;
    
    #[test]
    fn test_stop_running_node() {
        // Should send shutdown signal
        // Should wait for graceful shutdown
        // Should confirm node stopped
    }
    
    #[test]
    fn test_stop_with_timeout() {
        // Should force stop after timeout
        // Should clean up resources
        // Should report timeout error
    }
    
    #[test]
    fn test_stop_non_existent_node() {
        // Should detect no running node
        // Should provide clear message
        // Should exit cleanly
    }
    
    #[test]
    fn test_stop_multiple_nodes() {
        // Should stop specific node by port
        // Should handle multiple instances
        // Should not affect other nodes
    }
}
```

#### `status` Command Tests

```rust
#[cfg(test)]
mod status_command_tests {
    use super::*;
    
    #[test]
    fn test_status_running_node() {
        // Should show node state (running/stopped)
        // Should display uptime
        // Should show resource usage
        // Should list active connections
    }
    
    #[test]
    fn test_status_output_formats() {
        // Should support JSON output
        // Should support human-readable output
        // Should support machine-parseable output
    }
    
    #[test]
    fn test_status_performance_metrics() {
        // Should show message throughput
        // Should display consensus metrics
        // Should report DAG statistics
    }
    
    #[test]
    fn test_status_error_conditions() {
        // Should handle RPC connection failure
        // Should handle partial data availability
        // Should provide degraded status info
    }
}
```

### 2. Peer Management Commands

#### `peer list` Command Tests

```rust
#[cfg(test)]
mod peer_list_tests {
    use super::*;
    
    #[test]
    fn test_list_connected_peers() {
        // Should show peer addresses
        // Should display connection duration
        // Should show peer statistics
    }
    
    #[test]
    fn test_list_with_filters() {
        // Should filter by connection state
        // Should filter by peer type
        // Should support regex patterns
    }
    
    #[test]
    fn test_list_sorting_options() {
        // Should sort by connection time
        // Should sort by data transferred
        // Should sort by latency
    }
    
    #[test]
    fn test_list_pagination() {
        // Should handle large peer lists
        // Should support page navigation
        // Should maintain consistent ordering
    }
}
```

#### `peer add` Command Tests

```rust
#[cfg(test)]
mod peer_add_tests {
    use super::*;
    
    #[test]
    fn test_add_valid_peer() {
        // Should validate address format
        // Should establish connection
        // Should update peer list
    }
    
    #[test]
    fn test_add_dark_address_peer() {
        // Should resolve dark address
        // Should handle resolution failures
        // Should establish encrypted connection
    }
    
    #[test]
    fn test_add_duplicate_peer() {
        // Should detect existing connection
        // Should not create duplicate
        // Should report as already connected
    }
    
    #[test]
    fn test_add_unreachable_peer() {
        // Should timeout connection attempt
        // Should report connection failure
        // Should not corrupt peer list
    }
    
    #[test]
    fn test_add_with_connection_params() {
        // Should apply custom timeout
        // Should use specified encryption
        // Should respect retry policy
    }
}
```

#### `peer remove` Command Tests

```rust
#[cfg(test)]
mod peer_remove_tests {
    use super::*;
    
    #[test]
    fn test_remove_connected_peer() {
        // Should close connection gracefully
        // Should update peer list
        // Should notify peer of disconnect
    }
    
    #[test]
    fn test_remove_non_existent_peer() {
        // Should handle missing peer gracefully
        // Should provide clear message
        // Should not affect other peers
    }
    
    #[test]
    fn test_remove_with_force_flag() {
        // Should force immediate disconnect
        // Should not wait for graceful close
        // Should clean up resources
    }
    
    #[test]
    fn test_remove_all_peers() {
        // Should support wildcard removal
        // Should confirm before bulk remove
        // Should maintain node operation
    }
}
```

### 3. Network Commands

#### `network stats` Command Tests

```rust
#[cfg(test)]
mod network_stats_tests {
    use super::*;
    
    #[test]
    fn test_stats_basic_metrics() {
        // Should show bandwidth usage
        // Should display message counts
        // Should report connection stats
    }
    
    #[test]
    fn test_stats_time_windows() {
        // Should support 1m, 5m, 1h windows
        // Should calculate moving averages
        // Should show trend indicators
    }
    
    #[test]
    fn test_stats_per_peer_breakdown() {
        // Should show per-peer statistics
        // Should identify top consumers
        // Should detect anomalies
    }
    
    #[test]
    fn test_stats_export_formats() {
        // Should export to CSV
        // Should export to JSON
        // Should support Prometheus format
    }
}
```

#### `network test` Command Tests

```rust
#[cfg(test)]
mod network_test_tests {
    use super::*;
    
    #[test]
    fn test_connectivity_check() {
        // Should ping all peers
        // Should measure latency
        // Should detect packet loss
    }
    
    #[test]
    fn test_bandwidth_test() {
        // Should measure upload speed
        // Should measure download speed
        // Should test concurrent connections
    }
    
    #[test]
    fn test_nat_traversal_check() {
        // Should detect NAT type
        // Should test hole punching
        // Should verify accessibility
    }
    
    #[test]
    fn test_network_diagnostics() {
        // Should run comprehensive tests
        // Should generate diagnostic report
        // Should suggest optimizations
    }
}
```

### 4. Address Commands

#### `address register` Command Tests

```rust
#[cfg(test)]
mod address_register_tests {
    use super::*;
    
    #[test]
    fn test_register_valid_domain() {
        // Should validate domain format
        // Should check availability
        // Should complete registration
    }
    
    #[test]
    fn test_register_duplicate_domain() {
        // Should detect existing registration
        // Should suggest alternatives
        // Should handle gracefully
    }
    
    #[test]
    fn test_register_with_metadata() {
        // Should attach service info
        // Should set TTL values
        // Should configure encryption
    }
    
    #[test]
    fn test_register_validation_rules() {
        // Should enforce naming rules
        // Should check reserved names
        // Should validate character set
    }
}
```

#### `address resolve` Command Tests

```rust
#[cfg(test)]
mod address_resolve_tests {
    use super::*;
    
    #[test]
    fn test_resolve_existing_address() {
        // Should return network address
        // Should include metadata
        // Should verify signatures
    }
    
    #[test]
    fn test_resolve_non_existent() {
        // Should handle not found gracefully
        // Should suggest similar names
        // Should provide clear error
    }
    
    #[test]
    fn test_resolve_with_caching() {
        // Should cache results locally
        // Should respect TTL values
        // Should handle cache invalidation
    }
    
    #[test]
    fn test_resolve_security_checks() {
        // Should verify authenticity
        // Should check revocation status
        // Should validate encryption keys
    }
}
```

## Integration Tests for Command Interactions

### Node Lifecycle Integration Tests

```rust
#[cfg(test)]
mod node_lifecycle_integration {
    use super::*;
    
    #[test]
    async fn test_full_node_lifecycle() {
        // Start node
        // Verify status shows running
        // Add peers
        // Verify peer connections
        // Generate network traffic
        // Check statistics
        // Stop node gracefully
        // Verify cleanup
    }
    
    #[test]
    async fn test_node_restart_preserves_state() {
        // Start node with peers
        // Create some state (addresses, etc)
        // Stop node
        // Start node again
        // Verify state preservation
        // Verify peer reconnection
    }
    
    #[test]
    async fn test_multiple_node_coordination() {
        // Start multiple nodes
        // Connect them as peers
        // Verify message propagation
        // Test consensus formation
        // Coordinate shutdown
    }
}
```

### Peer Management Integration Tests

```rust
#[cfg(test)]
mod peer_management_integration {
    use super::*;
    
    #[test]
    async fn test_peer_discovery_flow() {
        // Start node
        // Add bootstrap peer
        // Verify peer discovery
        // Check peer list growth
        // Test peer rotation
    }
    
    #[test]
    async fn test_peer_reconnection_logic() {
        // Establish peer connections
        // Simulate network partition
        // Verify reconnection attempts
        // Check exponential backoff
        // Verify eventual consistency
    }
}
```

## Mock Strategies for Network/Node Components

### Mock Node Implementation

```rust
pub struct MockNode {
    state: Arc<RwLock<NodeState>>,
    peers: Arc<RwLock<Vec<PeerInfo>>>,
    message_log: Arc<RwLock<Vec<Message>>>,
}

impl MockNode {
    pub fn new() -> Self {
        // Initialize with configurable state
    }
    
    pub fn with_scenario(scenario: TestScenario) -> Self {
        // Create node with predefined behavior
    }
    
    pub fn simulate_failure(&self, failure_type: FailureType) {
        // Inject various failure conditions
    }
}
```

### Mock Network Layer

```rust
pub struct MockNetwork {
    latency_profile: LatencyProfile,
    packet_loss_rate: f64,
    bandwidth_limit: Option<usize>,
}

impl MockNetwork {
    pub fn simulate_conditions(&self, conditions: NetworkConditions) {
        // Simulate various network conditions
    }
    
    pub fn record_traffic(&self) -> TrafficLog {
        // Record all network operations for verification
    }
}
```

### Mock RPC Server

```rust
pub struct MockRpcServer {
    handlers: HashMap<String, Box<dyn RpcHandler>>,
    call_history: Arc<RwLock<Vec<RpcCall>>>,
}

impl MockRpcServer {
    pub fn expect_call(&mut self, method: &str) -> &mut Self {
        // Set up expected RPC calls
    }
    
    pub fn verify_calls(&self) {
        // Verify all expected calls were made
    }
}
```

## Performance and Stress Tests

### Command Latency Tests

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[test]
    fn test_command_execution_time() {
        // Measure startup time
        // Measure command parsing time
        // Measure RPC round-trip time
        // Assert under threshold
    }
    
    #[test]
    fn test_large_peer_list_performance() {
        // Create 1000+ peers
        // Measure list command time
        // Verify pagination performance
        // Check memory usage
    }
}
```

### Stress Tests

```rust
#[cfg(test)]
mod stress_tests {
    use super::*;
    
    #[test]
    #[ignore] // Run only in stress test mode
    fn test_concurrent_command_execution() {
        // Execute 100+ commands concurrently
        // Verify no race conditions
        // Check resource cleanup
        // Monitor error rates
    }
    
    #[test]
    #[ignore]
    fn test_sustained_operation() {
        // Run node for extended period
        // Execute commands continuously
        // Monitor resource leaks
        // Verify stability
    }
}
```

## Test Data and Fixtures

### Configuration Fixtures

```yaml
# test_configs/minimal.yaml
node:
  port: 8000
  data_dir: "./test_data"
  log_level: "debug"

# test_configs/full.yaml
node:
  port: 8000
  data_dir: "./test_data"
  log_level: "info"
  initial_peers:
    - "192.168.1.10:8000"
    - "peer1.dark:8000"
  max_connections: 100
  encryption: "ml-kem"
```

### Test Data Generation

```rust
pub mod test_data {
    pub fn generate_peer_addresses(count: usize) -> Vec<String> {
        // Generate realistic peer addresses
    }
    
    pub fn generate_dark_domains(count: usize) -> Vec<String> {
        // Generate valid dark domain names
    }
    
    pub fn generate_network_traffic(size: usize) -> Vec<Message> {
        // Generate realistic message patterns
    }
}
```

### Expected Output Fixtures

```json
// expected_outputs/status_running.json
{
  "status": "running",
  "uptime_seconds": 3600,
  "peers_connected": 5,
  "messages_processed": 1000,
  "dag_height": 100,
  "memory_usage_mb": 45.2,
  "cpu_usage_percent": 12.5
}
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: CLI Tests

on:
  push:
    paths:
      - 'tools/cli/**'
      - '.github/workflows/cli-tests.yml'
  pull_request:
    paths:
      - 'tools/cli/**'

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run unit tests
        run: |
          cd tools/cli
          cargo test --lib
          
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run integration tests
        run: |
          cd tools/cli
          cargo test --test '*' -- --test-threads=1
          
  performance-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run performance benchmarks
        run: |
          cd tools/cli
          cargo bench --bench cli_benchmarks
          
  stress-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run stress tests
        run: |
          cd tools/cli
          cargo test --test stress_tests --release -- --ignored
```

### Test Coverage Requirements

```toml
# .cargo/config.toml
[target.'cfg(all())']
rustflags = ["-C", "instrument-coverage"]

# Minimum coverage thresholds
[coverage]
unit_tests = 90
integration_tests = 80
total = 85
```

### Continuous Monitoring

```yaml
# monitoring/cli_metrics.yaml
metrics:
  - name: cli_command_duration
    type: histogram
    help: "Duration of CLI command execution"
    labels: ["command", "status"]
    
  - name: cli_error_rate
    type: counter
    help: "Number of CLI command errors"
    labels: ["command", "error_type"]
    
  - name: cli_active_connections
    type: gauge
    help: "Number of active RPC connections"
```

## Test Execution Strategy

### TDD Workflow

1. **RED Phase**
   - Write failing test for new feature
   - Ensure test captures requirements
   - Verify test fails for right reason

2. **GREEN Phase**
   - Implement minimal code to pass
   - Focus on correctness over optimization
   - Ensure all tests pass

3. **REFACTOR Phase**
   - Improve code structure
   - Optimize performance
   - Maintain test coverage

### Test Pyramid

```
         /\
        /  \    E2E Tests (5%)
       /    \   - Full system scenarios
      /──────\  - User journey tests
     /        \ 
    /          \ Integration Tests (25%)
   /            \ - Multi-component tests
  /──────────────\ - API contract tests
 /                \
/                  \ Unit Tests (70%)
────────────────────  - Fast, isolated tests
                      - High coverage
```

### Test Prioritization

1. **Critical Path Tests**
   - Node start/stop commands
   - Peer connection management
   - Network communication

2. **Security Tests**
   - Authentication/authorization
   - Encryption verification
   - Input validation

3. **Performance Tests**
   - Command response time
   - Resource usage
   - Scalability limits

4. **Edge Case Tests**
   - Error conditions
   - Recovery scenarios
   - Boundary conditions

## Maintenance and Evolution

### Test Maintenance Guidelines

- Keep tests independent and isolated
- Use descriptive test names
- Maintain test documentation
- Regular test cleanup and refactoring
- Monitor test execution time

### Test Evolution Strategy

- Add tests for each bug fix
- Update tests for API changes
- Expand test scenarios based on usage
- Performance regression prevention
- Security test enhancement

## Conclusion

This comprehensive TDD test plan provides a solid foundation for developing and maintaining the QuDAG CLI with high quality and reliability. The plan emphasizes:

- Complete command coverage
- Realistic test scenarios
- Effective mocking strategies
- Performance validation
- Continuous integration

Following this plan ensures the CLI remains robust, performant, and maintainable as the QuDAG protocol evolves.