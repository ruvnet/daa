#!/bin/bash
#
# Generate Flame Graphs for DAA Performance Profiling
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}DAA Performance Profiling with Flame Graphs${NC}"
echo "=============================================="

# Check if perf is available
if ! command -v perf &> /dev/null; then
    echo -e "${RED}Error: 'perf' is not installed. Please install linux-tools-generic${NC}"
    exit 1
fi

# Check if flamegraph is available
if ! command -v flamegraph &> /dev/null; then
    echo -e "${YELLOW}Installing flamegraph...${NC}"
    cargo install flamegraph
fi

# Create output directory
FLAME_OUTPUT_DIR="flame_graphs_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$FLAME_OUTPUT_DIR"

echo -e "${GREEN}Output directory: $FLAME_OUTPUT_DIR${NC}"

# Function to generate flame graph for a specific benchmark
generate_flame_graph() {
    local benchmark_name=$1
    local bench_target=$2
    local output_name=$3
    
    echo -e "${YELLOW}Generating flame graph for $benchmark_name...${NC}"
    
    # Run benchmark with flamegraph
    flamegraph --output "$FLAME_OUTPUT_DIR/${output_name}.svg" -- \
        cargo bench --bench "$bench_target" -- --sample-size 10 --measurement-time 30
    
    echo -e "${GREEN}✓ Flame graph generated: $FLAME_OUTPUT_DIR/${output_name}.svg${NC}"
}

# Function to run CPU profiling
profile_cpu_intensive() {
    echo -e "${YELLOW}Profiling CPU-intensive operations...${NC}"
    
    # Profile training benchmarks
    generate_flame_graph "Training Convergence" "training_benchmarks" "cpu_training_convergence"
    
    # Profile consensus benchmarks
    generate_flame_graph "Consensus Finalization" "consensus_benchmarks" "cpu_consensus_finalization"
    
    # Profile resource benchmarks
    generate_flame_graph "Resource Utilization" "resource_benchmarks" "cpu_resource_utilization"
}

# Function to run memory profiling
profile_memory_usage() {
    echo -e "${YELLOW}Profiling memory usage...${NC}"
    
    # Use valgrind massif for memory profiling
    if command -v valgrind &> /dev/null; then
        valgrind --tool=massif --massif-out-file="$FLAME_OUTPUT_DIR/memory_profile.out" \
            cargo bench --bench training_benchmarks -- --sample-size 5
        
        # Generate memory flame graph if ms_print is available
        if command -v ms_print &> /dev/null; then
            ms_print "$FLAME_OUTPUT_DIR/memory_profile.out" > "$FLAME_OUTPUT_DIR/memory_profile.txt"
            echo -e "${GREEN}✓ Memory profile generated: $FLAME_OUTPUT_DIR/memory_profile.txt${NC}"
        fi
    else
        echo -e "${RED}Warning: valgrind not available for memory profiling${NC}"
    fi
}

# Function to run network profiling
profile_network_io() {
    echo -e "${YELLOW}Profiling network I/O...${NC}"
    
    # Profile P2P benchmarks
    generate_flame_graph "P2P Network Operations" "p2p_benchmarks" "network_p2p_operations"
    
    # Profile bandwidth usage
    flamegraph --output "$FLAME_OUTPUT_DIR/network_bandwidth.svg" -- \
        cargo bench --bench resource_benchmarks -- bandwidth --sample-size 10
    
    echo -e "${GREEN}✓ Network I/O flame graphs generated${NC}"
}

# Function to profile specific scenarios
profile_scenarios() {
    echo -e "${YELLOW}Profiling specific scenarios...${NC}"
    
    # High-load scenario
    echo "Profiling high-load training scenario..."
    flamegraph --output "$FLAME_OUTPUT_DIR/scenario_high_load.svg" -- \
        cargo bench --bench training_benchmarks -- "large_model" --sample-size 5
    
    # Distributed consensus scenario
    echo "Profiling distributed consensus scenario..."
    flamegraph --output "$FLAME_OUTPUT_DIR/scenario_consensus.svg" -- \
        cargo bench --bench consensus_benchmarks -- "validator_count" --sample-size 5
    
    # P2P discovery scenario
    echo "Profiling P2P discovery scenario..."
    flamegraph --output "$FLAME_OUTPUT_DIR/scenario_p2p_discovery.svg" -- \
        cargo bench --bench p2p_benchmarks -- "peer_discovery" --sample-size 5
    
    echo -e "${GREEN}✓ Scenario-specific flame graphs generated${NC}"
}

