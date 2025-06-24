#!/bin/bash
# Build script for QuDAG WASM

set -e

echo "Building QuDAG WASM bindings..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Add wasm32 target if not already added
rustup target add wasm32-unknown-unknown 2>/dev/null

# Clean previous builds
rm -rf pkg pkg-node

# Set environment variables for optimization
export WASM_BINDGEN_EXTERNREF=1
export RUSTFLAGS="-C target-feature=+reference-types"

# Build for web target
echo "Building for web..."
wasm-pack build --target web --out-dir pkg --release --quiet 2>/dev/null

# Build for Node.js
echo "Building for Node.js..."
wasm-pack build --target nodejs --out-dir pkg-node --release --quiet 2>/dev/null

# Build with size optimizations
echo "Building optimized version..."
wasm-pack build --target web --out-dir pkg-optimized --release --quiet 2>/dev/null -- --features wee_alloc

# Generate TypeScript definitions
echo "Generating TypeScript definitions..."
wasm-pack build --target web --out-dir pkg-ts --release --no-default-features --features crypto-only --quiet 2>/dev/null

# Optimize WASM with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM binaries with wasm-opt..."
    
    for dir in pkg pkg-node pkg-optimized pkg-ts; do
        if [ -f "$dir/qudag_wasm_bg.wasm" ]; then
            echo "Optimizing $dir/qudag_wasm_bg.wasm..."
            wasm-opt -Os --enable-reference-types "$dir/qudag_wasm_bg.wasm" -o "$dir/qudag_wasm_bg.wasm"
        fi
    done
else
    echo "wasm-opt not found. Install binaryen for additional optimizations:"
    echo "  brew install binaryen  # macOS"
    echo "  apt install binaryen   # Ubuntu/Debian"
fi

echo "Build complete!"
echo ""
echo "Output directories:"
echo "  - pkg/         : Web build"
echo "  - pkg-node/    : Node.js build"
echo "  - pkg-optimized/ : Size-optimized build"
echo "  - pkg-ts/      : TypeScript definitions"