# QuDAG Exchange Interface Components

This document describes the three interface components for the QuDAG Exchange system: CLI, API, and WASM.

## Overview

The QuDAG Exchange provides multiple interfaces for interacting with the quantum-secure resource exchange protocol:

- **CLI (`qudag-exchange-cli`)**: Command-line interface for terminal users
- **API (`qudag-exchange-api`)**: HTTP REST API for programmatic access
- **WASM (`qudag-exchange-wasm`)**: WebAssembly bindings for browser/Node.js

All interfaces connect to the core exchange logic and provide access to rUv (Resource Utilization Voucher) operations.

## CLI Interface

### Installation

```bash
cd crates/cli
cargo install --path .
```

### Usage

```bash
# Initialize configuration
qudag-exchange config init

# Create a new account
qudag-exchange create-account --name alice

# Check balance
qudag-exchange balance --account alice

# Transfer rUv tokens
qudag-exchange transfer --to bob --amount 100

# Check resource usage
qudag-exchange resource-status --detailed

# View consensus information
qudag-exchange consensus-info --peers
```

### Configuration

The CLI stores configuration in:
- Linux/macOS: `~/.config/qudag/exchange/config.toml`
- Windows: `%APPDATA%\qudag\exchange\config.toml`

Example configuration:
```toml
default_account = "alice"
node_endpoint = "http://localhost:8585"
vault_path = "~/.local/share/qudag/exchange/vault"

[network]
bootstrap_peers = ["peer1.qudag.net:8585", "peer2.qudag.net:8585"]
timeout = 30
```

## API Interface

### Running the Server

```bash
cd crates/api
cargo run --release
```

The server will start on `http://localhost:8585` by default.

### API Endpoints

#### Account Management
- `POST /api/v1/accounts` - Create new account
- `GET /api/v1/accounts/:id/balance` - Get account balance

#### Transactions
- `POST /api/v1/transactions` - Submit transaction
- `GET /api/v1/transactions/:id` - Get transaction status

#### Resources
- `GET /api/v1/resources/status` - Get resource usage
- `GET /api/v1/resources/costs` - Get operation costs

#### Consensus
- `GET /api/v1/consensus/info` - Get consensus information
- `GET /api/v1/consensus/peers` - List connected peers

#### Health
- `GET /api/v1/health` - Health check

### Example API Usage

```bash
# Create account
curl -X POST http://localhost:8585/api/v1/accounts \
  -H "Content-Type: application/json" \
  -d '{"name": "alice", "initial_balance": 1000}'

# Check balance
curl http://localhost:8585/api/v1/accounts/alice/balance

# Submit transaction
curl -X POST http://localhost:8585/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "from": "alice",
    "to": "bob",
    "amount": 100,
    "signature": "...",
    "memo": "Payment for services"
  }'
```

### Authentication

The API uses JWT tokens for authentication. Include the token in the Authorization header:

```bash
curl http://localhost:8585/api/v1/accounts/alice/balance \
  -H "Authorization: Bearer <your-jwt-token>"
```

## WASM Interface

### Building

```bash
cd crates/wasm
./build.sh
```

This creates three packages:
- `pkg-web/` - For web browsers
- `pkg-node/` - For Node.js
- `pkg-bundler/` - For webpack/bundlers

### Browser Usage

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, { QuDAGExchange } from './pkg-web/qudag_exchange_wasm.js';
        
        async function run() {
            // Initialize WASM module
            await init();
            
            // Create exchange instance
            const exchange = new QuDAGExchange();
            
            // Create account
            const account = await exchange.create_account("alice");
            console.log("Account created:", account.id);
            
            // Check balance
            const balance = await exchange.get_balance("alice");
            console.log("Balance:", balance, "rUv");
            
            // Transfer tokens
            const tx = await exchange.transfer("alice", "bob", 100);
            console.log("Transaction:", tx.id);
        }
        
        run();
    </script>
</head>
</html>
```

### Node.js Usage

```javascript
const { QuDAGExchange } = require('./pkg-node/qudag_exchange_wasm.js');

async function main() {
    // Create exchange instance
    const exchange = new QuDAGExchange();
    
    // Create account
    const account = await exchange.create_account("alice");
    console.log("Account created:", account.id);
    
    // Check balance
    const balance = await exchange.get_balance("alice");
    console.log("Balance:", balance, "rUv");
}

main().catch(console.error);
```

### Webpack/Bundler Usage

```javascript
import init, { QuDAGExchange } from 'qudag-exchange-wasm';

async function initExchange() {
    await init();
    return new QuDAGExchange();
}

export { initExchange };
```

## Resource Costs

All operations consume rUv tokens:

| Operation | Cost (rUv) | Description |
|-----------|------------|-------------|
| Create Account | 10 | One-time account creation |
| Transfer | 1 | Per transaction fee |
| Store Data | 5/KB | Per kilobyte stored |
| Compute | 2/ms | Per millisecond of computation |

## Development

### Running Tests

```bash
# Test all interfaces
cargo test --workspace

# Test specific interface
cargo test -p qudag-exchange-cli
cargo test -p qudag-exchange-api
wasm-pack test --headless --chrome
```

### Building Documentation

```bash
cargo doc --workspace --no-deps --open
```

## Security Considerations

1. **Quantum-Resistant**: All cryptographic operations use post-quantum algorithms (ML-DSA, ML-KEM)
2. **Secure Storage**: Private keys are stored in QuDAG Vault with quantum-resistant encryption
3. **API Authentication**: JWT tokens with proper expiration
4. **WASM Isolation**: Runs in sandboxed environment with limited capabilities
5. **Input Validation**: All interfaces validate and sanitize inputs

## Integration with Core

All interfaces are thin wrappers around `qudag-exchange-core`. The core implementation (by Core Implementation Agent) handles:

- rUv ledger management
- Resource metering
- Consensus integration
- Vault security
- Transaction processing

The interfaces focus solely on user interaction and data presentation.