# Function to generate comparison flame graphs
profile_comparisons() {
    echo -e "${YELLOW}Profiling DAA vs PyTorch comparisons...${NC}"
    
    flamegraph --output "$FLAME_OUTPUT_DIR/comparison_pytorch_vs_daa.svg" -- \
        cargo bench --bench pytorch_comparison -- --sample-size 5
    
    echo -e "${GREEN}✓ Comparison flame graphs generated${NC}"
}

# Function to generate summary report
generate_summary_report() {
    echo -e "${YELLOW}Generating summary report...${NC}"
    
    cat > "$FLAME_OUTPUT_DIR/README.md" << EOF
# DAA Performance Profiling Results

Generated on: $(date)

## Flame Graphs

### CPU Profiling
- \`cpu_training_convergence.svg\` - Training convergence CPU usage
- \`cpu_consensus_finalization.svg\` - Consensus finalization CPU usage  
- \`cpu_resource_utilization.svg\` - Resource utilization CPU patterns

### Network Profiling
- \`network_p2p_operations.svg\` - P2P network operations
- \`network_bandwidth.svg\` - Bandwidth utilization patterns

### Scenario Analysis
- \`scenario_high_load.svg\` - High-load training scenario
- \`scenario_consensus.svg\` - Distributed consensus scenario
- \`scenario_p2p_discovery.svg\` - P2P peer discovery scenario

### Performance Comparisons
- \`comparison_pytorch_vs_daa.svg\` - DAA vs PyTorch performance comparison

### Memory Analysis
$([ -f "$FLAME_OUTPUT_DIR/memory_profile.txt" ] && echo "- \`memory_profile.txt\` - Memory usage analysis" || echo "- Memory profiling not available (valgrind required)")

## How to View Flame Graphs

1. Open the .svg files in a web browser
2. Click on stack frames to zoom in
3. Use Ctrl+F to search for specific function names
4. Hover over frames to see timing information

## Key Performance Insights

### CPU Hotspots
Look for wide bars in the flame graphs - these represent functions that consume the most CPU time.

### Memory Patterns
Check the memory profile for allocation patterns and potential leaks.

### Network Bottlenecks
Examine network flame graphs for I/O wait times and serialization overhead.

## Benchmark Configuration

- Sample Size: 5-10 samples per benchmark
- Measurement Time: 30 seconds per benchmark
- Profiling Tool: cargo flamegraph + perf
- Memory Profiler: valgrind massif (if available)

EOF

    echo -e "${GREEN}✓ Summary report generated: $FLAME_OUTPUT_DIR/README.md${NC}"
}

# Main execution
echo -e "${BLUE}Starting comprehensive performance profiling...${NC}"

# Build benchmarks first
echo -e "${YELLOW}Building benchmarks...${NC}"
cargo build --release --benches

# Run all profiling
profile_cpu_intensive
profile_memory_usage
profile_network_io
profile_scenarios
profile_comparisons

# Generate summary
generate_summary_report

echo ""
echo -e "${GREEN}🎉 Performance profiling complete!${NC}"
echo -e "${BLUE}Results saved to: $FLAME_OUTPUT_DIR/${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Open the flame graph SVG files in your browser"
echo "2. Review the README.md for analysis guidance"
echo "3. Look for performance bottlenecks in the wide flame bars"
echo "4. Compare DAA performance with PyTorch using comparison graphs"
echo ""

# Optional: Open the directory in file manager (Linux)
if command -v xdg-open &> /dev/null; then
    echo -e "${BLUE}Opening results directory...${NC}"
    xdg-open "$FLAME_OUTPUT_DIR" 2>/dev/null || true
fi