# Advanced Usage Guide

## Custom Node Deployment

### Multiple Node Setup
```bash
# Create data directories
mkdir -p /data/node1 /data/node2

# Initialize nodes with custom configs
qudag node init --data-dir /data/node1 --config node1.toml
qudag node init --data-dir /data/node2 --config node2.toml

# Start nodes
qudag node start --data-dir /data/node1 --daemon
qudag node start --data-dir /data/node2 --daemon
```

### Network Customization

Create private networks:
```bash
# Generate network key
qudag network keygen --output network.key

# Start node with custom network
qudag node start --network-key network.key --peers-file peers.txt
```

## Performance Optimization

### Resource Tuning
```bash
# Set resource limits
qudag config set performance.worker_threads 4
qudag config set performance.max_memory "4GB"

# Enable performance mode
qudag node start --performance-mode
```

### Monitoring and Profiling
```bash
# Start with profiling
qudag node start --profile

# Export metrics
qudag metrics export --format prometheus
```

## Security Hardening

### Cryptographic Operations
```bash
# Configure quantum-resistant algorithms
qudag config set crypto.kem_algorithm ML-KEM-768
qudag config set crypto.signature_algorithm ML-DSA-65

# Verify crypto implementation
qudag crypto verify --comprehensive
```

### Network Security
```bash
# Configure secure networking
qudag config set network.ssl_enabled true
qudag config set network.peer_verification strict

# Update trusted peers
qudag network peers trust <peer-id>
```

## DAG Management

### Consensus Control
```bash
# Adjust consensus parameters
qudag config set dag.consensus_threshold 0.75
qudag config set dag.finality_delay 15

# Monitor consensus
qudag dag consensus --watch
```

### Data Management
```bash
# Export DAG data
qudag dag export --format json --output dag-backup.json

# Import DAG data
qudag dag import dag-backup.json --verify
```

## Maintenance Operations

### Database Management
```bash
# Compact database
qudag maintenance compact-db

# Verify database integrity
qudag maintenance verify-db
```

### Network Maintenance
```bash
# Update peer lists
qudag network peers update

# Clear peer cache
qudag network peers clear-cache
```

## Development and Testing

### Local Testing
```bash
# Start test network
qudag test network --nodes 3

# Simulate network conditions
qudag test network --latency 100 --packet-loss 0.1
```

### Debugging
```bash
# Enable debug logging
qudag node start --log-level debug --trace

# Debug specific components
qudag debug network --verbose
qudag debug consensus --trace
```

## Integration

### API Access
```bash
# Enable API
qudag config set api.enabled true
qudag config set api.port 9000

# Generate API key
qudag api create-key --name "service1"
```

### Metrics Integration
```bash
# Configure Prometheus metrics
qudag config set metrics.prometheus_enabled true
qudag config set metrics.prometheus_port 9090

# Enable detailed metrics
qudag metrics enable --detailed
```

## Recovery Procedures

### Node Recovery
```bash
# Backup node data
qudag backup create --output backup.tar.gz

# Restore from backup
qudag backup restore backup.tar.gz --verify
```

### Network Recovery
```bash
# Force resync
qudag network resync --force

# Rebuild peer connections
qudag network rebuild
```

## Advanced Configuration

### Custom Plugins
```bash
# Load custom plugin
qudag plugin load ./my-plugin.so

# Configure plugin
qudag plugin config my-plugin --option value
```

### Advanced Routing
```bash
# Configure routing policies
qudag network route add --policy custom
qudag network route optimize
```

## Performance Testing

### Benchmarking
```bash
# Run comprehensive benchmarks
qudag benchmark full --duration 1h

# Test specific components
qudag benchmark crypto --algorithm ML-KEM-768
qudag benchmark network --peers 100
```

### Load Testing
```bash
# Generate test load
qudag test load --tps 1000 --duration 10m

# Monitor during load test
qudag monitor --metrics --interval 1
```