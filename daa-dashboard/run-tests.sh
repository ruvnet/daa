#!/bin/bash

# DAA Dashboard Test Runner Script

echo "🧪 DAA Dashboard Test Suite"
echo "=========================="

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to run tests with specific configuration
run_test_suite() {
    local suite_name=$1
    local test_pattern=$2
    
    echo -e "\n${YELLOW}Running ${suite_name}...${NC}"
    
    if npm test -- ${test_pattern}; then
        echo -e "${GREEN}✓ ${suite_name} passed${NC}"
        return 0
    else
        echo -e "${RED}✗ ${suite_name} failed${NC}"
        return 1
    fi
}

# Main test execution
main() {
    local failed=0
    
    # Run different test suites
    run_test_suite "API Layer Tests" "src/api/**/*.test.ts" || ((failed++))
    run_test_suite "Component Tests" "src/components/**/*.test.tsx" || ((failed++))
    run_test_suite "Hook Tests" "src/hooks/**/*.test.tsx" || ((failed++))
    run_test_suite "Integration Tests" "src/__tests__/integration/**/*.test.tsx" || ((failed++))
    
    # Generate coverage report
    echo -e "\n${YELLOW}Generating coverage report...${NC}"
    npm run test:coverage
    
    # Summary
    echo -e "\n=========================="
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}✓ All tests passed!${NC}"
        
        # Show coverage summary
        echo -e "\n${YELLOW}Coverage Summary:${NC}"
        cat coverage/coverage-summary.json | jq '.total' 2>/dev/null || echo "Coverage report available in coverage/index.html"
    else
        echo -e "${RED}✗ ${failed} test suite(s) failed${NC}"
        exit 1
    fi
}

# Check if dependencies are installed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

# Run the main function
main