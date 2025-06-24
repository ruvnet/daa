#!/bin/bash
set -euo pipefail

# QuDAG Testnet Monitoring Script
# This script monitors the health and status of all QuDAG nodes

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

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

# Monitoring options
REFRESH_INTERVAL=${REFRESH_INTERVAL:-5}
CONTINUOUS_MODE=false
VERBOSE_MODE=false
JSON_OUTPUT=false

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_header() {
    clear
    echo -e "${CYAN}"
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║               QuDAG Testnet Monitor v1.0                       ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo "Time: $(date '+%Y-%m-%d %H:%M:%S')"
    echo
}

check_prerequisites() {
    if ! command -v flyctl &> /dev/null; then
        log_error "flyctl is not installed"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        log_error "jq is not installed"
        exit 1
    fi
}

get_node_status() {
    local app_name=$1
    local status_json=$(flyctl status -a "$app_name" --json 2>/dev/null || echo "{}")
    
    if [ "$status_json" = "{}" ]; then
        echo "error"
        return
    fi
    
    local deployment_status=$(echo "$status_json" | jq -r '.DeploymentStatus // "unknown"')
    echo "$deployment_status"
}

get_node_instances() {
    local app_name=$1
    flyctl scale show -a "$app_name" --json 2>/dev/null | jq -r '.[] | select(.Process == "app") | .Count' || echo "0"
}

check_health_endpoint() {
    local app_name=$1
    local url="https://$app_name.fly.dev/health"
    
    if curl -sf --max-time 5 "$url" > /dev/null; then
        echo "healthy"
    else
        echo "unhealthy"
    fi
}

get_node_metrics() {
    local app_name=$1
    local metrics_url="https://$app_name.fly.dev:9090/metrics"
    
    # Try to fetch basic metrics
    local metrics=$(curl -sf --max-time 5 "$metrics_url" 2>/dev/null || echo "")
    
    if [ -n "$metrics" ]; then
        # Extract some key metrics
        local peers=$(echo "$metrics" | grep -E "^qudag_peers_total" | awk '{print $2}' | head -1 || echo "0")
        local blocks=$(echo "$metrics" | grep -E "^qudag_blocks_total" | awk '{print $2}' | head -1 || echo "0")
        local txs=$(echo "$metrics" | grep -E "^qudag_transactions_total" | awk '{print $2}' | head -1 || echo "0")
        
        echo "$peers|$blocks|$txs"
    else
        echo "0|0|0"
    fi
}

get_node_logs_summary() {
    local app_name=$1
    
    # Get last 10 log entries
    local logs=$(flyctl logs -a "$app_name" --no-tail -n 10 2>/dev/null || echo "")
    
    if [ -n "$logs" ]; then
        # Count errors and warnings
        local errors=$(echo "$logs" | grep -c "ERROR" || echo "0")
        local warnings=$(echo "$logs" | grep -c "WARN" || echo "0")
        
        echo "$errors|$warnings"
    else
        echo "0|0"
    fi
}

display_node_status() {
    local node=$1
    IFS=':' read -r location region app_name <<< "${NODES[$node]}"
    
    echo -n "  $location ($region): "
    
    # Get status
    local status=$(get_node_status "$app_name")
    local instances=$(get_node_instances "$app_name")
    local health=$(check_health_endpoint "$app_name")
    
    # Status indicator
    case "$status" in
        "successful")
            echo -ne "${GREEN}●${NC} "
            ;;
        "failed")
            echo -ne "${RED}●${NC} "
            ;;
        *)
            echo -ne "${YELLOW}●${NC} "
            ;;
    esac
    
    # Basic info
    echo -n "Status: $status | Instances: $instances | Health: "
    
    if [ "$health" = "healthy" ]; then
        echo -ne "${GREEN}$health${NC}"
    else
        echo -ne "${RED}$health${NC}"
    fi
    
    # Get metrics if verbose
    if [ "$VERBOSE_MODE" = true ]; then
        IFS='|' read -r peers blocks txs <<< "$(get_node_metrics "$app_name")"
        echo
        echo "     Peers: $peers | Blocks: $blocks | Transactions: $txs"
        
        IFS='|' read -r errors warnings <<< "$(get_node_logs_summary "$app_name")"
        echo "     Recent Errors: $errors | Warnings: $warnings"
    fi
    
    echo
}

display_network_summary() {
    echo -e "${CYAN}Network Summary:${NC}"
    echo "════════════════════════════════════════════════════════════════"
    
    local total_nodes=0
    local healthy_nodes=0
    
    for node in "${!NODES[@]}"; do
        ((total_nodes++))
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        if [ "$(check_health_endpoint "$app_name")" = "healthy" ]; then
            ((healthy_nodes++))
        fi
    done
    
    echo "Total Nodes: $total_nodes | Healthy: $healthy_nodes | Unhealthy: $((total_nodes - healthy_nodes))"
    echo
}

display_all_nodes() {
    echo -e "${CYAN}Node Status:${NC}"
    echo "════════════════════════════════════════════════════════════════"
    
    for node in node1 node2 node3 node4; do
        display_node_status "$node"
    done
}

json_output() {
    local output='{"timestamp":"'$(date -Iseconds)'","nodes":['
    
    local first=true
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        if [ "$first" != true ]; then
            output+=","
        fi
        first=false
        
        local status=$(get_node_status "$app_name")
        local instances=$(get_node_instances "$app_name")
        local health=$(check_health_endpoint "$app_name")
        IFS='|' read -r peers blocks txs <<< "$(get_node_metrics "$app_name")"
        IFS='|' read -r errors warnings <<< "$(get_node_logs_summary "$app_name")"
        
        output+='{"name":"'$app_name'","location":"'$location'","region":"'$region'",'
        output+='"status":"'$status'","instances":'$instances',"health":"'$health'",'
        output+='"metrics":{"peers":'$peers',"blocks":'$blocks',"transactions":'$txs'},'
        output+='"logs":{"errors":'$errors',"warnings":'$warnings'}}'
    done
    
    output+=']}'
    echo "$output" | jq .
}

monitor_loop() {
    while true; do
        if [ "$JSON_OUTPUT" = true ]; then
            json_output
        else
            print_header
            display_network_summary
            display_all_nodes
            
            echo
            echo "Press Ctrl+C to exit | Refreshing every ${REFRESH_INTERVAL}s"
        fi
        
        sleep "$REFRESH_INTERVAL"
    done
}

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -c, --continuous    Run in continuous monitoring mode"
    echo "  -i, --interval N    Set refresh interval in seconds (default: 5)"
    echo "  -v, --verbose       Show detailed metrics and logs"
    echo "  -j, --json          Output in JSON format"
    echo "  -h, --help          Show this help message"
    echo
    echo "Examples:"
    echo "  $0                  # One-time status check"
    echo "  $0 -c               # Continuous monitoring"
    echo "  $0 -c -v -i 10     # Verbose continuous monitoring every 10s"
    echo "  $0 -j               # JSON output for scripting"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--continuous)
            CONTINUOUS_MODE=true
            shift
            ;;
        -i|--interval)
            REFRESH_INTERVAL="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE_MODE=true
            shift
            ;;
        -j|--json)
            JSON_OUTPUT=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    check_prerequisites
    
    if [ "$CONTINUOUS_MODE" = true ]; then
        trap 'echo -e "\n${CYAN}Monitoring stopped.${NC}"; exit 0' INT
        monitor_loop
    else
        if [ "$JSON_OUTPUT" = true ]; then
            json_output
        else
            print_header
            display_network_summary
            display_all_nodes
        fi
    fi
}

main