# QuDAG Testnet - Docker Containerization

This directory contains the optimized Docker containerization for the QuDAG testnet deployment. It provides a complete 4-node testnet environment with monitoring, metrics collection, and management tools.

## 🚀 Quick Start

```bash
# 1. Setup the testnet (generate keys, build images)
./scripts/setup.sh

# 2. Start the testnet
docker-compose up -d

# 3. Monitor the testnet
./scripts/monitor.sh -c

# 4. Access services
# - Grafana: http://localhost:3000 (admin/admin123)
# - Prometheus: http://localhost:9094
# - Node APIs: http://localhost:8080-8083
```

## 📁 Directory Structure

```
qudag-testnet/
├── Dockerfile                 # Multi-stage optimized Dockerfile
├── docker-compose.yml         # 4-node + monitoring stack
├── README.md                  # This file
├── configs/                   # Node configuration files
│   ├── node1.toml            # Bootstrap node config
│   ├── node2.toml            # Validator node config
│   ├── node3.toml            # Validator node config
│   └── node4.toml            # Validator node config
├── monitoring/               # Monitoring configuration
│   ├── prometheus.yml        # Prometheus configuration
│   ├── alerts.yml           # Alert rules
│   └── grafana/             # Grafana configuration
│       ├── provisioning/    # Auto-provisioning
│       └── dashboards/      # Dashboard definitions
├── keys/                    # Generated cryptographic keys
│   ├── node1/              # Node 1 keys
│   ├── node2/              # Node 2 keys
│   ├── node3/              # Node 3 keys
│   └── node4/              # Node 4 keys
└── scripts/                # Management scripts
    ├── setup.sh           # Setup and initialization
    └── monitor.sh         # Health monitoring
```

## 🔧 Features

### Docker Optimization
- **Multi-stage build** with Cargo Chef for efficient dependency caching
- **Minimal runtime image** based on Debian Bookworm Slim
- **Multi-architecture support** (AMD64/ARM64)
- **Optimized Rust compilation** with target-cpu=native
- **Security hardening** with non-root user execution
- **Health checks** for all services

### Networking
- **Isolated Docker network** (172.28.0.0/16)
- **P2P connectivity** between all nodes
- **Port mapping** for external access
- **Service discovery** using hostnames
- **Bootstrap node** configuration

### Monitoring Stack
- **Prometheus** metrics collection
- **Grafana** visualization dashboards
- **Node Exporter** for host metrics
- **cAdvisor** for container metrics
- **Alert rules** for proactive monitoring

### Management Tools
- **Setup script** for automated initialization
- **Monitoring script** for health checks
- **Key generation** for node authentication
- **Configuration management**

## 🛠️ Setup Instructions

### Prerequisites

```bash
# Required tools
sudo apt-get update && sudo apt-get install -y \
    docker.io \
    docker-compose \
    openssl \
    jq \
    curl
```

### Step 1: Initialize Testnet

Run the setup script to generate keys and build the Docker image:

```bash
./scripts/setup.sh
```

This will:
- Generate Ed25519 keypairs for all nodes
- Create peer IDs and bootstrap configuration
- Build the optimized Docker image
- Create environment configuration
- Validate all configurations

### Step 2: Start Services

```bash
# Start all services
docker-compose up -d

# Start only nodes (no monitoring)
docker-compose up -d node1 node2 node3 node4

# Start with logs
docker-compose up
```

### Step 3: Verify Deployment

```bash
# Check status
./scripts/monitor.sh

# Continuous monitoring
./scripts/monitor.sh -c

# JSON output for automation
./scripts/monitor.sh -j
```

## 📊 Monitoring & Observability

### Grafana Dashboard

Access Grafana at http://localhost:3000:
- **Username**: admin
- **Password**: admin123
- **Dashboard**: QuDAG Testnet Overview

Key metrics monitored:
- Node health and uptime
- CPU and memory usage
- Peer connection counts
- DAG vertex creation rates
- Network latency
- Storage usage

### Prometheus Metrics

Access Prometheus at http://localhost:9094:
- Raw metrics collection
- Alert rule management
- Query interface
- Target health monitoring

### Node APIs

Each node exposes an API endpoint:
- **Node 1**: http://localhost:8080
- **Node 2**: http://localhost:8081
- **Node 3**: http://localhost:8082
- **Node 4**: http://localhost:8083

API endpoints:
- `/api/v1/health` - Health check
- `/api/v1/peers` - Peer information
- `/api/v1/dag/info` - DAG statistics
- `/metrics` - Prometheus metrics

## 🔐 Security Features

### Cryptographic Security
- Ed25519 keypairs for node authentication
- Quantum-resistant cryptographic algorithms
- Secure key storage and management
- TLS encryption for RPC endpoints

