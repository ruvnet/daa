# 🔧 DAA Troubleshooting Guide

> **Comprehensive problem-solving guide for DAA agents** - From common issues to advanced debugging techniques.

[![Support](https://img.shields.io/badge/Support-24%2F7-green)](https://discord.gg/daa)
[![Documentation](https://img.shields.io/badge/Docs-Up%20to%20Date-blue)](https://docs.daa.dev)
[![Community](https://img.shields.io/discord/123456789)](https://discord.gg/daa)

---

## 📋 Table of Contents

1. [Quick Diagnostics](#-quick-diagnostics)
2. [Common Issues](#-common-issues)
3. [Installation Problems](#-installation-problems)
4. [Configuration Issues](#-configuration-issues)
5. [Network Problems](#-network-problems)
6. [Performance Issues](#-performance-issues)
7. [AI Integration Problems](#-ai-integration-problems)
8. [Blockchain Connectivity](#-blockchain-connectivity)
9. [Advanced Debugging](#-advanced-debugging)
10. [Getting Help](#-getting-help)

---

## 🩺 Quick Diagnostics

### Health Check Script

Run this script to quickly identify common issues:

```bash
#!/bin/bash
# daa-health-check.sh

echo "🔍 DAA Health Check Starting..."
echo "================================"

# Check system requirements
echo "📋 System Requirements:"
echo "CPU Cores: $(nproc)"
echo "RAM: $(free -h | awk '/^Mem:/ {print $2}')"
echo "Disk Space: $(df -h / | awk 'NR==2 {print $4}')"
echo ""

# Check DAA installation
if command -v daa-cli &> /dev/null; then
    echo "✅ DAA CLI installed: $(daa-cli --version)"
else
    echo "❌ DAA CLI not found"
fi

# Check Rust installation
if command -v rustc &> /dev/null; then
    echo "✅ Rust installed: $(rustc --version)"
else
    echo "❌ Rust not found"
fi

# Check Docker
if command -v docker &> /dev/null; then
    echo "✅ Docker installed: $(docker --version)"
    if docker ps &> /dev/null; then
        echo "✅ Docker daemon running"
    else
        echo "❌ Docker daemon not running"
    fi
else
    echo "❌ Docker not found"
fi

# Check network connectivity
echo ""
echo "🌐 Network Connectivity:"
if ping -c 1 google.com &> /dev/null; then
    echo "✅ Internet connectivity"
else
    echo "❌ No internet connectivity"
fi

# Check DAA service status
if systemctl is-active --quiet daa-agent; then
    echo "✅ DAA agent service running"
else
    echo "❌ DAA agent service not running"
fi

# Check ports
echo ""
echo "🔌 Port Status:"
for port in 8080 9090 5432 6379; do
    if netstat -tuln | grep -q ":$port "; then
        echo "✅ Port $port is open"
    else
        echo "⚠️  Port $port is not listening"
    fi
done

echo ""
echo "📊 Process Information:"
ps aux | grep -E "(daa|postgres|redis)" | grep -v grep

echo ""
echo "💾 Disk Usage:"
du -sh ~/.daa/ 2>/dev/null || echo "DAA data directory not found"

echo ""
echo "🏁 Health check completed!"
```

### Quick Status Check

```bash
# Check agent status
daa-cli status

# Check logs for errors
daa-cli logs --level error --tail 50

# Check network connectivity
daa-cli network diagnose

# Check economic status
daa-cli economy stats
```

---

## 🚨 Common Issues

### Issue 1: Agent Won't Start

**Symptoms:**
- Agent process exits immediately
- "Configuration error" messages
- Port binding failures

**Diagnostic Steps:**

```bash
# Check configuration
daa-cli config validate

# Check for port conflicts
sudo netstat -tulpn | grep :8080
sudo netstat -tulpn | grep :9090

# Check permissions
ls -la ~/.daa/
```

**Solutions:**

```bash
# Fix configuration
daa-cli config init --reset

# Use different ports
daa-cli config set network.http_port 8081
daa-cli config set network.p2p_port 9091

# Fix permissions
sudo chown -R $USER:$USER ~/.daa/
chmod 755 ~/.daa/
```

### Issue 2: Database Connection Failed

**Symptoms:**
- "Connection refused" errors
- "Authentication failed" messages
- Database timeout errors

**Diagnostic Steps:**

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test connection
psql -h localhost -U daa -d daa -c "SELECT 1;"

# Check database logs
sudo journalctl -u postgresql -n 50
```

**Solutions:**

```bash
# Start PostgreSQL
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Reset database password
sudo -u postgres psql -c "ALTER USER daa PASSWORD 'newpassword';"

# Update DAA configuration
daa-cli config set database.password "newpassword"

# Recreate database
dropdb -U postgres daa
createdb -U postgres -O daa daa
daa-cli database migrate
```

### Issue 3: P2P Network Issues

**Symptoms:**
- No peers connected
- "Bootstrap failed" messages
- Network timeout errors

**Diagnostic Steps:**

```bash
# Check peer status
daa-cli network peers

# Test bootstrap nodes
telnet bootstrap1.daa.network 9090

# Check firewall
sudo ufw status
```

**Solutions:**

```bash
# Configure firewall
sudo ufw allow 9090/tcp
sudo ufw allow 8080/tcp

# Update bootstrap peers
daa-cli config set network.bootstrap_peers "node1.daa.network:9090,node2.daa.network:9090"

# Check NAT/router configuration
# Ensure ports 8080 and 9090 are forwarded

# Reset network state
daa-cli network reset
```

### Issue 4: High Memory Usage

**Symptoms:**
- System becoming slow
- Out of memory errors
- Agent crashes

**Diagnostic Steps:**

```bash
# Check memory usage
free -h
ps aux --sort=-%mem | head -10

# Check DAA memory usage
daa-cli metrics memory

# Check for memory leaks
valgrind --tool=memcheck --leak-check=full daa-agent
```

**Solutions:**

```bash
# Increase system memory (if possible)
# Configure memory limits
daa-cli config set orchestrator.memory_limit "2GB"

# Enable garbage collection
daa-cli config set runtime.gc_enabled true

# Reduce concurrent workflows
daa-cli config set orchestrator.max_concurrent_workflows 5

# Clear caches
daa-cli cache clear
```

---

## 📦 Installation Problems

### Rust Installation Issues

**Problem:** Rust toolchain not installing or updating

```bash
# Remove existing installation
rustup self uninstall

# Fresh installation
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update PATH
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Cargo Build Failures

**Problem:** DAA compilation fails

```bash
# Update Rust toolchain
rustup update

# Clear cargo cache
cargo clean

# Install required system dependencies
# Ubuntu/Debian:
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev libpq-dev

# CentOS/RHEL:
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel postgresql-devel

# macOS:
xcode-select --install
brew install postgresql openssl
```

### Docker Issues

**Problem:** Docker containers won't start

```bash
# Check Docker daemon
sudo systemctl status docker

# Start Docker
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Pull latest images
docker-compose pull

# Reset containers
docker-compose down -v
docker-compose up -d
```

---

## ⚙️ Configuration Issues

### Invalid Configuration Format

**Problem:** Configuration file syntax errors

```bash
# Validate configuration
daa-cli config validate

# Show configuration schema
daa-cli config schema

# Reset to defaults
daa-cli config init --reset

# Common fixes:
# - Check TOML syntax
# - Verify string quoting
# - Check indentation
# - Validate data types
```

### Missing Environment Variables

**Problem:** Required environment variables not set

```bash
# Check required variables
daa-cli config check-env

# Set missing variables
export ANTHROPIC_API_KEY="your-api-key"
export ETHEREUM_RPC_URL="https://mainnet.infura.io/v3/your-key"

# Make permanent
echo 'export ANTHROPIC_API_KEY="your-api-key"' >> ~/.bashrc
source ~/.bashrc

# Use .env file
cat > .env << EOF
ANTHROPIC_API_KEY=your-api-key
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/your-key
EOF
```

### Permission Errors

**Problem:** Access denied to configuration files

```bash
# Fix file permissions
chmod 600 ~/.daa/config.toml
chmod 700 ~/.daa/

# Fix ownership
sudo chown -R $USER:$USER ~/.daa/

# Check SELinux (if applicable)
sudo setsebool -P httpd_can_network_connect 1
```

---

## 🌐 Network Problems

### Firewall Configuration

**Problem:** Network connections blocked by firewall

```bash
# Ubuntu/Debian (UFW)
sudo ufw allow 8080/tcp comment 'DAA HTTP API'
sudo ufw allow 9090/tcp comment 'DAA P2P Network'
sudo ufw reload

# CentOS/RHEL (firewalld)
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --permanent --add-port=9090/tcp
sudo firewall-cmd --reload

# Check current rules
sudo ufw status verbose
# or
sudo firewall-cmd --list-all
```

### DNS Resolution Issues

**Problem:** Cannot resolve peer addresses

```bash
# Test DNS resolution
nslookup bootstrap1.daa.network
dig +short bootstrap1.daa.network

# Use alternative DNS
echo "nameserver 8.8.8.8" | sudo tee -a /etc/resolv.conf

# Configure local DNS cache
sudo systemd-resolve --set-dns=8.8.8.8 --interface=eth0
```

### NAT/Router Configuration

**Problem:** Agents behind NAT cannot connect

```bash
# Enable UPnP (if available)
daa-cli config set network.upnp_enabled true

# Manual port forwarding
# Forward external port 9090 to internal IP:9090

# Use STUN servers
daa-cli config set network.stun_servers "stun.l.google.com:19302,stun1.l.google.com:19302"

# Enable hole punching
daa-cli config set network.hole_punching true
```

---

## ⚡ Performance Issues

### Slow Response Times

**Problem:** API requests taking too long

**Diagnostics:**

```bash
# Check response times
curl -w "Total time: %{time_total}s\n" -o /dev/null -s http://localhost:8080/health

# Monitor system resources
htop
iotop -a

# Check database performance
daa-cli database analyze

# Profile the application
daa-cli profile start
# ... perform operations ...
daa-cli profile stop --output profile.txt
```

**Solutions:**

```bash
# Increase worker threads
daa-cli config set runtime.worker_threads 8

# Enable connection pooling
daa-cli config set database.pool_size 20

# Add Redis cache
daa-cli config set cache.enabled true
daa-cli config set cache.redis_url "redis://localhost:6379"

# Optimize database
psql -U daa -d daa -c "VACUUM ANALYZE;"
psql -U daa -d daa -c "REINDEX DATABASE daa;"
```

### High CPU Usage

**Problem:** Agent consuming excessive CPU

**Diagnostics:**

```bash
# Identify CPU-intensive processes
top -H -p $(pgrep daa-agent)

# Check for infinite loops
strace -p $(pgrep daa-agent) -c

# Profile CPU usage
perf record -g -p $(pgrep daa-agent)
perf report
```

**Solutions:**

```bash
# Reduce processing frequency
daa-cli config set orchestrator.autonomy_interval "120s"

# Limit concurrent operations
daa-cli config set orchestrator.max_concurrent_workflows 3

# Enable CPU limiting (cgroups)
sudo systemctl edit daa-agent
# Add:
[Service]
CPUQuota=200%
```

### Memory Leaks

**Problem:** Memory usage continuously increasing

**Diagnostics:**

```bash
# Monitor memory over time
while true; do
    ps -p $(pgrep daa-agent) -o pid,vsz,rss,pcpu,pmem,time,cmd
    sleep 60
done > memory_usage.log

# Use memory profiler
valgrind --tool=massif daa-agent
ms_print massif.out.* > memory_profile.txt
```

**Solutions:**

```bash
# Enable periodic garbage collection
daa-cli config set runtime.gc_interval "300s"

# Reduce cache sizes
daa-cli config set cache.max_size "500MB"

# Implement memory limits
ulimit -v 2097152  # 2GB virtual memory limit

# Restart agent periodically (as last resort)
sudo systemctl edit daa-agent
# Add:
[Service]
RuntimeMaxSec=86400  # 24 hours
```

---

## 🤖 AI Integration Problems

### Claude API Issues

**Problem:** AI requests failing or timing out

**Diagnostics:**

```bash
# Test API key
curl -H "Authorization: Bearer $ANTHROPIC_API_KEY" \
     -H "Content-Type: application/json" \
     https://api.anthropic.com/v1/messages

# Check quota
daa-cli ai quota

# Monitor API usage
daa-cli ai usage --last-24h
```

**Solutions:**

```bash
# Update API key
daa-cli config set ai.api_key "new-api-key"

# Implement retry logic
daa-cli config set ai.retry_attempts 3
daa-cli config set ai.retry_delay "5s"

# Use different model
daa-cli config set ai.model "claude-3-haiku-20240307"

# Add request caching
daa-cli config set ai.cache_responses true
```

### MCP Connection Issues

**Problem:** MCP server connectivity problems

**Diagnostics:**

```bash
# Test MCP server
daa-cli mcp test --server localhost:3000

# Check MCP logs
daa-cli mcp logs --server localhost:3000

# List available tools
daa-cli mcp tools --server localhost:3000
```

**Solutions:**

```bash
# Restart MCP server
daa-cli mcp restart --server localhost:3000

# Update MCP configuration
daa-cli config set mcp.servers "localhost:3000,localhost:3001"

# Check MCP server health
curl http://localhost:3000/health
```

---

## ⛓️ Blockchain Connectivity

### Ethereum Connection Issues

**Problem:** Cannot connect to Ethereum network

**Diagnostics:**

```bash
# Test RPC endpoint
curl -X POST -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
     $ETHEREUM_RPC_URL

# Check network ID
daa-cli chain ethereum network-id

# Verify account balance
daa-cli chain ethereum balance --address 0x...
```

**Solutions:**

```bash
# Try different RPC provider
daa-cli config set chains.ethereum.rpc_url "https://eth-mainnet.alchemyapi.io/v2/your-key"

# Configure multiple endpoints
daa-cli config set chains.ethereum.rpc_urls "https://mainnet.infura.io/v3/key1,https://eth-mainnet.alchemyapi.io/v2/key2"

# Adjust timeout settings
daa-cli config set chains.ethereum.timeout "30s"
```

### Transaction Failures

**Problem:** Blockchain transactions failing

**Diagnostics:**

```bash
# Check gas price
daa-cli chain ethereum gas-price

# Check nonce
daa-cli chain ethereum nonce --address 0x...

# Analyze failed transaction
daa-cli chain ethereum tx-receipt --hash 0x...
```

**Solutions:**

```bash
# Increase gas limit
daa-cli config set chains.ethereum.gas_limit 500000

# Use dynamic gas pricing
daa-cli config set chains.ethereum.gas_price_strategy "dynamic"

# Add transaction retry logic
daa-cli config set chains.ethereum.retry_failed_tx true
```

---

## 🔬 Advanced Debugging

### Logging Configuration

**Enable comprehensive logging:**

```toml
# config.toml
[logging]
level = "debug"
format = "json"
output = "/var/log/daa/agent.log"

# Module-specific logging
[logging.modules]
"daa_orchestrator" = "debug"
"daa_rules" = "trace"
"daa_economy" = "info"
"daa_ai" = "debug"
"daa_chain" = "warn"
```

**Structured logging analysis:**

```bash
# Filter by component
jq 'select(.module == "daa_orchestrator")' /var/log/daa/agent.log

# Find error patterns
jq 'select(.level == "ERROR")' /var/log/daa/agent.log | jq -r '.message'

# Analyze timing
jq 'select(.event == "workflow_completed") | .duration' /var/log/daa/agent.log
```

### Performance Profiling

**CPU Profiling:**

```bash
# Install profiling tools
sudo apt-get install linux-perf

# Record CPU profile
perf record -g -p $(pgrep daa-agent) -- sleep 60

# Generate report
perf report --stdio > cpu_profile.txt

# Flame graph
git clone https://github.com/brendangregg/FlameGraph
perf script | ./FlameGraph/stackcollapse-perf.pl | ./FlameGraph/flamegraph.pl > flame.svg
```

**Memory Profiling:**

```bash
# Use jemalloc profiling
export MALLOC_CONF="prof:true,prof_prefix:jeprof"
daa-agent

# Generate heap profile
jeprof --pdf daa-agent jeprof.*.heap > heap_profile.pdf
```

### Network Analysis

**Packet Capture:**

```bash
# Capture DAA network traffic
sudo tcpdump -i any -w daa_traffic.pcap port 9090

# Analyze with Wireshark
wireshark daa_traffic.pcap

# Command-line analysis
tcpdump -r daa_traffic.pcap -A | grep -i "daa"
```

**Connection Tracking:**

```bash
# Monitor active connections
ss -tuln | grep -E "(8080|9090)"

# Track connection states
netstat -an | grep -E "(8080|9090)"

# Monitor connection changes
watch -n 1 'ss -tuln | grep -E "(8080|9090)"'
```

### Database Debugging

**Query Analysis:**

```sql
-- Enable query logging
ALTER SYSTEM SET log_statement = 'all';
SELECT pg_reload_conf();

-- Find slow queries
SELECT query, mean_time, calls, total_time
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 10;

-- Check locks
SELECT * FROM pg_locks WHERE NOT granted;

-- Analyze table statistics
SELECT schemaname, tablename, n_tup_ins, n_tup_upd, n_tup_del
FROM pg_stat_user_tables;
```

**Connection Pool Monitoring:**

```bash
# Check active connections
psql -U daa -d daa -c "SELECT count(*) FROM pg_stat_activity;"

# Monitor connection pool
daa-cli database pool-stats

# Reset connections
daa-cli database reset-connections
```

---

## 🆘 Getting Help

### Community Support

**Discord Server:**
- Join: [https://discord.gg/daa](https://discord.gg/daa)
- Channels: #troubleshooting, #technical-help
- Office hours: Monday-Friday 9AM-5PM UTC

**GitHub Issues:**
- Bug reports: [https://github.com/ruvnet/daa/issues](https://github.com/ruvnet/daa/issues)
- Feature requests: [https://github.com/ruvnet/daa/discussions](https://github.com/ruvnet/daa/discussions)

### Information to Include

When asking for help, include:

```bash
# Generate support bundle
daa-cli support bundle --output daa-support.tar.gz

# Manual information collection
echo "=== System Information ===" > support-info.txt
uname -a >> support-info.txt
echo "" >> support-info.txt

echo "=== DAA Version ===" >> support-info.txt
daa-cli --version >> support-info.txt
echo "" >> support-info.txt

echo "=== Configuration ===" >> support-info.txt
daa-cli config show --redacted >> support-info.txt
echo "" >> support-info.txt

echo "=== Recent Logs ===" >> support-info.txt
daa-cli logs --tail 100 >> support-info.txt
echo "" >> support-info.txt

echo "=== System Status ===" >> support-info.txt
daa-cli status >> support-info.txt
```

### Professional Support

**Enterprise Support:**
- Email: enterprise@daa.dev
- Response time: 4 hours (business days)
- Includes: Priority bug fixes, custom deployment assistance

**Consulting Services:**
- Architecture review
- Performance optimization
- Custom integration development
- Training and workshops

### Documentation

**Official Documentation:**
- Website: [https://docs.daa.dev](https://docs.daa.dev)
- API Docs: [https://docs.rs/daa-orchestrator](https://docs.rs/daa-orchestrator)
- Examples: [https://github.com/ruvnet/daa/tree/main/examples](https://github.com/ruvnet/daa/tree/main/examples)

**Community Resources:**
- Blog: [https://blog.daa.dev](https://blog.daa.dev)
- YouTube: [https://youtube.com/@DAAProtocol](https://youtube.com/@DAAProtocol)
- Twitter: [@DAAProtocol](https://twitter.com/DAAProtocol)

---

## 📊 Known Issues & Workarounds

### Current Known Issues

| Issue | Affected Versions | Workaround | Fix ETA |
|-------|------------------|------------|---------|
| Memory leak in AI module | v0.2.0 | Restart agent daily | v0.2.1 |
| Slow startup on macOS | All | Use `--fast-start` flag | v0.3.0 |
| P2P connection drops | v0.1.x | Enable keep-alive | Fixed in v0.2.0 |

### Compatibility Matrix

| Component | Version | Status | Notes |
|-----------|---------|--------|-------|
| Rust | 1.70+ | ✅ Supported | Required |
| PostgreSQL | 12+ | ✅ Supported | 15+ recommended |
| Redis | 6+ | ✅ Supported | Optional but recommended |
| Docker | 20+ | ✅ Supported | For containerized deployment |
| Kubernetes | 1.20+ | ✅ Supported | For production deployment |

---

## 🔄 Regular Maintenance

### Daily Tasks

```bash
#!/bin/bash
# daily-maintenance.sh

# Check system health
daa-cli health-check

# Rotate logs
logrotate /etc/logrotate.d/daa

# Clean temporary files
find /tmp -name "daa-*" -mtime +1 -delete

# Update metrics
daa-cli metrics collect
```

### Weekly Tasks

```bash
#!/bin/bash
# weekly-maintenance.sh

# Database maintenance
psql -U daa -d daa -c "VACUUM ANALYZE;"

# Clear old cache entries
daa-cli cache cleanup --older-than 7d

# Check for updates
daa-cli update check

# Backup configuration
cp ~/.daa/config.toml ~/.daa/config.toml.backup.$(date +%Y%m%d)
```

### Monthly Tasks

```bash
#!/bin/bash
# monthly-maintenance.sh

# Full database backup
pg_dump -U daa daa | gzip > daa-backup-$(date +%Y%m%d).sql.gz

# Security updates
daa-cli security update

# Performance analysis
daa-cli analyze performance --period 30d

# Clean old logs
find /var/log/daa -name "*.log" -mtime +30 -delete
```

---

*This troubleshooting guide is continuously updated with new solutions and community feedback. If you encounter an issue not covered here, please report it on our [GitHub issues](https://github.com/ruvnet/daa/issues) page.*