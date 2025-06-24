# QuDAG Testnet Deployment Scripts

This directory contains comprehensive test and deployment scripts for the QuDAG testnet.

## Scripts Overview

### 1. `test-local.sh` - Local Testing Script
Tests a 4-node testnet locally using Docker Compose.

**Features:**
- Builds Docker image locally
- Runs 4-node testnet with docker-compose
- Tests health endpoints on all nodes
- Verifies P2P connectivity between nodes
- Checks metrics endpoints
- Tests Prometheus and Grafana integration
- Provides detailed pass/fail reporting
- Option to keep testnet running for manual inspection

**Usage:**
```bash
./test-local.sh
```

### 2. `deploy-fixed.sh` - Fly.io Deployment Script
Deploys nodes to Fly.io with health verification and rollback support.

**Features:**
- Sequential deployment with health checks
- Automatic rollback on failure
- P2P connectivity verification
- DNS configuration instructions
- Deployment summary with URLs
- Support for multiple regions (Toronto, Amsterdam, Singapore, San Francisco)

**Usage:**
```bash
./deploy-fixed.sh
```

### 3. `verify-deployment.sh` - Deployment Verification Script
Comprehensive verification of deployed nodes.

**Features:**
- Health endpoint testing
- Metrics endpoint verification
- P2P connectivity matrix
- Exchange server endpoint checks
- Consensus synchronization verification
- TLS/SSL certificate validation
- Performance metrics
- Detailed report generation

**Usage:**
```bash
./verify-deployment.sh
```

### 4. `update-fly-configs.sh` - Configuration Update Script
Updates all fly.node*.toml files with consistent settings.

**Features:**
- Backs up existing configurations
- Ensures consistent health check settings
- Sets proper environment variables
- Configures unique volume names
- Implements staggered startup delays

**Usage:**
```bash
./update-fly-configs.sh
```

## Deployment Workflow

### Local Testing
1. First test locally to ensure everything works:
   ```bash
   ./test-local.sh
   ```

2. The script will:
   - Build the Docker image
   - Start a 4-node testnet
   - Run comprehensive tests
   - Report success/failure
   - Optionally keep running for inspection

### Production Deployment to Fly.io
1. Ensure you're logged in to Fly.io:
   ```bash
   fly auth login
   ```

2. Update configurations if needed:
   ```bash
   ./update-fly-configs.sh
   ```

3. Deploy to Fly.io:
   ```bash
   ./deploy-fixed.sh
   ```

4. Verify the deployment:
   ```bash
   ./verify-deployment.sh
   ```

## Node Configuration

The testnet consists of 4 nodes:

1. **Node 1 (Toronto)** - Bootstrap node
   - Region: yyz (Toronto)
   - Role: Bootstrap/Registry Authority
   - Resources: 2 CPUs, 4GB RAM

2. **Node 2 (Amsterdam)** - Validator node
   - Region: ams (Amsterdam)
   - Role: Validator
   - Resources: 1 CPU, 2GB RAM

3. **Node 3 (Singapore)** - Validator node
   - Region: sin (Singapore)
   - Role: Validator
   - Resources: 1 CPU, 2GB RAM

4. **Node 4 (San Francisco)** - Validator node
   - Region: sjc (San Jose)
   - Role: Validator
   - Resources: 1 CPU, 2GB RAM

## Endpoints

After deployment, the following endpoints are available:

### Health Endpoints
- Node 1: `https://qudag-testnet-node1.fly.dev/health`
- Node 2: `https://qudag-testnet-node2.fly.dev/health`
- Node 3: `https://qudag-testnet-node3.fly.dev/health`
- Node 4: `https://qudag-testnet-node4.fly.dev/health`

### Metrics Endpoints
- Node 1: `https://qudag-testnet-node1.fly.dev:9090/metrics`
- Node 2: `https://qudag-testnet-node2.fly.dev:9090/metrics`
- Node 3: `https://qudag-testnet-node3.fly.dev:9090/metrics`
- Node 4: `https://qudag-testnet-node4.fly.dev:9090/metrics`

## Troubleshooting

### Local Testing Issues
- If Docker build fails, check the Dockerfile and ensure all dependencies are available
- If health checks fail, increase the grace period in docker-compose.yml
- Check container logs: `docker-compose logs <service-name>`

### Deployment Issues
- If deployment fails, check Fly.io logs: `fly logs --app qudag-testnet-node<N>`
- For P2P connectivity issues, verify firewall rules allow port 4001
- For health check failures, increase grace_period in fly.toml files

### Verification Failures
- Review the generated deployment report for specific failures
- Check individual node logs for errors
- Ensure all nodes have the correct bootstrap peer addresses
- Verify DNS resolution for node domains

## Monitoring

### Local Monitoring
- Prometheus: `http://localhost:9094`
- Grafana: `http://localhost:3000` (admin/admin)

### Production Monitoring
- Fly.io Dashboard: `https://fly.io/apps/qudag-testnet-node<N>`
- Use `fly status --app qudag-testnet-node<N>` for CLI monitoring

## Security Considerations

- All nodes use TLS/SSL provided by Fly.io proxy
- P2P communications on port 4001 (TCP/UDP)
- Health checks are performed over HTTPS in production
- Metrics endpoints should be secured in production environments

## Rollback Procedure

If deployment issues occur:

1. The deploy script automatically attempts rollback on failure
2. Manual rollback: `fly deploy --app qudag-testnet-node<N> --image-label <previous-version>`
3. Check previous versions: `fly releases --app qudag-testnet-node<N>`

## DNS Configuration

After successful deployment:

1. Add CNAME records for custom domain:
   ```
   qudag-testnet.yourdomain.com -> qudag-testnet-node1.fly.dev
   ```

2. Configure SSL certificates in Fly.io dashboard

3. Update any load balancer configurations as needed