# QuDAG Testnet Deployment Guide - Enhanced Configuration

This guide covers the enhanced deployment configuration for the QuDAG testnet with proper security, TLS, and network connectivity.

## Overview of Improvements

### 1. **Security Enhancements**
- **TLS 1.3 Support**: All nodes now support HTTPS with TLS 1.3
- **Certificate Management**: Automated CA and certificate generation
- **API Authentication**: Bearer token and API key authentication
- **Secure Secrets**: Proper secret management with Docker secrets and Fly.io secrets

### 2. **Network Configuration**
- **Dynamic Peer Discovery**: Bootstrap peer IDs are dynamically resolved
- **Proper Multiaddresses**: Correctly formatted libp2p multiaddresses
- **NAT Traversal**: Enhanced NAT traversal configuration
- **P2P Security**: Peer verification and secure handshakes

### 3. **Monitoring & Observability**
- **HTTPS Health Checks**: Secure health endpoints with TLS
- **Comprehensive Metrics**: P2P, consensus, crypto, and domain metrics
- **Alert Configuration**: Prometheus alerting rules
- **Grafana Dashboards**: Pre-configured dashboards for all metrics

### 4. **Deployment Automation**
- **Enhanced Scripts**: Robust deployment scripts with error handling
- **Parallel Deployment**: Validator nodes deploy in parallel
- **Health Verification**: Automatic health checks post-deployment
- **Rollback Support**: Automatic rollback on deployment failure

## Prerequisites

1. **Required Tools**:
   ```bash
   # Install required tools
   curl -L https://fly.io/install.sh | sh
   sudo apt-get install -y jq openssl docker docker-compose
   ```

2. **Fly.io Account**:
   ```bash
   flyctl auth login
   flyctl auth whoami
   ```

3. **Environment Configuration**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   nano .env
   ```

## Quick Start

### Local Development Deployment

1. **Setup Security Components**:
   ```bash
   ./scripts/setup-secrets.sh
   ./scripts/setup-tls.sh
   ```

2. **Start Local Testnet**:
   ```bash
   docker-compose -f docker-compose.enhanced.yml up -d
   ```

3. **Verify Health**:
   ```bash
   ./tls/verify-tls.sh local
   ```

### Production Deployment (Fly.io)

1. **Complete Setup**:
   ```bash
   ./scripts/deploy-enhanced.sh deploy
   ```

   This script will:
   - Check prerequisites
   - Generate secrets and TLS certificates
   - Create Fly.io applications
   - Deploy nodes with proper configuration
   - Update bootstrap peers dynamically
   - Verify deployment health

2. **Post-Deployment Verification**:
   ```bash
   ./scripts/deploy-enhanced.sh verify
   ./tls/verify-tls.sh production
   ```

## Configuration Details

### Node Configuration

Each node uses the secure template with:
- **Quantum-resistant cryptography** (ML-DSA-65 signatures)
- **Dark domain support** with quantum fingerprints
- **Secure RPC endpoints** (HTTP and HTTPS)
- **Resource limits** and monitoring

### TLS Configuration

- **Certificate Authority**: Self-signed CA for testnet
- **Server Certificates**: Unique certificates per node
- **Client Certificates**: For API authentication
- **Mutual TLS**: Optional client certificate verification

### Environment Variables

Key environment variables to configure:

```bash
# Fly.io Configuration
FLY_API_TOKEN=your_fly_api_token
FLY_ORG=your_fly_organization

# Security
GRAFANA_ADMIN_PASSWORD=secure_password_here
QUDAG_API_AUTH_ENABLED=true
QUDAG_TLS_ENABLED=true

# Network
QUDAG_NETWORK_ID=qudag-testnet
QUDAG_ENVIRONMENT=testnet

