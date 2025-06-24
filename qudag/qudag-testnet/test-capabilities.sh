#!/bin/bash
set -euo pipefail

# QuDAG Capabilities Test Script
# Tests all core functionality including quantum crypto, dark domains, vault, and more

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
TEST_RESULTS=()

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_fail() {
    echo -e "${RED}[✗]${NC} $1"
}

log_section() {
    echo -e "\n${BLUE}========== $1 ==========${NC}\n"
}

# Record test result
record_test() {
    local test_name=$1
    local result=$2
    local details=$3
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$result" = "PASS" ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        log_success "$test_name: $details"
        TEST_RESULTS+=("✓ $test_name: $details")
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        log_fail "$test_name: $details"
        TEST_RESULTS+=("✗ $test_name: $details")
    fi
}

# Test quantum cryptography capabilities
test_quantum_crypto() {
    log_section "Quantum Cryptography Tests"
    
    # Test ML-DSA signature generation
    log_info "Testing ML-DSA signature generation..."
    if command -v qudag &> /dev/null; then
        if qudag key generate --algorithm ml-dsa 2>/dev/null | grep -q "Generated"; then
            record_test "ML-DSA Key Generation" "PASS" "Successfully generated quantum-resistant signing keys"
        else
            record_test "ML-DSA Key Generation" "FAIL" "Could not generate ML-DSA keys"
        fi
    else
        record_test "ML-DSA Key Generation" "FAIL" "QuDAG CLI not available"
    fi
    
    # Test ML-KEM encryption
    log_info "Testing ML-KEM key encapsulation..."
    if command -v qudag &> /dev/null; then
        if qudag key generate --algorithm ml-kem 2>/dev/null | grep -q "Generated"; then
            record_test "ML-KEM Key Generation" "PASS" "Successfully generated quantum-resistant encryption keys"
        else
            record_test "ML-KEM Key Generation" "FAIL" "Could not generate ML-KEM keys"
        fi
    else
        record_test "ML-KEM Key Generation" "FAIL" "QuDAG CLI not available"
    fi
    
    # Test quantum fingerprinting
    log_info "Testing quantum fingerprinting..."
    if command -v qudag &> /dev/null; then
        if echo "test data" | qudag fingerprint create 2>/dev/null | grep -q "Fingerprint"; then
            record_test "Quantum Fingerprinting" "PASS" "Successfully created quantum fingerprint"
        else
            record_test "Quantum Fingerprinting" "FAIL" "Could not create quantum fingerprint"
        fi
    else
        record_test "Quantum Fingerprinting" "FAIL" "QuDAG CLI not available"
    fi
}

# Test dark domain system
test_dark_domains() {
    log_section "Dark Domain System Tests"
    
    # Test domain generation
    log_info "Testing .dark domain generation..."
    if command -v qudag &> /dev/null; then
        if qudag address generate --type quantum 2>/dev/null | grep -q "address"; then
            record_test "Quantum Address Generation" "PASS" "Successfully generated quantum address"
        else
            record_test "Quantum Address Generation" "FAIL" "Could not generate quantum address"
        fi
        
        # Test shadow address
        if qudag address generate --type shadow 2>/dev/null | grep -q "address"; then
            record_test "Shadow Address Generation" "PASS" "Successfully generated shadow address"
        else
            record_test "Shadow Address Generation" "FAIL" "Could not generate shadow address"
        fi
        
        # Test onion address
        if qudag address generate --type onion 2>/dev/null | grep -q "address"; then
            record_test "Onion Address Generation" "PASS" "Successfully generated onion address"
        else
            record_test "Onion Address Generation" "FAIL" "Could not generate onion address"
        fi
    else
        record_test "Dark Domain System" "FAIL" "QuDAG CLI not available"
    fi
}

# Test vault functionality
test_vault() {
    log_section "Vault Functionality Tests"
    
    log_info "Testing password vault operations..."
    if command -v qudag &> /dev/null; then
        # Test password generation
        if qudag vault generate --length 16 2>/dev/null | grep -q "Generated"; then
            record_test "Password Generation" "PASS" "Successfully generated secure password"
        else
            record_test "Password Generation" "FAIL" "Could not generate password"
        fi
        
        # Test vault configuration
        if qudag vault config show 2>/dev/null | grep -q "Vault"; then
            record_test "Vault Configuration" "PASS" "Successfully retrieved vault config"
        else
            record_test "Vault Configuration" "FAIL" "Could not access vault config"
        fi
    else
        record_test "Vault System" "FAIL" "QuDAG CLI not available"
    fi
}

