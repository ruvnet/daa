#!/bin/bash
# QuDAG Testnet Monitoring Script
# Real-time monitoring and health checks for testnet nodes

set -euo pipefail

# Configuration
TESTNET_NAME="${TESTNET_NAME:-qudag-testnet}"
REFRESH_INTERVAL="${REFRESH_INTERVAL:-5}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Terminal control
clear_screen() {
    printf '\033[2J\033[H'
}

move_cursor() {
    printf '\033[%d;%dH' "$1" "$2"
}

# Get node status
get_node_status() {
    local container=$1
    
    if ! docker ps --format "{{.Names}}" | grep -q "^${container}$"; then
        echo "OFFLINE"
        return
    fi
    
    local health=$(docker inspect --format='{{.State.Health.Status}}' "$container" 2>/dev/null || echo "unknown")
    
    case $health in
        healthy)
            echo "HEALTHY"
            ;;
        starting)
            echo "STARTING"
            ;;
        unhealthy)
            echo "UNHEALTHY"
            ;;
        *)
            echo "UNKNOWN"
            ;;
    esac
}

# Get node metrics
get_node_metrics() {
    local container=$1
    local port=$2
    
    local metrics=$(curl -s "http://localhost:${port}/metrics" 2>/dev/null || echo "")
    
    if [ -z "$metrics" ]; then
        echo "peers:0 blocks:0 tps:0"
        return
    fi
    
    local peers=$(echo "$metrics" | grep -E "^qudag_peers_connected" | awk '{print $2}' || echo "0")
    local blocks=$(echo "$metrics" | grep -E "^qudag_blocks_total" | awk '{print $2}' || echo "0")
    local tps=$(echo "$metrics" | grep -E "^qudag_transactions_per_second" | awk '{print $2}' || echo "0")
    
    echo "peers:${peers:-0} blocks:${blocks:-0} tps:${tps:-0}"
}

# Get container stats
get_container_stats() {
    local container=$1
    
    local stats=$(docker stats --no-stream --format "{{.CPUPerc}} {{.MemUsage}}" "$container" 2>/dev/null || echo "0% 0/0")
    echo "$stats"
}

# Format status with color
format_status() {
    local status=$1
    
    case $status in
        HEALTHY)
            echo -e "${GREEN}● HEALTHY${NC}"
            ;;
        STARTING)
            echo -e "${YELLOW}● STARTING${NC}"
            ;;
        UNHEALTHY|OFFLINE)
            echo -e "${RED}● $status${NC}"
            ;;
        *)
            echo -e "${CYAN}● $status${NC}"
            ;;
    esac
}

# Display header
display_header() {
    clear_screen
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}                        QuDAG Testnet Monitor v1.0                            ${BLUE}║${NC}"
    echo -e "${BLUE}╠══════════════════════════════════════════════════════════════════════════════╣${NC}"
}

# Display node info
display_node_info() {
    local row=$1
    local name=$2
    local container=$3
    local metrics_port=$4
    
    move_cursor $row 1
    
    # Get status and metrics
    local status=$(get_node_status "$container")
    local metrics=$(get_node_metrics "$container" "$metrics_port")
    local stats=$(get_container_stats "$container")
    
    # Parse metrics
    local peers=$(echo "$metrics" | grep -o "peers:[0-9]*" | cut -d: -f2)
    local blocks=$(echo "$metrics" | grep -o "blocks:[0-9]*" | cut -d: -f2)
    local tps=$(echo "$metrics" | grep -o "tps:[0-9.]*" | cut -d: -f2)
    
    # Parse stats
    local cpu=$(echo "$stats" | awk '{print $1}')
    local mem=$(echo "$stats" | awk '{print $2}')
    
    # Format and display
    printf "${BLUE}║${NC} %-15s $(format_status "$status") │ Peers: %3s │ Blocks: %6s │ TPS: %6s │ CPU: %6s │ Mem: %-12s ${BLUE}║${NC}\n" \
        "$name" "$peers" "$blocks" "$tps" "$cpu" "$mem"
}

# Display footer
display_footer() {
    echo -e "${BLUE}╠══════════════════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${BLUE}║${NC} Prometheus: http://localhost:9093  │  Grafana: http://localhost:3000        ${BLUE}║${NC}"
    echo -e "${BLUE}║${NC} Press Ctrl+C to exit               │  Refresh: ${REFRESH_INTERVAL}s                          ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
}

