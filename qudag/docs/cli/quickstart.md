# Quick Start Guide

Get started with QuDAG CLI in minutes.

## First Steps

1. After [installation](installation.md), verify the CLI:
```bash
qudag --version
```

2. Initialize a new node:
```bash
qudag node init
```

3. Start the node:
```bash
qudag node start
```

## Basic Commands

### Node Management
```bash
# Check node status
qudag node status

# View node metrics
qudag node metrics

# Stop node
qudag node stop
```

### Network Operations
```bash
# List connected peers
qudag network peers

# View network stats
qudag network stats

# Test network connectivity
qudag network test
```

### DAG Operations
```bash
# View DAG status
qudag dag status

# List recent vertices
qudag dag vertices

# Check consensus status
qudag dag consensus
```

## Configuration

1. Basic node configuration:
```bash
qudag config set network.port 8000
qudag config set network.max_peers 50
```

2. View current configuration:
```bash
qudag config show
```

## Monitoring

Monitor node performance:
```bash
qudag monitor --metrics
```

## Next Steps

- Read the [Command Reference](commands.md) for detailed command information
- Configure your node using the [Configuration Guide](configuration.md)
- Explore [Advanced Usage](advanced-usage.md) for more features

## Common Issues

If you encounter problems:
1. Check node logs:
```bash
qudag logs show
```

2. Verify network connectivity:
```bash
qudag network test
```

3. See [Troubleshooting](troubleshooting.md) for more help