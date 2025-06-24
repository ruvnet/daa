#!/bin/bash

# Script to run dark addressing integration tests
set -e

echo "Setting up QuDAG environment..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Must be run from the QuDAG root directory"
    exit 1
fi

echo "Building the project..."
if command -v cargo &> /dev/null; then
    # Try to build the core components first
    cargo check --lib -p qudag-network
    cargo check --lib -p qudag-crypto
    
    echo "Running dark addressing integration tests..."
    
    # Run the specific integration tests
    echo "Running dark domain tests..."
    cargo test --test '*' dark_domain_tests --features test
    
    echo "Running shadow address tests..."
    cargo test --test '*' shadow_address_tests --features test
    
    echo "Running quantum fingerprint tests..."
    cargo test --test '*' quantum_fingerprint_tests --features test
    
    echo "Running DNS integration tests..."
    cargo test --test '*' dns_integration_tests --features test
    
    echo "Running full system tests..."
    cargo test --test '*' full_system_tests --features test
    
    echo "All dark addressing tests completed successfully!"
else
    echo "Warning: Cargo not found. Please install Rust and Cargo to run tests."
    echo "You can install Rust from: https://rustup.rs/"
    
    echo "For now, showing the test structure that has been created:"
    find tests/integration/dark_addressing -name "*.rs" -exec echo "Test file: {}" \; -exec wc -l {} \;
fi

echo "Dark addressing integration test structure:"
echo "============================================"
echo "✓ Dark domain registration and resolution tests"
echo "✓ Shadow address routing and resolution tests" 
echo "✓ Quantum fingerprint verification tests"
echo "✓ DNS record management and resolution tests"
echo "✓ Full system integration tests"
echo ""
echo "Test files created:"
ls -la tests/integration/dark_addressing/

echo ""
echo "To run individual test files when Cargo is available:"
echo "cargo test --test dark_domain_tests"
echo "cargo test --test shadow_address_tests"
echo "cargo test --test quantum_fingerprint_tests"
echo "cargo test --test dns_integration_tests"
echo "cargo test --test full_system_tests"