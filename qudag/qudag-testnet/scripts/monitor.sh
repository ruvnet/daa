#!/bin/bash
# QuDAG Testnet Monitoring Script

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTNET_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default settings
CONTINUOUS=false
INTERVAL=10
VERBOSE=false
JSON_OUTPUT=false

# Node endpoints
NODES=(
    "node1:8080"
    "node2:8081"
    "node3:8082"
    "node4:8083"
)

# Usage function
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Monitor QuDAG testnet nodes health and status.

OPTIONS:
    -c, --continuous    Run continuously with updates
    -i, --interval N    Set update interval in seconds (default: 10)
    -v, --verbose       Show verbose output
    -j, --json          Output in JSON format
    -h, --help          Show this help message

EXAMPLES:
    $0                   # Single status check
    $0 -c                # Continuous monitoring
    $0 -c -i 5           # Continuous monitoring every 5 seconds
    $0 -v                # Verbose single check
    $0 -j                # JSON output
EOF
}

# Logging functions
log_info() {
    [ "$JSON_OUTPUT" = false ] && echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    [ "$JSON_OUTPUT" = false ] && echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    [ "$JSON_OUTPUT" = false ] && echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    [ "$JSON_OUTPUT" = false ] && echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker containers are running
check_containers() {
    local status_data=()
    
    # Check node containers
    for i in {1..4}; do
        local container_name="qudag-testnet-node$i"
        local status=$(docker inspect "$container_name" --format='{{.State.Status}}' 2>/dev/null || echo "not_found")
        local health=$(docker inspect "$container_name" --format='{{.State.Health.Status}}' 2>/dev/null || echo "unknown")
        
        status_data+=("{\"node\":\"node$i\",\"container_status\":\"$status\",\"health_status\":\"$health\"}")
    done
    
    # Check monitoring containers
    local monitoring_containers=("prometheus" "grafana" "node-exporter" "cadvisor")
    for container in "${monitoring_containers[@]}"; do
        local container_name="qudag-$container"
        local status=$(docker inspect "$container_name" --format='{{.State.Status}}' 2>/dev/null || echo "not_found")
        
        status_data+=("{\"service\":\"$container\",\"container_status\":\"$status\"}")
    done
    
    echo "${status_data[@]}"
}

# Check node APIs
check_node_apis() {
    local api_data=()
    
    for i in {1..4}; do
        local port=$((8080 + i - 1))
        local url="http://localhost:$port/api/v1/health"
        
        local response=$(curl -s -w "%{http_code}" -o /dev/null --max-time 5 "$url" 2>/dev/null || echo "000")
        local status="unknown"
        
        case $response in
            200) status="healthy" ;;
            000) status="unreachable" ;;
            *) status="unhealthy" ;;
        esac
        
        api_data+=("{\"node\":\"node$i\",\"port\":$port,\"status\":\"$status\",\"http_code\":\"$response\"}")
    done
    
    echo "${api_data[@]}"
}

# Get node metrics
get_node_metrics() {
    local metrics_data=()
    
    for i in {1..4}; do
        local port=$((9090 + i))
        local url="http://localhost:$port/metrics"
        
        # Get basic metrics
        local peer_count=$(curl -s --max-time 5 "$url" 2>/dev/null | grep "qudag_p2p_peer_count" | tail -1 | awk '{print $2}' || echo "0")
        local cpu_usage=$(docker stats "qudag-testnet-node$i" --no-stream --format "{{.CPUPerc}}" 2>/dev/null | sed 's/%//' || echo "0")
        local memory_usage=$(docker stats "qudag-testnet-node$i" --no-stream --format "{{.MemUsage}}" 2>/dev/null || echo "0B / 0B")
        
        metrics_data+=("{\"node\":\"node$i\",\"peer_count\":$peer_count,\"cpu_percent\":\"$cpu_usage\",\"memory_usage\":\"$memory_usage\"}")
    done
    
    echo "${metrics_data[@]}"
}

