# QuDAG Exchange

## Quick Start Guide

QuDAG Exchange is a modular, quantum-secure, multi-agent resource exchange protocol that enables trustless computational resource trading through rUv (Resource Utilization Voucher) tokens.

### Installation

```bash
# Clone the repository
git clone https://github.com/qudag/qudag-exchange
cd qudag-exchange

# Build the project
cargo build --release

# Install CLI globally
cargo install --path qudag-exchange-cli

# For WASM support
wasm-pack build --target web --out-dir pkg
```

### Basic Usage

```bash
# Create a new account
qudag-exchange-cli create-account --name alice

# Check balance
qudag-exchange-cli balance --account alice

# Transfer rUv tokens
qudag-exchange-cli transfer --from alice --to bob --amount 100

# Start a node
qudag-exchange-cli node start --port 8080

# Query network status
qudag-exchange-cli network status
```

### Core Components

- **rUv Tokens**: Resource Utilization Vouchers for computational resource trading
- **Quantum-Resistant Security**: ML-DSA signatures and ML-KEM encryption
- **DAG Consensus**: QR-Avalanche protocol for scalable transaction finality
- **Vault Integration**: Secure key management with QuDAG Vault
- **WASM Support**: Run in browsers and sandboxed environments

### Documentation

- [Getting Started Guide](docs/getting-started.md)
- [Architecture Overview](docs/architecture.md)
- [API Reference](docs/api-reference.md)
- [CLI Command Reference](docs/cli-reference.md)
- [rUv Token Economics](docs/ruv-economics.md)
- [Security Considerations](docs/security.md)

### Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

### License

Apache 2.0 - See [LICENSE](LICENSE) for details.