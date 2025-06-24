# Network Setup Tutorial

This tutorial explains how to set up and manage QuDAG networks using the CLI.

## Network Configuration

### Basic Network Setup

```bash
# Initialize a new network
qudag network init

# Generate network configuration
qudag network generate-config --nodes 5

# Start network with generated config
qudag network start --config network.toml
```

### Bootstrap Node Setup

```bash
# Start a bootstrap node
qudag network bootstrap --port 9000

# Get bootstrap node connection info
qudag network bootstrap-info

# Add bootstrap node to config
qudag network add-bootstrap /ip4/127.0.0.1/tcp/9000/p2p/QmBootstrapId
```

## Network Management

### Peer Management

```bash
# List connected peers
qudag network peers list

# Add peer manually
qudag network peers add /ip4/1.2.3.4/tcp/9000/p2p/QmPeerId

# Remove peer
qudag network peers remove QmPeerId

# Set peer connection limits
qudag network peers set-limit 100
```

### Network Health

```bash
# Check network health
qudag network health

# View network metrics
qudag network metrics

# Get network topology
qudag network topology
```

## Routing Configuration

### Anonymous Routing

```bash
# Enable anonymous routing
qudag network routing anonymous enable

# Configure routing parameters
qudag network routing set-params --ttl 10 --redundancy 3

# View routing status
qudag network routing status
```

### Connection Management

```bash
# List active connections
qudag network connections list

# Close specific connection
qudag network connections close QmPeerId

# Set connection timeouts
qudag network connections set-timeout 30s
```

## Security Configuration

### Network Access Control

```bash
# Enable network ACLs
qudag network acl enable

# Add allowed peers
qudag network acl allow QmPeerId

# Block malicious peers
qudag network acl block QmPeerId
```

### Encryption Settings

```bash
# Configure network encryption
qudag network security set-encryption

# Rotate network keys
qudag network security rotate-keys

# View security status
qudag network security status
```

## Performance Optimization

### Network Tuning

```bash
# Optimize network for throughput
qudag network tune throughput

# Configure bandwidth limits
qudag network tune bandwidth --up 10M --down 10M

# Reset network tuning
qudag network tune reset
```

### Load Balancing

```bash
# Enable load balancing
qudag network lb enable

# Configure load balancer
qudag network lb configure --method round-robin

# View load balancer status
qudag network lb status
```

## Testing and Diagnostics

### Network Testing

```bash
# Run network connectivity test
qudag network test connectivity

# Measure network latency
qudag network test latency

# Check routing performance
qudag network test routing
```

### Network Diagnostics

```bash
# Run full network diagnostic
qudag network diagnose

# Check peer reachability
qudag network ping QmPeerId

# View network logs
qudag network logs --level debug
```

## Tips and Best Practices

1. Always start with bootstrap nodes when setting up a new network
2. Monitor network health and performance metrics regularly
3. Use appropriate security settings for your network type
4. Keep routing configuration optimized for your use case
5. Regularly test network connectivity and performance