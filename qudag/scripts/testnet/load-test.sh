#!/bin/bash
# QuDAG Testnet Load Testing Script
# Performs various load tests on the deployed testnet

set -euo pipefail

# Configuration
TESTNET_NAME="${TESTNET_NAME:-qudag-testnet}"
TEST_DURATION="${TEST_DURATION:-300}" # 5 minutes default
TPS_TARGET="${TPS_TARGET:-100}"
CONCURRENT_CLIENTS="${CONCURRENT_CLIENTS:-10}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Logging
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Generate test transactions
generate_transaction() {
    local from_address=$1
    local to_address=$2
    local amount=$3
    local nonce=$4
    
    cat << EOF
{
  "from": "$from_address",
  "to": "$to_address",
  "amount": $amount,
  "nonce": $nonce,
  "timestamp": $(date +%s),
  "data": "Load test transaction $nonce"
}
EOF
}

# Generate test addresses
generate_test_addresses() {
    local count=$1
    local addresses=()
    
    for i in $(seq 1 "$count"); do
        # Generate quantum-resistant address using the node
        local address=$(docker exec "${TESTNET_NAME}-bootstrap" qudag address generate --type quantum 2>/dev/null | grep "Address:" | awk '{print $2}')
        addresses+=("$address")
    done
    
    echo "${addresses[@]}"
}

