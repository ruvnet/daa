#!/bin/bash

echo "Testing cargo functionality..."

# Clean any existing build artifacts
rm -rf target

# Test 1: Check cargo version
echo "1. Checking cargo version:"
cargo --version

# Test 2: Try to build a simple hello world
echo -e "\n2. Creating minimal test project:"
mkdir -p test_minimal
cd test_minimal

cat > Cargo.toml << EOF
[package]
name = "test_minimal"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF

mkdir -p src
cat > src/main.rs << EOF
fn main() {
    println!("Hello, cargo works!");
}
EOF

echo "3. Building minimal project:"
cargo build

echo -e "\n4. Running minimal project:"
./target/debug/test_minimal

cd ..
rm -rf test_minimal

echo -e "\nCargo test completed!"