#!/bin/bash

# QuDAG Comprehensive Benchmark Suite
# Tests all performance targets from CLAUDE.md:
# - Sub-second consensus finality (99th percentile)
# - 10,000+ messages/second throughput per node
# - Linear scalability with node count
# - <100MB memory usage for base node

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Benchmark results directory
RESULTS_DIR="benchmark_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULTS_FILE="${RESULTS_DIR}/benchmark_results_${TIMESTAMP}.txt"

# Create results directory
mkdir -p "${RESULTS_DIR}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}QuDAG Comprehensive Benchmark Suite${NC}"
echo -e "${BLUE}Testing Performance Targets:${NC}"
echo -e "${BLUE}- Sub-second consensus finality (99th percentile)${NC}"
echo -e "${BLUE}- 10,000+ messages/second throughput${NC}"
echo -e "${BLUE}- Linear scalability with node count${NC}"
echo -e "${BLUE}- <100MB memory usage for base node${NC}"
echo -e "${BLUE}========================================${NC}"

# Function to run benchmark with timing and error handling
run_benchmark() {
    local crate_name=$1
    local bench_name=$2
    local description=$3
    
    echo -e "${YELLOW}Running ${description}...${NC}"
    echo "Running ${description}" >> "${RESULTS_FILE}"
    echo "Started at: $(date)" >> "${RESULTS_FILE}"
    
    local start_time=$(date +%s)
    
    if cargo bench --manifest-path "${crate_name}/Cargo.toml" --bench "${bench_name}" 2>&1 | tee -a "${RESULTS_FILE}"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "${GREEN}✓ ${description} completed in ${duration}s${NC}"
        echo "Completed successfully in ${duration}s" >> "${RESULTS_FILE}"
        echo "" >> "${RESULTS_FILE}"
        return 0
    else
        echo -e "${RED}✗ ${description} failed${NC}"
        echo "BENCHMARK FAILED" >> "${RESULTS_FILE}"
        echo "" >> "${RESULTS_FILE}"
        return 1
    fi
}

# Function to check if benchmark meets performance targets
check_performance_targets() {
    echo -e "${BLUE}Checking Performance Targets...${NC}"
    echo "=== PERFORMANCE TARGET ANALYSIS ===" >> "${RESULTS_FILE}"
    
    # Check for warning messages in benchmark output
    local warnings=$(grep -i "warning" "${RESULTS_FILE}" | wc -l)
    local latency_warnings=$(grep -i "latency.*exceeds" "${RESULTS_FILE}" | wc -l)
    local throughput_warnings=$(grep -i "throughput.*below" "${RESULTS_FILE}" | wc -l)
    local memory_warnings=$(grep -i "memory.*exceeds" "${RESULTS_FILE}" | wc -l)
    
    echo "Performance Analysis:" >> "${RESULTS_FILE}"
    echo "- Total warnings: ${warnings}" >> "${RESULTS_FILE}"
    echo "- Latency warnings: ${latency_warnings}" >> "${RESULTS_FILE}"
    echo "- Throughput warnings: ${throughput_warnings}" >> "${RESULTS_FILE}"
    echo "- Memory warnings: ${memory_warnings}" >> "${RESULTS_FILE}"
    
    if [ ${warnings} -eq 0 ]; then
        echo -e "${GREEN}✓ All performance targets met!${NC}"
        echo "STATUS: ALL TARGETS MET" >> "${RESULTS_FILE}"
        return 0
    else
        echo -e "${YELLOW}⚠ ${warnings} performance warnings found${NC}"
        echo "STATUS: SOME TARGETS NOT MET" >> "${RESULTS_FILE}"
        return 1
    fi
}

# Main benchmark execution
echo "Starting comprehensive benchmark suite at $(date)" > "${RESULTS_FILE}"
echo "Platform: $(uname -a)" >> "${RESULTS_FILE}"
echo "Rust version: $(rustc --version)" >> "${RESULTS_FILE}"
echo "Cargo version: $(cargo --version)" >> "${RESULTS_FILE}"
echo "" >> "${RESULTS_FILE}"

# Build all components first
echo -e "${YELLOW}Building all components...${NC}"
if ! cargo build --release --workspace; then
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Build completed${NC}"

