#!/bin/bash

# QuDAG Exchange Performance Benchmarking Script
# Run comprehensive benchmarks and generate reports

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
RESULTS_DIR="$PROJECT_ROOT/target/benchmark-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}QuDAG Exchange Performance Benchmarking${NC}"
echo "========================================="

# Create results directory
mkdir -p "$RESULTS_DIR/$TIMESTAMP"

# Function to run benchmarks with profiling
run_benchmarks() {
    echo -e "\n${YELLOW}Running Criterion benchmarks...${NC}"
    
    # Run standard benchmarks
    cargo bench --bench exchange_benchmarks -- --save-baseline "$TIMESTAMP" 2>&1 | tee "$RESULTS_DIR/$TIMESTAMP/criterion.log"
    
    # Copy HTML reports
    if [ -d "target/criterion" ]; then
        cp -r target/criterion "$RESULTS_DIR/$TIMESTAMP/"
        echo -e "${GREEN}✓ Criterion reports saved${NC}"
    fi
}

# Function to run flamegraph profiling
run_flamegraph() {
    if command -v cargo-flamegraph &> /dev/null; then
        echo -e "\n${YELLOW}Generating flamegraph...${NC}"
        
        # Build with debug symbols
        cargo build --release --features "timing-attack-tests"
        
        # Run with flamegraph
        cargo flamegraph --bench exchange_benchmarks -o "$RESULTS_DIR/$TIMESTAMP/flamegraph.svg" -- --bench
        
        echo -e "${GREEN}✓ Flamegraph saved to $RESULTS_DIR/$TIMESTAMP/flamegraph.svg${NC}"
    else
        echo -e "${YELLOW}! cargo-flamegraph not installed. Skipping flamegraph generation.${NC}"
        echo "  Install with: cargo install flamegraph"
    fi
}

# Function to check for performance regressions
check_regressions() {
    echo -e "\n${YELLOW}Checking for performance regressions...${NC}"
    
    if [ -d "target/criterion" ]; then
        # Look for regression reports
        if grep -r "Regression" target/criterion/*.json 2>/dev/null; then
            echo -e "${RED}⚠ Performance regressions detected!${NC}"
            echo "Check the criterion reports for details."
        else
            echo -e "${GREEN}✓ No performance regressions detected${NC}"
        fi
    fi
}

# Function to generate memory profile
run_memory_profile() {
    if command -v valgrind &> /dev/null; then
        echo -e "\n${YELLOW}Running memory profiling with valgrind...${NC}"
        
        # Build test binary
        cargo build --release --bin qudag-exchange-bench 2>/dev/null || {
            echo "Skipping valgrind - no bench binary available yet"
            return
        }
        
        valgrind --tool=massif --massif-out-file="$RESULTS_DIR/$TIMESTAMP/massif.out" \
            target/release/qudag-exchange-bench
        
        # Generate readable output
        if command -v ms_print &> /dev/null; then
            ms_print "$RESULTS_DIR/$TIMESTAMP/massif.out" > "$RESULTS_DIR/$TIMESTAMP/memory-profile.txt"
            echo -e "${GREEN}✓ Memory profile saved${NC}"
        fi
    else
        echo -e "${YELLOW}! valgrind not installed. Skipping memory profiling.${NC}"
    fi
}

# Function to run WASM size analysis
analyze_wasm_size() {
    echo -e "\n${YELLOW}Analyzing WASM bundle size...${NC}"
    
    if command -v wasm-pack &> /dev/null; then
        # Build WASM
        wasm-pack build --release --target web --out-dir pkg 2>&1 | tee "$RESULTS_DIR/$TIMESTAMP/wasm-build.log"
        
        # Analyze size
        if [ -f "pkg/*_bg.wasm" ]; then
            WASM_SIZE=$(ls -lh pkg/*_bg.wasm | awk '{print $5}')
            echo "WASM bundle size: $WASM_SIZE" > "$RESULTS_DIR/$TIMESTAMP/wasm-size.txt"
            
            # Check against target
            WASM_BYTES=$(stat -c%s pkg/*_bg.wasm 2>/dev/null || stat -f%z pkg/*_bg.wasm 2>/dev/null)
            if [ "$WASM_BYTES" -lt 512000 ]; then
                echo -e "${GREEN}✓ WASM size within target (<500KB): $WASM_SIZE${NC}"
            else
                echo -e "${RED}⚠ WASM size exceeds target (>500KB): $WASM_SIZE${NC}"
            fi
        fi
    else
        echo -e "${YELLOW}! wasm-pack not installed. Skipping WASM analysis.${NC}"
    fi
}

# Function to generate performance report
generate_report() {
    echo -e "\n${YELLOW}Generating performance report...${NC}"
    
    cat > "$RESULTS_DIR/$TIMESTAMP/report.md" << EOF
# QuDAG Exchange Performance Report
Generated: $(date)

## Benchmark Results
- Criterion reports: [HTML Reports](criterion/report/index.html)
- Flamegraph: [flamegraph.svg](flamegraph.svg)
- Memory profile: [memory-profile.txt](memory-profile.txt)

## Key Metrics
$(grep -E "(time:|thrpt:|Transaction)" "$RESULTS_DIR/$TIMESTAMP/criterion.log" | head -20 || echo "Benchmarks pending implementation")

## WASM Bundle Size
$(cat "$RESULTS_DIR/$TIMESTAMP/wasm-size.txt" 2>/dev/null || echo "WASM build pending")

## Performance Targets
- Transaction throughput: >10,000 TPS
- Ledger lookup latency: <1ms
- WASM bundle size: <500KB
- Memory usage: <100MB for 1M accounts

## Notes
This report tracks performance metrics for the QuDAG Exchange implementation.
Regular benchmarking helps identify optimization opportunities and regressions.
EOF

    echo -e "${GREEN}✓ Performance report saved to $RESULTS_DIR/$TIMESTAMP/report.md${NC}"
}

# Main execution
main() {
    echo "Starting benchmarks at $(date)"
    echo "Results will be saved to: $RESULTS_DIR/$TIMESTAMP"
    
    # Change to project directory
    cd "$PROJECT_ROOT"
    
    # Run all benchmarks
    run_benchmarks
    run_flamegraph
    run_memory_profile
    analyze_wasm_size
    check_regressions
    generate_report
    
    echo -e "\n${GREEN}Benchmarking complete!${NC}"
    echo "Results saved to: $RESULTS_DIR/$TIMESTAMP"
    echo "View the performance report at: $RESULTS_DIR/$TIMESTAMP/report.md"
}

# Run main function
main