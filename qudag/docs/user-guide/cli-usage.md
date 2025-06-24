# QuDAG CLI Usage Guide

## Command Overview

QuDAG provides a comprehensive command-line interface (CLI) for managing nodes, networks, and protocol operations.

### Global Options
```bash
--config PATH    Path to config file
--verbose        Enable verbose output
--json          Output in JSON format
--network NAME  Specify network (mainnet/testnet)
```

## Node Management

### Node Initialization
```bash
# Create a new node
qudag init --node-id <name>

# Start a node
qudag start [--port PORT]

# Stop a node
qudag stop [--force]

# View node status
qudag status
```

### Configuration
```bash
# View current config
qudag config show

# Set configuration values
qudag config set <key> <value>

# Reset configuration
qudag config reset
```

## Network Operations

### Peer Management
```bash
# Add a peer
qudag peer add <multiaddr>

# List peers
qudag peer list

# Remove a peer
qudag peer remove <peer-id>

# Get peer info
qudag peer info <peer-id>
```

### Message Operations
```bash
# Send a message
qudag message send --to <address> --content <message>

# List messages
qudag message list

# Get message info
qudag message info <message-id>
```

## Monitoring and Diagnostics

### Network Monitoring
```bash
# View network status
qudag network status

# Show network metrics
qudag network metrics

# Check peer connectivity
qudag network ping <peer-id>
```

### Logging
```bash
# View logs
qudag logs [--tail N]

# Set log level
qudag logs set-level <level>

# Export logs
qudag logs export <file>
```

## Advanced Operations

### Identity Management
```bash
# Create new identity
qudag identity create

# List identities
qudag identity list

# Export identity
qudag identity export <id>

# Import identity
qudag identity import <file>
```

### Security Operations
```bash
# Generate new keys
qudag security keygen

# Rotate keys
qudag security rotate-keys

# Security audit
qudag security audit
```

### Consensus Management
```bash
# View consensus status
qudag consensus status

# List validators
qudag consensus validators

# Show round info
qudag consensus round <round-id>
```

## Development Tools

### Testing
```bash
# Run node tests
qudag test node

# Network simulation
qudag test network

# Performance tests
qudag test performance
```

### Debugging
```bash
# Enable debug mode
qudag debug enable

# Profile performance
qudag debug profile

# Memory analysis
qudag debug memory
```

## Best Practices

1. **Security**
   - Always verify peer identities
   - Use secure key storage
   - Keep software updated

2. **Performance**
   - Monitor resource usage
   - Optimize network settings
   - Regular maintenance

3. **Maintenance**
   - Regular backups
   - Log rotation
   - Health checks