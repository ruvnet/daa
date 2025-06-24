# QuDAG DevOps & CI/CD Infrastructure

This directory contains the complete CI/CD pipeline and DevOps infrastructure for QuDAG Exchange.

## GitHub Actions Workflows

### 1. CI Pipeline (`ci.yml`)
Main continuous integration workflow that runs on every push and pull request.

**Features:**
- 🔍 Format and lint checks (rustfmt, clippy)
- 🔒 Security audit with cargo-audit
- ✅ Comprehensive test suite across Linux, macOS, and Windows
- 🧪 Specialized quantum cryptography tests
- 🏗️ Multi-platform binary builds (x64, ARM64)
- 📦 WASM builds for web and Node.js
- 📊 Code coverage reporting
- 🐳 Docker image builds
- 🔗 Multi-node integration tests

**Triggers:**
- Push to main, develop, quadag-exchange branches
- Pull requests to main, develop
- Manual workflow dispatch

### 2. Release Automation (`release.yml`)
Automated release workflow triggered by version tags.

**Features:**
- 📝 Automatic changelog generation
- 🏗️ Cross-platform binary builds
- 📦 WASM package creation
- 🐳 Multi-architecture Docker images
- 📚 Publishing to crates.io
- 🎯 GitHub Release creation with artifacts

**Usage:**
```bash
git tag v1.2.3
git push origin v1.2.3
```

### 3. NPM Publishing (`npm-publish.yml`)
Automated WASM package publishing to npm registry.

**Features:**
- 🔄 Version change detection
- 🧪 WASM testing in Node.js and browsers
- 📦 Multi-target builds (web, nodejs, bundler)
- 🚀 Automatic npm publishing
- 🏷️ Prerelease support

**Packages Published:**
- `@qudag/wasm` - For bundlers
- `@qudag/wasm-web` - For browsers
- `@qudag/wasm-node` - For Node.js

### 4. Security Scanning (`security.yml`)
Comprehensive security analysis workflow.

**Features:**
- 🔒 Dependency vulnerability scanning
- 📜 License compliance checks
- 🔍 Static security analysis
- 🕵️ Secret detection
- 🐳 Container vulnerability scanning
- ⚡ Quantum crypto security tests
- 📊 Security report generation

**Schedule:** Runs weekly and on dependency changes

### 5. Performance Benchmarks (`benchmarks.yml`)
Performance tracking and regression detection.

**Features:**
- ⚡ Crypto operation benchmarks
- 🔗 DAG consensus performance
- 🌐 Network throughput tests
- 📦 WASM performance tracking
- 💾 Memory usage profiling
- 📊 Comparative analysis on PRs
- 📈 Historical performance tracking

## Docker Configurations

### Production Images

#### 1. Main Dockerfile (`Dockerfile`)
Multi-stage production build with minimal runtime footprint.
- Based on Debian slim
- Non-root user execution
- Health checks included
- Volume mounts for data persistence

#### 2. Alpine Dockerfile (`Dockerfile.alpine`)
Ultra-minimal Alpine-based image for resource-constrained environments.
- Static binary compilation
- ~50MB final image size
- Tini for proper signal handling

#### 3. Development Dockerfile (`Dockerfile.dev`)
Development environment with debugging tools.
- Hot reloading support
- Debugging tools (gdb, valgrind)
- Performance profiling tools
- Development utilities

### Docker Compose Configurations

#### 1. Production Setup (`docker-compose.yml`)
Full production deployment with:
- 3 QuDAG nodes in cluster
- Prometheus monitoring
- Grafana dashboards
- Redis caching
- PostgreSQL storage
- NGINX load balancer

#### 2. Development Setup (`docker-compose.dev.yml`)
Development environment with:
- Hot-reloading dev container
- Jaeger distributed tracing
- Enhanced monitoring
- Database management tools
- Email testing (MailHog)
- Documentation server

#### 3. Testnet Setup (`docker-compose.testnet.yml`)
Generated dynamically by deployment scripts for flexible testnet configurations.

## Testnet Orchestration Scripts

### 1. Deploy Script (`scripts/testnet/deploy-testnet.sh`)
Complete testnet deployment automation.

