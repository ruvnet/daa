#!/bin/bash
set -euo pipefail

# QuDAG Protocol Project Scaffolding Script
# This script creates the complete project structure for TDD development with Claude Code

PROJECT_NAME="qudag-protocol"
PROJECT_ROOT="$(pwd)/$PROJECT_NAME"

echo "🚀 Setting up QuDAG Protocol project structure..."

# Create main project directory
mkdir -p "$PROJECT_ROOT"
cd "$PROJECT_ROOT"

# Initialize Rust workspace
cat > Cargo.toml << 'HEREDOC_END'
[workspace]
members = [
"core/crypto",
"core/dag",
"core/network",
"core/protocol",
"tools/cli",
"tools/simulator",
"benchmarks"
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
anyhow = "1.0"
thiserror = "1.0"
proptest = "1.0"
criterion = { version = "0.5", features = ["html_reports"] }
pqcrypto = "0.16"
libp2p = "0.53"
futures = "0.3"
rand = "0.8"
sha3 = "0.10"
ed25519-dalek = "2.0"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"

[profile.bench]
debug = true
HEREDOC_END

echo "✅ Workspace Cargo.toml created"

# Create core directory structure
mkdir -p core/{crypto,dag,network,protocol}/src
mkdir -p core/{crypto,dag,network,protocol}/tests
mkdir -p tools/{cli,simulator}/src
mkdir -p benchmarks/src

# Create test directories
mkdir -p tests/{unit,integration,security,performance}
mkdir -p fuzz/fuzz_targets

# Create Claude Code directories
mkdir -p .claude/{commands,workflows,agents,contexts}

# Create documentation directories
mkdir -p docs/{api,user-guide,security,architecture}

# Create infrastructure directories
mkdir -p infra/{docker,k8s,terraform}
mkdir -p scripts/{dev,deploy,test}

# Create GitHub workflows
mkdir -p .github/{workflows,ISSUE_TEMPLATE}

echo "📁 Directory structure created"

# Create a simple starter file
cat > core/crypto/src/lib.rs << 'HEREDOC_END'
//! Quantum-resistant cryptographic primitives for QuDAG Protocol

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod kem;
pub mod signatures;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
HEREDOC_END

echo "✅ Basic source files created"

# Create core crate configurations
cat > core/crypto/Cargo.toml << 'HEREDOC_END'
[package]
name = "qudag-crypto"
version = "0.1.0"
edition = "2021"

[dependencies]
pqcrypto = { workspace = true }
rand = { workspace = true }
sha3 = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }
criterion = { workspace = true }
HEREDOC_END

cat > core/dag/Cargo.toml << 'HEREDOC_END'
[package]
name = "qudag-dag"
version = "0.1.0"
edition = "2021"

[dependencies]
qudag-crypto = { path = "../crypto" }
tokio = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }
tokio-test = "0.4"
HEREDOC_END

cat > core/network/Cargo.toml << 'HEREDOC_END'
[package]
name = "qudag-network"
version = "0.1.0"
edition = "2021"

[dependencies]
qudag-crypto = { path = "../crypto" }
qudag-dag = { path = "../dag" }
libp2p = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
futures = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
proptest = { workspace = true }
HEREDOC_END

cat > core/protocol/Cargo.toml << 'HEREDOC_END'
[package]
name = "qudag-protocol"
version = "0.1.0"
edition = "2021"

[dependencies]
qudag-crypto = { path = "../crypto" }
qudag-dag = { path = "../dag" }
qudag-network = { path = "../network" }
tokio = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
proptest = { workspace = true }
HEREDOC_END

echo "✅ Core crate configurations created"

# Create README
cat > README.md << 'HEREDOC_END'
# QuDAG Protocol

A quantum-resistant DAG-based anonymous communication system implemented in Rust using Test-Driven Development.

## Features

- **Quantum Resistance**: Post-quantum cryptography (ML-KEM, ML-DSA, HQC)
- **DAG Consensus**: QR-Avalanche algorithm for parallel transaction processing
- **Anonymous Communication**: Multi-path onion routing with traffic obfuscation
- **High Performance**: Sub-second finality, 10,000+ messages/second throughput
- **Security First**: Constant-time operations, side-channel resistance

## Quick Start

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Development

This project is optimized for development with Claude Code and follows TDD methodology.

## License

Licensed under either Apache License 2.0 or MIT License at your option.
HEREDOC_END

echo "✅ README.md created"

# Create .gitignore
cat > .gitignore << 'HEREDOC_END'
# Rust
/target/
Cargo.lock
*.pdb

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Test artifacts
/coverage/
/fuzz/corpus/
/fuzz/artifacts/

# Benchmarks
/benchmarks/target/

# Data
/data/
*.db

# Secrets
*.key
*.pem
.env

# Claude Code cache
.claude/cache/
.claude/logs/
HEREDOC_END

echo "✅ .gitignore created"

echo ""
echo "🎉 QuDAG Protocol project scaffolding complete!"
echo ""
echo "Project structure created at: $PROJECT_ROOT"
echo ""
echo "Next steps:"
echo "1. cd $PROJECT_NAME"
echo "2. cargo build"
echo "3. cargo test"
echo ""
echo "Happy coding! 🚀"