#!/bin/bash

echo "QuDAG Performance Benchmarks Report"
echo "=================================="
echo "Date: $(date)"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found"
    exit 1
fi

echo "1. Build Status"
echo "---------------"
if cargo build -p qudag-dag --release 2>&1 | tail -5; then
    echo "✓ Build successful"
else
    echo "✗ Build failed"
fi

echo ""
echo "2. Test Execution (Performance-related tests)"
echo "--------------------------------------------"
# Run tests that contain performance/benchmark keywords
cargo test -p qudag-dag --release performance 2>&1 | grep -E "(test result:|passed|FAILED|test.*performance)"

echo ""
echo "3. Available Benchmarks"
echo "----------------------"
ls -la /workspaces/QuDAG/core/dag/benches/

echo ""
echo "4. Performance Requirements from CLAUDE.md"
echo "-----------------------------------------"
echo "- Sub-second consensus finality (99th percentile)"
echo "- 10,000+ messages/second throughput per node"
echo "- Linear scalability with node count"
echo "- <100MB memory usage for base node"

echo ""
echo "5. Benchmark Summary"
echo "-------------------"
echo "The DAG benchmarks measure:"
echo "- Consensus finality times across different network sizes (10-500 nodes)"
echo "- Throughput in vertices/transactions per second"
echo "- Scalability with increasing DAG sizes (1k-100k vertices)"
echo "- Byzantine resistance with up to 33% malicious nodes"
echo "- Memory usage efficiency"
echo ""
echo "To run full benchmarks when build succeeds:"
echo "  cargo bench -p qudag-dag"
echo ""
echo "Individual benchmark suites:"
echo "  - finality_benchmarks: Consensus finality latency"
echo "  - consensus_benchmarks: Basic consensus rounds"
echo "  - qr_avalanche_benchmarks: QR-Avalanche algorithm performance"
echo "  - dag_benchmarks: Core DAG operations"