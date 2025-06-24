#!/bin/bash
set -euo pipefail

# QuDAG Testnet Deployment Verification Script
# Checks all deployed nodes for health, connectivity, and functionality

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NODES=("node1" "node2" "node3" "node4")
PROJECT_DIR=$(dirname "$0")
cd "$PROJECT_DIR"

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

# Get node URL
get_node_url() {
    local node=$1
    local app_name="qudag-testnet-$node"
    
    # Use IP addresses directly since DNS resolution is not working
    case "$node" in
        "node1")
            echo "109.105.222.156"  # Toronto
            ;;
        "node2")
            echo "149.248.199.86"  # Amsterdam
            ;;
        "node3")
            echo "149.248.218.16"  # Singapore
            ;;
        "node4")
            echo "137.66.62.149"  # San Francisco
            ;;
        *)
            echo ""
            ;;
    esac
}

# Test health endpoint
test_health_endpoint() {
    local node=$1
    local url=$(get_node_url "$node")
    
    if [ -z "$url" ]; then
        record_test "$node Health" "FAIL" "Could not determine URL"
        return 1
    fi
    
    log_info "Testing health endpoint for $node..."
    
    if response=$(curl -sf "http://$url/health" 2>/dev/null); then
        local synced=$(echo "$response" | jq -r '.details.synced // .synced // false')
        local peers=$(echo "$response" | jq -r '.details.peers // .peers // 0')
        local height=$(echo "$response" | jq -r '.details.height // .height // 0')
        
        record_test "$node Health" "PASS" "synced=$synced, peers=$peers, height=$height"
        return 0
    else
        record_test "$node Health" "FAIL" "Endpoint not responding"
        return 1
    fi
}

# Test metrics endpoint
test_metrics_endpoint() {
    local node=$1
    local url=$(get_node_url "$node")
    
    if [ -z "$url" ]; then
        record_test "$node Metrics" "FAIL" "Could not determine URL"
        return 1
    fi
    
    log_info "Testing metrics endpoint for $node..."
    
    # Metrics are on port 9090
    if response=$(curl -sf "http://$url:9090/metrics" 2>/dev/null); then
        local metric_count=$(echo "$response" | grep -c "^qudag_" || echo "0")
        record_test "$node Metrics" "PASS" "Found $metric_count QuDAG metrics"
        return 0
    else
        # Try without port (if behind proxy)
        if response=$(curl -sf "http://$url/metrics" 2>/dev/null); then
            local metric_count=$(echo "$response" | grep -c "^qudag_" || echo "0")
            record_test "$node Metrics" "PASS" "Found $metric_count QuDAG metrics (via proxy)"
            return 0
        else
            record_test "$node Metrics" "FAIL" "Endpoint not responding"
            return 1
        fi
    fi
}

# Test P2P connectivity
test_p2p_connectivity() {
    log_section "P2P Connectivity Matrix"
    
    local connected_nodes=0
    local total_peer_connections=0
    
    for node in "${NODES[@]}"; do
        local url=$(get_node_url "$node")
        
        if [ -n "$url" ]; then
            if health_data=$(curl -sf "http://$url/health" 2>/dev/null); then
                local peer_count=$(echo "$health_data" | jq -r '.details.peers // .peers // 0')
                
                if [ "$peer_count" -gt 0 ]; then
                    connected_nodes=$((connected_nodes + 1))
                    total_peer_connections=$((total_peer_connections + peer_count))
                    record_test "$node P2P" "PASS" "Connected to $peer_count peers"
                else
                    record_test "$node P2P" "FAIL" "No peers connected"
                fi
            else
                record_test "$node P2P" "FAIL" "Could not retrieve peer info"
            fi
        fi
    done
    
    # Overall P2P health
    if [ "$connected_nodes" -eq "${#NODES[@]}" ]; then
        record_test "P2P Network" "PASS" "All nodes connected"
    else
        record_test "P2P Network" "FAIL" "Only $connected_nodes/${#NODES[@]} nodes have peers"
    fi
}

# Test Exchange Server endpoints (if applicable)
test_exchange_endpoints() {
    log_section "Exchange Server Endpoints"
    
    # Test on node1 (bootstrap node) which might run exchange server
    local url=$(get_node_url "node1")
    
    if [ -n "$url" ]; then
        # Test exchange info endpoint
        if curl -sf "http://$url/api/v1/exchange/info" >/dev/null 2>&1; then
            record_test "Exchange Info API" "PASS" "Endpoint responding"
        else
            record_test "Exchange Info API" "FAIL" "Endpoint not available"
        fi
        
        # Test orderbook endpoint
        if curl -sf "http://$url/api/v1/exchange/orderbook/QUDAG-USDT" >/dev/null 2>&1; then
            record_test "Exchange Orderbook API" "PASS" "Endpoint responding"
        else
            record_test "Exchange Orderbook API" "FAIL" "Endpoint not available"
        fi
    else
        record_test "Exchange Server" "FAIL" "Could not determine URL"
    fi
}

