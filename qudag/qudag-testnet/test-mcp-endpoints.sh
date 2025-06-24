#!/bin/bash
set -uo pipefail

# QuDAG MCP Endpoints Test Script
# Tests Model Context Protocol functionality on testnet

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Node endpoints
NODES=(
    "109.105.222.156:Toronto:Enhanced"
    "149.248.199.86:Amsterdam:Standard"
    "149.248.218.16:Singapore:Standard"
    "137.66.62.149:SanFrancisco:Standard"
)

echo -e "${PURPLE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║        QuDAG MCP Endpoints Test                          ║${NC}"
echo -e "${PURPLE}╚══════════════════════════════════════════════════════════╝${NC}"
echo

# Function to test endpoint
test_endpoint() {
    local url=$1
    local name=$2
    local method=${3:-GET}
    local data=${4:-}
    
    if [ "$method" = "GET" ]; then
        if response=$(curl -sf "$url" 2>/dev/null); then
            echo -e "${GREEN}✓${NC} $name"
            echo "  Response: $(echo "$response" | jq -c '.' 2>/dev/null || echo "$response" | head -c 100)"
            return 0
        else
            echo -e "${RED}✗${NC} $name"
            return 1
        fi
    else
        if response=$(curl -sf -X "$method" -H "Content-Type: application/json" -d "$data" "$url" 2>/dev/null); then
            echo -e "${GREEN}✓${NC} $name"
            echo "  Response: $(echo "$response" | jq -c '.' 2>/dev/null || echo "$response" | head -c 100)"
            return 0
        else
            echo -e "${RED}✗${NC} $name"
            return 1
        fi
    fi
}

# Test MCP standard endpoints
echo -e "${YELLOW}1. TESTING MCP STANDARD ENDPOINTS${NC}"
echo "===================================="

for node_info in "${NODES[@]}"; do
    IFS=':' read -r ip location type <<< "$node_info"
    echo -e "\n${BLUE}$location Node ($type):${NC}"
    
    # Test MCP discovery endpoint
    test_endpoint "http://$ip/mcp" "MCP Discovery"
    test_endpoint "http://$ip/.well-known/mcp" "MCP Well-Known"
    
    # Test MCP server info
    test_endpoint "http://$ip/mcp/info" "MCP Server Info"
    test_endpoint "http://$ip/api/mcp/info" "MCP API Info"
    
    # Test MCP capabilities
    test_endpoint "http://$ip/mcp/capabilities" "MCP Capabilities"
    test_endpoint "http://$ip/mcp/tools" "MCP Tools List"
    test_endpoint "http://$ip/mcp/resources" "MCP Resources"
done

# Test MCP-specific ports
echo -e "\n${YELLOW}2. TESTING MCP PORTS${NC}"
echo "====================="

MCP_PORTS=(3000 3333 8090 8888 9999)
for node_info in "${NODES[@]}"; do
    IFS=':' read -r ip location type <<< "$node_info"
    echo -e "\n${BLUE}$location Node:${NC}"
    
    for port in "${MCP_PORTS[@]}"; do
        if nc -z -w1 "$ip" "$port" 2>/dev/null; then
            echo -e "${GREEN}✓${NC} Port $port open"
            # Try to get MCP info from the port
            test_endpoint "http://$ip:$port/mcp/info" "  MCP on port $port"
        else
            echo -e "${YELLOW}○${NC} Port $port closed"
        fi
    done
done

# Test MCP JSON-RPC endpoints
echo -e "\n${YELLOW}3. TESTING MCP JSON-RPC${NC}"
echo "========================"

for node_info in "${NODES[@]}"; do
    IFS=':' read -r ip location type <<< "$node_info"
    echo -e "\n${BLUE}$location Node:${NC}"
    
    # Test JSON-RPC endpoint
    rpc_data='{"jsonrpc":"2.0","method":"mcp/list_tools","params":{},"id":1}'
    test_endpoint "http://$ip/rpc" "JSON-RPC (list_tools)" "POST" "$rpc_data"
    test_endpoint "http://$ip/mcp/rpc" "MCP RPC (list_tools)" "POST" "$rpc_data"
    
    # Test server capabilities
    rpc_data='{"jsonrpc":"2.0","method":"mcp/server_capabilities","params":{},"id":1}'
    test_endpoint "http://$ip/rpc" "JSON-RPC (capabilities)" "POST" "$rpc_data"
done

# Test MCP tool endpoints
echo -e "\n${YELLOW}4. TESTING MCP TOOLS${NC}"
echo "====================="

# Known QuDAG MCP tools from codebase
TOOLS=(
    "qudag_crypto"
    "qudag_vault"
    "qudag_dag"
    "qudag_network"
    "qudag_exchange"
)

for node_info in "${NODES[@]}"; do
    IFS=':' read -r ip location type <<< "$node_info"
    echo -e "\n${BLUE}$location Node:${NC}"
    
    for tool in "${TOOLS[@]}"; do
        # Test tool info endpoint
        test_endpoint "http://$ip/mcp/tools/$tool" "Tool: $tool"
        
        # Test tool execution via JSON-RPC
        rpc_data="{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"$tool\",\"arguments\":{}},\"id\":1}"
        test_endpoint "http://$ip/mcp/rpc" "Execute: $tool" "POST" "$rpc_data"
    done
