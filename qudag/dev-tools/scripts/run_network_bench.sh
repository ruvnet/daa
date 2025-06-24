#!/bin/bash

echo "Network Benchmark Runner"
echo "======================="
echo ""

# Clean and prepare
echo "Cleaning build artifacts..."
rm -rf target/release/deps/qudag_network*
rm -rf target/debug/deps/qudag_network*

# Try to build first
echo "Building network module..."
if cargo build -p qudag-network --release; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

# Run benchmarks
echo ""
echo "Running benchmarks..."

# Run throughput benchmark
echo "1. Throughput benchmark:"
cargo bench -p qudag-network --bench throughput -- --sample-size 10 --warm-up-time 1 --measurement-time 5 2>&1 | grep -E "(test|time:|thrpt:|Benchmarking|faster|slower)" || echo "Failed"

echo ""
echo "2. Network benchmarks:"
cargo bench -p qudag-network --bench network_benchmarks -- --sample-size 10 --warm-up-time 1 --measurement-time 5 2>&1 | grep -E "(test|time:|thrpt:|Benchmarking|faster|slower)" || echo "Failed"

echo ""
echo "Done!"