# Test consensus synchronization
test_consensus_sync() {
    log_section "Consensus Synchronization"
    
    local heights=()
    local synced_nodes=0
    
    for node in "${NODES[@]}"; do
        local url=$(get_node_url "$node")
        
        if [ -n "$url" ]; then
            if health_data=$(curl -sf "http://$url/health" 2>/dev/null); then
                local height=$(echo "$health_data" | jq -r '.details.height // .height // 0')
                local synced=$(echo "$health_data" | jq -r '.details.synced // .synced // false')
                
                heights+=("$height")
                
                if [ "$synced" = "true" ]; then
                    synced_nodes=$((synced_nodes + 1))
                fi
            fi
        fi
    done
    
    # Check if all heights are similar (within 2 blocks)
    if [ ${#heights[@]} -gt 1 ]; then
        local min_height=${heights[0]}
        local max_height=${heights[0]}
        
        for height in "${heights[@]}"; do
            if [ "$height" -lt "$min_height" ]; then
                min_height=$height
            fi
            if [ "$height" -gt "$max_height" ]; then
                max_height=$height
            fi
        done
        
        local height_diff=$((max_height - min_height))
        
        if [ "$height_diff" -le 2 ]; then
            record_test "Consensus Sync" "PASS" "All nodes within 2 blocks (heights: ${heights[*]})"
        else
            record_test "Consensus Sync" "FAIL" "Height difference: $height_diff blocks"
        fi
    else
        record_test "Consensus Sync" "FAIL" "Could not retrieve block heights"
    fi
    
    # Sync status
    if [ "$synced_nodes" -eq "${#NODES[@]}" ]; then
        record_test "Sync Status" "PASS" "All nodes report synced state"
    else
        record_test "Sync Status" "FAIL" "Only $synced_nodes/${#NODES[@]} nodes synced"
    fi
}

# Test TLS/SSL certificates
test_tls_certificates() {
    log_section "TLS Certificate Verification"
    
    for node in "${NODES[@]}"; do
        local url=$(get_node_url "$node")
        
        if [ -n "$url" ]; then
            # Skip TLS check for IP addresses
            record_test "$node TLS" "SKIP" "Using direct IP access"
        else
            record_test "$node TLS" "FAIL" "Could not determine URL"
        fi
    done
}

# Performance check
test_performance() {
    log_section "Performance Metrics"
    
    for node in "${NODES[@]}"; do
        local url=$(get_node_url "$node")
        
        if [ -n "$url" ]; then
            # Measure health endpoint response time
            local start_time=$(date +%s%N)
            if curl -sf "http://$url/health" >/dev/null 2>&1; then
                local end_time=$(date +%s%N)
                local response_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
                
                if [ "$response_time" -lt 1000 ]; then
                    record_test "$node Response Time" "PASS" "${response_time}ms"
                else
                    record_test "$node Response Time" "FAIL" "${response_time}ms (>1000ms)"
                fi
            else
                record_test "$node Response Time" "FAIL" "No response"
            fi
        fi
    done
}

# Generate deployment report
generate_report() {
    log_section "Deployment Verification Report"
    
    echo "Timestamp: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo
    echo "Test Results:"
    for result in "${TEST_RESULTS[@]}"; do
        echo "  $result"
    done
    
    # Node URLs
    echo
    echo "Node URLs:"
    for node in "${NODES[@]}"; do
        local url=$(get_node_url "$node")
        if [ -n "$url" ]; then
            echo "  - $node: http://$url"
        fi
    done
    
    # Recommendations
    echo
    echo "Recommendations:"
    if [ "$FAILED_TESTS" -eq 0 ]; then
        echo "  ✓ Deployment is healthy and fully operational"
    else
        echo "  ⚠ Review failed tests and take corrective action"
        
        if [[ " ${TEST_RESULTS[*]} " =~ "P2P" ]] && [[ " ${TEST_RESULTS[*]} " =~ "✗" ]]; then
            echo "  - Check firewall rules for P2P ports (4001)"
            echo "  - Verify bootstrap peer addresses"
        fi
        
        if [[ " ${TEST_RESULTS[*]} " =~ "Health" ]] && [[ " ${TEST_RESULTS[*]} " =~ "✗" ]]; then
            echo "  - Check application logs with: fly logs --app qudag-testnet-nodeX"
            echo "  - Verify node configuration files"
        fi
        
        if [[ " ${TEST_RESULTS[*]} " =~ "Metrics" ]] && [[ " ${TEST_RESULTS[*]} " =~ "✗" ]]; then
            echo "  - Ensure metrics port (9090) is properly exposed"
            echo "  - Check Prometheus scrape configuration"
        fi
    fi
}

# Main execution
main() {
    log_info "QuDAG Testnet Deployment Verification"
    log_info "Starting comprehensive deployment checks..."
    
    # Test all health endpoints
    log_section "Health Endpoint Tests"
    for node in "${NODES[@]}"; do
        test_health_endpoint "$node"
    done
    
    # Test all metrics endpoints
    log_section "Metrics Endpoint Tests"
    for node in "${NODES[@]}"; do
        test_metrics_endpoint "$node"
    done
    
    # Test P2P connectivity
    test_p2p_connectivity
    
    # Test Exchange endpoints
    test_exchange_endpoints
    
    # Test consensus synchronization
    test_consensus_sync
    
    # Test TLS certificates
    test_tls_certificates
    
    # Performance tests
    test_performance
    
    # Generate final report
    generate_report
    
    # Save report to file
    local report_file="deployment-report-$(date +%Y%m%d-%H%M%S).txt"
    {
        echo "QuDAG Testnet Deployment Verification Report"
        echo "==========================================="
        generate_report
    } > "$report_file"
    
    log_info "Report saved to: $report_file"
    
    # Exit with appropriate code
    if [ "$FAILED_TESTS" -eq 0 ]; then
        log_success "All verification tests passed!"
        exit 0
    else
        log_error "Some verification tests failed"
        exit 1
    fi
}

# Run main function
main "$@"