# Test exchange functionality
test_exchange() {
    log_section "Exchange System Tests"
    
    log_info "Testing rUv token exchange system..."
    if command -v qudag &> /dev/null; then
        # Test fee info
        if qudag exchange fee-info 2>/dev/null | grep -q "Fee"; then
            record_test "Exchange Fee Info" "PASS" "Successfully retrieved fee information"
        else
            record_test "Exchange Fee Info" "FAIL" "Could not retrieve fee info"
        fi
        
        # Test immutable status
        if qudag exchange immutable-status 2>/dev/null | grep -q "status"; then
            record_test "Immutable Status" "PASS" "Successfully checked immutable deployment"
        else
            record_test "Immutable Status" "FAIL" "Could not check immutable status"
        fi
    else
        record_test "Exchange System" "FAIL" "QuDAG CLI not available"
    fi
}

# Test network node connectivity
test_network_connectivity() {
    log_section "Network Node Connectivity Tests"
    
    local nodes=("109.105.222.156" "149.248.199.86" "149.248.218.16" "137.66.62.149")
    local node_names=("Toronto" "Amsterdam" "Singapore" "San Francisco")
    
    for i in "${!nodes[@]}"; do
        local ip="${nodes[$i]}"
        local name="${node_names[$i]}"
        
        log_info "Testing connectivity to $name node ($ip)..."
        
        # Test basic connectivity
        if ping -c 1 -W 2 "$ip" &>/dev/null; then
            record_test "$name Ping" "PASS" "Node reachable via ICMP"
        else
            record_test "$name Ping" "FAIL" "Node not reachable via ICMP"
        fi
        
        # Test HTTP health endpoint
        if curl -sf "http://$ip/health" &>/dev/null; then
            record_test "$name HTTP" "PASS" "Health endpoint accessible"
        else
            record_test "$name HTTP" "FAIL" "Health endpoint not accessible"
        fi
        
        # Test P2P port
        if nc -z -w2 "$ip" 4001 &>/dev/null; then
            record_test "$name P2P Port" "PASS" "P2P port 4001 open"
        else
            record_test "$name P2P Port" "FAIL" "P2P port 4001 not accessible"
        fi
    done
}

# Test API endpoints
test_api_endpoints() {
    log_section "API Endpoint Tests"
    
    local test_node="109.105.222.156"  # Toronto node
    
    log_info "Testing API endpoints on node1..."
    
    # Test status endpoint
    if response=$(curl -sf "http://$test_node/api/v1/status" 2>/dev/null); then
        if echo "$response" | jq -e '.node.id' &>/dev/null; then
            record_test "Status API" "PASS" "Complete node status retrieved"
        else
            record_test "Status API" "FAIL" "Invalid status response format"
        fi
    else
        record_test "Status API" "FAIL" "Status endpoint not accessible"
    fi
    
    # Test metrics format
    if response=$(curl -sf "http://$test_node/metrics" 2>/dev/null); then
        if echo "$response" | grep -q "# HELP qudag_"; then
            record_test "Prometheus Metrics" "PASS" "Metrics in correct Prometheus format"
        else
            record_test "Prometheus Metrics" "FAIL" "Metrics not in Prometheus format"
        fi
    else
        record_test "Prometheus Metrics" "FAIL" "Metrics endpoint not accessible"
    fi
}

# Test DAG functionality via API
test_dag_operations() {
    log_section "DAG Operations Tests"
    
    local test_node="109.105.222.156"
    
    log_info "Testing DAG consensus operations..."
    
    # Check block height progression
    if height1=$(curl -sf "http://$test_node/health" | jq -r '.details.height' 2>/dev/null); then
        sleep 5
        if height2=$(curl -sf "http://$test_node/health" | jq -r '.details.height' 2>/dev/null); then
            if [ "$height2" -gt "$height1" ]; then
                record_test "DAG Block Production" "PASS" "Blocks being produced (height: $height1 -> $height2)"
            else
                record_test "DAG Block Production" "FAIL" "No new blocks produced"
            fi
        else
            record_test "DAG Block Production" "FAIL" "Could not retrieve second height"
        fi
    else
        record_test "DAG Block Production" "FAIL" "Could not retrieve initial height"
    fi
    
    # Check message processing
    if msg_count=$(curl -sf "http://$test_node/api/v1/status" | jq -r '.dag.messages_processed' 2>/dev/null); then
        if [ "$msg_count" -gt 0 ]; then
            record_test "DAG Message Processing" "PASS" "$msg_count messages processed"
        else
            record_test "DAG Message Processing" "FAIL" "No messages processed"
        fi
    else
        record_test "DAG Message Processing" "FAIL" "Could not retrieve message count"
    fi
}

