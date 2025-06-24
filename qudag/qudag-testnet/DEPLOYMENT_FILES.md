# QuDAG Testnet Deployment Files

This directory contains all the necessary files for deploying a 4-node QuDAG testnet on Fly.io.

## Directory Structure

```
qudag-testnet/
├── README.md                 # Comprehensive deployment guide
├── DEPLOYMENT_FILES.md       # This file - deployment file inventory
├── .env.example              # Environment variables template
├── .gitignore                # Git ignore for security files
├── Dockerfile                # Optimized multi-stage Docker build
├── docker-compose.yml        # Local testing configuration
│
├── scripts/                  # Deployment and management scripts
│   ├── deployment.sh         # Main deployment automation
│   ├── setup-secrets.sh      # Node key and secret generation
│   ├── monitor-nodes.sh      # Real-time monitoring tool
│   ├── cleanup.sh            # Testnet teardown script
│   ├── backup-nodes.sh       # Backup utility
│   └── restore-node.sh       # Restore from backup
│
├── nodes/                    # Fly.io configuration files
│   ├── fly.node1.toml        # Toronto node (bootstrap)
│   ├── fly.node2.toml        # Amsterdam node
│   ├── fly.node3.toml        # Singapore node
│   └── fly.node4.toml        # San Francisco node
│
├── configs/                  # Node and monitoring configurations
│   ├── node1.toml            # Bootstrap node configuration
│   ├── node2.toml            # Amsterdam node configuration
│   ├── node3.toml            # Singapore node configuration
│   ├── node4.toml            # San Francisco node configuration
│   ├── prometheus.yml        # Prometheus monitoring config
│   ├── grafana-datasources.yml  # Grafana data source setup
│   ├── grafana-dashboards.yml   # Grafana dashboard provisioning
│   ├── alerts.yml            # Prometheus alert rules
│   └── dashboards/           # Grafana dashboard definitions
│       ├── qudag-overview.json      # Main monitoring dashboard
│       ├── qudag-performance.json   # Performance metrics dashboard
│       └── qudag-network.json       # Network topology dashboard
│
└── .secrets/                 # Generated secrets (git ignored)
    ├── node_keys/            # Ed25519 keypairs
    ├── api_tokens/           # API authentication tokens
    └── peer_ids/             # Generated peer IDs
```

## File Descriptions

### Core Files

#### `Dockerfile`
Multi-stage build using Cargo Chef for efficient caching:
- Stage 1: Recipe preparation
- Stage 2: Dependency building
- Stage 3: Application building
- Stage 4: Minimal runtime image

#### `docker-compose.yml`
Local 4-node testnet configuration with:
- 4 QuDAG nodes with proper networking
- Prometheus metrics collection
- Grafana visualization
- Health checks and dependencies

#### `.env.example`
Template for environment configuration including:
- Fly.io API tokens
- Network configuration
- Node settings
- Security parameters

#### `.gitignore`
Ensures security files are not committed:
- `.secrets/` directory
- Private keys
- API tokens
- Local data volumes

### Scripts

#### `deployment.sh`
Main deployment automation script that:
- Creates Fly.io applications
- Provisions persistent volumes
- Configures secrets
- Deploys nodes in sequence
- Verifies deployment health

#### `setup-secrets.sh`
Security setup script that generates:
- Ed25519 keypairs for each node
- API authentication tokens
- Peer IDs for bootstrap configuration
- TLS certificates (optional)

#### `monitor-nodes.sh`
Interactive monitoring tool with features:
- Real-time status updates
- Health check monitoring
- Performance metrics
- JSON output for automation
- Continuous mode with refresh

#### `cleanup.sh`
Safe teardown script with options:
- Keep data volumes (default)
- Remove volumes (`-v` flag)
- Complete removal (`--all` flag)
- Backup before cleanup

#### `backup-nodes.sh`
Automated backup utility that:
- Backs up node data
- Saves configuration
- Exports secrets (encrypted)
- Creates timestamped archives

#### `restore-node.sh`
Restoration utility for:
- Restoring from backups
- Migrating nodes
- Disaster recovery

### Node Configurations

Each node has two configuration files:

#### Fly.io Configuration (`nodes/fly.nodeX.toml`)
- Application name and organization
- Build configuration
- Resource allocation (CPU, memory)
- Volume mounts
- Network settings
- Health checks

#### QuDAG Configuration (`configs/nodeX.toml`)
- Network identity
- P2P settings
- Bootstrap peers
- Consensus parameters
- Storage configuration
- RPC/API settings
- Dark domain configuration

### Monitoring Stack

#### `prometheus.yml`
Prometheus configuration for:
- Scraping all 4 nodes
- Retention policies
- Alert rules integration
- Service discovery

#### `grafana-datasources.yml`
Automatic provisioning of:
- Prometheus data source
- Alert notification channels
- Query configurations

#### `grafana-dashboards.yml`
Dashboard provisioning for:
- Automatic dashboard loading
- Folder organization
- Default dashboard selection

#### `alerts.yml`
Critical alerts for:
- Node downtime
- Consensus failures
- Resource exhaustion
- Network partitions

### Dashboards

#### `qudag-overview.json`
Main monitoring dashboard showing:
- Network topology
- Node health status
- Transaction throughput
- Block production rate

#### `qudag-performance.json`
Performance metrics including:
- CPU and memory usage
- Disk I/O
- Network bandwidth
- P2P connection metrics

#### `qudag-network.json`
Network topology visualization:
- Peer connections
- Geographic distribution
- Latency measurements
- Connection health

## Quick Reference

```bash
# Deploy testnet
./scripts/deployment.sh

# Monitor nodes
./scripts/monitor-nodes.sh -c -v

# View logs
flyctl logs -a qudag-testnet-node1

# Backup data
./scripts/backup-nodes.sh

# Cleanup
./scripts/cleanup.sh --all
```

## Security Files (Generated)

The following files are created by `setup-secrets.sh` and stored in `.secrets/`:
- Node private keys (Ed25519)
- API authentication tokens
- Peer IDs for bootstrap configuration
- TLS certificates (if enabled)

These are automatically added to `.gitignore` and should never be committed.

## Environment Variables

Key variables configured in `.env`:
- `FLY_API_TOKEN`: Fly.io authentication
- `QUDAG_NETWORK_ID`: Network identifier
- `QUDAG_DARK_DOMAIN_ENABLED`: Dark domain feature flag
- `QUDAG_NODE_PREFIX`: Node naming prefix
- `MONITORING_ENABLED`: Enable monitoring stack
- `BACKUP_RETENTION_DAYS`: Backup retention period

## Deployment Workflow

1. **Environment Setup**: Configure `.env` file
2. **Secret Generation**: Run `setup-secrets.sh`
3. **Local Testing**: Use `docker-compose up`
4. **Production Deploy**: Execute `deployment.sh`
5. **Monitoring**: Use `monitor-nodes.sh`
6. **Maintenance**: Regular backups and updates

## Notes

- All scripts include comprehensive error handling
- Deployment is idempotent - safe to run multiple times
- Monitoring stack is optional but recommended
- Backup strategy should be customized for production use