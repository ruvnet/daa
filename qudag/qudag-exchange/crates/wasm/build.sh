#!/bin/bash
# Build script for QuDAG Exchange WASM

set -e

echo "Building QuDAG Exchange WASM..."

# Build for web target
echo "Building for web browsers..."
wasm-pack build --target web --out-dir pkg-web --release

# Build for Node.js
echo "Building for Node.js..."
wasm-pack build --target nodejs --out-dir pkg-node --release

# Build for bundlers (webpack, etc.)
echo "Building for bundlers..."
wasm-pack build --target bundler --out-dir pkg-bundler --release

echo "WASM build complete!"
echo "Outputs:"
echo "  - pkg-web/     (for browsers)"
echo "  - pkg-node/    (for Node.js)"
echo "  - pkg-bundler/ (for webpack/bundlers)"