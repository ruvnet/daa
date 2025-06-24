#!/bin/bash
# QuDAG Vault Publishing Script

set -e

echo "🚀 Preparing QuDAG Vault for crates.io publishing..."

# Load environment variables if .env exists
if [ -f .env ]; then
    echo "📂 Loading environment variables from .env..."
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check if CARGO_REGISTRY_TOKEN is set
if [ -z "$CARGO_REGISTRY_TOKEN" ]; then
    echo "❌ CARGO_REGISTRY_TOKEN not set!"
    echo "Please set your crates.io token:"
    echo "  export CARGO_REGISTRY_TOKEN=your_token_here"
    echo "  or add it to .env file"
    exit 1
fi

echo "✅ Cargo token configured"

# Navigate to vault directory
cd core/vault

echo "🔧 Building vault in standalone mode..."
cargo build --no-default-features --features standalone

echo "🧪 Running tests..."
cargo test --no-default-features --features standalone

echo "📚 Generating documentation..."
cargo doc --no-deps --no-default-features --features standalone

echo "🔍 Running publish dry-run..."
cargo publish --dry-run --no-default-features --features standalone

echo "📦 Publishing to crates.io..."
cargo publish --no-default-features --features standalone

echo "✅ QuDAG Vault Core published successfully!"

# Navigate back and publish CLI
cd ../../tools/cli

echo "🔧 Building CLI..."
cargo build

echo "🧪 Running CLI tests..."
cargo test

echo "🔍 Running CLI publish dry-run..."
cargo publish --dry-run

echo "📦 Publishing CLI to crates.io..."
cargo publish

echo "✅ QuDAG CLI published successfully!"
echo "🎉 All packages published to crates.io!"