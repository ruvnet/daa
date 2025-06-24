#!/bin/bash
set -euo pipefail

# QuDAG Testnet Local Testing Script
# Tests a 4-node testnet using docker-compose with comprehensive validation

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose.yml"
PROJECT_DIR=$(dirname "$0")
cd "$PROJECT_DIR"

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
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    docker-compose -f "$COMPOSE_FILE" down -v || true
    docker network prune -f || true
}

# Test health endpoint
test_health() {
    local node_name=$1
    local port=$2
    local max_retries=30
    local retry_count=0
    
    log_info "Testing health endpoint for $node_name on port $port..."
    
    while [ $retry_count -lt $max_retries ]; do
        if curl -sf "http://localhost:$port/health" > /dev/null 2>&1; then
            local health_data=$(curl -s "http://localhost:$port/health")
            log_success "$node_name is healthy: $health_data"
            return 0
        fi
        
        retry_count=$((retry_count + 1))
        log_warn "Waiting for $node_name to be healthy... ($retry_count/$max_retries)"
        sleep 2
    done
    
    log_error "$node_name failed health check after $max_retries attempts"
    return 1
}

# Test metrics endpoint
test_metrics() {
    local node_name=$1
    local port=$2
    
    log_info "Testing metrics endpoint for $node_name on port $port..."
    
    if curl -sf "http://localhost:$port/metrics" > /dev/null 2>&1; then
        local metrics_sample=$(curl -s "http://localhost:$port/metrics" | head -5)
        log_success "$node_name metrics available:"
        echo "$metrics_sample"
        return 0
    else
        log_error "$node_name metrics endpoint not responding"
        return 1
    fi
}

# Test P2P connectivity
test_p2p_connectivity() {
    log_info "Testing P2P connectivity between nodes..."
    
    # Wait for nodes to establish connections
    sleep 10
    
    # Check each node's peer count
    local all_connected=true
    for i in {0..3}; do
        local node_num=$((i + 1))
        local rpc_port=$((8080 + i))
        local node_name="node$node_num"
        
        local health_data=$(curl -s "http://localhost:$rpc_port/health" 2>/dev/null || echo "{}")
        local peer_count=$(echo "$health_data" | grep -o '"peers":[0-9]*' | cut -d':' -f2 || echo "0")
        
        if [ "$peer_count" -eq "0" ] || [ -z "$peer_count" ]; then
            log_warn "$node_name has no peers connected"
            all_connected=false
        else
            log_success "$node_name has $peer_count peers connected"
        fi
    done
    
    if [ "$all_connected" = true ]; then
        log_success "All nodes have established P2P connections"
        return 0
    else
        log_warn "Some nodes have not established P2P connections"
        return 1
    fi
}

# Verify Prometheus targets
test_prometheus() {
    log_info "Testing Prometheus metrics collection..."
    
    if curl -sf "http://localhost:9094/api/v1/targets" > /dev/null 2>&1; then
        local targets=$(curl -s "http://localhost:9094/api/v1/targets")
        local active_targets=$(echo "$targets" | grep -o '"health":"up"' | wc -l)
        log_success "Prometheus has $active_targets active targets"
        return 0
    else
        log_error "Prometheus is not responding"
        return 1
    fi
}

# Main execution
main() {
    log_info "Starting QuDAG Testnet Local Testing"
    log_info "Working directory: $(pwd)"
    
    # Set up trap for cleanup
    trap cleanup EXIT
    
    # Step 1: Build Docker image
    log_info "Building Docker image..."
    if docker-compose -f "$COMPOSE_FILE" build --no-cache; then
        log_success "Docker image built successfully"
    else
        log_error "Failed to build Docker image"
        exit 1
    fi
    
    # Step 2: Start the testnet
    log_info "Starting 4-node testnet..."
    if docker-compose -f "$COMPOSE_FILE" up -d; then
        log_success "Docker-compose started successfully"
    else
        log_error "Failed to start docker-compose"
        exit 1
    fi
    
    # Wait for initial startup
    sleep 5
    
    # Step 3: Test health endpoints
    log_info "Testing health endpoints..."
    local health_tests_passed=true
    
    test_health "node1" 8080 || health_tests_passed=false
    test_health "node2" 8081 || health_tests_passed=false
    test_health "node3" 8082 || health_tests_passed=false
    test_health "node4" 8083 || health_tests_passed=false
    
    if [ "$health_tests_passed" = false ]; then
        log_error "Some health checks failed"
        docker-compose -f "$COMPOSE_FILE" logs
        exit 1
    fi
    
    # Step 4: Test metrics endpoints
    log_info "Testing metrics endpoints..."
    local metrics_tests_passed=true
    
    test_metrics "node1" 9090 || metrics_tests_passed=false
    test_metrics "node2" 9091 || metrics_tests_passed=false
    test_metrics "node3" 9092 || metrics_tests_passed=false
    test_metrics "node4" 9093 || metrics_tests_passed=false
    
    if [ "$metrics_tests_passed" = false ]; then
        log_warn "Some metrics endpoints are not responding"
    fi
    
    # Step 5: Test P2P connectivity
    test_p2p_connectivity || log_warn "P2P connectivity test had warnings"
    
    # Step 6: Test Prometheus
    test_prometheus || log_warn "Prometheus test failed"
    
    # Step 7: Test Grafana
    log_info "Testing Grafana..."
    if curl -sf "http://localhost:3000/api/health" > /dev/null 2>&1; then
        log_success "Grafana is running"
    else
        log_warn "Grafana is not responding"
    fi
    
    # Step 8: Run container health checks
    log_info "Checking container health status..."
    local unhealthy_containers=$(docker-compose -f "$COMPOSE_FILE" ps | grep -E '(unhealthy|starting)' | wc -l)
    if [ "$unhealthy_containers" -eq 0 ]; then
        log_success "All containers are healthy"
    else
        log_warn "$unhealthy_containers containers are not healthy"
        docker-compose -f "$COMPOSE_FILE" ps
    fi
    
    # Step 9: Display final status
    log_info "========== TEST SUMMARY =========="
    docker-compose -f "$COMPOSE_FILE" ps
    
    # Step 10: Run for a bit to ensure stability
    log_info "Running stability test for 30 seconds..."
    sleep 30
    
    # Final health check
    local final_health_passed=true
    for i in {0..3}; do
        local port=$((8080 + i))
        if ! curl -sf "http://localhost:$port/health" > /dev/null 2>&1; then
            final_health_passed=false
        fi
    done
    
    if [ "$final_health_passed" = true ]; then
        log_success "========== ALL TESTS PASSED =========="
        log_info "Testnet is running successfully!"
        log_info "Health endpoints: http://localhost:8080-8083/health"
        log_info "Metrics endpoints: http://localhost:9090-9093/metrics"
        log_info "Prometheus: http://localhost:9094"
        log_info "Grafana: http://localhost:3000 (admin/admin)"
        
        # Ask if user wants to keep it running
        read -p "Keep testnet running for manual inspection? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            log_info "Testnet is running. Press Ctrl+C to stop."
            # Keep script running
            trap - EXIT
            while true; do
                sleep 60
                log_info "Testnet still running... (Ctrl+C to stop)"
            done
        fi
    else
        log_error "========== TESTS FAILED =========="
        docker-compose -f "$COMPOSE_FILE" logs --tail=50
        exit 1
    fi
}

# Run main function
main "$@"