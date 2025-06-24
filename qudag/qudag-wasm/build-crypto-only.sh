#!/bin/bash
# Simplified build script for QuDAG WASM - Crypto module only

set -e

echo "Building QuDAG WASM bindings (Crypto module only)..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Add wasm32 target if not already added
rustup target add wasm32-unknown-unknown 2>/dev/null

# Clean previous builds
rm -rf pkg pkg-crypto

# Build for web target (crypto only)
echo "Building crypto module for web..."
wasm-pack build --target web --out-dir pkg-crypto --release --no-default-features --quiet 2>/dev/null

echo "Build complete!"
echo ""
echo "Output directory:"
echo "  - pkg-crypto/  : Crypto-only web build"
echo ""
echo "Note: This build includes only the quantum-resistant cryptography"
echo "features that are compatible with WASM. Network-dependent features"
echo "are not available in this build."