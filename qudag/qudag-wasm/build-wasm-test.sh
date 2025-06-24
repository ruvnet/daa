#!/bin/bash

# Test WASM build with different feature combinations

echo "=== Testing QuDAG WASM Build ==="
echo ""

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf pkg/

# Test 1: Default build (crypto-only)
echo ""
echo "1. Testing default build (crypto-only)..."
wasm-pack build --target web --release --quiet 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ Default build successful!"
    ls -la pkg/
else
    echo "❌ Default build failed!"
    exit 1
fi

# Test 2: Build with DAG feature
echo ""
echo "2. Testing build with DAG feature..."
wasm-pack build --target web --release --quiet 2>/dev/null -- --features dag

if [ $? -eq 0 ]; then
    echo "✅ DAG feature build successful!"
else
    echo "❌ DAG feature build failed!"
fi

# Test 3: Build for Node.js
echo ""
echo "3. Testing Node.js build..."
wasm-pack build --target nodejs --release --quiet 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ Node.js build successful!"
else
    echo "❌ Node.js build failed!"
fi

# Test 4: Check generated files
echo ""
echo "4. Checking generated files..."
if [ -f "pkg/qudag_wasm.js" ] && [ -f "pkg/qudag_wasm_bg.wasm" ] && [ -f "pkg/qudag_wasm.d.ts" ]; then
    echo "✅ All expected files generated!"
    echo ""
    echo "Generated files:"
    ls -la pkg/
    echo ""
    echo "WASM file size:"
    ls -lh pkg/*.wasm
else
    echo "❌ Some expected files are missing!"
fi

echo ""
echo "=== Build Test Complete ==="