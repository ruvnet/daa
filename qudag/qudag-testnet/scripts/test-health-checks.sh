#!/bin/bash
set -euo pipefail

# QuDAG Testnet Health Check Test Script
# Tests different health check approaches to diagnose issues

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Node configuration
declare -A NODES=(
    ["node1"]="toronto:yyz:qudag-testnet-node1"
    ["node2"]="amsterdam:ams:qudag-testnet-node2"
    ["node3"]="singapore:sin:qudag-testnet-node3"
    ["node4"]="sanfrancisco:sjc:qudag-testnet-node4"
)

echo -e "${CYAN}QuDAG Testnet Health Check Diagnostics${NC}"
echo "======================================="
echo

for node in node1 node2 node3 node4; do
    IFS=':' read -r location region app_name <<< "${NODES[$node]}"
    
    echo -e "${BLUE}Testing $app_name ($location)${NC}"
    echo "-----------------------------------"
    
    # Test HTTPS with certificate validation
    echo -n "  HTTPS (strict): "
    if curl -sf --max-time 5 "https://$app_name.fly.dev/health" &>/dev/null; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${RED}✗ FAIL${NC}"
    fi
    
    # Test HTTPS ignoring certificates
    echo -n "  HTTPS (no cert check): "
    if curl -sfk --max-time 5 "https://$app_name.fly.dev/health" &>/dev/null; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${RED}✗ FAIL${NC}"
    fi
    
    # Test HTTP
    echo -n "  HTTP: "
    if curl -sf --max-time 5 "http://$app_name.fly.dev/health" &>/dev/null; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${RED}✗ FAIL${NC}"
    fi
    
    # Test metrics endpoint
    echo -n "  Metrics (port 9090): "
    if curl -sf --max-time 5 "http://$app_name.fly.dev:9090/metrics" &>/dev/null; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${RED}✗ FAIL${NC}"
    fi
    
    # Test P2P port connectivity
    echo -n "  P2P Port (4001): "
    if nc -zv -w5 "$app_name.fly.dev" 4001 &>/dev/null; then
        echo -e "${GREEN}✓ OPEN${NC}"
    else
        echo -e "${YELLOW}⚠ CLOSED/FILTERED${NC}"
    fi
    
    # Get actual response
    echo "  Response sample:"
    curl -sk --max-time 5 "https://$app_name.fly.dev/health" 2>/dev/null | head -1 | sed 's/^/    /' || echo "    (no response)"
    
    echo
done

echo -e "${BLUE}Summary:${NC}"
echo "- If HTTPS strict fails but no-cert-check passes: Certificate issue"
echo "- If both HTTPS fail but HTTP passes: force_https or TLS proxy issue"
echo "- If metrics fail: Port 9090 not properly exposed"
echo "- If P2P closed: Network isolation between nodes"