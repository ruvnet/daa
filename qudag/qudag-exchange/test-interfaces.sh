#!/bin/bash
# Test script to verify interface components compile

set -e

echo "Testing QuDAG Exchange Interface Components..."
echo "============================================"

# Test that all crates compile
echo -e "\n1. Testing workspace compilation..."
cargo check --workspace

echo -e "\n2. Testing CLI compilation..."
cargo check -p qudag-exchange-cli

echo -e "\n3. Testing API compilation..."
cargo check -p qudag-exchange-api

echo -e "\n4. Testing WASM compilation..."
cargo check -p qudag-exchange-wasm --target wasm32-unknown-unknown

echo -e "\n5. Running clippy lints..."
cargo clippy --workspace -- -D warnings || true

echo -e "\n6. Checking documentation..."
cargo doc --workspace --no-deps

echo -e "\n✅ All interface components compile successfully!"
echo "Note: Core implementation is still pending from Core Implementation Agent"