# QuDAG Deployment Guide

This guide provides comprehensive instructions for deploying QuDAG nodes in various environments, from development to production.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Building from Source](#building-from-source)
4. [Configuration](#configuration)
5. [Deployment Methods](#deployment-methods)
6. [Monitoring and Maintenance](#monitoring-and-maintenance)
7. [Security Considerations](#security-considerations)
8. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

#### Minimum Requirements
- **CPU**: 2 cores, x86_64 or ARM64
- **Memory**: 2GB RAM
- **Storage**: 10GB available disk space
- **Network**: Stable internet connection with open ports

#### Recommended Requirements
- **CPU**: 4+ cores, x86_64 or ARM64
- **Memory**: 8GB+ RAM
- **Storage**: 50GB+ SSD storage
- **Network**: High-bandwidth connection (10+ Mbps)

### Software Dependencies

#### Operating System Support
- Linux (Ubuntu 20.04+, CentOS 8+, Debian 11+)
- macOS (10.15+)
- Windows (with WSL2)

#### Required Software
- **Rust**: 1.70+ (latest stable recommended)
- **Git**: For source code management
- **OpenSSL**: Development libraries
- **pkg-config**: Build dependencies

#### Installation Commands

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y curl build-essential libssl-dev pkg-config git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**CentOS/RHEL:**
```bash
sudo yum groupinstall -y "Development Tools"
sudo yum install -y openssl-devel pkg-config git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**macOS:**
```bash
# Install Homebrew if not present
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install openssl pkg-config git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## Environment Setup

### Development Environment

1. **Clone Repository**
   ```bash
   git clone https://github.com/ruvnet/QuDAG.git
   cd QuDAG
   ```

2. **Verify Build Environment**
   ```bash
   cargo --version
   rustc --version
   ```

3. **Run Initial Tests**
   ```bash
   cargo test --workspace
   ```

### Production Environment

1. **Create Dedicated User**
   ```bash
   sudo useradd -m -s /bin/bash qudag
   sudo usermod -aG sudo qudag
   ```

2. **Set Up Directory Structure**
   ```bash
   sudo mkdir -p /opt/qudag/{bin,config,data,logs}
   sudo chown -R qudag:qudag /opt/qudag
   ```

3. **Configure Firewall**
   ```bash
   # Allow QuDAG default port
   sudo ufw allow 8000/tcp
   sudo ufw enable
   ```

## Building from Source

### Standard Build

1. **Debug Build (Development)**
   ```bash
   cargo build --workspace
   ```

2. **Release Build (Production)**
   ```bash
   cargo build --release --workspace
   ```

3. **Install Binary**
   ```bash
   # Copy to system location
   sudo cp target/release/qudag /usr/local/bin/
   sudo chmod +x /usr/local/bin/qudag
   ```

### Optimized Build

For production deployments with maximum performance:

```bash
# Build with native CPU optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release --workspace

# Build with specific features
cargo build --release --features "optimized-crypto,high-performance"
```

### Cross-Compilation

For deploying to different architectures:

```bash
# Install cross-compilation target
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl
```

## Configuration

### Node Configuration

Create a configuration file at `/opt/qudag/config/node.toml`:

```toml
[node]
# Node identity and networking
node_id = "node-001"
listen_address = "0.0.0.0:8000"
external_address = "your-external-ip:8000"

# Maximum number of peer connections
max_peers = 50

# Bootstrap nodes for network discovery
bootstrap_nodes = [
    "bootstrap1.qudag.io:8000",
    "bootstrap2.qudag.io:8000"
]

[consensus]
# QR-Avalanche consensus parameters
query_sample_size = 20
finality_threshold = 0.80
finality_timeout = "5s"
confirmation_depth = 4

[crypto]
# Cryptographic configuration
key_rotation_interval = "24h"
enable_quantum_resistance = true

[network]
# Network behavior
enable_anonymous_routing = true
circuit_min_hops = 3
circuit_max_hops = 7
enable_dark_addressing = true

[storage]
# Data storage configuration
data_directory = "/opt/qudag/data"
log_directory = "/opt/qudag/logs"
max_storage_size = "10GB"

[performance]
# Performance tuning
worker_threads = 4
async_runtime_threads = 8
memory_limit = "2GB"

[monitoring]
# Metrics and monitoring
enable_metrics = true
metrics_port = 9090
log_level = "info"
```

### Security Configuration

Create `/opt/qudag/config/security.toml`:

```toml
[security]
# TLS configuration
tls_cert_path = "/opt/qudag/config/tls.crt"
tls_key_path = "/opt/qudag/config/tls.key"

# Key management
key_storage_path = "/opt/qudag/data/keys"
enable_hardware_security = false

# Network security
enable_traffic_obfuscation = true
require_peer_authentication = true
max_connection_rate = 10  # connections per second

[access_control]
# Administrative access
admin_api_enabled = false
admin_api_bind = "127.0.0.1:8001"
admin_api_key = "your-secure-admin-key"

# Rate limiting
message_rate_limit = 100  # messages per second
bandwidth_limit = "10MB"  # per second
```

### Environment Variables

For sensitive configuration, use environment variables:

```bash
# Create environment file
cat > /opt/qudag/config/environment << EOF
QUDAG_NODE_ID=node-001
QUDAG_LISTEN_ADDR=0.0.0.0:8000
QUDAG_BOOTSTRAP_NODES=bootstrap1.qudag.io:8000,bootstrap2.qudag.io:8000
QUDAG_DATA_DIR=/opt/qudag/data
QUDAG_LOG_LEVEL=info
QUDAG_ADMIN_KEY=your-secure-admin-key
EOF

# Set proper permissions
chmod 600 /opt/qudag/config/environment
```

## Deployment Methods

### Systemd Service (Recommended for Linux)

1. **Create Service File**
   ```bash
   sudo tee /etc/systemd/system/qudag.service << EOF
   [Unit]
   Description=QuDAG Protocol Node
   After=network.target
   Wants=network.target

   [Service]
   Type=exec
   User=qudag
   Group=qudag
   ExecStart=/usr/local/bin/qudag start --config /opt/qudag/config/node.toml
   ExecReload=/bin/kill -HUP \$MAINPID
   KillMode=process
   Restart=on-failure
   RestartSec=5s

   # Security settings
   NoNewPrivileges=true
   PrivateTmp=true
   ProtectSystem=strict
   ProtectHome=true
   ReadWritePaths=/opt/qudag

   # Resource limits
   LimitNOFILE=65536
   LimitNPROC=4096

   # Environment
   EnvironmentFile=-/opt/qudag/config/environment

   [Install]
   WantedBy=multi-user.target
   EOF
   ```

2. **Enable and Start Service**
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable qudag
   sudo systemctl start qudag
   ```

3. **Check Service Status**
   ```bash
   sudo systemctl status qudag
   sudo journalctl -u qudag -f
   ```

### Docker Deployment

1. **Create Dockerfile**
   ```dockerfile
   FROM rust:1.70-slim as builder

   WORKDIR /usr/src/app
   COPY . .

   RUN apt-get update && \
       apt-get install -y pkg-config libssl-dev && \
       cargo build --release

   FROM debian:bullseye-slim

   RUN apt-get update && \
       apt-get install -y ca-certificates libssl1.1 && \
       rm -rf /var/lib/apt/lists/*

   COPY --from=builder /usr/src/app/target/release/qudag /usr/local/bin/qudag

   RUN useradd -m qudag
   USER qudag

   EXPOSE 8000 9090

   CMD ["qudag", "start"]
   ```

2. **Build and Run Container**
   ```bash
   # Build image
   docker build -t qudag:latest .

   # Run container
   docker run -d \
     --name qudag-node \
     -p 8000:8000 \
     -p 9090:9090 \
     -v qudag-data:/opt/qudag/data \
     -v qudag-config:/opt/qudag/config \
     --restart unless-stopped \
     qudag:latest
   ```

3. **Docker Compose**
   ```yaml
   version: '3.8'

   services:
     qudag:
       build: .
       ports:
         - "8000:8000"
         - "9090:9090"
       volumes:
         - qudag-data:/opt/qudag/data
         - qudag-config:/opt/qudag/config
         - qudag-logs:/opt/qudag/logs
       environment:
         - QUDAG_LOG_LEVEL=info
         - QUDAG_LISTEN_ADDR=0.0.0.0:8000
       restart: unless-stopped
       healthcheck:
         test: ["CMD", "qudag", "status"]
         interval: 30s
         timeout: 10s
         retries: 3

   volumes:
     qudag-data:
     qudag-config:
     qudag-logs:
   ```

### Kubernetes Deployment

1. **ConfigMap**
   ```yaml
   apiVersion: v1
   kind: ConfigMap
   metadata:
     name: qudag-config
   data:
     node.toml: |
       [node]
       node_id = "k8s-node"
       listen_address = "0.0.0.0:8000"
       max_peers = 50
       
       [consensus]
       query_sample_size = 20
       finality_threshold = 0.80
   ```

2. **Deployment**
   ```yaml
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: qudag-node
   spec:
     replicas: 3
     selector:
       matchLabels:
         app: qudag
     template:
       metadata:
         labels:
           app: qudag
       spec:
         containers:
         - name: qudag
           image: qudag:latest
           ports:
           - containerPort: 8000
           - containerPort: 9090
           volumeMounts:
           - name: config
             mountPath: /opt/qudag/config
           - name: data
             mountPath: /opt/qudag/data
           resources:
             requests:
               memory: "1Gi"
               cpu: "500m"
             limits:
               memory: "2Gi"
               cpu: "1"
         volumes:
         - name: config
           configMap:
             name: qudag-config
         - name: data
           persistentVolumeClaim:
             claimName: qudag-data
   ```

3. **Service**
   ```yaml
   apiVersion: v1
   kind: Service
   metadata:
     name: qudag-service
   spec:
     selector:
       app: qudag
     ports:
     - name: p2p
       port: 8000
       targetPort: 8000
     - name: metrics
       port: 9090
       targetPort: 9090
     type: LoadBalancer
   ```

## Monitoring and Maintenance

### Health Checks

1. **Service Health**
   ```bash
   # Check if node is running
   qudag status

   # Check peer connections
   qudag peer list

   # Check network statistics
   qudag network stats
   ```

2. **System Resources**
   ```bash
   # Monitor CPU and memory usage
   top -p $(pgrep qudag)

   # Check disk usage
   df -h /opt/qudag

   # Monitor network connections
   netstat -tlpn | grep :8000
   ```

### Metrics Collection

1. **Prometheus Integration**
   ```yaml
   # prometheus.yml
   global:
     scrape_interval: 15s

   scrape_configs:
   - job_name: 'qudag'
     static_configs:
     - targets: ['localhost:9090']
   ```

2. **Grafana Dashboard**
   - Import QuDAG dashboard from `monitoring/grafana-dashboard.json`
   - Monitor key metrics: TPS, latency, peer count, memory usage

### Log Management

1. **Log Rotation**
   ```bash
   # Create logrotate configuration
   sudo tee /etc/logrotate.d/qudag << EOF
   /opt/qudag/logs/*.log {
       daily
       rotate 30
       compress
       delaycompress
       missingok
       notifempty
       create 644 qudag qudag
       postrotate
           systemctl reload qudag
       endscript
   }
   EOF
   ```

2. **Centralized Logging**
   ```bash
   # Ship logs to centralized system
   tail -F /opt/qudag/logs/qudag.log | \
     filebeat -e -c /etc/filebeat/filebeat.yml
   ```

### Backup and Recovery

1. **Data Backup**
   ```bash
   #!/bin/bash
   # Daily backup script
   BACKUP_DIR="/backup/qudag/$(date +%Y-%m-%d)"
   mkdir -p "$BACKUP_DIR"
   
   # Stop node
   systemctl stop qudag
   
   # Create backup
   tar -czf "$BACKUP_DIR/qudag-data.tar.gz" /opt/qudag/data
   tar -czf "$BACKUP_DIR/qudag-config.tar.gz" /opt/qudag/config
   
   # Start node
   systemctl start qudag
   
   # Cleanup old backups (keep 30 days)
   find /backup/qudag -type d -mtime +30 -exec rm -rf {} \;
   ```

2. **Configuration Backup**
   ```bash
   # Version control for configuration
   cd /opt/qudag/config
   git init
   git add .
   git commit -m "Initial configuration"
   ```

### Updates and Upgrades

1. **Update Process**
   ```bash
   #!/bin/bash
   # Update script
   
   # Stop service
   systemctl stop qudag
   
   # Backup current binary
   cp /usr/local/bin/qudag /usr/local/bin/qudag.backup
   
   # Download and install new version
   wget https://github.com/ruvnet/QuDAG/releases/latest/download/qudag-linux-x86_64
   chmod +x qudag-linux-x86_64
   sudo mv qudag-linux-x86_64 /usr/local/bin/qudag
   
   # Start service
   systemctl start qudag
   
   # Verify update
   qudag --version
   ```

2. **Rolling Updates (Kubernetes)**
   ```bash
   # Update deployment with new image
   kubectl set image deployment/qudag-node qudag=qudag:v0.2.0
   
   # Monitor rollout
   kubectl rollout status deployment/qudag-node
   ```

## Security Considerations

### Network Security

1. **Firewall Configuration**
   ```bash
   # UFW configuration
   sudo ufw default deny incoming
   sudo ufw default allow outgoing
   sudo ufw allow 22/tcp    # SSH
   sudo ufw allow 8000/tcp  # QuDAG P2P
   sudo ufw limit ssh       # Rate limit SSH
   sudo ufw enable
   ```

2. **TLS Certificates**
   ```bash
   # Generate self-signed certificate (development)
   openssl req -x509 -newkey rsa:4096 -keyout tls.key -out tls.crt -days 365 -nodes
   
   # Use Let's Encrypt (production)
   certbot certonly --standalone -d your-domain.com
   ```

### Access Control

1. **SSH Hardening**
   ```bash
   # /etc/ssh/sshd_config
   PermitRootLogin no
   PasswordAuthentication no
   PubkeyAuthentication yes
   AllowUsers qudag
   ```

2. **File Permissions**
   ```bash
   # Secure configuration files
   chmod 600 /opt/qudag/config/*.toml
   chmod 700 /opt/qudag/data
   chown -R qudag:qudag /opt/qudag
   ```

### System Hardening

1. **Fail2ban Configuration**
   ```ini
   # /etc/fail2ban/jail.local
   [sshd]
   enabled = true
   port = ssh
   filter = sshd
   logpath = /var/log/auth.log
   maxretry = 3
   bantime = 3600
   ```

2. **Automated Security Updates**
   ```bash
   # Ubuntu/Debian
   sudo apt install unattended-upgrades
   sudo dpkg-reconfigure -plow unattended-upgrades
   ```

## Troubleshooting

### Common Issues

1. **Node Won't Start**
   ```bash
   # Check configuration
   qudag config validate

   # Check permissions
   ls -la /opt/qudag/
   sudo -u qudag ls -la /opt/qudag/data

   # Check logs
   journalctl -u qudag -n 50
   ```

2. **Network Connectivity Issues**
   ```bash
   # Test network connectivity
   telnet bootstrap1.qudag.io 8000

   # Check firewall
   sudo ufw status
   sudo iptables -L

   # Check DNS resolution
   nslookup bootstrap1.qudag.io
   ```

3. **Performance Issues**
   ```bash
   # Check system resources
   htop
   iostat -x 1

   # Check QuDAG metrics
   curl http://localhost:9090/metrics

   # Profile the application
   perf record -g qudag start
   perf report
   ```

### Debug Mode

1. **Enable Debug Logging**
   ```bash
   # Set log level to debug
   export QUDAG_LOG_LEVEL=debug
   systemctl restart qudag
   ```

2. **Performance Profiling**
   ```bash
   # CPU profiling
   cargo build --release
   CPUPROFILE=/tmp/qudag.prof ./target/release/qudag start

   # Memory profiling
   valgrind --tool=massif ./target/release/qudag start
   ```

### Recovery Procedures

1. **Data Corruption Recovery**
   ```bash
   # Stop node
   systemctl stop qudag

   # Restore from backup
   tar -xzf /backup/qudag/latest/qudag-data.tar.gz -C /

   # Verify data integrity
   qudag data verify

   # Start node
   systemctl start qudag
   ```

2. **Network Partition Recovery**
   ```bash
   # Clear peer cache
   rm -rf /opt/qudag/data/peers.db

   # Reset routing tables
   qudag network reset

   # Restart with fresh bootstrap
   systemctl restart qudag
   ```

## Support and Resources

### Documentation
- [API Reference](api/)
- [Architecture Guide](architecture/)
- [Security Best Practices](security/best_practices.md)

### Community
- GitHub Issues: https://github.com/ruvnet/QuDAG/issues
- Discord: https://discord.gg/qudag
- Forum: https://forum.qudag.io

### Professional Support
- Enterprise Support: support@qudag.io
- Consulting Services: consulting@qudag.io
- Training: training@qudag.io

This deployment guide provides comprehensive instructions for setting up QuDAG nodes in various environments. Always test deployments in a staging environment before production deployment.