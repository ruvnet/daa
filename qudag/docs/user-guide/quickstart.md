# QuDAG Quick Start Guide

This guide will help you get up and running with QuDAG quickly.

## Prerequisites

- Rust toolchain (1.75.0 or later)
- Git
- CMake (3.12 or later)
- C++ compiler supporting C++17
- OpenSSL development libraries

## Quick Installation

```bash
# Clone the repository
git clone https://github.com/qudag/qudag
cd qudag

# Build QuDAG
cargo build --release

# Run tests to verify installation
cargo test --all-features --workspace
```

## First Node Setup

1. Initialize a node:
   ```bash
   qudag init --node-id node1
   ```

2. Configure the node:
   ```bash
   qudag config set --network testnet --port 8000
   ```

3. Start the node:
   ```bash
   qudag start
   ```

## Create a Simple Network

1. Initialize additional nodes:
   ```bash
   qudag init --node-id node2
   qudag init --node-id node3
   ```

2. Configure peer connections:
   ```bash
   qudag peer add /ip4/127.0.0.1/tcp/8000 --to node2
   qudag peer add /ip4/127.0.0.1/tcp/8000 --to node3
   ```

3. Start all nodes:
   ```bash
   qudag start --node-id node2 --port 8001
   qudag start --node-id node3 --port 8002
   ```

## Basic Operations

### Send Messages
```bash
qudag message send --to QD... --content "Hello QuDAG!"
```

### Monitor Network
```bash
qudag network status
```

### View Node Status
```bash
qudag node info
```

## Next Steps

- Read the [Full Installation Guide](installation.md) for detailed setup
- Explore [CLI Usage](cli-usage.md) for more commands
- Review [Security Best Practices](security.md)
- Check [Troubleshooting](troubleshooting.md) if you encounter issues