#!/bin/bash
# Simple QuDAG Vault Publishing Script

set -e

echo "🚀 Publishing QuDAG Vault (standalone mode)..."

# Check if CARGO_REGISTRY_TOKEN is set
if [ -z "$CARGO_REGISTRY_TOKEN" ]; then
    echo "❌ CARGO_REGISTRY_TOKEN not set!"
    echo "Please run: export CARGO_REGISTRY_TOKEN=your_token_here"
    exit 1
fi

# Create a clean build
echo "🧹 Cleaning build artifacts..."
cargo clean

# Navigate to vault directory
cd core/vault

echo "🔧 Building vault without QuDAG dependencies..."
cargo build --no-default-features

echo "🧪 Running basic tests..."
cargo test --no-default-features --lib

echo "📚 Generating documentation..."
cargo doc --no-deps --no-default-features

echo "🔍 Running publish dry-run..."
cargo publish --dry-run --no-default-features

read -p "🚀 Ready to publish? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📦 Publishing to crates.io..."
    cargo publish --no-default-features
    echo "✅ QuDAG Vault Core published successfully!"
else
    echo "❌ Publishing cancelled."
fi