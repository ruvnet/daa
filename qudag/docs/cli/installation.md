# Installation Guide

## Prerequisites

- Rust toolchain (1.75.0 or later)
- Cargo package manager
- OpenSSL development libraries
- CMake (3.15 or later)

## Installation Methods

### 1. From Source

```bash
# Clone the repository
git clone https://github.com/qudag/qudag
cd qudag

# Build and install the CLI
cargo install --path tools/cli
```

### 2. From Crates.io

```bash
cargo install qudag-cli
```

### 3. Using Pre-built Binaries

Download the appropriate binary for your platform from the [releases page](https://github.com/qudag/qudag/releases).

#### Linux
```bash
curl -LO https://github.com/qudag/qudag/releases/latest/download/qudag-linux-x86_64.tar.gz
tar xzf qudag-linux-x86_64.tar.gz
sudo mv qudag /usr/local/bin/
```

#### macOS
```bash
curl -LO https://github.com/qudag/qudag/releases/latest/download/qudag-macos-x86_64.tar.gz
tar xzf qudag-macos-x86_64.tar.gz
sudo mv qudag /usr/local/bin/
```

#### Windows
Download the Windows installer from the releases page and follow the installation wizard.

## Verifying Installation

After installation, verify the CLI is working correctly:

```bash
qudag --version
qudag node status
```

## System Requirements

- **CPU**: 2+ cores recommended
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 1GB minimum for binary and basic operation
- **Network**: Stable internet connection required for node operation

## Updating

To update an existing installation:

```bash
# If installed from source
git pull
cargo install --path tools/cli --force

# If installed from crates.io
cargo install qudag-cli --force
```

## Troubleshooting Installation

If you encounter issues during installation:

1. Ensure all prerequisites are properly installed
2. Check system requirements are met
3. Verify network connectivity
4. See [troubleshooting guide](troubleshooting.md) for common issues