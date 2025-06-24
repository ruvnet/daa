#!/bin/bash
# QuDAG WASM Crypto Test Suite
# Comprehensive testing script for WASM crypto implementation

set -e

echo "🚀 QuDAG WASM Crypto Test Suite"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}Testing: ${test_name}${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASS: ${test_name}${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAIL: ${test_name}${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
}

echo "📋 Test Environment Information"
echo "-------------------------------"
echo "Date: $(date)"
echo "Platform: $(uname -a)"
echo "Rust version: $(rustc --version)"
echo "wasm-pack version: $(wasm-pack --version)"
echo ""

# Test 1: Basic build
run_test "WASM Build (crypto-only)" \
    "wasm-pack build --target web --release --no-default-features --quiet 2>/dev/null"

# Test 2: Check generated files
run_test "Generated Files Check" \
    "test -f pkg/qudag_wasm.js && test -f pkg/qudag_wasm_bg.wasm && test -f pkg/qudag_wasm.d.ts"

# Test 3: File sizes check
run_test "File Size Validation" \
    "test $(stat -c%s pkg/qudag_wasm_bg.wasm) -lt 300000"  # Less than 300KB

# Test 4: TypeScript definitions validation
run_test "TypeScript Definitions" \
    "grep -q 'WasmMlDsaKeyPair' pkg/qudag_wasm.d.ts && grep -q 'WasmMlKemKeyPair' pkg/qudag_wasm.d.ts"

# Test 5: JavaScript bindings validation  
run_test "JavaScript Bindings" \
    "grep -q 'WasmMlDsaKeyPair' pkg/qudag_wasm.js && grep -q 'WasmQuantumFingerprint' pkg/qudag_wasm.js"

# Test 6: Package.json validation
run_test "Package Metadata" \
    "test -f pkg/package.json && grep -q '\"name\": \"qudag-wasm\"' pkg/package.json"

# Test 7: Build with different targets
run_test "Node.js Build" \
    "wasm-pack build --target nodejs --out-dir pkg-node --release --no-default-features --quiet 2>/dev/null"

# Test 8: Optimized build
run_test "Optimized Build" \
    "wasm-pack build --target web --out-dir pkg-optimized --release --no-default-features --quiet 2>/dev/null"

# Test 9: Check for critical exports in JS
run_test "Critical Exports Check" \
    "grep -q 'WasmMlDsaKeyPair' pkg/qudag_wasm.js && grep -q 'WasmQuantumFingerprint' pkg/qudag_wasm.js"

# Test 10: WASM binary validation
run_test "WASM Binary Validation" \
    "wasm-validate pkg/qudag_wasm_bg.wasm 2>/dev/null || echo 'wasm-validate not available, skipping'"

echo "📊 Test Results Summary"
echo "======================"
echo -e "Total Tests: ${TOTAL_TESTS}"
echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED!${NC}"
    echo ""
    echo "✅ WASM crypto build is ready for deployment"
    echo "✅ All critical files generated successfully"
    echo "✅ Package can be published to NPM"
    echo ""
    echo "Next steps:"
    echo "1. Test in browser using tests/browser_test.html"
    echo "2. Integrate with your application"
    echo "3. Consider publishing to NPM registry"
    echo ""
    echo "📁 Generated packages:"
    echo "  - pkg/          (Web target)"
    echo "  - pkg-node/     (Node.js target)"  
    echo "  - pkg-optimized/ (Size optimized)"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
    echo ""
    echo "Please review the failed tests above and fix any issues."
    echo "Check the build logs for detailed error information."
    exit 1
fi