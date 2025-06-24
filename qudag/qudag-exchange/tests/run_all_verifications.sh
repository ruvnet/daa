#!/bin/bash
# QuDAG Exchange Comprehensive Verification Suite
# Run all verification tests and generate reports

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="$SCRIPT_DIR/verification_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="$RESULTS_DIR/verification_report_$TIMESTAMP.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$RESULTS_DIR"

echo "=== QuDAG Exchange Verification Suite ==="
echo "Timestamp: $(date)"
echo "Results will be saved to: $RESULTS_DIR"
echo

# Initialize report
cat > "$REPORT_FILE" << EOF
# QuDAG Exchange Verification Results
Generated: $(date)

## Summary

EOF

# Function to run a test suite
run_test_suite() {
    local name=$1
    local command=$2
    local description=$3
    
    echo -e "${YELLOW}Running $name...${NC}"
    echo "### $name" >> "$REPORT_FILE"
    echo "$description" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    
    if eval "$command" >> "$REPORT_FILE" 2>&1; then
        echo -e "${GREEN}✓ $name completed successfully${NC}"
        echo "Status: PASSED" >> "$REPORT_FILE"
    else
        echo -e "${RED}✗ $name failed${NC}"
        echo "Status: FAILED" >> "$REPORT_FILE"
    fi
    
    echo '```' >> "$REPORT_FILE"
    echo >> "$REPORT_FILE"
}

# Check if we're in the right directory
if [ ! -f "$PROJECT_ROOT/qudag-exchange/Cargo.toml" ]; then
    echo -e "${RED}Error: Not in QuDAG project root${NC}"
    exit 1
fi

cd "$PROJECT_ROOT/qudag-exchange"

# 1. Run Fuzzing Tests (short duration for CI)
if command -v cargo-fuzz &> /dev/null; then
    run_test_suite "Fuzzing Tests" \
        "cd fuzz && FUZZ_TIME=10 ./run_verification_fuzzing.sh" \
        "Fuzzing all exchange components for crashes and invariant violations"
else
    echo -e "${YELLOW}Skipping fuzzing tests (cargo-fuzz not installed)${NC}"
fi

# 2. Run Property-Based Tests
run_test_suite "Property-Based Tests" \
    "cargo test --test ledger_properties -- --nocapture" \
    "Testing ledger invariants with proptest"

# 3. Run Model Checking
run_test_suite "Model Checking" \
    "cargo test --test consensus_model -- --nocapture" \
    "Exhaustive state space exploration for consensus"

# 4. Run Crypto Verification
run_test_suite "Cryptographic Verification" \
    "cargo test --test test_vectors -- --nocapture" \
    "Verifying crypto implementations against test vectors"

# 5. Run Standard Tests
run_test_suite "Unit Tests" \
    "cargo test --lib" \
    "Standard unit tests for all modules"

# 6. Run Integration Tests
run_test_suite "Integration Tests" \
    "cargo test --test '*' -- --nocapture" \
    "Integration tests across modules"

# 7. Check for Security Issues
echo -e "${YELLOW}Running security checks...${NC}"
echo "### Security Audit" >> "$REPORT_FILE"

# Check for unsafe code
echo "#### Unsafe Code Usage" >> "$REPORT_FILE"
if grep -r "unsafe" --include="*.rs" . | grep -v "forbid(unsafe_code)"; then
    echo "Found unsafe code usage:" >> "$REPORT_FILE"
    grep -r "unsafe" --include="*.rs" . | grep -v "forbid(unsafe_code)" >> "$REPORT_FILE"
else
    echo "✓ No unsafe code found" >> "$REPORT_FILE"
fi
echo >> "$REPORT_FILE"

# Check dependencies
if command -v cargo-audit &> /dev/null; then
    echo "#### Dependency Audit" >> "$REPORT_FILE"
    cargo audit >> "$REPORT_FILE" 2>&1 || true
else
    echo "cargo-audit not installed, skipping dependency audit" >> "$REPORT_FILE"
fi

# 8. Generate Coverage Report (if available)
if command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}Generating coverage report...${NC}"
    cargo tarpaulin --out Html --output-dir "$RESULTS_DIR/coverage" || true
fi

# 9. Generate Summary
echo "## Overall Results" >> "$REPORT_FILE"
echo >> "$REPORT_FILE"

passed=$(grep -c "Status: PASSED" "$REPORT_FILE" || true)
failed=$(grep -c "Status: FAILED" "$REPORT_FILE" || true)
total=$((passed + failed))

echo "- Total test suites: $total" >> "$REPORT_FILE"
echo "- Passed: $passed" >> "$REPORT_FILE"
echo "- Failed: $failed" >> "$REPORT_FILE"
echo >> "$REPORT_FILE"

if [ "$failed" -eq 0 ]; then
    echo "✅ All verification tests passed!" >> "$REPORT_FILE"
    STATUS="SUCCESS"
else
    echo "❌ Some verification tests failed. Please review the results above." >> "$REPORT_FILE"
    STATUS="FAILURE"
fi

# 10. Save summary to JSON for Memory system
cat > "$RESULTS_DIR/verification_summary.json" << EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "status": "$STATUS",
    "total_suites": $total,
    "passed": $passed,
    "failed": $failed,
    "report_file": "$REPORT_FILE",
    "coverage_dir": "$RESULTS_DIR/coverage"
}
EOF

echo
echo "=== Verification Complete ==="
echo "Status: $STATUS"
echo "Full report: $REPORT_FILE"
echo "Summary: $RESULTS_DIR/verification_summary.json"

# Exit with appropriate code
[ "$failed" -eq 0 ] && exit 0 || exit 1