# Display status in table format
display_status_table() {
    local container_data=($1)
    local api_data=($2)
    local metrics_data=($3)
    
    echo -e "\n${CYAN}=== QuDAG Testnet Status ===${NC}"
    echo -e "${BLUE}$(date)${NC}\n"
    
    # Node status table
    printf "%-8s %-12s %-12s %-10s %-8s %-15s %-10s\n" \
        "Node" "Container" "Health" "API" "Peers" "CPU" "Memory"
    printf "%-8s %-12s %-12s %-10s %-8s %-15s %-10s\n" \
        "----" "---------" "------" "---" "-----" "---" "------"
    
    for i in {1..4}; do
        # Extract data for this node
        local container_status=$(echo "${container_data[$((i-1))]}" | jq -r .container_status)
        local health_status=$(echo "${container_data[$((i-1))]}" | jq -r .health_status)
        local api_status=$(echo "${api_data[$((i-1))]}" | jq -r .status)
        local peer_count=$(echo "${metrics_data[$((i-1))]}" | jq -r .peer_count)
        local cpu_usage=$(echo "${metrics_data[$((i-1))]}" | jq -r .cpu_percent)
        local memory_usage=$(echo "${metrics_data[$((i-1))]}" | jq -r .memory_usage | cut -d'/' -f1 | xargs)
        
        # Color coding
        local container_color=""
        local api_color=""
        
        case $container_status in
            "running") container_color="$GREEN" ;;
            *) container_color="$RED" ;;
        esac
        
        case $api_status in
            "healthy") api_color="$GREEN" ;;
            "unreachable") api_color="$RED" ;;
            *) api_color="$YELLOW" ;;
        esac
        
        printf "%-8s ${container_color}%-12s${NC} %-12s ${api_color}%-10s${NC} %-8s %-15s %-10s\n" \
            "node$i" "$container_status" "$health_status" "$api_status" "$peer_count" "${cpu_usage}%" "$memory_usage"
    done
    
    # Monitoring services status
    echo -e "\n${CYAN}=== Monitoring Services ===${NC}"
    printf "%-15s %-12s\n" "Service" "Status"
    printf "%-15s %-12s\n" "-------" "------"
    
    for i in {4..7}; do
        local service_data="${container_data[$i]}"
        local service_name=$(echo "$service_data" | jq -r .service)
        local service_status=$(echo "$service_data" | jq -r .container_status)
        
        local status_color=""
        case $service_status in
            "running") status_color="$GREEN" ;;
            *) status_color="$RED" ;;
        esac
        
        printf "%-15s ${status_color}%-12s${NC}\n" "$service_name" "$service_status"
    done
    
    # Summary
    local running_nodes=$(echo "${container_data[@]}" | jq -s '[.[] | select(.node != null and .container_status == "running")] | length')
    local healthy_apis=$(echo "${api_data[@]}" | jq -s '[.[] | select(.status == "healthy")] | length')
    
    echo -e "\n${CYAN}=== Summary ===${NC}"
    echo "Running Nodes: $running_nodes/4"
    echo "Healthy APIs:  $healthy_apis/4"
    
    if [ "$running_nodes" -eq 4 ] && [ "$healthy_apis" -eq 4 ]; then
        echo -e "${GREEN}Testnet Status: HEALTHY${NC}"
    elif [ "$running_nodes" -gt 0 ]; then
        echo -e "${YELLOW}Testnet Status: DEGRADED${NC}"
    else
        echo -e "${RED}Testnet Status: DOWN${NC}"
    fi
}

# Display JSON output
display_json_output() {
    local container_data=($1)
    local api_data=($2)
    local metrics_data=($3)
    
    local json_output=$(jq -n \
        --argjson containers "$(printf '%s\n' "${container_data[@]}" | jq -s '.')" \
        --argjson apis "$(printf '%s\n' "${api_data[@]}" | jq -s '.')" \
        --argjson metrics "$(printf '%s\n' "${metrics_data[@]}" | jq -s '.')" \
        '{
            "timestamp": now | strftime("%Y-%m-%dT%H:%M:%SZ"),
            "containers": $containers,
            "apis": $apis,
            "metrics": $metrics
        }')
    
    echo "$json_output"
}

# Main monitoring function
monitor_once() {
    local container_data=($(check_containers))
    local api_data=($(check_node_apis))
    local metrics_data=($(get_node_metrics))
    
    if [ "$JSON_OUTPUT" = true ]; then
        display_json_output "${container_data[*]}" "${api_data[*]}" "${metrics_data[*]}"
    else
        display_status_table "${container_data[*]}" "${api_data[*]}" "${metrics_data[*]}"
    fi
}

# Continuous monitoring
monitor_continuous() {
    log_info "Starting continuous monitoring (interval: ${INTERVAL}s)"
    log_info "Press Ctrl+C to stop"
    
    while true; do
        if [ "$JSON_OUTPUT" = false ]; then
            clear
        fi
        
        monitor_once
        
        if [ "$JSON_OUTPUT" = false ]; then
            echo -e "\n${BLUE}Next update in ${INTERVAL}s...${NC}"
        fi
        
        sleep "$INTERVAL"
    done
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--continuous)
            CONTINUOUS=true
            shift
            ;;
        -i|--interval)
            INTERVAL="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -j|--json)
            JSON_OUTPUT=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Check dependencies
if ! command -v jq &> /dev/null; then
    log_error "jq is required but not installed"
    exit 1
fi

if ! command -v docker &> /dev/null; then
    log_error "docker is required but not installed"
    exit 1
fi

# Main execution
if [ "$CONTINUOUS" = true ]; then
    monitor_continuous
else
    monitor_once
fi