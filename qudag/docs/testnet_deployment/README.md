# QuDAG Exchange Testnet Deployment Guide

This guide provides comprehensive instructions for deploying QuDAG testnet with full Exchange functionality, including quantum-resistant rUv token operations, dynamic fee models, and immutable deployment capabilities.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Network Architecture](#network-architecture)
4. [Quick Start](#quick-start)
5. [Detailed Deployment Guide](#detailed-deployment-guide)
6. [Configuration](#configuration)
7. [Monitoring](#monitoring)
8. [Troubleshooting](#troubleshooting)
9. [Security Considerations](#security-considerations)
10. [Cost Breakdown](#cost-breakdown)
11. [Maintenance](#maintenance)

## 🎯 Overview

The QuDAG Exchange testnet deployment supports:

- **rUv Token System**: Quantum-resistant Resource Utilization Vouchers
- **Dynamic Fee Model**: Tiered fees with verified agent benefits (0.1%-1.0% unverified, 0.25%-0.5% verified)
- **Immutable Deployment**: Optional post-initialization configuration locking
- **Multi-Region Distribution**: Resilient network across multiple geographic regions
- **Quantum-Resistant Security**: ML-DSA-87 signatures throughout

### Node Architecture

| Node Type | Location | Storage | Functions |
|-----------|----------|---------|-----------|
| Bootstrap | Toronto (yyz) | 26GB | Genesis, Exchange init, Full operations |
| Exchange Full | Montreal (yul) | 24GB | Complete Exchange, Agent verification |
| Validator | Chicago (ord) | 19GB | Transaction validation, Consensus |
| Light | New York (ewr) | 15GB | Client operations, Query relay |

Each node runs the complete QuDAG protocol with:
- **P2P networking** using libp2p with quantum-resistant encryption
- **rUv Token Exchange** with dynamic fee calculations
- **Dark domain registration** system with quantum fingerprints
- **QR-Avalanche consensus** with ML-DSA-87 signatures
- **Immutable deployment** capabilities for production security
- **Persistent storage** for DAG and Exchange data
- **Comprehensive monitoring** with Exchange-specific metrics

## Prerequisites

### Required Tools
- [Fly.io CLI](https://fly.io/docs/hands-on/install-flyctl/) (flyctl)
- Docker (for local testing)
- Git
- jq (for JSON processing)
- OpenSSL (for key generation)

### Fly.io Account Setup
1. Create a Fly.io account: https://fly.io/signup
2. Install flyctl: `curl -L https://fly.io/install.sh | sh`
3. Authenticate: `flyctl auth login`
4. Add credit card (required for persistent volumes)

### System Requirements
- Minimum 8GB RAM for local testing
- 20GB free disk space
- Linux/macOS (Windows via WSL2)

## Network Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        QuDAG Testnet Topology                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│     ┌─────────────┐                      ┌─────────────┐          │
│     │   Node 1    │                      │   Node 2    │          │
│     │  Toronto    │◄────────────────────►│ Amsterdam   │          │
│     │   (yyz)     │                      │   (ams)     │          │
│     │ [Bootstrap] │                      │ [Validator] │          │
│     └──────┬──────┘                      └──────┬──────┘          │
│            │                                     │                  │
│            │              P2P Mesh               │                  │
│            │              Network                │                  │
│            │                                     │                  │
│     ┌──────┴──────┐                      ┌──────┴──────┐          │
│     │   Node 3    │                      │   Node 4    │          │
│     │ Singapore   │◄────────────────────►│San Francisco│          │
│     │   (sin)     │                      │   (sjc)     │          │
│     │ [Validator] │                      │ [Validator] │          │
│     └─────────────┘                      └─────────────┘          │
│                                                                     │
│ Legend:                                                             │
│ ◄────► P2P Connection (libp2p)                                     │
│ [Role] Node Role in Network                                        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Port Configuration
- **4001**: P2P networking (TCP/UDP)
- **8080**: RPC API endpoint
- **9090**: Prometheus metrics

## 🚀 Deployment Options

### 1. Fly.io Deployment (Recommended)

**Features:**
- 4-node distributed testnet with Exchange functionality
- Built-in private networking (6PN) for quantum-secure communication
- Persistent storage for DAG and Exchange data
- Automatic scaling and monitoring
- Cost: ~$45/month for full Exchange setup

### 2. Docker Compose Deployment

**Features:**
- Local development and testing with Exchange
- Complete rUv token functionality
- Isolated network environment
- Easy debugging and monitoring

### 3. Kubernetes Deployment

**Features:**
- Production-ready orchestration
- Exchange auto-scaling capabilities
- High availability configuration
- Enterprise monitoring integration

## Quick Start

### Fly.io Exchange Deployment

```bash
# 1. Clone and navigate to deployment directory
cd /workspaces/QuDAG/docs/testnet_deployment

# 2. Deploy Exchange-enabled testnet
./deploy-exchange-testnet-flyio.sh deploy

# 3. Verify Exchange functionality
./deploy-exchange-testnet-flyio.sh verify

# 4. Check deployment information
./deploy-exchange-testnet-flyio.sh info
```

### Local Exchange Testing

```bash
# 1. Build with Exchange support
cargo build --release --features "exchange"

# 2. Test Exchange CLI
./target/release/qudag exchange --help

# 3. Run local Exchange operations
./target/release/qudag exchange create-account --name alice
./target/release/qudag exchange balance --account alice
```

## Detailed Deployment Guide

### Step 1: Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Edit configuration
vim .env
```

Key environment variables:
- `FLY_API_TOKEN`: Your Fly.io API token
- `QUDAG_NETWORK_ID`: Network identifier (default: qudag-testnet)
- `QUDAG_DARK_DOMAIN_ENABLED`: Enable dark domain system

### Step 2: Generate Node Keys

```bash
# Generate cryptographic keys for all nodes
./scripts/setup-secrets.sh

# This creates:
# - Ed25519 keypairs for each node
# - API authentication tokens
# - Peer IDs for bootstrap configuration
```

### Step 3: Local Testing (Optional)

```bash
# Test the setup locally with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f

# Stop local test
docker-compose down
```

### Step 4: Deploy to Fly.io

```bash
# Run the deployment script
./scripts/deployment.sh

# The script will:
# 1. Create Fly.io applications
# 2. Create persistent volumes (10GB each)
# 3. Set secrets
# 4. Deploy nodes sequentially
# 5. Configure bootstrap peers
# 6. Verify deployment
```

### Step 5: Verify Deployment

```bash
# Check all nodes status
./scripts/monitor-nodes.sh

# Check individual node
flyctl status -a qudag-testnet-node1

# View logs
flyctl logs -a qudag-testnet-node1
```

## Configuration

### Node Configuration Files

Each node has a TOML configuration file in `configs/`:
- `node1.toml`: Bootstrap node configuration
- `node2.toml`, `node3.toml`, `node4.toml`: Validator configurations

### Key Configuration Parameters

```toml
[network]
network_id = "qudag-testnet"
listen_address = "/ip4/0.0.0.0/tcp/4001"
external_address = "/dns4/qudag-testnet-node1.fly.dev/tcp/4001"

[p2p]
max_peers = 50
min_peers = 3
bootstrap_peers = ["/dns4/qudag-testnet-node1.fly.dev/tcp/4001/p2p/PEER_ID"]

[dark_domain]
enabled = true
registration_fee = 100
namespace = "testnet"

[consensus]
type = "dag"
block_time = "5s"
```

### Updating Configuration

```bash
# Edit configuration
vim configs/node1.toml

# Redeploy specific node
flyctl deploy -a qudag-testnet-node1 --config nodes/fly.node1.toml
```

## Monitoring

### Prometheus + Grafana Stack

Local monitoring setup:
```bash
# Start monitoring stack
docker-compose up prometheus grafana -d

# Access dashboards
# Prometheus: http://localhost:9094
# Grafana: http://localhost:3000 (admin/admin)
```

### Real-time Node Monitoring

```bash
# Basic monitoring
./scripts/monitor-nodes.sh

# Continuous monitoring with 10s refresh
./scripts/monitor-nodes.sh -c -i 10

# Verbose mode with metrics
./scripts/monitor-nodes.sh -c -v

# JSON output for automation
./scripts/monitor-nodes.sh -j
```

### Key Metrics to Monitor

- **Peer Count**: Should be ≥ 3 for healthy networking
- **Block Production**: New blocks every ~5 seconds
- **Memory Usage**: Should stay under 1.5GB
- **CPU Usage**: Normal range 10-30%
- **Network Latency**: P2P latency < 200ms optimal

## Troubleshooting

### Common Issues and Solutions

#### Node Won't Start
```bash
# Check logs
flyctl logs -a qudag-testnet-node1 --tail 100

# SSH into container
flyctl ssh console -a qudag-testnet-node1

# Check configuration
cat /data/qudag/config.toml
```

#### Connectivity Issues
```bash
# Check network status
flyctl ips list -a qudag-testnet-node1

# Verify P2P port is open
flyctl ssh console -a qudag-testnet-node1
nc -zv localhost 4001
```

#### Consensus Problems
```bash
# Check peer connections
curl https://qudag-testnet-node1.fly.dev/api/v1/peers

# Verify bootstrap configuration
flyctl secrets list -a qudag-testnet-node1
```

#### Storage Issues
```bash
# Check volume usage
flyctl volumes list -a qudag-testnet-node1

# Resize volume if needed
flyctl volumes extend <volume-id> -s 20
```

### Debug Commands

```bash
# Full system diagnostics
for node in qudag-testnet-node{1..4}; do
  echo "=== $node ==="
  flyctl status -a $node
  flyctl checks list -a $node
done

# Export all logs
for node in qudag-testnet-node{1..4}; do
  flyctl logs -a $node > logs/$node.log
done
```

## Security Considerations

### Network Security

1. **TLS Encryption**: All RPC endpoints use TLS
2. **Token Authentication**: API access requires authentication tokens
3. **Firewall Rules**: Only required ports are exposed
4. **Private Keys**: Stored as Fly.io secrets, never in code

### Best Practices

```bash
# Rotate API keys periodically
./scripts/setup-secrets.sh --rotate-keys

# Update node software
flyctl deploy -a qudag-testnet-node1 --image qudag:latest

# Backup critical data
flyctl ssh console -a qudag-testnet-node1
tar -czf backup.tar.gz /data/qudag/db
```

### Security Checklist

- [ ] Change default API tokens in `.env`
- [ ] Enable 2FA on Fly.io account
- [ ] Restrict CORS origins in production
- [ ] Monitor for unusual activity
- [ ] Keep node software updated
- [ ] Regular security audits

## Cost Breakdown

### Fly.io Pricing (as of 2024)

| Resource | Unit Price | Monthly Cost (4 nodes) |
|----------|-----------|------------------------|
| Shared CPU (2 vCPU) | $0.0000100/s | ~$26.00 |
| RAM (2GB) | $0.0000019/GB/s | ~$20.00 |
| Persistent Storage (10GB) | $0.15/GB/month | $6.00 |
| Bandwidth | $0.02/GB | ~$10.00 |
| **Total Estimated** | | **~$62/month** |

### Cost Optimization Tips

1. Use shared CPUs for testnet
2. Scale down during low activity
3. Implement data pruning
4. Use Fly.io free tier allowances

## Maintenance

### Regular Maintenance Tasks

#### Daily
- Monitor node health
- Check error logs
- Verify consensus participation

#### Weekly
- Review metrics trends
- Update dependencies
- Backup configuration

#### Monthly
- Rotate secrets
- Update node software
- Audit security logs
- Review costs

### Backup Procedures

```bash
# Backup all node data
./scripts/backup-nodes.sh

# Restore from backup
./scripts/restore-node.sh qudag-testnet-node1 backup-20240615.tar.gz
```

### Scaling Operations

```bash
# Add more resources
flyctl scale vm shared-cpu-2x -a qudag-testnet-node1

# Add more nodes
cp nodes/fly.node1.toml nodes/fly.node5.toml
# Edit configuration
flyctl deploy -a qudag-testnet-node5 --config nodes/fly.node5.toml
```

## Cleanup

To completely remove the testnet:

```bash
# Safe cleanup (keeps data)
./scripts/cleanup.sh

# Remove volumes too
./scripts/cleanup.sh -v

# Complete removal
./scripts/cleanup.sh --all
```

## Support and Resources

- QuDAG Documentation: https://github.com/yourusername/QuDAG
- Fly.io Documentation: https://fly.io/docs
- Community Discord: [Join Discord]
- Issue Tracker: [GitHub Issues]

## License

This deployment configuration is part of the QuDAG project and follows the same license terms.