# Monitoring
ALERT_WEBHOOK_URL=https://your-webhook.com/alerts
```

## Monitoring

### Access Points

- **Grafana**: http://localhost:3000 (local) or configure Fly.io proxy
- **Prometheus**: http://localhost:9094 (local)
- **Node Health**: https://qudag-testnet-nodeX.fly.dev/health
- **Metrics**: https://qudag-testnet-nodeX.fly.dev/metrics

### Key Metrics to Monitor

1. **Network Health**:
   - `qudag_peer_count`: Number of connected peers
   - `qudag_p2p_latency`: P2P network latency
   - `qudag_network_bandwidth`: Network throughput

2. **Consensus**:
   - `qudag_block_height`: Current block height
   - `qudag_consensus_rounds`: Consensus round metrics
   - `qudag_finality_time`: Time to finality

3. **Quantum Crypto**:
   - `qudag_signature_verification_time`: ML-DSA verification time
   - `qudag_key_generation_time`: Key generation performance
   - `qudag_crypto_operations_total`: Total crypto operations

4. **Dark Domains**:
   - `qudag_dark_domains_registered`: Total registered domains
   - `qudag_domain_resolution_time`: Resolution latency
   - `qudag_fingerprint_collisions`: Fingerprint collision count

## Troubleshooting

### Common Issues

1. **Peer Connection Issues**:
   ```bash
   # Update bootstrap peers
   ./scripts/update-bootstrap-peers.sh
   
   # Check peer connectivity
   flyctl ssh console -a qudag-testnet-node1 -C "qudag-node peers list"
   ```

2. **TLS Certificate Issues**:
   ```bash
   # Regenerate certificates
   ./scripts/setup-tls.sh setup
   
   # Verify certificates
   openssl s_client -connect qudag-testnet-node1.fly.dev:443 -CAfile tls/ca/ca.pem
   ```

3. **Health Check Failures**:
   ```bash
   # Check logs
   flyctl logs -a qudag-testnet-node1
   
   # SSH into node
   flyctl ssh console -a qudag-testnet-node1
   ```

4. **Secret Management Issues**:
   ```bash
   # List secrets
   flyctl secrets list -a qudag-testnet-node1
   
   # Update secrets
   ./scripts/deploy-enhanced.sh update-secrets
   ```

## Security Best Practices

1. **Secrets Management**:
   - Never commit secrets to git
   - Use strong passwords for all services
   - Rotate API tokens regularly
   - Back up private keys securely

2. **Network Security**:
   - Keep CORS origins restricted
   - Use IP whitelisting for production
   - Enable rate limiting
   - Monitor for suspicious activity

3. **TLS Security**:
   - Use TLS 1.3 minimum
   - Rotate certificates before expiry
   - Verify certificate chains
   - Enable mutual TLS for sensitive APIs

## Maintenance

### Regular Tasks

1. **Daily**:
   - Check node health status
   - Monitor consensus participation
   - Review error logs

2. **Weekly**:
   - Update bootstrap peer configurations
   - Check disk usage
   - Review security alerts

3. **Monthly**:
   - Rotate API tokens
   - Update TLS certificates
   - Performance optimization
   - Security audit

### Backup and Recovery

1. **Backup Node Data**:
   ```bash
   ./scripts/backup-nodes.sh
   ```

2. **Restore Node**:
   ```bash
   ./scripts/restore-node.sh node1 backup-file.tar.gz
   ```

## Advanced Configuration

### Custom Node Deployment

To deploy a custom node configuration:

1. Create custom config from template:
   ```bash
   cp configs/node-template-secure.toml configs/node5.toml
   # Edit node5.toml
   ```

2. Create Fly.io configuration:
   ```bash
   cp nodes/fly.node1.toml nodes/fly.node5.toml
   # Edit fly.node5.toml
   ```

3. Deploy:
   ```bash
   flyctl deploy --app qudag-testnet-node5 --config nodes/fly.node5.toml
   ```

### Performance Tuning

1. **Database Optimization**:
   - Adjust RocksDB cache size
   - Enable compression
   - Tune write buffer size

2. **Network Optimization**:
   - Adjust peer limits
   - Configure connection timeouts
   - Enable connection pooling

3. **Resource Allocation**:
   - Scale Fly.io instances
   - Adjust memory limits
   - Configure CPU allocation

## Support

For issues or questions:
1. Check logs: `flyctl logs -a <app-name>`
2. Review this guide
3. Check QuDAG documentation
4. Submit issues to the repository