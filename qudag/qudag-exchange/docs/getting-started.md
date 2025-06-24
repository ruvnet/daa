# Getting Started with QuDAG Exchange

## Overview

QuDAG Exchange is a decentralized resource exchange protocol that enables secure, quantum-resistant trading of computational resources through rUv (Resource Utilization Voucher) tokens. This guide will help you set up and start using QuDAG Exchange.

## Prerequisites

- Rust 1.75+ (for building from source)
- Node.js 18+ (for WASM usage)
- Git
- 4GB RAM minimum
- 10GB free disk space

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/qudag/qudag-exchange
cd qudag-exchange

# Build all components
cargo build --release

# Run tests to verify installation
cargo test

# Install the CLI tool
cargo install --path qudag-exchange-cli
```

### Using Pre-built Binaries

Download the latest release for your platform:

```bash
# Linux
wget https://github.com/qudag/qudag-exchange/releases/latest/download/qudag-exchange-linux-amd64
chmod +x qudag-exchange-linux-amd64

# macOS
wget https://github.com/qudag/qudag-exchange/releases/latest/download/qudag-exchange-darwin-amd64
chmod +x qudag-exchange-darwin-amd64

# Windows
# Download qudag-exchange-windows-amd64.exe from releases page
```

### WASM Installation

For browser or Node.js usage:

```bash
# Build WASM package
wasm-pack build --target web --out-dir pkg

# Or install from npm
npm install @qudag/exchange-wasm
```

## First Steps

### 1. Create Your First Account

```bash
# Create a new account with a secure vault
qudag-exchange-cli create-account --name alice

# You'll be prompted to create a master password
# This password protects your quantum-resistant keys
```

### 2. Generate Quantum-Resistant Keys

```bash
# Generate ML-DSA signing keys
qudag-exchange-cli key generate --type signing --account alice

# Generate ML-KEM encryption keys  
qudag-exchange-cli key generate --type encryption --account alice
```

### 3. Check Your Balance

```bash
# View your rUv token balance
qudag-exchange-cli balance --account alice

# Initial balance is 0 rUv
# You'll need to either mine, receive transfers, or provide resources
```

### 4. Start Contributing Resources

```bash
# Start a node to contribute resources and earn rUv
qudag-exchange-cli node start \
  --account alice \
  --port 8080 \
  --resources cpu=2,memory=4GB,storage=100GB

# Your node will join the network and start earning rUv
```

## Basic Operations

### Transferring rUv Tokens

```bash
# Transfer 50 rUv to another account
qudag-exchange-cli transfer \
  --from alice \
  --to bob \
  --amount 50 \
  --memo "Payment for compute time"

# Transaction will be submitted to DAG consensus
# Confirmation typically takes 2-5 seconds
```

### Resource Trading

```bash
# Offer compute resources for rUv
qudag-exchange-cli offer create \
  --type compute \
  --specs "cpu=4,memory=8GB" \
  --price "10 rUv/hour" \
  --duration 24h

# Browse available resource offers
qudag-exchange-cli offer list --type compute

# Purchase compute time
qudag-exchange-cli offer accept \
  --offer-id abc123 \
  --duration 2h
```

### Monitoring Your Node

```bash
# Check node status
qudag-exchange-cli node status

# View network statistics
qudag-exchange-cli network stats

# Monitor resource usage
qudag-exchange-cli resources monitor
```

## Using the API

### Starting the API Server

```bash
# Start HTTP API server
qudag-exchange-server \
  --port 3000 \
  --account alice

# API will be available at http://localhost:3000
```

### Basic API Usage

```bash
# Check balance via API
curl http://localhost:3000/api/v1/balance/alice

# Submit a transaction
curl -X POST http://localhost:3000/api/v1/transaction \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "from": "alice",
    "to": "bob", 
    "amount": 25,
    "memo": "API transfer"
  }'
```

## WASM Usage

### Browser Integration

```html
<!DOCTYPE html>
<html>
<head>
  <script type="module">
    import init, { QuDagExchange } from './pkg/qudag_exchange_wasm.js';
    
    async function run() {
      await init();
      
      // Create exchange instance
      const exchange = new QuDagExchange();
      
      // Create account
      const account = await exchange.createAccount('alice', 'password');
      
      // Check balance
      const balance = await exchange.getBalance('alice');
      console.log('Balance:', balance);
    }
    
    run();
  </script>
</head>
</html>
```

### Node.js Integration

```javascript
const { QuDagExchange } = require('@qudag/exchange-wasm');

async function main() {
  // Initialize exchange
  const exchange = new QuDagExchange();
  
  // Create account
  const account = await exchange.createAccount('alice', 'securePassword');
  
  // Transfer tokens
  const txId = await exchange.transfer({
    from: 'alice',
    to: 'bob',
    amount: 50,
    password: 'securePassword'
  });
  
  console.log('Transaction ID:', txId);
}

main().catch(console.error);
```

## Common Use Cases

### 1. Distributed Computing Provider

```bash
# Register as compute provider
qudag-exchange-cli provider register \
  --type compute \
  --specs "gpu=4xA100,cpu=64,memory=256GB"

# Set pricing
qudag-exchange-cli provider set-price \
  --resource gpu \
  --price "100 rUv/hour"

# Start accepting jobs
qudag-exchange-cli provider start
```

### 2. Storage Provider

```bash
# Offer storage space
qudag-exchange-cli provider register \
  --type storage \
  --capacity 10TB \
  --redundancy 3

# Monitor storage usage
qudag-exchange-cli storage status
```

### 3. Resource Consumer

```bash
# Find available GPUs
qudag-exchange-cli market search \
  --resource gpu \
  --min-memory 40GB

# Reserve compute time
qudag-exchange-cli market reserve \
  --provider provider123 \
  --duration 4h \
  --auto-renew
```

## Troubleshooting

### Connection Issues

```bash
# Check network connectivity
qudag-exchange-cli network diagnose

# Manually add bootstrap peers
qudag-exchange-cli peer add \
  --address "/ip4/1.2.3.4/tcp/8080/p2p/QmPeer..."

# Check firewall settings
qudag-exchange-cli network check-nat
```

### Vault/Key Issues

```bash
# Recover vault from backup
qudag-exchange-cli vault restore \
  --backup vault-backup.json \
  --password

# Export keys (handle with care!)
qudag-exchange-cli key export \
  --account alice \
  --output alice-keys.json
```

### Performance Optimization

```bash
# Optimize DAG storage
qudag-exchange-cli maintenance compact

# Adjust resource limits
qudag-exchange-cli config set \
  --max-connections 100 \
  --cache-size 2GB
```

## Next Steps

- Read the [Architecture Overview](architecture.md) to understand system design
- Explore the [API Reference](api-reference.md) for programmatic access
- Learn about [rUv Token Economics](ruv-economics.md)
- Review [Security Best Practices](security.md)
- Join our [Discord](https://discord.gg/qudag) for community support

## Getting Help

- **Documentation**: https://docs.qudag.io
- **GitHub Issues**: https://github.com/qudag/qudag-exchange/issues
- **Discord**: https://discord.gg/qudag
- **Email**: support@qudag.io