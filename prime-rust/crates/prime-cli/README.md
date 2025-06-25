# DAA Prime CLI

[![Crates.io](https://img.shields.io/crates/v/daa-prime-cli.svg)](https://crates.io/crates/daa-prime-cli)
[![Documentation](https://docs.rs/daa-prime-cli/badge.svg)](https://docs.rs/daa-prime-cli)
[![License](https://img.shields.io/crates/l/daa-prime-cli.svg)](https://github.com/yourusername/daa/blob/main/LICENSE)

Command-line interface for Prime distributed machine learning operations. Provides easy access to training, coordination, and management functions for the Prime distributed ML framework.

## Overview

DAA Prime CLI is the primary interface for interacting with the Prime distributed ML ecosystem. It provides:

- **Node Management**: Start and manage trainer and coordinator nodes
- **Training Operations**: Launch training jobs and monitor progress  
- **System Status**: Check system health and performance metrics
- **Configuration**: Manage node configurations and network settings
- **Integration**: Seamless integration with all Prime components

## Features

- 🖥️ **User-Friendly Interface**: Intuitive commands for all operations
- 🔧 **Comprehensive Management**: Full control over distributed ML infrastructure  
- 📊 **Real-Time Monitoring**: Live status updates and progress tracking
- ⚙️ **Flexible Configuration**: Support for various deployment scenarios
- 🔗 **Full Integration**: Access to all Prime ecosystem capabilities

## Installation

### From Crates.io

```bash
cargo install daa-prime-cli
```

### From Source

```bash
git clone https://github.com/yourusername/daa.git
cd daa/prime-rust/crates/prime-cli
cargo install --path .
```

### Binary Releases

Download pre-built binaries from the [releases page](https://github.com/yourusername/daa/releases).

## Quick Start

### Check System Status

```bash
# View Prime system status and available crates
prime status
```

Output:
```
Prime system status:
  - System: Ready
  - Version: 0.2.1
  - Framework: Distributed ML with DAA
  - Available crates:
    * daa-prime-core v0.2.1
    * daa-prime-dht v0.2.1
    * daa-prime-trainer v0.2.1
    * daa-prime-coordinator v0.2.1
```

### Start a Trainer Node

```bash
# Start trainer with random ID
prime trainer

# Start trainer with specific ID
prime trainer --id trainer-gpu-001
```

### Start a Coordinator Node

```bash
# Start coordinator with random ID
prime coordinator

# Start coordinator with specific ID  
prime coordinator --id coordinator-main
```

## Command Reference

### Global Options

```bash
prime [OPTIONS] <COMMAND>

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

### Commands

#### `prime status`

Display system status and component information.

```bash
prime status
```

Shows:
- System readiness
- Version information
- Available crate versions
- Framework details

#### `prime trainer`

Start a distributed training node.

```bash
prime trainer [OPTIONS]

Options:
  -i, --id <ID>    Node ID (defaults to random UUID)
  -h, --help       Print help information
```

**Examples:**

```bash
# Start with random ID
prime trainer

# Start with specific ID
prime trainer --id gpu-worker-001

# Start with custom configuration
prime trainer --id trainer-001 --config trainer.json
```

**Note:** The current CLI provides a simplified interface. For full training functionality, use the `daa-prime-trainer` crate directly in your Rust applications.

#### `prime coordinator`

Start a coordination node for distributed training.

```bash
prime coordinator [OPTIONS]

Options:
  -i, --id <ID>    Node ID (defaults to random UUID)
  -h, --help       Print help information
```

**Examples:**

```bash
# Start coordinator with random ID
prime coordinator

# Start coordinator with specific ID
prime coordinator --id coord-main

# Start coordinator cluster node
prime coordinator --id coord-002 --cluster-config cluster.json
```

**Note:** The current CLI provides a simplified interface. For full coordination functionality, use the `daa-prime-coordinator` crate directly in your Rust applications.

## Configuration

### Environment Variables

```bash
# Set logging level
export RUST_LOG=info

# Set custom data directory
export PRIME_DATA_DIR=/path/to/data

# Set network configuration
export PRIME_NETWORK_PORT=8080
export PRIME_NETWORK_HOST=0.0.0.0
```

### Configuration Files

#### Trainer Configuration (`trainer.json`)

```json
{
  "training": {
    "batch_size": 32,
    "learning_rate": 0.001,
    "max_epochs": 100,
    "optimizer": {
      "type": "AdamW",
      "beta1": 0.9,
      "beta2": 0.999,
      "weight_decay": 0.01
    }
  },
  "network": {
    "timeout": 30,
    "max_message_size": 1048576,
    "enable_compression": true
  },
  "data": {
    "data_path": "/path/to/training/data",
    "validation_split": 0.2,
    "batch_size": 32
  }
}
```

#### Coordinator Configuration (`coordinator.json`)

```json
{
  "coordination": {
    "min_nodes_for_round": 3,
    "heartbeat_timeout_ms": 5000,
    "task_timeout_ms": 60000,
    "consensus_threshold": 0.66
  },
  "economic": {
    "enable_rewards": true,
    "base_reward_amount": 100,
    "quality_threshold": 0.8
  },
  "network": {
    "port": 8080,
    "max_connections": 1000,
    "enable_tls": true
  }
}
```

#### Cluster Configuration (`cluster.json`)

```json
{
  "cluster_id": "main_cluster",
  "coordinators": [
    {
      "id": "coordinator-001",
      "address": "10.0.1.10:8080",
      "is_bootstrap": true
    },
    {
      "id": "coordinator-002", 
      "address": "10.0.1.11:8080"
    },
    {
      "id": "coordinator-003",
      "address": "10.0.1.12:8080"
    }
  ],
  "leader_election": "raft",
  "failover_timeout": 30
}
```

## Usage Examples

### Development Workflow

```bash
# 1. Check system status
prime status

# 2. Start a coordinator
prime coordinator --id dev-coordinator &

# 3. Start multiple trainers
prime trainer --id trainer-1 &
prime trainer --id trainer-2 &
prime trainer --id trainer-3 &

# 4. Monitor progress (in separate terminal)
watch -n 5 prime status
```

### Production Deployment

```bash
# Start coordinator cluster
prime coordinator --id coord-1 --config prod-coordinator.json --cluster-config cluster.json &
prime coordinator --id coord-2 --config prod-coordinator.json --cluster-config cluster.json &
prime coordinator --id coord-3 --config prod-coordinator.json --cluster-config cluster.json &

# Start trainer nodes on different machines
prime trainer --id gpu-node-001 --config gpu-trainer.json &
prime trainer --id gpu-node-002 --config gpu-trainer.json &
prime trainer --id cpu-node-001 --config cpu-trainer.json &
```

### Testing Setup

```bash
# Quick test with minimal nodes
prime coordinator --id test-coord &
sleep 2
prime trainer --id test-trainer-1 &
prime trainer --id test-trainer-2 &

# Check everything is running
prime status
```

## Integration with DAA Ecosystem

### DAA AI Integration

```bash
# Trainers automatically integrate with DAA AI for:
# - Model management and versioning
# - Inference capabilities  
# - ML pipeline orchestration

# View AI integration status
prime status | grep -A 5 "AI Integration"
```

### DAA Rules Integration

```bash
# Coordinators use DAA Rules for:
# - Governance policy enforcement
# - Training parameter validation
# - Node behavior compliance

# Check rules integration
prime status | grep -A 5 "Rules Integration"
```

### DAA Economy Integration

```bash
# Automatic token rewards for:
# - Quality training contributions
# - Reliable node operation
# - Successful task completion

# View economic integration
prime status | grep -A 5 "Economy Integration"
```

## Advanced Usage

### Docker Integration

```dockerfile
# Dockerfile for Prime CLI
FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .
RUN cargo install --path crates/prime-cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/prime /usr/local/bin/

ENTRYPOINT ["prime"]
CMD ["status"]
```

```bash
# Build and run
docker build -t prime-cli .

# Run trainer
docker run --rm -d --name trainer-1 prime-cli trainer --id docker-trainer-1

# Run coordinator  
docker run --rm -d --name coordinator prime-cli coordinator --id docker-coord

# Check status
docker run --rm prime-cli status
```

### Kubernetes Deployment

```yaml
# prime-trainer.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prime-trainer
spec:
  replicas: 3
  selector:
    matchLabels:
      app: prime-trainer
  template:
    metadata:
      labels:
        app: prime-trainer
    spec:
      containers:
      - name: trainer
        image: prime-cli:latest
        command: ["prime", "trainer"]
        args: ["--id", "k8s-trainer-$(POD_NAME)"]
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: RUST_LOG
          value: "info"
```

```yaml
# prime-coordinator.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prime-coordinator
spec:
  replicas: 1
  selector:
    matchLabels:
      app: prime-coordinator
  template:
    metadata:
      labels:
        app: prime-coordinator
    spec:
      containers:
      - name: coordinator
        image: prime-cli:latest
        command: ["prime", "coordinator"]
        args: ["--id", "k8s-coordinator"]
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
```

### Scripting and Automation

```bash
#!/bin/bash
# deploy-prime-cluster.sh

set -e

COORDINATOR_ID="auto-coord-$(date +%s)"
TRAINER_COUNT=${1:-5}

echo "Deploying Prime cluster with $TRAINER_COUNT trainers..."

# Start coordinator
echo "Starting coordinator: $COORDINATOR_ID"
prime coordinator --id "$COORDINATOR_ID" &
COORD_PID=$!

# Wait for coordinator to be ready
sleep 5

# Start trainers
for i in $(seq 1 $TRAINER_COUNT); do
    TRAINER_ID="auto-trainer-$i-$(date +%s)"
    echo "Starting trainer: $TRAINER_ID"
    prime trainer --id "$TRAINER_ID" &
done

echo "Cluster deployed successfully!"
echo "Coordinator PID: $COORD_PID"
echo "Use 'prime status' to check cluster status"

# Wait for interrupt
trap "kill $COORD_PID; exit" INT
wait $COORD_PID
```

## Monitoring and Logging

### Logging Configuration

```bash
# Set log levels
export RUST_LOG=debug           # Full debug logging
export RUST_LOG=info            # Default info logging  
export RUST_LOG=warn            # Warnings and errors only
export RUST_LOG=error           # Errors only

# Component-specific logging
export RUST_LOG=daa_prime_trainer=debug,daa_prime_coordinator=info

# Log to file
prime trainer --id trainer-001 2>&1 | tee trainer.log
```

### Status Monitoring

```bash
# Continuous status monitoring
watch -n 10 prime status

# Status with timestamp
while true; do
    echo "$(date): $(prime status | head -1)"
    sleep 30
done

# Check if nodes are responsive
timeout 30 prime status || echo "Prime system not responding"
```

### Process Management

```bash
# Start with process management
prime coordinator --id coord-main &
echo $! > coordinator.pid

prime trainer --id trainer-main & 
echo $! > trainer.pid

# Stop processes
kill $(cat coordinator.pid)
kill $(cat trainer.pid)

# Check if processes are running
kill -0 $(cat coordinator.pid) 2>/dev/null && echo "Coordinator running" || echo "Coordinator stopped"
```

## Troubleshooting

### Common Issues

1. **Command Not Found**
   ```bash
   # Ensure cargo bin directory is in PATH
   export PATH="$HOME/.cargo/bin:$PATH"
   
   # Or use full path
   ~/.cargo/bin/prime status
   ```

2. **Connection Issues**
   ```bash
   # Check network connectivity
   prime status
   
   # If coordinator unreachable, verify it's running
   ps aux | grep prime
   ```

3. **Permission Issues**
   ```bash
   # Ensure proper permissions for data directory
   sudo chown -R $USER:$USER ~/.prime/
   chmod 755 ~/.prime/
   ```

4. **Port Conflicts**
   ```bash
   # Check if ports are in use
   netstat -tuln | grep 8080
   
   # Use different port
   export PRIME_NETWORK_PORT=8081
   prime coordinator
   ```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug prime trainer --id debug-trainer

# Trace all network activity
RUST_LOG=trace prime coordinator --id debug-coordinator

# Check configuration parsing
prime status --verbose
```

### Getting Help

```bash
# General help
prime --help

# Command-specific help
prime trainer --help
prime coordinator --help

# Version information
prime --version
```

## Limitations and Future Development

### Current Limitations

- **Simplified Interface**: Current CLI provides basic node management; full functionality requires using crates directly
- **Configuration**: Limited configuration options through CLI flags
- **Monitoring**: Basic status reporting; advanced metrics require integration with crate APIs
- **Management**: No built-in cluster management commands

### Planned Enhancements

- [ ] **Enhanced Training Commands**: Full training job management and monitoring
- [ ] **Advanced Configuration**: Comprehensive CLI-based configuration management
- [ ] **Cluster Management**: Built-in cluster deployment and management commands
- [ ] **Interactive Mode**: TUI-based interactive interface for complex operations
- [ ] **Plugin System**: Extensible plugin architecture for custom commands
- [ ] **Integration Commands**: Direct integration with DAA AI, Rules, and Economy

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/yourusername/daa.git
cd daa/prime-rust/crates/prime-cli

# Build
cargo build --release

# Run
./target/release/prime status
```

### Contributing

Contributions are welcome! Areas for improvement:

- **Command Extensions**: Additional commands for cluster management
- **Configuration Management**: Enhanced configuration handling
- **Interactive Features**: TUI components for better user experience
- **Documentation**: More examples and use cases
- **Testing**: Integration tests for CLI workflows

See our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Related Crates

- [`daa-prime-core`](../prime-core): Core types and protocol definitions
- [`daa-prime-trainer`](../prime-trainer): Full training node functionality
- [`daa-prime-coordinator`](../prime-coordinator): Full coordination capabilities
- [`daa-prime-dht`](../prime-dht): Distributed hash table operations

## Support

- 📖 [Documentation](https://docs.rs/daa-prime-cli)
- 🐛 [Issue Tracker](https://github.com/yourusername/daa/issues)
- 💬 [Discussions](https://github.com/yourusername/daa/discussions)
- 🚀 [Feature Requests](https://github.com/yourusername/daa/issues/new?template=feature_request.md)