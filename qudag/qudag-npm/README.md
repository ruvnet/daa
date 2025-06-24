# QuDAG NPM Package

> 🌐 Quantum-Resistant Distributed Communication Platform

[![npm version](https://img.shields.io/npm/v/qudag.svg)](https://www.npmjs.com/package/qudag)
[![License](https://img.shields.io/npm/l/qudag.svg)](https://github.com/ruvnet/QuDAG/blob/main/LICENSE)
[![Platform Support](https://img.shields.io/badge/platforms-linux%20%7C%20macos%20%7C%20windows-brightgreen)](https://www.npmjs.com/package/qudag)

QuDAG is a revolutionary quantum-resistant distributed communication platform built on a Directed Acyclic Graph (DAG) architecture. This NPM package provides easy access to the QuDAG CLI through Node.js and NPX.

## 🚀 Quick Start

### Using NPX (Recommended)

No installation required! Just run:

```bash
npx qudag@latest --help
```

### Global Installation

Install globally for direct CLI access:

```bash
npm install -g qudag
qudag --help
```

### Project Installation

Add to your project as a dependency:

```bash
npm install qudag
```

## 📋 Prerequisites

- Node.js >= 16.0.0
- Supported platforms:
  - Linux (x64, arm64)
  - macOS (x64, arm64)
  - Windows (x64)

## 🛠️ CLI Usage

### Start a QuDAG Node

```bash
# Using npx
npx qudag@latest start --port 8000

# If installed globally
qudag start --port 8000
```

### Dark Address Management

```bash
# Register a .dark domain
npx qudag@latest address register mynode.dark

# Resolve a dark address
npx qudag@latest address resolve mynode.dark

# Generate a temporary shadow address
npx qudag@latest address shadow --ttl 3600

# Create a quantum fingerprint
npx qudag@latest address fingerprint --data "Hello QuDAG!"
```

### Peer Management

```bash
# List connected peers
npx qudag@latest peer list

# Add a new peer
npx qudag@latest peer add /ip4/192.168.1.100/tcp/8000

# Remove a peer
npx qudag@latest peer remove 12D3KooWExample...
```

### Node Operations

```bash
# Check node status
npx qudag@latest status

# View network statistics
npx qudag@latest network stats

# Stop the node
npx qudag@latest stop
```

## 💻 Programmatic Usage

### JavaScript/TypeScript API

```javascript
const { QuDAG } = require('qudag');
// or
import { QuDAG } from 'qudag';

// Start a node
const startResult = await QuDAG.start(8000);
console.log('Node started:', startResult.stdout);

// Get node status
const status = await QuDAG.status();
console.log('Status:', status.stdout);

// Register a dark address
const register = await QuDAG.registerAddress('myapp.dark');
console.log('Registered:', register.stdout);

// List peers
const peers = await QuDAG.listPeers();
console.log('Peers:', peers.stdout);

// Execute raw commands
const result = await QuDAG.raw(['--version']);
console.log('Version:', result.stdout);
```

### Advanced Usage

```javascript
const { execute, getPlatformInfo, isInstalled } = require('qudag');

// Check if binary is installed
if (!isInstalled()) {
  console.log('QuDAG binary not installed');
}

// Get platform information
const info = getPlatformInfo();
console.log('Platform:', info.platform);
console.log('Architecture:', info.arch);
console.log('Binary path:', info.binaryPath);

// Execute custom commands with options
const result = await execute(['peer', 'list', '--format', 'json'], {
  env: { ...process.env, RUST_LOG: 'debug' }
});

const peers = JSON.parse(result.stdout);
console.log(`Connected to ${peers.length} peers`);
```

## 🏗️ Architecture

This NPM package acts as a wrapper around the native QuDAG binary, providing:

- **Automatic Binary Management**: Downloads the correct binary for your platform
- **Cross-Platform Support**: Works on Linux, macOS, and Windows
- **TypeScript Support**: Full type definitions included
- **Programmatic API**: Use QuDAG from your Node.js applications
- **NPX Support**: Run without installation using `npx qudag@latest`

## 🔧 Troubleshooting

### Binary Download Issues

If the binary fails to download during installation:

1. Check your internet connection
2. Verify your platform is supported
3. Check if a proxy is blocking GitHub access
4. Try manual installation:

```bash
# The binary will be downloaded on first use
npx qudag@latest --help
```

### Permission Issues on Linux/macOS

If you get permission errors:

```bash
# Make the binary executable
chmod +x $(npm root -g)/qudag/bin/platform/qudag
```

### Platform Not Supported

If your platform is not supported, you can build from source:

```bash
# Clone the repository
git clone https://github.com/ruvnet/QuDAG
cd QuDAG

# Build the project
cargo build --release

# Copy the binary to the npm package
cp target/release/qudag node_modules/qudag/bin/platform/
```

## 🤝 Contributing

Contributions are welcome! Please see the [main QuDAG repository](https://github.com/ruvnet/QuDAG) for contribution guidelines.

## 📄 License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

## 🔗 Links

- [GitHub Repository](https://github.com/ruvnet/QuDAG)
- [Documentation](https://docs.qudag.io)
- [NPM Package](https://www.npmjs.com/package/qudag)
- [Report Issues](https://github.com/ruvnet/QuDAG/issues)

---

Created by [rUv](https://github.com/ruvnet)