#!/bin/bash
# QuDAG Benchmarking Script
# Run comprehensive benchmarks and generate reports

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
BENCHMARK_DIR="benchmark_results_$(date +%Y%m%d_%H%M%S)"
PYTHON_CMD="python3"

echo -e "${GREEN}QuDAG Benchmarking Suite${NC}"
echo "================================"
echo "Starting comprehensive benchmarks..."
echo "Output directory: $BENCHMARK_DIR"
echo ""

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"
if ! command -v $PYTHON_CMD &> /dev/null; then
    echo -e "${RED}Error: Python 3 not found${NC}"
    exit 1
fi

if ! $PYTHON_CMD -c "import psutil" &> /dev/null; then
    echo -e "${YELLOW}Installing psutil...${NC}"
    pip3 install psutil
fi

# Check if QuDAG CLI exists
if [ ! -f "./claude-flow" ]; then
    echo -e "${YELLOW}Warning: QuDAG CLI (./claude-flow) not found${NC}"
    echo "CLI benchmarks will be skipped"
fi

# Make benchmark tool executable
chmod +x qudag_benchmark.py

# Run benchmarks
echo -e "\n${GREEN}Running all benchmark suites...${NC}"
$PYTHON_CMD qudag_benchmark.py --suite all --output "$BENCHMARK_DIR" --verbose

# Check if benchmarks completed successfully
if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}Benchmarks completed successfully!${NC}"
    
    # Display summary
    echo -e "\n${YELLOW}Benchmark Summary:${NC}"
    if [ -f "$BENCHMARK_DIR/benchmark_summary.md" ]; then
        cat "$BENCHMARK_DIR/benchmark_summary.md"
    fi
    
    # Store results in QuDAG Memory if CLI is available
    if [ -f "./claude-flow" ]; then
        echo -e "\n${YELLOW}Storing results in QuDAG Memory...${NC}"
        TIMESTAMP=$(date +%Y%m%d_%H%M%S)
        ./claude-flow memory store "benchmark_results_$TIMESTAMP" "$(cat $BENCHMARK_DIR/qudag_benchmark_results.json)" || true
    fi
    
    echo -e "\n${GREEN}Results saved to: $BENCHMARK_DIR/${NC}"
    echo "View detailed results:"
    echo "  - Summary: $BENCHMARK_DIR/benchmark_summary.md"
    echo "  - Full results: $BENCHMARK_DIR/qudag_benchmark_results.json"
else
    echo -e "\n${RED}Benchmarks failed!${NC}"
    exit 1
fi