# Initialize counters
total_benchmarks=0
passed_benchmarks=0
failed_benchmarks=0

# Crypto benchmarks
echo -e "\n${BLUE}=== CRYPTO BENCHMARKS ===${NC}"
benchmarks=(
    "core/crypto:crypto_benchmarks:ML-KEM Basic Operations"
    "core/crypto:crypto_optimized:Optimized Crypto Operations"
    "core/crypto:ml_kem_benchmarks:ML-KEM Detailed Performance"
    "core/crypto:mldsa_benchmarks:ML-DSA Signature Performance"
    "core/crypto:hqc_benchmarks:HQC Encryption Performance"
)

for benchmark in "${benchmarks[@]}"; do
    IFS=':' read -r crate bench_name description <<< "$benchmark"
    total_benchmarks=$((total_benchmarks + 1))
    if run_benchmark "$crate" "$bench_name" "$description"; then
        passed_benchmarks=$((passed_benchmarks + 1))
    else
        failed_benchmarks=$((failed_benchmarks + 1))
    fi
done

# Network benchmarks
echo -e "\n${BLUE}=== NETWORK BENCHMARKS ===${NC}"
benchmarks=(
    "core/network:throughput:Network Throughput (Original)"
    "core/network:throughput_optimized:Network Throughput (Optimized)"
    "core/network:network_benchmarks:Network Operations"
    "core/network:peer_benchmarks:Peer Management"
    "core/network:routing_benchmarks:Routing Performance"
)

for benchmark in "${benchmarks[@]}"; do
    IFS=':' read -r crate bench_name description <<< "$benchmark"
    total_benchmarks=$((total_benchmarks + 1))
    if run_benchmark "$crate" "$bench_name" "$description"; then
        passed_benchmarks=$((passed_benchmarks + 1))
    else
        failed_benchmarks=$((failed_benchmarks + 1))
    fi
done

# DAG consensus benchmarks
echo -e "\n${BLUE}=== DAG CONSENSUS BENCHMARKS ===${NC}"
benchmarks=(
    "core/dag:consensus_benchmarks:Basic Consensus Operations"
    "core/dag:finality_benchmarks:Consensus Finality Performance"
    "core/dag:dag_benchmarks:DAG Operations"
)

for benchmark in "${benchmarks[@]}"; do
    IFS=':' read -r crate bench_name description <<< "$benchmark"
    total_benchmarks=$((total_benchmarks + 1))
    if run_benchmark "$crate" "$bench_name" "$description"; then
        passed_benchmarks=$((passed_benchmarks + 1))
    else
        failed_benchmarks=$((failed_benchmarks + 1))
    fi
done

# Protocol benchmarks
echo -e "\n${BLUE}=== PROTOCOL BENCHMARKS ===${NC}"
benchmarks=(
    "core/protocol:protocol_benchmarks:Protocol Operations"
)

for benchmark in "${benchmarks[@]}"; do
    IFS=':' read -r crate bench_name description <<< "$benchmark"
    total_benchmarks=$((total_benchmarks + 1))
    if run_benchmark "$crate" "$bench_name" "$description"; then
        passed_benchmarks=$((passed_benchmarks + 1))
    else
        failed_benchmarks=$((failed_benchmarks + 1))
    fi
done

# System benchmarks
echo -e "\n${BLUE}=== SYSTEM BENCHMARKS ===${NC}"
benchmarks=(
    "benchmarks:system_benchmarks:System-wide Performance"
    "benchmarks:dark_addressing_benchmarks:Dark Addressing Performance"
)

for benchmark in "${benchmarks[@]}"; do
    IFS=':' read -r crate bench_name description <<< "$benchmark"
    total_benchmarks=$((total_benchmarks + 1))
    if run_benchmark "$crate" "$bench_name" "$description"; then
        passed_benchmarks=$((passed_benchmarks + 1))
    else
        failed_benchmarks=$((failed_benchmarks + 1))
    fi
done

# Simulator benchmarks
echo -e "\n${BLUE}=== SIMULATOR BENCHMARKS ===${NC}"
benchmarks=(
    "tools/simulator:simulator_benchmarks:Network Simulation Performance"
)