**Features:**
- Dynamic node configuration generation
- Docker image building
- Service orchestration
- Health monitoring
- Network setup

**Usage:**
```bash
# Deploy 5-node testnet
NODE_COUNT=5 ./scripts/testnet/deploy-testnet.sh deploy

# Stop testnet
./scripts/testnet/deploy-testnet.sh stop

# Clean up everything
./scripts/testnet/deploy-testnet.sh clean
```

### 2. Monitor Script (`scripts/testnet/monitor-testnet.sh`)
Real-time testnet monitoring dashboard.

**Features:**
- Live node status
- Performance metrics
- Network statistics
- Resource usage
- Recent activity logs

**Usage:**
```bash
./scripts/testnet/monitor-testnet.sh
```

### 3. Load Test Script (`scripts/testnet/load-test.sh`)
Comprehensive load testing suite.

**Test Suites:**
- Transaction throughput testing
- Quantum crypto performance
- Consensus stability
- Dark addressing performance

**Usage:**
```bash
# Run all tests
./scripts/testnet/load-test.sh all

# Run specific test
./scripts/testnet/load-test.sh transactions
```

## Quick Start

### Local Development
```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up

# Run tests
docker-compose -f docker-compose.dev.yml exec dev cargo test

# Access services
# - Dev container: http://localhost:8080
# - Grafana: http://localhost:3001
# - Jaeger: http://localhost:16686
```

### Production Deployment
```bash
# Build and start production cluster
docker-compose up -d

# Scale nodes
docker-compose up -d --scale qudag-node-2=3

# View logs
docker-compose logs -f
```

### Testnet Deployment
```bash
# Deploy 10-node testnet
NODE_COUNT=10 ./scripts/testnet/deploy-testnet.sh

# Monitor testnet
./scripts/testnet/monitor-testnet.sh

# Run load tests
./scripts/testnet/load-test.sh all
```

## CI/CD Best Practices

1. **Branch Protection**
   - Require PR reviews
   - Enforce status checks
   - Require up-to-date branches

2. **Secrets Management**
   - `GITHUB_TOKEN` - Automatically provided
   - `CRATES_IO_TOKEN` - For crates.io publishing
   - `NPM_TOKEN` - For npm publishing
   - `DOCKER_USERNAME/PASSWORD` - Docker Hub credentials

3. **Performance Monitoring**
   - Benchmark results tracked over time
   - Automatic regression detection
   - Performance reports on PRs

4. **Security**
   - Weekly vulnerability scans
   - License compliance checks
   - Container security scanning
   - Quantum crypto security tests

## Monitoring & Observability

### Metrics Collection
- Prometheus scrapes all nodes
- Custom QuDAG metrics exposed
- System resource metrics
- Network performance metrics

### Visualization
- Pre-configured Grafana dashboards
- Real-time performance monitoring
- Historical trend analysis
- Alert configuration

### Distributed Tracing
- Jaeger integration for request tracing
- Performance bottleneck identification
- Cross-service correlation

## Troubleshooting

### Common Issues

1. **Build Failures**
   ```bash
   # Clear Docker cache
   docker system prune -a
   
   # Rebuild without cache
   docker-compose build --no-cache
   ```

2. **Test Failures**
   ```bash
   # Run specific test
   docker-compose exec dev cargo test test_name
   
   # Enable debug logging
   RUST_LOG=debug cargo test
   ```

3. **Performance Issues**
   ```bash
   # Profile CPU usage
   docker stats
   
   # Check resource limits
   docker-compose exec node cat /proc/meminfo
   ```

## Contributing

When adding new DevOps features:

1. Update relevant workflows in `.github/workflows/`
2. Test locally using act: `act -j build`
3. Update this README with new features
4. Ensure all scripts are executable
5. Add appropriate error handling

## Security Considerations

- All containers run as non-root users
- Secrets are never logged or exposed
- Network isolation between services
- Regular security scanning
- Quantum-resistant cryptography throughout

## Performance Optimization

- Multi-stage Docker builds for smaller images
- Build caching for faster CI/CD
- Parallel job execution where possible
- Resource limits to prevent runaway processes
- Optimized WASM builds with release flags

This infrastructure provides a complete DevOps solution for QuDAG Exchange, enabling reliable development, testing, and deployment workflows.