### Container Security
- Non-root user execution
- Minimal attack surface
- Resource limits and isolation
- Security-hardened base image

### Network Security
- Isolated Docker network
- Firewall-friendly port mapping
- Traffic obfuscation support
- Dark domain addressing

## ⚡ Performance Optimization

### Build Optimization
- Cargo Chef for dependency caching
- Multi-stage builds reduce image size
- Rust compilation with target-cpu=native
- Efficient layer caching

### Runtime Optimization
- Resource limits prevent resource exhaustion
- Optimized logging configuration
- Efficient storage management
- Connection pooling and reuse

### Monitoring Optimization
- Efficient metrics collection
- Configurable retention policies
- Optimized dashboard queries
- Alert rule optimization

## 🔧 Configuration

### Node Configuration

Each node has a TOML configuration file in `configs/`:

```toml
[node]
name = "testnet-validator-2"
role = "validator"
data_dir = "/data/qudag"

[network]
network_id = "qudag-testnet-local"
max_peers = 50
min_peers = 3

[p2p]
enable_nat = true
enable_relay = true
bootstrap_peers = ["..."]

[dark_domain]
enabled = true
namespace = "testnet"

[consensus]
type = "qr-avalanche"
block_time = "5s"
```

### Environment Variables

Key environment variables:

```bash
# Network configuration
QUDAG_NETWORK_ID=qudag-testnet-local
QUDAG_DARK_DOMAIN_ENABLED=true

# Port configuration
QUDAG_P2P_PORT=4001
QUDAG_RPC_PORT=8080
QUDAG_METRICS_PORT=9090

# Logging
RUST_LOG=info,qudag=debug
RUST_BACKTRACE=1
```

## 🚨 Troubleshooting

### Common Issues

1. **Nodes not connecting**
   ```bash
   # Check network connectivity
   docker network ls
   docker network inspect qudag-testnet_qudag_testnet
   ```

2. **Health checks failing**
   ```bash
   # Check logs
   docker-compose logs node1
   
   # Check API endpoints
   curl http://localhost:8080/api/v1/health
   ```

3. **Monitoring not working**
   ```bash
   # Restart monitoring stack
   docker-compose restart prometheus grafana
   
   # Check Prometheus targets
   curl http://localhost:9094/api/v1/targets
   ```

### Debug Commands

```bash
# View logs
docker-compose logs -f node1

# Execute commands in container
docker-compose exec node1 bash

# Check container stats
docker stats

# Inspect container
docker inspect qudag-testnet-node1

# Reset everything
docker-compose down -v
docker system prune -f
```

## 📝 Management Commands

### Start/Stop Services

```bash
# Start all services
docker-compose up -d

# Stop all services
docker-compose down

# Restart a specific node
docker-compose restart node1

# Scale services (if needed)
docker-compose up -d --scale node1=1
```

### Data Management

```bash
# Backup node data
docker run --rm -v qudag-testnet_node1_data:/data -v $(pwd):/backup \
    alpine tar czf /backup/node1-backup.tar.gz /data

# Restore node data
docker run --rm -v qudag-testnet_node1_data:/data -v $(pwd):/backup \
    alpine tar xzf /backup/node1-backup.tar.gz -C /

# Clean up volumes
docker-compose down -v
```

### Key Management

```bash
# Regenerate keys only
./scripts/setup.sh keys-only

# View peer IDs
for i in {1..4}; do
    echo "Node $i: $(cat keys/node$i/peer_id.txt)"
done
```

## 📈 Scaling and Production

### Production Considerations

1. **Resource Allocation**
   - Increase memory limits for production workloads
   - Adjust CPU limits based on expected traffic
   - Configure appropriate storage volumes

2. **Security Hardening**
   - Enable TLS for all endpoints
   - Implement proper authentication
   - Regular security updates

3. **Monitoring and Alerting**
   - Configure alert destinations (email, Slack, etc.)
   - Set up log aggregation
   - Implement health checks

4. **Backup and Recovery**
   - Automated backup procedures
   - Disaster recovery planning
   - Data replication strategies

### Horizontal Scaling

To add more nodes:

1. Create new configuration files
2. Update docker-compose.yml
3. Generate new keys
4. Update bootstrap configuration

## 📄 License

This configuration is part of the QuDAG project and follows the same license terms (MIT OR Apache-2.0).

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📞 Support

- GitHub Issues: [QuDAG Issues](https://github.com/ruvnet/QuDAG/issues)
- Documentation: [QuDAG Docs](https://github.com/ruvnet/QuDAG)
- Community: [Discord](https://discord.gg/qudag)