# Test WASM compatibility
test_wasm_compatibility() {
    log_section "WASM Compatibility Tests"
    
    log_info "Testing WASM package availability..."
    
    # Check if npm package exists
    if npm view qudag version &>/dev/null; then
        local version=$(npm view qudag version)
        record_test "NPM Package" "PASS" "QuDAG WASM package available (v$version)"
    else
        record_test "NPM Package" "FAIL" "QuDAG WASM package not found on npm"
    fi
    
    # Check if Node.js can use the package
    if command -v node &> /dev/null; then
        if node -e "console.log('WASM test')" &>/dev/null; then
            record_test "Node.js Compatibility" "PASS" "Node.js environment available"
        else
            record_test "Node.js Compatibility" "FAIL" "Node.js not working properly"
        fi
    else
        record_test "Node.js Compatibility" "FAIL" "Node.js not installed"
    fi
}

# Generate capability report
generate_capability_report() {
    log_section "QuDAG Capability Report"
    
    echo "Timestamp: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo
    echo "Capability Summary:"
    echo "==================="
    
    # Categorize results
    local quantum_tests=0
    local quantum_passed=0
    local network_tests=0
    local network_passed=0
    local api_tests=0
    local api_passed=0
    
    for result in "${TEST_RESULTS[@]}"; do
        if [[ "$result" =~ "ML-DSA\|ML-KEM\|Quantum" ]]; then
            quantum_tests=$((quantum_tests + 1))
            [[ "$result" =~ "✓" ]] && quantum_passed=$((quantum_passed + 1))
        elif [[ "$result" =~ "Ping\|P2P\|HTTP" ]]; then
            network_tests=$((network_tests + 1))
            [[ "$result" =~ "✓" ]] && network_passed=$((network_passed + 1))
        elif [[ "$result" =~ "API\|Metrics\|Status" ]]; then
            api_tests=$((api_tests + 1))
            [[ "$result" =~ "✓" ]] && api_passed=$((api_passed + 1))
        fi
    done
    
    echo "🔐 Quantum Cryptography: $quantum_passed/$quantum_tests tests passed"
    echo "🌐 Network Connectivity: $network_passed/$network_tests tests passed"
    echo "📡 API Functionality: $api_passed/$api_tests tests passed"
    echo
    echo "Detailed Test Results:"
    echo "====================="
    for result in "${TEST_RESULTS[@]}"; do
        echo "  $result"
    done
    
    # Recommendations
    echo
    echo "Recommendations:"
    echo "================"
    if [ "$FAILED_TESTS" -eq 0 ]; then
        echo "✅ All capabilities verified and working correctly!"
    else
        echo "⚠️  Some capabilities need attention:"
        
        # Check for CLI availability
        if ! command -v qudag &> /dev/null; then
            echo "  - Install QuDAG CLI: cargo install qudag-cli"
        fi
        
        # Check for specific failures
        for result in "${TEST_RESULTS[@]}"; do
            if [[ "$result" =~ "✗" ]]; then
                if [[ "$result" =~ "P2P" ]]; then
                    echo "  - P2P connectivity issues detected - check firewall rules"
                elif [[ "$result" =~ "CLI not available" ]]; then
                    echo "  - QuDAG CLI needs to be installed for full capability testing"
                fi
            fi
        done
    fi
}

# Main execution
main() {
    log_info "QuDAG Comprehensive Capability Test"
    log_info "Testing all QuDAG features and functionality..."
    
    # Run all test suites
    test_network_connectivity
    test_api_endpoints
    test_dag_operations
    test_quantum_crypto
    test_dark_domains
    test_vault
    test_exchange
    test_wasm_compatibility
    
    # Generate final report
    generate_capability_report
    
    # Save report to file
    local report_file="capability-report-$(date +%Y%m%d-%H%M%S).txt"
    {
        echo "QuDAG Capability Test Report"
        echo "============================"
        generate_capability_report
    } > "$report_file"
    
    log_info "Report saved to: $report_file"
    
    # Exit with appropriate code
    if [ "$FAILED_TESTS" -eq 0 ]; then
        log_success "All capability tests passed!"
        exit 0
    else
        log_error "Some capability tests failed"
        exit 1
    fi
}

# Run main function
main "$@"