for benchmark in "${benchmarks[@]}"; do
    IFS=':' read -r crate bench_name description <<< "$benchmark"
    total_benchmarks=$((total_benchmarks + 1))
    if run_benchmark "$crate" "$bench_name" "$description"; then
        passed_benchmarks=$((passed_benchmarks + 1))
    else
        failed_benchmarks=$((failed_benchmarks + 1))
    fi
done

# Final summary
echo -e "\n${BLUE}=== BENCHMARK SUMMARY ===${NC}"
echo "Total benchmarks: ${total_benchmarks}"
echo "Passed: ${passed_benchmarks}"
echo "Failed: ${failed_benchmarks}"

echo "" >> "${RESULTS_FILE}"
echo "=== FINAL SUMMARY ===" >> "${RESULTS_FILE}"
echo "Total benchmarks: ${total_benchmarks}" >> "${RESULTS_FILE}"
echo "Passed: ${passed_benchmarks}" >> "${RESULTS_FILE}"
echo "Failed: ${failed_benchmarks}" >> "${RESULTS_FILE}"
echo "Completed at: $(date)" >> "${RESULTS_FILE}"

# Check performance targets
check_performance_targets

# Generate report
echo -e "\n${BLUE}=== GENERATING PERFORMANCE REPORT ===${NC}"
REPORT_FILE="${RESULTS_DIR}/performance_report_${TIMESTAMP}.md"

cat > "${REPORT_FILE}" << EOF
# QuDAG Performance Benchmark Report

**Generated:** $(date)
**Platform:** $(uname -a)
**Rust Version:** $(rustc --version)

## Performance Targets

QuDAG has the following performance requirements:

- **Sub-second consensus finality (99th percentile)**: Target < 1000ms
- **Message throughput**: Target > 10,000 messages/second per node
- **Linear scalability**: Performance should scale linearly with node count
- **Memory usage**: Target < 100MB for base node

## Benchmark Results Summary

- **Total Benchmarks:** ${total_benchmarks}
- **Passed:** ${passed_benchmarks}
- **Failed:** ${failed_benchmarks}
- **Success Rate:** $(( passed_benchmarks * 100 / total_benchmarks ))%

## Performance Analysis

EOF

# Add warning analysis to report
if [ ${warnings} -eq 0 ]; then
    echo "✅ **All performance targets met!**" >> "${REPORT_FILE}"
else
    echo "⚠️ **Performance Issues Found:**" >> "${REPORT_FILE}"
    echo "- ${latency_warnings} latency warnings" >> "${REPORT_FILE}"
    echo "- ${throughput_warnings} throughput warnings" >> "${REPORT_FILE}"
    echo "- ${memory_warnings} memory warnings" >> "${REPORT_FILE}"
fi

cat >> "${REPORT_FILE}" << EOF

## Detailed Results

See full benchmark results in: \`${RESULTS_FILE}\`

## Recommendations

EOF

if [ ${failed_benchmarks} -gt 0 ]; then
    echo "- Fix failed benchmarks before deployment" >> "${REPORT_FILE}"
fi

if [ ${latency_warnings} -gt 0 ]; then
    echo "- Optimize components with high latency" >> "${REPORT_FILE}"
fi

if [ ${throughput_warnings} -gt 0 ]; then
    echo "- Improve throughput in underperforming components" >> "${REPORT_FILE}"
fi

if [ ${memory_warnings} -gt 0 ]; then
    echo "- Reduce memory usage to meet 100MB target" >> "${REPORT_FILE}"
fi

echo -e "${GREEN}✓ Performance report generated: ${REPORT_FILE}${NC}"

# Exit with appropriate code
if [ ${failed_benchmarks} -eq 0 ] && [ ${warnings} -eq 0 ]; then
    echo -e "${GREEN}🎉 All benchmarks passed and performance targets met!${NC}"
    exit 0
elif [ ${failed_benchmarks} -eq 0 ]; then
    echo -e "${YELLOW}⚠ All benchmarks passed but some performance targets not met${NC}"
    exit 1
else
    echo -e "${RED}❌ Some benchmarks failed${NC}"
    exit 2
fi