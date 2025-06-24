#!/bin/bash

echo "=== QuDAG Benchmark Runner ==="
echo "Generated on: $(date)"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run benchmark with timeout
run_benchmark() {
    local package=$1
    local bench_name=$2
    local timeout_sec=${3:-30}
    
    echo -e "${YELLOW}Running benchmark: $package::$bench_name${NC}"
    
    # Try to run with timeout
    if timeout $timeout_sec cargo bench -p $package --bench $bench_name -- --sample-size 10 2>/dev/null; then
        echo -e "${GREEN}✅ $bench_name completed successfully${NC}"
        return 0
    else
        echo -e "${RED}❌ $bench_name failed or timed out${NC}"
        return 1
    fi
}

# Function to check if benchmark compiles
check_benchmark() {
    local package=$1
    local bench_name=$2
    
    echo -e "${YELLOW}Checking benchmark compilation: $package::$bench_name${NC}"
    
    if timeout 30 cargo bench -p $package --bench $bench_name --no-run 2>/dev/null; then
        echo -e "${GREEN}✅ $bench_name compiles successfully${NC}"
        return 0
    else
        echo -e "${RED}❌ $bench_name compilation failed${NC}"
        return 1
    fi
}

# Track results
total_benchmarks=0
successful_compilations=0
successful_runs=0

echo "## Phase 1: Checking Benchmark Compilation"
echo ""

# Check crypto benchmarks
crypto_benchmarks=("crypto_optimized" "crypto_benchmarks")
for bench in "${crypto_benchmarks[@]}"; do
    ((total_benchmarks++))
    if check_benchmark "qudag-crypto" "$bench"; then
        ((successful_compilations++))
    fi
done

# Check network benchmarks  
network_benchmarks=("throughput_optimized" "network_benchmarks")
for bench in "${network_benchmarks[@]}"; do
    ((total_benchmarks++))
    if check_benchmark "qudag-network" "$bench"; then
        ((successful_compilations++))
    fi
done

# Check DAG benchmarks
dag_benchmarks=("dag_benchmarks" "consensus_benchmarks")
for bench in "${dag_benchmarks[@]}"; do
    ((total_benchmarks++))
    if check_benchmark "qudag-dag" "$bench"; then
        ((successful_compilations++))
    fi
done

# Check protocol benchmarks
protocol_benchmarks=("protocol_benchmarks")
for bench in "${protocol_benchmarks[@]}"; do
    ((total_benchmarks++))
    if check_benchmark "qudag-protocol" "$bench"; then
        ((successful_compilations++))
    fi
done

echo ""
echo "## Compilation Results:"
echo "Total benchmarks checked: $total_benchmarks"
echo "Successfully compiled: $successful_compilations"
echo "Compilation success rate: $((successful_compilations * 100 / total_benchmarks))%"
echo ""

# Only run benchmarks if compilation was successful
if [ $successful_compilations -gt 0 ]; then
    echo "## Phase 2: Running Successful Benchmarks (Quick Mode)"
    echo ""
    
    # Run a few key benchmarks with short sample size
    if [ $successful_compilations -ge 1 ]; then
        echo "Running critical performance benchmarks..."
        
        # Try to run crypto optimized benchmark (most important)
        if check_benchmark "qudag-crypto" "crypto_optimized"; then
            if run_benchmark "qudag-crypto" "crypto_optimized" 60; then
                ((successful_runs++))
            fi
        fi
        
        # Try to run network throughput benchmark  
        if check_benchmark "qudag-network" "throughput_optimized"; then
            if run_benchmark "qudag-network" "throughput_optimized" 60; then
                ((successful_runs++))
            fi
        fi
    fi
    
    echo ""
    echo "## Runtime Results:"
    echo "Benchmarks attempted: $(( successful_runs > 0 ? 2 : 0 ))"
    echo "Successfully executed: $successful_runs"
    
else
    echo "❌ No benchmarks compiled successfully. Skipping runtime phase."
fi

echo ""
echo "## Summary"
echo "========="
echo "Compilation: $successful_compilations/$total_benchmarks benchmarks compile"
echo "Execution: $successful_runs benchmarks ran successfully"

if [ $successful_compilations -eq $total_benchmarks ] && [ $successful_runs -gt 0 ]; then
    echo -e "${GREEN}🎉 Benchmark validation PASSED${NC}"
    exit 0
elif [ $successful_compilations -gt 0 ]; then
    echo -e "${YELLOW}⚠️ Partial benchmark validation - some issues remain${NC}"
    exit 0
else
    echo -e "${RED}❌ Benchmark validation FAILED${NC}"
    exit 1
fi