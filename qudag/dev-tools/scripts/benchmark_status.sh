#!/bin/bash

echo "=== QuDAG Benchmark Status Report ==="
echo "Generated on: $(date)"
echo ""

echo "## Benchmark Files Status"
echo ""

# Check core modules for benchmark files
modules=("crypto" "dag" "network" "protocol")

for module in "${modules[@]}"; do
    echo "### $module module:"
    bench_dir="core/$module/benches"
    if [ -d "$bench_dir" ]; then
        echo "  ✓ Benchmark directory exists"
        bench_files=$(find "$bench_dir" -name "*.rs" | wc -l)
        echo "  ✓ Benchmark files: $bench_files"
        find "$bench_dir" -name "*.rs" | sed 's/.*\//    - /'
    else
        echo "  ✗ No benchmark directory found"
    fi
    echo ""
done

echo "## Cargo.toml Benchmark Configuration"
echo ""

for module in "${modules[@]}"; do
    toml_file="core/$module/Cargo.toml"
    echo "### $module:"
    if grep -q "criterion.workspace = true" "$toml_file" 2>/dev/null; then
        echo "  ✓ criterion dependency configured"
    else
        echo "  ✗ criterion dependency missing"
    fi
    
    bench_count=$(grep -c "\[\[bench\]\]" "$toml_file" 2>/dev/null || echo "0")
    echo "  ✓ [[bench]] entries: $bench_count"
    echo ""
done

echo "## Performance Targets"
echo ""
echo "Based on project requirements:"
echo "- Sub-second consensus finality (99th percentile)"  
echo "- 10,000+ messages/second throughput per node"
echo "- Linear scalability with node count"
echo "- <100MB memory usage for base node"
echo ""

echo "## Critical Paths That Need Benchmarks"
echo ""
echo "### Crypto Operations:"
echo "- ML-KEM key generation, encapsulation, decapsulation"
echo "- ML-DSA signature generation and verification"
echo "- BLAKE3 hashing throughput"
echo "- HQC operations"
echo ""

echo "### Network Operations:"
echo "- Message throughput and latency"
echo "- Anonymous routing performance"
echo "- Connection management overhead"
echo "- P2P discovery and handshakes"
echo ""

echo "### DAG Operations:"
echo "- Node addition and validation"
echo "- Consensus algorithm performance"
echo "- Finality timing"
echo "- Graph traversal operations"
echo ""

echo "### Protocol Coordination:"
echo "- Multi-component message flows"
echo "- System initialization time"
echo "- Resource coordination overhead"
echo ""

echo "## Recommended Actions"
echo ""
echo "1. Fix compilation/dependency issues preventing benchmark runs"
echo "2. Add missing benchmarks for identified critical paths"
echo "3. Implement performance regression testing"
echo "4. Set up continuous benchmarking in CI/CD"
echo "5. Create performance monitoring dashboard"
echo ""

echo "=== End Report ==="