# Get network statistics
get_network_stats() {
    local total_peers=0
    local total_blocks=0
    local total_tps=0
    local active_nodes=0
    
    # Check bootstrap node
    local bootstrap_metrics=$(get_node_metrics "${TESTNET_NAME}-bootstrap" "9090")
    if [[ "$bootstrap_metrics" != "peers:0 blocks:0 tps:0" ]]; then
        ((active_nodes++))
        total_peers=$((total_peers + $(echo "$bootstrap_metrics" | grep -o "peers:[0-9]*" | cut -d: -f2)))
        total_blocks=$(echo "$bootstrap_metrics" | grep -o "blocks:[0-9]*" | cut -d: -f2)
        total_tps=$(echo "$bootstrap_metrics" | grep -o "tps:[0-9.]*" | cut -d: -f2)
    fi
    
    # Check other nodes
    local node_count=$(docker ps --filter "name=${TESTNET_NAME}-node-" --format "{{.Names}}" | wc -l)
    for i in $(seq 1 "$node_count"); do
        local metrics=$(get_node_metrics "${TESTNET_NAME}-node-${i}" "$((9090 + i))")
        if [[ "$metrics" != "peers:0 blocks:0 tps:0" ]]; then
            ((active_nodes++))
            total_peers=$((total_peers + $(echo "$metrics" | grep -o "peers:[0-9]*" | cut -d: -f2)))
        fi
    done
    
    move_cursor 5 1
    echo -e "${BLUE}║${NC} Network Stats: Active Nodes: ${GREEN}$active_nodes${NC} │ Total Connections: ${GREEN}$total_peers${NC} │ Chain Height: ${GREEN}$total_blocks${NC} │ Network TPS: ${GREEN}$total_tps${NC}   ${BLUE}║${NC}"
}

# Monitor loop
monitor_loop() {
    while true; do
        display_header
        
        # Network statistics
        get_network_stats
        
        # Node table header
        move_cursor 7 1
        echo -e "${BLUE}╠═════════════════╤═════════════╤═════════╤═════════════╤═════════════╤═══════╤═══════════════╣${NC}"
        move_cursor 8 1
        echo -e "${BLUE}║${NC} Node            │ Status      │ Peers   │ Blocks      │ TPS         │ CPU   │ Memory        ${BLUE}║${NC}"
        move_cursor 9 1
        echo -e "${BLUE}╠═════════════════╪═════════════╪═════════╪═════════════╪═════════════╪═══════╪═══════════════╣${NC}"
        
        # Display bootstrap node
        display_node_info 10 "Bootstrap" "${TESTNET_NAME}-bootstrap" "9090"
        
        # Display other nodes
        local row=11
        local node_count=$(docker ps --filter "name=${TESTNET_NAME}-node-" --format "{{.Names}}" | wc -l)
        
        for i in $(seq 1 "$node_count"); do
            display_node_info $row "Node $i" "${TESTNET_NAME}-node-${i}" "$((9090 + i))"
            ((row++))
        done
        
        # Fill empty rows
        while [ $row -lt 20 ]; do
            move_cursor $row 1
            echo -e "${BLUE}║${NC}                                                                                              ${BLUE}║${NC}"
            ((row++))
        done
        
        move_cursor 20 1
        echo -e "${BLUE}╠══════════════════════════════════════════════════════════════════════════════╣${NC}"
        
        # Recent logs
        move_cursor 21 1
        echo -e "${BLUE}║${NC} Recent Activity:                                                             ${BLUE}║${NC}"
        
        # Get recent logs from bootstrap node
        local logs=$(docker logs --tail 3 "${TESTNET_NAME}-bootstrap" 2>&1 | tail -3)
        local log_row=22
        
        while IFS= read -r line && [ $log_row -lt 25 ]; do
            move_cursor $log_row 1
            # Truncate log line if too long
            local truncated_line=$(echo "$line" | cut -c1-75)
            printf "${BLUE}║${NC} %-76s ${BLUE}║${NC}\n" "$truncated_line"
            ((log_row++))
        done <<< "$logs"
        
        # Fill empty log rows
        while [ $log_row -lt 25 ]; do
            move_cursor $log_row 1
            echo -e "${BLUE}║${NC}                                                                              ${BLUE}║${NC}"
            ((log_row++))
        done
        
        move_cursor 25 1
        display_footer
        
        # Hide cursor
        printf '\033[?25l'
        
        sleep "$REFRESH_INTERVAL"
    done
}

# Cleanup on exit
cleanup() {
    # Show cursor
    printf '\033[?25h'
    clear_screen
    exit 0
}

# Set up signal handlers
trap cleanup INT TERM

# Main execution
main() {
    # Check if testnet is running
    if ! docker ps --format "{{.Names}}" | grep -q "${TESTNET_NAME}"; then
        echo -e "${RED}Error: No testnet containers found with prefix '${TESTNET_NAME}'${NC}"
        echo "Please deploy the testnet first using: ./deploy-testnet.sh"
        exit 1
    fi
    
    # Start monitoring
    monitor_loop
}

# Run main
main