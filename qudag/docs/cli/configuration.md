# Configuration Guide

## Configuration File

The QuDAG node configuration can be specified via a configuration file in TOML format or through command line arguments.

## Configuration Sections

### Node Configuration
```toml
[node]
# Data directory path
data_dir = "./data"

# Network port for P2P connections
port = 8000

# Initial peer addresses
peers = [
    "127.0.0.1:8001",
    "127.0.0.1:8002"
]
```

### Network Configuration
```toml
[network]
# Listen address for P2P connections
listen_addr = "0.0.0.0"
port = 8000

# Maximum number of peers
max_peers = 50

# Bootstrap nodes
bootstrap_nodes = [
    "node1.qudag.network:8000",
    "node2.qudag.network:8000"
]

# Peer discovery configuration
[network.discovery]
enabled = true
interval = 300  # seconds
```

### DAG Configuration
```toml
[dag]
# Consensus parameters
consensus_threshold = 0.67
finality_delay = 10

# Transaction validation
max_tx_size = 1048576  # 1MB
max_block_size = 5242880  # 5MB
```

### Crypto Configuration
```toml
[crypto]
# Quantum-resistant algorithm selection
kem_algorithm = "ML-KEM-768"
signature_algorithm = "ML-DSA-65"

# Security parameters
memory_protection = true
constant_time = true
```

## Command Line Arguments

All configuration options can be specified via command line arguments:

```bash
# Start node with custom config
qudag start --config config.toml

# Start node with direct options
qudag start --data-dir /var/qudag/data --port 8001 --peers "127.0.0.1:8000"
```

## Priority Order

Configuration values are resolved in the following order:

1. Command line arguments (highest priority)
2. Configuration file specified with --config
3. Default values (lowest priority)

## Security Considerations

1. Protect configuration file permissions:
```bash
chmod 600 ~/.qudag/config.toml
```

2. Use secure network settings:
```toml
[network]
listen_addr = "localhost"  # For private nodes
ssl_enabled = true
```

3. Enable security features:
```toml
[security]
memory_protection = true
constant_time_crypto = true
secure_rng = true
```

## Performance Tuning

Optimize performance with these settings:

```toml
[performance]
# Thread pool configuration
worker_threads = 4
crypto_threads = 2

# Memory limits
max_memory = "4GB"
cache_size = "1GB"

# Network optimizations
tcp_keepalive = 60
max_concurrent_requests = 1000
```

## Monitoring Configuration

Configure monitoring and metrics:

```toml
[metrics]
enabled = true
interval = 60  # seconds
prometheus_port = 9090

[logging]
file = "/var/log/qudag.log"
rotation = "daily"
retention = 7  # days
```

## Example Configurations

### Minimal Node
```toml
[node]
node_id = "minimal-node"
log_level = "info"

[network]
port = 8000
max_peers = 10
```

### Full Node
```toml
[node]
node_id = "full-node"
log_level = "debug"
metrics_enabled = true

[network]
port = 8000
max_peers = 50
bootstrap_nodes = ["node1.qudag.network:8000"]

[dag]
consensus_threshold = 0.67
finality_delay = 10

[crypto]
kem_algorithm = "ML-KEM-768"
signature_algorithm = "ML-DSA-65"
```

### Development Node
```toml
[node]
node_id = "dev-node"
log_level = "debug"

[network]
port = 9000
max_peers = 5

[development]
test_mode = true
fast_sync = true
```