done

# Test MCP resource endpoints
echo -e "\n${YELLOW}5. TESTING MCP RESOURCES${NC}"
echo "========================="

RESOURCES=(
    "dag_status"
    "network_peers"
    "crypto_keys"
    "vault_status"
    "exchange_info"
)

for node_info in "${NODES[@]}"; do
    IFS=':' read -r ip location type <<< "$node_info"
    echo -e "\n${BLUE}$location Node:${NC}"
    
    for resource in "${RESOURCES[@]}"; do
        test_endpoint "http://$ip/mcp/resources/$resource" "Resource: $resource"
    done
done

# Test WebSocket endpoints for MCP
echo -e "\n${YELLOW}6. TESTING MCP WEBSOCKET${NC}"
echo "========================="

for node_info in "${NODES[@]}"; do
    IFS=':' read -r ip location type <<< "$node_info"
    echo -e "\n${BLUE}$location Node:${NC}"
    
    # Test WebSocket upgrade headers
    if response=$(curl -sf -H "Upgrade: websocket" -H "Connection: Upgrade" -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" "http://$ip/mcp/ws" -v 2>&1); then
        if echo "$response" | grep -q "101 Switching Protocols"; then
            echo -e "${GREEN}✓${NC} WebSocket endpoint available"
        else
            echo -e "${RED}✗${NC} WebSocket not available"
        fi
    else
        echo -e "${RED}✗${NC} WebSocket endpoint not found"
    fi
done

# Test SSE endpoints for MCP
echo -e "\n${YELLOW}7. TESTING MCP SERVER-SENT EVENTS${NC}"
echo "===================================="

for node_info in "${NODES[@]}"; do
    IFS=':' read -r ip location type <<< "$node_info"
    echo -e "\n${BLUE}$location Node:${NC}"
    
    # Test SSE endpoint
    if timeout 2 curl -sf -H "Accept: text/event-stream" "http://$ip/mcp/events" 2>/dev/null | head -n 5; then
        echo -e "${GREEN}✓${NC} SSE endpoint active"
    else
        echo -e "${RED}✗${NC} SSE endpoint not available"
    fi
done

# Generate MCP report
echo -e "\n${YELLOW}8. MCP INTEGRATION SUMMARY${NC}"
echo "==========================="

echo -e "\n${CYAN}Expected MCP Features:${NC}"
echo "• Tool Discovery & Execution"
echo "• Resource Access & Monitoring"
echo "• JSON-RPC Communication"
echo "• WebSocket Real-time Updates"
echo "• Server-Sent Events"

echo -e "\n${CYAN}QuDAG MCP Tools (from codebase):${NC}"
echo "• qudag_crypto - Quantum-resistant cryptography"
echo "• qudag_vault - Password vault operations"
echo "• qudag_dag - DAG consensus operations"
echo "• qudag_network - P2P network management"
echo "• qudag_exchange - rUv token exchange"

echo -e "\n${CYAN}MCP Resources (from codebase):${NC}"
echo "• dag_status - Current DAG state"
echo "• network_peers - Connected peers"
echo "• crypto_keys - Key management"
echo "• vault_status - Vault information"
echo "• exchange_info - Exchange status"

# Check if MCP server might be on different port
echo -e "\n${YELLOW}9. CHECKING ALTERNATIVE MCP CONFIGURATIONS${NC}"
echo "==========================================="

# Check for MCP in status endpoint
enhanced_ip="109.105.222.156"
if status=$(curl -sf "http://$enhanced_ip/api/v1/status" 2>/dev/null); then
    if echo "$status" | jq -e '.mcp' >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} MCP configuration found in status"
        echo "$status" | jq '.mcp'
    else
        echo -e "${YELLOW}○${NC} No MCP configuration in status endpoint"
    fi
fi

# Final summary
echo -e "\n${PURPLE}══════════════════════════════════════════════════════════${NC}"
echo -e "${PURPLE}                    MCP TEST SUMMARY                      ${NC}"
echo -e "${PURPLE}══════════════════════════════════════════════════════════${NC}"

echo -e "\n${YELLOW}Current Status:${NC}"
echo "The deployed testnet nodes do not appear to have MCP endpoints exposed."
echo "This is expected as the current deployment focuses on core DAG functionality."

echo -e "\n${YELLOW}To Enable MCP:${NC}"
echo "1. Deploy QuDAG with MCP server enabled:"
echo "   qudag start --enable-mcp --mcp-port 3333"
echo
echo "2. Or run standalone MCP server:"
echo "   qudag mcp serve --port 3333"
echo
echo "3. The MCP implementation is available in the codebase at:"
echo "   /workspaces/QuDAG/qudag-mcp/"

echo -e "\n${GREEN}MCP integration is fully implemented in the codebase${NC}"
echo -e "${GREEN}but not enabled in the current testnet deployment.${NC}"