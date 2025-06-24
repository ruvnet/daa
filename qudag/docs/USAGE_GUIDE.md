# QuDAG Usage Guide

This comprehensive guide covers all aspects of using the QuDAG system, from basic setup to advanced development workflows.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Running a QuDAG Node](#running-a-qudag-node)
3. [Using the CLI Commands](#using-the-cli-commands)
4. [Peer Management](#peer-management)
5. [Network Operations](#network-operations)
6. [Running Tests](#running-tests)
7. [Running Benchmarks](#running-benchmarks)
8. [Using the Simulator](#using-the-simulator)
9. [Development Workflow](#development-workflow)
10. [Examples and Common Scenarios](#examples-and-common-scenarios)

## Quick Start

### Prerequisites

- Rust 1.70+ (stable)
- Git
- 4GB+ RAM
- 10GB+ disk space

### Installation

#### Method 1: Using the Installation Script (Recommended)

```bash
# Clone the repository
git clone https://github.com/ruvnet/QuDAG
cd QuDAG

# Run the installation script
./install.sh

# For system-wide installation (requires sudo)
sudo ./install.sh --system

# For custom installation directory
./install.sh --prefix /opt/qudag

# View all installation options
./install.sh --help
```

#### Method 2: Using Make

```bash
# Install to ~/.local (user directory)
make install

# Install system-wide (requires sudo)
make install-system

# Install debug build for development
make install-debug

# Install to CARGO_HOME/bin
make install-cargo
```

#### Method 3: Manual Build

```bash
# Build the project (optimized release build)
cargo build --release

# The binary will be at ./target/release/qudag-cli
# You can manually copy it to your preferred location
cp ./target/release/qudag-cli ~/.local/bin/qudag

# Add to PATH if needed
export PATH="$HOME/.local/bin:$PATH"
```

#### Post-Installation

After installation, verify the CLI is working:

```bash
# Check version
qudag --version

# View help
qudag --help

# If installed to ~/.local/bin, you may need to reload your shell
source ~/.bashrc  # or ~/.zshrc, etc.
```

#### Uninstallation

```bash
# Using the uninstall script
./uninstall.sh

# Or using make
make uninstall

# Manual uninstallation
rm ~/.local/bin/qudag
```

### First Run

```bash
# Start your first node with default settings
qudag start

# In another terminal, check node status
qudag status

# Stop the node gracefully
qudag stop
```

## Running a QuDAG Node

### Basic Node Operations

#### Starting a Node

```bash
# Start with default settings (port 8000, data in ./data)
./target/release/qudag start

# Start with custom port
./target/release/qudag start --port 8080

# Start with custom data directory
./target/release/qudag start --data-dir /path/to/data

# Start with verbose logging
./target/release/qudag start --log-level debug

# Start with all options
./target/release/qudag start \
  --port 8080 \
  --data-dir ./my-node \
  --log-level debug
```

#### Checking Node Status

```bash
# Basic status check (default port 8000)
./target/release/qudag status

# Status with custom port
./target/release/qudag status --port 8080

# Status with JSON output
./target/release/qudag status --format json

# Status with table format
./target/release/qudag status --format table

# Verbose status information
./target/release/qudag status --verbose

# Status with custom timeout
./target/release/qudag status --timeout-seconds 60
```

#### Stopping a Node

```bash
# Stop node on default port
./target/release/qudag stop

# Stop node on custom port
./target/release/qudag stop --port 8080
```

### Node Configuration

The node can be configured through command-line arguments:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `--port` | 8000 | Network port for node communication |
| `--data-dir` | `./data` | Directory for node data storage |
| `--log-level` | `info` | Logging level (trace, debug, info, warn, error) |
| `--max-peers` | 50 | Maximum number of peer connections |

### Node Data Structure

When running, the node creates the following directory structure:

```
data/
├── config/           # Node configuration
├── dag/             # DAG state data
├── keys/            # Cryptographic keys
├── network/         # Network state
└── logs/            # Application logs
```

## Using the CLI Commands

### Command Structure

The QuDAG CLI follows a hierarchical command structure:

```
qudag <command> [subcommand] [options]
```

### Available Commands

#### Node Management

```bash
# Start a node
qudag start [--port PORT] [--data-dir DIR] [--log-level LEVEL]

# Stop a node
qudag stop [--port PORT]

# Get node status
qudag status [--port PORT] [--format FORMAT] [--verbose]
```

#### Dark Addressing

```bash
# Register a .dark domain
qudag address register myservice.dark

# Resolve a .dark domain
qudag address resolve myservice.dark

# Generate a shadow address (temporary)
qudag address shadow --ttl 3600  # 1 hour TTL

# Create a quantum fingerprint
qudag address fingerprint --data "Important data to fingerprint"
```

#### Peer Management

```bash
# List all connected peers
qudag peer list

# Add a new peer
qudag peer add <peer-address>

# Remove a peer
qudag peer remove <peer-address>

# Ban a peer
qudag peer ban <peer-address>

# Get peer statistics
qudag peer stats <peer-address>

# Export peer list
qudag peer export --output peers.json
```

#### Network Operations

```bash
# Get network statistics
qudag network stats

# Run network connectivity tests
qudag network test
```

### Command Options

Most commands support these common options:

- `--help`: Show help information
- `--version`: Show version information
- `--verbose`: Enable verbose output
- `--quiet`: Suppress non-error output

## Peer Management

### Listing Peers

```bash
# List all peers
qudag peer list

# List peers with specific status
qudag peer list --status connected

# List peers in JSON format
qudag peer list --format json
```

### Adding Peers

```bash
# Add a single peer
qudag peer add /ip4/192.168.1.100/tcp/8000/p2p/QmPeer123

# Add peers from a file
qudag peer add --file peers.txt

# Add peer with connection timeout
qudag peer add /ip4/192.168.1.100/tcp/8000/p2p/QmPeer123 --timeout 30
```

### Managing Peer Connections

```bash
# Remove a specific peer
qudag peer remove QmPeer123

# Force disconnect a peer
qudag peer remove QmPeer123 --force

# Ban a peer (prevents reconnection)
qudag peer ban QmPeer123

# Get detailed peer statistics
qudag peer stats QmPeer123
```

### Peer List Export/Import

```bash
# Export current peer list
qudag peer export --output my-peers.json

# Import peers from file (when adding)
qudag peer add --file imported-peers.json
```

## Network Operations

### Network Statistics

```bash
# Get comprehensive network statistics
qudag network stats

# Example output:
Network Statistics:
==================
  Total Connections:    42
  Active Connections:   38
  Messages Sent:        125,432
  Messages Received:    118,765
  Bytes Sent:           1.2 GB
  Bytes Received:       1.1 GB
  Average Latency:      45.3 ms
  Uptime:              2h 34m 12s
```

### Network Testing

```bash
# Run full network test suite
qudag network test

# Test includes:
# - Port accessibility
# - NAT traversal
# - Bandwidth testing
# - Latency measurements
# - Peer discovery
# - Message routing
```

### Network Diagnostics

```bash
# Check network connectivity
qudag network test

# Monitor network performance
watch -n 5 qudag network stats
```

## Running Tests

### Unit Tests

```bash
# Run all unit tests
cargo test

# Run tests for specific module
cargo test -p qudag-crypto
cargo test -p qudag-network
cargo test -p qudag-dag
cargo test -p qudag-protocol
cargo test -p qudag-cli

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_ml_kem_encryption

# Run tests in parallel
cargo test -- --test-threads=8
```

### Integration Tests

```bash
# Run all integration tests
cargo test --test integration

# Run specific integration test suite
cargo test --test network_integration
cargo test --test dag_integration
cargo test --test protocol_integration
```

### Security Tests

```bash
# Run security-focused tests
cargo test --features security-tests

# Run timing attack tests
cargo test test_constant_time

# Run memory safety tests
cargo test --features memory-tests
```

### Property-Based Tests

```bash
# Run property tests (uses proptest)
cargo test --features proptest

# Run with more test cases
PROPTEST_CASES=10000 cargo test
```

### Fuzz Testing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzing campaign
cd fuzz
cargo fuzz run crypto_fuzzer

# Run all fuzzers
./run_all_fuzz_tests.sh
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View coverage report
open tarpaulin-report.html
```

## Running Benchmarks

### Performance Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench crypto_benchmarks
cargo bench --bench network_benchmarks
cargo bench --bench consensus_benchmarks
cargo bench --bench system_benchmarks

# Run benchmarks and save baseline
cargo bench -- --save-baseline my-baseline

# Compare against baseline
cargo bench -- --baseline my-baseline
```

### Benchmark Categories

#### Cryptographic Benchmarks

```bash
# ML-KEM operations
cargo bench --bench crypto_benchmarks -- ml_kem

# ML-DSA operations
cargo bench --bench crypto_benchmarks -- ml_dsa

# BLAKE3 hashing
cargo bench --bench crypto_benchmarks -- blake3

# Quantum fingerprinting
cargo bench --bench crypto_benchmarks -- fingerprint
```

#### Network Benchmarks

```bash
# Message routing
cargo bench --bench network_benchmarks -- routing

# Peer discovery
cargo bench --bench network_benchmarks -- discovery

# Onion encryption
cargo bench --bench network_benchmarks -- onion
```

#### Consensus Benchmarks

```bash
# QR-Avalanche consensus
cargo bench --bench consensus_benchmarks -- avalanche

# DAG operations
cargo bench --bench consensus_benchmarks -- dag

# Tip selection
cargo bench --bench consensus_benchmarks -- tip_selection
```

### Profiling

```bash
# CPU profiling with perf
cargo build --release
perf record --call-graph=dwarf ./target/release/qudag start
perf report

# Memory profiling with valgrind
valgrind --tool=memcheck --leak-check=full ./target/release/qudag start

# Heap profiling
valgrind --tool=massif ./target/release/qudag start
ms_print massif.out.*
```

## Using the Simulator

### Running Simulations

```bash
# Build the simulator
cargo build -p qudag-simulator --release

# Run basic simulation
./target/release/qudag-simulator

# Run specific scenario
./target/release/qudag-simulator --scenario high-load

# Run with custom parameters
./target/release/qudag-simulator \
  --nodes 100 \
  --duration 3600 \
  --message-rate 1000
```

### Simulation Scenarios

#### Available Scenarios

1. **Basic Network**: Simple network with default parameters
2. **High Load**: Network under heavy message load
3. **Network Partition**: Simulates network splits
4. **Byzantine Nodes**: Tests consensus with malicious nodes
5. **Churn**: High peer join/leave rate
6. **Latency Variation**: Variable network conditions

```bash
# Run specific scenarios
./target/release/qudag-simulator --scenario basic-network
./target/release/qudag-simulator --scenario high-load
./target/release/qudag-simulator --scenario network-partition
./target/release/qudag-simulator --scenario byzantine-nodes
./target/release/qudag-simulator --scenario churn
./target/release/qudag-simulator --scenario latency-variation
```

### Simulation Parameters

```bash
# Configure simulation parameters
./target/release/qudag-simulator \
  --nodes 50 \                    # Number of nodes
  --duration 1800 \               # Duration in seconds
  --message-rate 500 \            # Messages per second
  --peer-connections 10 \         # Connections per node
  --byzantine-percentage 10 \     # Percentage of byzantine nodes
  --network-latency 50 \          # Base network latency (ms)
  --packet-loss 0.01              # Packet loss rate (0-1)
```

### Analyzing Results

```bash
# Generate simulation report
./target/release/qudag-simulator --scenario high-load --report

# Export metrics
./target/release/qudag-simulator --export metrics.json

# Visualize network topology
./target/release/qudag-simulator --visualize network.dot
dot -Tpng network.dot -o network.png
```

## Development Workflow

### Setting Up Development Environment

```bash
# Install development tools
rustup component add rustfmt clippy
cargo install cargo-watch cargo-audit cargo-outdated

# Set up pre-commit hooks
cp scripts/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
```

### Development Commands

```bash
# Watch for changes and rebuild
cargo watch -x build

# Watch and run tests
cargo watch -x test

# Format code
cargo fmt

# Check for common issues
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated
```

### Test-Driven Development

```bash
# 1. Write failing test
echo "Write test in appropriate test file"

# 2. Run test to see it fail
cargo test test_new_feature

# 3. Implement feature
echo "Implement in src/"

# 4. Run test to see it pass
cargo test test_new_feature

# 5. Refactor if needed
cargo fmt && cargo clippy
```

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run -- start

# Run with specific module logging
RUST_LOG=qudag_network=debug cargo run -- start

# Run with backtrace
RUST_BACKTRACE=1 cargo run -- start

# Run with full backtrace
RUST_BACKTRACE=full cargo run -- start
```

### Performance Optimization

```bash
# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bin qudag -- start

# Profile with cargo-profiling
cargo install cargo-profiling
cargo profiling callgrind --bin qudag

# Check binary size
cargo bloat --release
```

## Examples and Common Scenarios

### Scenario 1: Setting Up a Test Network

```bash
# Terminal 1: Start first node
./target/release/qudag start --port 8000 --data-dir ./node1

# Terminal 2: Start second node
./target/release/qudag start --port 8001 --data-dir ./node2

# Terminal 3: Connect nodes
./target/release/qudag peer add /ip4/127.0.0.1/tcp/8000 --port 8001

# Check network status
./target/release/qudag network stats --port 8000
./target/release/qudag network stats --port 8001
```

### Scenario 2: Dark Address Communication

```bash
# Node A: Register dark address
./target/release/qudag address register alice.dark

# Node B: Register dark address
./target/release/qudag address register bob.dark

# Resolve addresses
./target/release/qudag address resolve alice.dark
./target/release/qudag address resolve bob.dark

# Create secure fingerprints
./target/release/qudag address fingerprint --data "Secure message from Alice"
```

### Scenario 3: Performance Testing

```bash
# Start node with performance monitoring
./target/release/qudag start --log-level debug

# In another terminal, run benchmarks
cargo bench --bench system_benchmarks

# Monitor system resources
htop  # In another terminal

# Generate load
./scripts/generate_load.sh --messages 10000 --rate 100
```

### Scenario 4: Security Testing

```bash
# Run security audit
cargo audit

# Run timing attack tests
cargo test --features security-tests test_timing_attacks

# Fuzz cryptographic operations
cd fuzz && cargo fuzz run crypto_fuzzer -- -max_total_time=3600

# Check for memory leaks
valgrind --leak-check=full ./target/release/qudag start
```

### Scenario 5: Development and Debugging

```bash
# Start development cycle
cargo watch -x "test -p qudag-network"

# Debug specific module
RUST_LOG=qudag_network=trace cargo run -- start

# Profile hot paths
cargo flamegraph --bin qudag -- start

# Check test coverage
cargo tarpaulin --out Html --output-dir coverage
```

### Scenario 6: Production Deployment Preparation

```bash
# Build optimized binary
cargo build --release --target x86_64-unknown-linux-gnu

# Strip debug symbols
strip target/release/qudag

# Run comprehensive tests
./scripts/run_all_tests.sh

# Generate documentation
cargo doc --no-deps --open

# Package for distribution
./scripts/package_release.sh
```

## Troubleshooting

### Common Issues

#### Port Already in Use

```bash
# Check what's using the port
lsof -i :8000

# Use a different port
./target/release/qudag start --port 8001
```

#### Connection Refused

```bash
# Ensure node is running
./target/release/qudag status

# Check firewall settings
sudo iptables -L | grep 8000
```

#### High Memory Usage

```bash
# Monitor memory usage
./target/release/qudag status --verbose

# Limit peer connections
./target/release/qudag start --max-peers 20
```

#### Performance Issues

```bash
# Enable performance monitoring
RUST_LOG=qudag_performance=debug ./target/release/qudag start

# Run profiler
cargo flamegraph --bin qudag
```

### Getting Help

- Check the logs: `tail -f ./data/logs/qudag.log`
- Run with debug logging: `RUST_LOG=debug ./target/release/qudag start`
- Check documentation: `cargo doc --open`
- File an issue: https://github.com/ruvnet/QuDAG/issues

## Best Practices

1. **Always run tests before committing**: `cargo test`
2. **Format code before committing**: `cargo fmt`
3. **Check for issues**: `cargo clippy`
4. **Keep dependencies updated**: `cargo update`
5. **Monitor performance**: Regular benchmarking
6. **Security first**: Run security tests frequently
7. **Document changes**: Update relevant documentation
8. **Use proper error handling**: No unwraps in production code

## Additional Resources

- [Architecture Documentation](docs/architecture/README.md)
- [Security Guide](docs/security/README.md)
- [API Documentation](https://docs.rs/qudag)
- [Contributing Guide](CONTRIBUTING.md)
- [Performance Report](performance_report.md)

---

For more information, visit the [QuDAG GitHub repository](https://github.com/ruvnet/QuDAG).