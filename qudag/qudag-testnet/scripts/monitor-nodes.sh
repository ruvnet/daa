#!/bin/bash
set -euo pipefail

# QuDAG Testnet Monitoring Script
# Real-time monitoring of all nodes with health checks and metrics

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Node configuration
declare -A NODES=(
    ["node1"]="toronto:yyz:qudag-testnet-node1"
    ["node2"]="amsterdam:ams:qudag-testnet-node2"
    ["node3"]="singapore:sin:qudag-testnet-node3"
    ["node4"]="sanfrancisco:sjc:qudag-testnet-node4"
)

# Default settings
CONTINUOUS=false
INTERVAL=30
VERBOSE=false
JSON_OUTPUT=false

# Functions
log_info() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${BLUE}[INFO]${NC} $1"
    fi
}

log_success() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${GREEN}[SUCCESS]${NC} $1"
    fi
}

log_warning() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${YELLOW}[WARNING]${NC} $1"
    fi
}

log_error() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${RED}[ERROR]${NC} $1"
    fi
}

check_prerequisites() {
    # Check for flyctl
    if ! command -v flyctl &> /dev/null; then
        log_error "flyctl is not installed. Please install it from https://fly.io/docs/hands-on/install-flyctl/"
        exit 1
    fi
    
    # Check for curl
    if ! command -v curl &> /dev/null; then
        log_error "curl is not installed. Please install it for health checks."
        exit 1
    fi
    
    # Check for jq if JSON output is requested
    if [ "$JSON_OUTPUT" = true ] && ! command -v jq &> /dev/null; then
        log_error "jq is not installed. Please install it for JSON processing."
        exit 1
    fi
}

get_node_status() {
    local app_name="$1"
    local location="$2"
    local region="$3"
    
    # Initialize status object
    local status="{"
    status+="\"name\": \"$app_name\","
    status+="\"location\": \"$location\","
    status+="\"region\": \"$region\","
    
    # Get Fly.io app status
    if flyctl status -a "$app_name" &>/dev/null; then
        local fly_status=$(flyctl status -a "$app_name" --json 2>/dev/null || echo '{}')
        local app_status=$(echo "$fly_status" | jq -r '.Status // "unknown"' 2>/dev/null || echo "unknown")
        status+="\"fly_status\": \"$app_status\","
        
        # Get instance info
        local instances=$(echo "$fly_status" | jq -r '.Allocations | length' 2>/dev/null || echo "0")
        status+="\"instances\": $instances,"
    else
        status+="\"fly_status\": \"error\","
        status+="\"instances\": 0,"
    fi
    
    # Check health endpoint
    local health_url="https://$app_name.fly.dev/health"
    # Use -k to ignore certificate issues and increase timeout
    if curl -sfk --max-time 10 "$health_url" &>/dev/null; then
        status+="\"health\": \"healthy\","
    else
        # Fallback to HTTP if HTTPS fails
        local http_health_url="http://$app_name.fly.dev/health"
        if curl -sf --max-time 10 "$http_health_url" &>/dev/null; then
            status+="\"health\": \"healthy (http)\","
        else
            status+="\"health\": \"unhealthy\","
        fi
    fi
    
    # Get metrics if verbose
    if [ "$VERBOSE" = true ]; then
        # Metrics are on port 9090, not through HTTPS proxy
        local metrics_url="http://$app_name.fly.dev:9090/metrics"
        local metrics_data=$(curl -sf --max-time 10 "$metrics_url" 2>/dev/null || echo "")
        
        if [ -n "$metrics_data" ]; then
            # Extract basic metrics (simplified parsing)
            local peers=$(echo "$metrics_data" | grep -o 'qudag_peers_total [0-9]*' | cut -d' ' -f2 || echo "0")
            local blocks=$(echo "$metrics_data" | grep -o 'qudag_blocks_total [0-9]*' | cut -d' ' -f2 || echo "0")
            
            status+="\"metrics\": {"
            status+="\"peers\": ${peers:-0},"
            status+="\"blocks\": ${blocks:-0}"
            status+="},"
        else
            status+="\"metrics\": null,"
        fi
    fi
    
    # Get timestamp
    status+="\"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\""
    status+="}"
    
    echo "$status"
}

