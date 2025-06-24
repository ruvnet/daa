#!/bin/bash

echo "Testing QuDAG Protocol Module - Simple Test"
echo "============================================"

cd /workspaces/QuDAG

# Test 1: Check if protocol compiles
echo "1. Checking protocol compilation..."
timeout 30s cargo check -p qudag-protocol 2>&1 | head -20

echo -e "\n2. Trying to run a single test..."
timeout 30s cargo test -p qudag-protocol test_coordinator_lifecycle 2>&1 | head -20

echo -e "\n3. Testing protocol unit tests..."
timeout 30s cargo test -p qudag-protocol --lib 2>&1 | head -20

echo -e "\nTest completed."