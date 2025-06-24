#!/bin/bash

# Test script for protocol module
echo "Testing QuDAG Protocol Module"
echo "=============================="

# Change to protocol directory
cd /workspaces/QuDAG/core/protocol

echo "1. Checking if protocol module compiles..."
cargo check --lib 2>&1

echo -e "\n2. Running unit tests..."
cargo test --lib --tests 2>&1

echo -e "\n3. Running integration tests..."
cargo test --test '*' 2>&1

echo -e "\n4. Running all tests with verbose output..."
cargo test --all --verbose 2>&1

echo -e "\nTest run completed."