# Transaction sender worker
transaction_sender() {
    local worker_id=$1
    local node_port=$2
    local addresses=($3)
    local target_tps=$4
    local duration=$5
    
    log_info "Worker $worker_id starting (target TPS: $target_tps)"
    
    local start_time=$(date +%s)
    local end_time=$((start_time + duration))
    local tx_count=0
    local nonce=0
    
    # Calculate delay between transactions
    local delay=$(echo "scale=3; 1 / $target_tps" | bc)
    
    while [ $(date +%s) -lt $end_time ]; do
        # Random from/to addresses
        local from=${addresses[$((RANDOM % ${#addresses[@]}))]}
        local to=${addresses[$((RANDOM % ${#addresses[@]}))]}
        local amount=$((RANDOM % 1000 + 1))
        
        # Generate and send transaction
        local tx=$(generate_transaction "$from" "$to" "$amount" "$nonce")
        
        # Send transaction
        local response=$(curl -s -X POST \
            -H "Content-Type: application/json" \
            -d "$tx" \
            "http://localhost:${node_port}/api/v1/transaction" 2>/dev/null)
        
        if [[ "$response" == *"success"* ]]; then
            ((tx_count++))
        fi
        
        ((nonce++))
        
        # Rate limiting
        sleep "$delay"
    done
    
    echo "$worker_id:$tx_count"
}

# Quantum crypto load test
quantum_crypto_test() {
    log_info "Running quantum cryptography load test..."
    
    local operations=("sign" "verify" "encrypt" "decrypt")
    local algorithms=("ml-dsa" "ml-kem" "hqc")
    local results=()
    
    for algo in "${algorithms[@]}"; do
        log_info "Testing $algo algorithm..."
        
        local start_time=$(date +%s%N)
        local op_count=0
        
        # Run operations for 30 seconds
        local end_time=$(($(date +%s) + 30))
        
        while [ $(date +%s) -lt $end_time ]; do
            # Generate key pair
            docker exec "${TESTNET_NAME}-bootstrap" qudag key generate --algorithm "$algo" &>/dev/null
            
            # Test operations
            case $algo in
                ml-dsa)
                    docker exec "${TESTNET_NAME}-bootstrap" qudag sign "test message" &>/dev/null
                    docker exec "${TESTNET_NAME}-bootstrap" qudag verify "signature" "test message" &>/dev/null
                    ((op_count+=2))
                    ;;
                ml-kem|hqc)
                    docker exec "${TESTNET_NAME}-bootstrap" qudag encrypt "test data" &>/dev/null
                    docker exec "${TESTNET_NAME}-bootstrap" qudag decrypt "ciphertext" &>/dev/null
                    ((op_count+=2))
                    ;;
            esac
        done
        
        local elapsed=$(($(date +%s%N) - start_time))
        local ops_per_sec=$(echo "scale=2; $op_count * 1000000000 / $elapsed" | bc)
        
        results+=("$algo: $ops_per_sec ops/sec")
    done
    
    log_success "Quantum crypto test complete:"
    for result in "${results[@]}"; do
        echo "  - $result"
    done
}

# Network stress test
network_stress_test() {
    log_info "Running network stress test..."
    
    # Get all node RPC ports
    local ports=(8080 8081 8082 8083 8084)
    
    # Start concurrent workers
    local pids=()
    local worker_results=()
    
    # Generate test addresses
    log_info "Generating test addresses..."
    local addresses=($(generate_test_addresses 20))
    
    # Calculate TPS per worker
    local tps_per_worker=$(echo "scale=2; $TPS_TARGET / $CONCURRENT_CLIENTS" | bc)
    
    # Start workers
    for i in $(seq 1 "$CONCURRENT_CLIENTS"); do
        local port=${ports[$((i % ${#ports[@]}))]}
        transaction_sender "$i" "$port" "${addresses[*]}" "$tps_per_worker" "$TEST_DURATION" &
        pids+=($!)
    done
    
    # Monitor progress
    local elapsed=0
    while [ $elapsed -lt $TEST_DURATION ]; do
        echo -ne "\rProgress: $elapsed/$TEST_DURATION seconds"
        sleep 1
        ((elapsed++))
    done
    echo
    
    # Wait for workers to complete
    log_info "Waiting for workers to complete..."
    for pid in "${pids[@]}"; do
        if wait $pid; then
            worker_results+=("$(cat /tmp/worker_result_$pid 2>/dev/null || echo "0")")
        fi
    done
    
    # Calculate total transactions
    local total_tx=0
    for result in "${worker_results[@]}"; do
        local tx_count=$(echo "$result" | cut -d: -f2)
        total_tx=$((total_tx + tx_count))
    done
    
    local actual_tps=$(echo "scale=2; $total_tx / $TEST_DURATION" | bc)
    
    log_success "Network stress test complete:"
    echo "  - Total transactions: $total_tx"
    echo "  - Average TPS: $actual_tps"
    echo "  - Target TPS: $TPS_TARGET"
}

# Consensus stability test
consensus_test() {
    log_info "Running consensus stability test..."
    
    # Monitor consensus metrics
    local start_time=$(date +%s)
    local test_duration=60
    local consensus_rounds=()
    local finality_times=()
    
    while [ $(($(date +%s) - start_time)) -lt $test_duration ]; do
        # Query consensus metrics from each node
        for i in $(seq 0 4); do
            local port=$((9090 + i))
            local metrics=$(curl -s "http://localhost:${port}/metrics" 2>/dev/null || echo "")
            
            if [ -n "$metrics" ]; then
                local rounds=$(echo "$metrics" | grep "qudag_consensus_rounds_total" | awk '{print $2}')
                local finality=$(echo "$metrics" | grep "qudag_finality_time_ms" | awk '{print $2}')
                
                consensus_rounds+=("$rounds")
                finality_times+=("$finality")
            fi
        done
        
        sleep 5
    done
    
    # Calculate averages
    local avg_finality=$(echo "${finality_times[@]}" | awk '{s+=$1} END {print s/NR}')
    
    log_success "Consensus test complete:"
    echo "  - Average finality time: ${avg_finality}ms"
    echo "  - Total consensus rounds: ${#consensus_rounds[@]}"
}

# Dark addressing test
dark_addressing_test() {
    log_info "Running dark addressing test..."
    
    local domains=()
    local resolution_times=()
    
    # Register test domains
    for i in $(seq 1 10); do
        local domain="loadtest${i}.dark"
        docker exec "${TESTNET_NAME}-bootstrap" qudag dark register "$domain" &>/dev/null
        domains+=("$domain")
    done
    
    # Test resolution performance
    for domain in "${domains[@]}"; do
        local start=$(date +%s%N)
        docker exec "${TESTNET_NAME}-bootstrap" qudag dark resolve "$domain" &>/dev/null
        local end=$(date +%s%N)
        
        local elapsed_ms=$(echo "scale=2; ($end - $start) / 1000000" | bc)
        resolution_times+=("$elapsed_ms")
    done
    
    # Calculate average
    local avg_resolution=$(echo "${resolution_times[@]}" | awk '{s+=$1} END {print s/NR}')
    
    log_success "Dark addressing test complete:"
    echo "  - Registered domains: ${#domains[@]}"
    echo "  - Average resolution time: ${avg_resolution}ms"
}

# Generate load test report
generate_report() {
    local report_file="testnet-load-test-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" << EOF
# QuDAG Testnet Load Test Report

Date: $(date)
Testnet: $TESTNET_NAME

## Test Configuration
- Duration: $TEST_DURATION seconds
- Target TPS: $TPS_TARGET
- Concurrent Clients: $CONCURRENT_CLIENTS

## Test Results

### Transaction Load Test
- Total transactions processed: $TOTAL_TX
- Average TPS achieved: $ACTUAL_TPS
- Success rate: $SUCCESS_RATE%

### Quantum Cryptography Performance
$CRYPTO_RESULTS

### Consensus Stability
- Average finality time: $AVG_FINALITY ms
- Consensus rounds: $CONSENSUS_ROUNDS
- Fork incidents: $FORK_COUNT

### Dark Addressing Performance
- Domains tested: $DOMAIN_COUNT
- Average resolution time: $AVG_RESOLUTION ms

### Resource Usage
$RESOURCE_USAGE

## Recommendations
$RECOMMENDATIONS

EOF
    
    log_success "Load test report generated: $report_file"
}

# Main execution
main() {
    log_info "Starting QuDAG testnet load tests..."
    
    # Check if testnet is running
    if ! docker ps --format "{{.Names}}" | grep -q "${TESTNET_NAME}"; then
        log_error "Testnet not running. Deploy it first with: ./deploy-testnet.sh"
        exit 1
    fi
    
    # Run test suites
    case "${1:-all}" in
        transactions)
            network_stress_test
            ;;
        crypto)
            quantum_crypto_test
            ;;
        consensus)
            consensus_test
            ;;
        addressing)
            dark_addressing_test
            ;;
        all)
            network_stress_test
            quantum_crypto_test
            consensus_test
            dark_addressing_test
            generate_report
            ;;
        *)
            echo "Usage: $0 {transactions|crypto|consensus|addressing|all}"
            echo
            echo "Test suites:"
            echo "  transactions - Network transaction load test"
            echo "  crypto       - Quantum cryptography performance test"
            echo "  consensus    - Consensus stability test"
            echo "  addressing   - Dark addressing performance test"
            echo "  all          - Run all tests (default)"
            exit 1
            ;;
    esac
    
    log_success "Load testing complete!"
}

# Run main
main "$@"