print_status_table() {
    if [ "$JSON_OUTPUT" = true ]; then
        return
    fi
    
    # Clear screen for continuous mode
    if [ "$CONTINUOUS" = true ]; then
        clear
    fi
    
    echo -e "${CYAN}==========================================${NC}"
    echo -e "${CYAN}    QuDAG Testnet Node Status${NC}"
    echo -e "${CYAN}==========================================${NC}"
    echo
    
    printf "%-12s %-12s %-8s %-12s %-12s" "NODE" "LOCATION" "REGION" "FLY STATUS" "HEALTH"
    if [ "$VERBOSE" = true ]; then
        printf " %-8s %-8s" "PEERS" "BLOCKS"
    fi
    echo
    echo "------------------------------------------------------------------------"
    
    local all_healthy=true
    
    for node in node1 node2 node3 node4; do
        if [ -z "${NODES[$node]:-}" ]; then
            continue
        fi
        
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        local status_json=$(get_node_status "$app_name" "$location" "$region")
        
        local fly_status=$(echo "$status_json" | jq -r '.fly_status')
        local health=$(echo "$status_json" | jq -r '.health')
        
        # Color coding
        local fly_color="$RED"
        local health_color="$RED"
        
        if [ "$fly_status" = "deployed" ] || [ "$fly_status" = "running" ]; then
            fly_color="$GREEN"
        fi
        
        if [ "$health" = "healthy" ]; then
            health_color="$GREEN"
        else
            all_healthy=false
        fi
        
        printf "%-12s %-12s %-8s ${fly_color}%-12s${NC} ${health_color}%-12s${NC}" \
            "$node" "$location" "$region" "$fly_status" "$health"
        
        if [ "$VERBOSE" = true ]; then
            local peers=$(echo "$status_json" | jq -r '.metrics.peers // 0')
            local blocks=$(echo "$status_json" | jq -r '.metrics.blocks // 0')
            printf " %-8s %-8s" "$peers" "$blocks"
        fi
        
        echo
    done
    
    echo
    
    if [ "$all_healthy" = true ]; then
        echo -e "${GREEN}✓ All nodes are healthy${NC}"
    else
        echo -e "${YELLOW}⚠ Some nodes need attention${NC}"
    fi
    
    echo -e "\nLast updated: $(date)"
    
    if [ "$CONTINUOUS" = true ]; then
        echo -e "\nPress Ctrl+C to exit continuous monitoring"
        echo "Refreshing in ${INTERVAL}s..."
    fi
}

print_json_status() {
    if [ "$JSON_OUTPUT" = false ]; then
        return
    fi
    
    echo "{"
    echo '  "timestamp": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",'
    echo '  "nodes": ['
    
    local first=true
    for node in node1 node2 node3 node4; do
        if [ -z "${NODES[$node]:-}" ]; then
            continue
        fi
        
        if [ "$first" = false ]; then
            echo ","
        fi
        first=false
        
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        local status_json=$(get_node_status "$app_name" "$location" "$region")
        echo "    $status_json"
    done
    
    echo
    echo '  ]'
    echo "}"
}

monitor_once() {
    if [ "$JSON_OUTPUT" = true ]; then
        print_json_status
    else
        print_status_table
    fi
}

monitor_continuous() {
    if [ "$JSON_OUTPUT" = true ]; then
        log_error "Continuous mode is not supported with JSON output"
        exit 1
    fi
    
    log_info "Starting continuous monitoring (refresh every ${INTERVAL}s)"
    
    while true; do
        print_status_table
        sleep "$INTERVAL"
    done
}

show_help() {
    cat << EOF
QuDAG Testnet Node Monitor

Usage: $0 [OPTIONS]

Options:
  -c, --continuous    Run in continuous mode
  -i, --interval N    Set refresh interval in seconds (default: 30)
  -v, --verbose       Show detailed metrics
  -j, --json         Output in JSON format
  -h, --help         Show this help message

Examples:
  $0                 # Single status check
  $0 -c              # Continuous monitoring
  $0 -c -i 10        # Continuous with 10s refresh
  $0 -v              # Verbose output with metrics
  $0 -j              # JSON output

EOF
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
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main execution
check_prerequisites

if [ "$CONTINUOUS" = true ]; then
    monitor_continuous
else
    monitor_once
fi