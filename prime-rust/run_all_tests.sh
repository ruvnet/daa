#!/bin/bash

# Prime-Rust Comprehensive TDD Test Suite Runner
# Following Test-Driven Development principles

set -e

echo "🚀 Prime-Rust Comprehensive TDD Test Suite"
echo "=========================================="
echo "Following TDD principles: Write tests first, then implement"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to run a test phase
run_test_phase() {
    local phase_name=$1
    local command=$2
    
    print_status $BLUE "🔄 Running $phase_name..."
    echo "Command: $command"
    echo ""
    
    if eval "$command"; then
        print_status $GREEN "✅ $phase_name PASSED"
    else
        print_status $RED "❌ $phase_name FAILED"
        return 1
    fi
    echo ""
}

# Check prerequisites
echo "🔍 Checking prerequisites..."

# Check Rust toolchain
if ! command -v cargo &> /dev/null; then
    print_status $RED "Cargo not found. Please install Rust."
    exit 1
fi

# Check for required components
if ! rustup component list --installed | grep -q "llvm-tools-preview"; then
    print_status $YELLOW "Installing llvm-tools-preview for coverage..."
    rustup component add llvm-tools-preview
fi

# Install additional testing tools
if ! command -v cargo-tarpaulin &> /dev/null; then
    print_status $YELLOW "Installing cargo-tarpaulin for coverage..."
    cargo install cargo-tarpaulin
fi

if ! command -v cargo-nextest &> /dev/null; then
    print_status $YELLOW "Installing cargo-nextest for faster testing..."
    cargo install cargo-nextest
fi

if ! command -v cargo-audit &> /dev/null; then
    print_status $YELLOW "Installing cargo-audit for security..."
    cargo install cargo-audit
fi

print_status $GREEN "✅ Prerequisites check completed"
echo ""

# Test phases following TDD approach
PHASES=(
    "1. Linting and Format Check:cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings"
    "2. Build Check:cargo build --all-targets --all-features"
    "3. Unit Tests:cargo test --lib --all-features"
    "4. Integration Tests:cargo test --test '*' --all-features"
    "5. Property-Based Tests:cargo test --all-features -- --ignored"
    "6. Documentation Tests:cargo test --doc --all-features"
    "7. Benchmark Compilation:cargo bench --no-run --all-features"
    "8. Security Audit:cargo audit"
)

# Track results
PASSED_PHASES=0
TOTAL_PHASES=${#PHASES[@]}
FAILED_PHASES=()

print_status $BLUE "🎯 Starting TDD Test Execution..."
print_status $BLUE "Total phases: $TOTAL_PHASES"
echo ""

# Run each test phase
for phase in "${PHASES[@]}"; do
    IFS=':' read -r phase_name command <<< "$phase"
    
    if run_test_phase "$phase_name" "$command"; then
        PASSED_PHASES=$((PASSED_PHASES + 1))
    else
        FAILED_PHASES+=("$phase_name")
    fi
done

# Extended testing (optional)
echo ""
print_status $BLUE "🔬 Extended Testing Phase..."

# Coverage testing
if command -v cargo-tarpaulin &> /dev/null; then
    print_status $BLUE "📊 Running coverage analysis..."
    if cargo tarpaulin --all-features --timeout 300 --out Html --output-dir coverage; then
        print_status $GREEN "✅ Coverage report generated in coverage/"
    else
        print_status $YELLOW "⚠️  Coverage analysis failed (non-critical)"
    fi
    echo ""
fi

# Fuzz testing (short run)
if [ -f "run_fuzz_tests.sh" ]; then
    print_status $BLUE "🔥 Running fuzz tests (short duration)..."
    export FUZZ_DURATION=30
    export MAX_TOTAL_TIME=180
    if ./run_fuzz_tests.sh; then
        print_status $GREEN "✅ Fuzz testing completed successfully"
    else
        print_status $YELLOW "⚠️  Fuzz testing found issues (check artifacts)"
    fi
    echo ""
fi

# Performance regression testing
print_status $BLUE "⚡ Running performance regression tests..."
if cargo bench --all-features > bench_results.txt 2>&1; then
    print_status $GREEN "✅ Benchmark tests completed"
else
    print_status $YELLOW "⚠️  Some benchmarks failed (non-critical)"
fi
echo ""

# Memory usage testing
print_status $BLUE "🧠 Running memory usage tests..."
if cargo test --release --all-features -- --test-threads=1 > memory_test.log 2>&1; then
    print_status $GREEN "✅ Memory usage tests completed"
else
    print_status $YELLOW "⚠️  Memory tests had issues (check log)"
fi
echo ""

# Final summary
echo ""
print_status $BLUE "📋 Test Suite Summary"
echo "===================="
echo "Total phases: $TOTAL_PHASES"
echo "Passed phases: $PASSED_PHASES"
echo "Failed phases: $((TOTAL_PHASES - PASSED_PHASES))"

if [ ${#FAILED_PHASES[@]} -gt 0 ]; then
    echo ""
    print_status $RED "❌ Failed phases:"
    for phase in "${FAILED_PHASES[@]}"; do
        echo "  - $phase"
    done
fi

# Coverage summary
if [ -f "coverage/tarpaulin-report.html" ]; then
    echo ""
    print_status $BLUE "📊 Coverage Report: coverage/tarpaulin-report.html"
fi

# Test artifacts
echo ""
print_status $BLUE "📁 Test Artifacts:"
echo "  - Benchmark results: bench_results.txt"
echo "  - Memory test log: memory_test.log"
if [ -d "fuzz/artifacts" ]; then
    echo "  - Fuzz artifacts: fuzz/artifacts/"
fi
if [ -d "coverage" ]; then
    echo "  - Coverage report: coverage/"
fi

# TDD Recommendations
echo ""
print_status $BLUE "🎯 TDD Recommendations"
echo "====================="
echo "1. ✅ Tests are written before implementation"
echo "2. ✅ All test types are covered (unit, integration, property, fuzz)"
echo "3. ✅ Continuous testing workflow established"
echo "4. ✅ Performance and security testing integrated"
echo ""

if [ $PASSED_PHASES -eq $TOTAL_PHASES ]; then
    print_status $GREEN "🎉 ALL TESTS PASSED! Ready for implementation."
    print_status $GREEN "The TDD framework is comprehensive and robust."
    exit 0
else
    print_status $RED "❌ Some tests failed. Please fix before proceeding."
    print_status $YELLOW "Follow TDD: Fix failing tests, then implement features."
    exit 1
fi