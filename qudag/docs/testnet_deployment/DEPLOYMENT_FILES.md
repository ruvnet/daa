# QuDAG Testnet Deployment Files

This directory contains all the necessary files for deploying a 4-node QuDAG testnet on Fly.io.

## Directory Structure

```
testnet_deployment/
├── README.md                 # Comprehensive deployment guide
├── DEPLOYMENT_FILES.md       # This file - deployment file inventory
├── .env.example              # Environment variables template
├── Dockerfile                # Optimized multi-stage Docker build
├── docker-compose.yml        # Local testing configuration
│
├── scripts/                  # Deployment and management scripts
│   ├── deployment.sh         # Main deployment automation
│   ├── setup-secrets.sh      # Node key and secret generation
│   ├── monitor-nodes.sh      # Real-time monitoring tool
│   └── cleanup.sh            # Testnet teardown script
│
├── nodes/                    # Fly.io configuration files
│   ├── fly.node1.toml        # Toronto node (bootstrap)
│   ├── fly.node2.toml        # Amsterdam node
│   ├── fly.node3.toml        # Singapore node
│   └── fly.node4.toml        # San Francisco node
│
└── configs/                  # Node and monitoring configurations
    ├── node1.toml            # Bootstrap node configuration
    ├── node2.toml            # Amsterdam node configuration
    ├── node3.toml            # Singapore node configuration
    ├── node4.toml            # San Francisco node configuration
    ├── prometheus.yml        # Prometheus monitoring config
    ├── grafana-datasources.yml  # Grafana data source setup
    ├── grafana-dashboards.yml   # Grafana dashboard provisioning
    ├── alerts.yml            # Prometheus alert rules
    ├── qudag-node-stub.rs    # Placeholder node binary (for testing)
    └── dashboards/           # Grafana dashboard definitions
        └── qudag-overview.json  # Main monitoring dashboard
```

## File Descriptions

### Core Files

- **Dockerfile**: Multi-stage build using Cargo Chef for efficient caching
- **docker-compose.yml**: Local 4-node testnet with Prometheus/Grafana
- **.env.example**: Template for environment configuration

### Scripts

- **deployment.sh**: Automated deployment with app creation, volumes, and verification
- **setup-secrets.sh**: Generates Ed25519 keys, API tokens, and peer IDs
- **monitor-nodes.sh**: Interactive monitoring with health checks and metrics
- **cleanup.sh**: Safe teardown with backup options

### Node Configurations

Each node has:
- **fly.nodeX.toml**: Fly.io deployment configuration
- **configs/nodeX.toml**: QuDAG node runtime configuration

### Monitoring Stack

- **prometheus.yml**: Scrapes metrics from all 4 nodes
- **grafana-*.yml**: Auto-provisioning for dashboards and data sources
- **alerts.yml**: Critical alerts for node health and consensus
- **dashboards/**: Pre-built Grafana dashboards

## Quick Reference

```bash
# Deploy testnet
./scripts/deployment.sh

# Monitor nodes
./scripts/monitor-nodes.sh -c -v

# View logs
flyctl logs -a qudag-testnet-node1

# Cleanup
./scripts/cleanup.sh --all
```

## Security Files (Generated)

The following files are created by `setup-secrets.sh` and stored in `.secrets/`:
- Node private keys (Ed25519)
- API authentication tokens
- Peer IDs for bootstrap configuration

These are automatically added to `.gitignore` and should never be committed.