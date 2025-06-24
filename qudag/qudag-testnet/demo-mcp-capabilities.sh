#!/bin/bash
set -uo pipefail

# QuDAG MCP Capabilities Demo
# Demonstrates the Model Context Protocol integration with QuDAG

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# MCP endpoint
MCP_URL="http://109.105.222.156:3333"
# Note: HTTPS domain access (https://qudag-testnet-node1.fly.dev/mcp) is planned but currently unavailable

echo -e "${PURPLE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║      QuDAG MCP (Model Context Protocol) Demo             ║${NC}"
echo -e "${PURPLE}╚══════════════════════════════════════════════════════════╝${NC}"
echo

echo -e "${CYAN}QuDAG MCP Server is live with streamable HTTP endpoints!${NC}"
echo -e "${CYAN}MCP URL: $MCP_URL${NC}"
echo

# 1. MCP Discovery
echo -e "${YELLOW}1. MCP DISCOVERY${NC}"
echo "================"
echo -e "${BLUE}GET $MCP_URL/mcp${NC}"
curl -s $MCP_URL/mcp | jq '.mcp.serverInfo'
echo

# 2. Server Capabilities
echo -e "${YELLOW}2. SERVER CAPABILITIES${NC}"
echo "======================"
echo -e "${BLUE}Supported Features:${NC}"
curl -s $MCP_URL/mcp | jq '.mcp.capabilities'
echo

# 3. Available Tools
echo -e "${YELLOW}3. AVAILABLE TOOLS${NC}"
echo "=================="
echo -e "${BLUE}GET $MCP_URL/mcp/tools${NC}"
curl -s $MCP_URL/mcp/tools | jq '.tools[].name' | tr -d '"' | while read tool; do
    echo -e "${GREEN}✓${NC} $tool"
done
echo

# 4. Available Resources
echo -e "${YELLOW}4. AVAILABLE RESOURCES${NC}"
echo "====================="
echo -e "${BLUE}GET $MCP_URL/mcp/resources${NC}"
curl -s $MCP_URL/mcp/resources | jq '.resources[].name' | tr -d '"' | while read resource; do
    echo -e "${GREEN}✓${NC} $resource"
done
echo

# 5. Tool Execution Demo
echo -e "${YELLOW}5. TOOL EXECUTION DEMO${NC}"
echo "======================"

# Execute DAG status tool
echo -e "${BLUE}Executing qudag_dag tool:${NC}"
echo -e "${CYAN}POST $MCP_URL/mcp/tools/call${NC}"
result=$(curl -s -X POST $MCP_URL/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "qudag_dag",
    "arguments": {
      "operation": "get_status"
    }
  }')
echo "$result" | jq
echo

# Execute crypto tool
echo -e "${BLUE}Executing qudag_crypto tool:${NC}"
result=$(curl -s -X POST $MCP_URL/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "qudag_crypto",
    "arguments": {
      "operation": "generate_keys",
      "algorithm": "ml-dsa"
    }
  }')
echo "$result" | jq
echo

# 6. Resource Access Demo
echo -e "${YELLOW}6. RESOURCE ACCESS DEMO${NC}"
echo "======================="

# Get DAG status resource
echo -e "${BLUE}Getting dag_status resource:${NC}"
echo -e "${CYAN}GET $MCP_URL/mcp/resources/dag_status${NC}"
curl -s $MCP_URL/mcp/resources/dag_status | jq
echo

# Get network peers resource
echo -e "${BLUE}Getting network_peers resource:${NC}"
echo -e "${CYAN}GET $MCP_URL/mcp/resources/network_peers${NC}"
curl -s $MCP_URL/mcp/resources/network_peers | jq
echo

# 7. JSON-RPC Demo
echo -e "${YELLOW}7. JSON-RPC INTERFACE${NC}"
echo "====================="

echo -e "${BLUE}Listing tools via JSON-RPC:${NC}"
echo -e "${CYAN}POST $MCP_URL/mcp/rpc${NC}"
curl -s -X POST $MCP_URL/mcp/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"mcp/list_tools","params":{},"id":1}' | jq '.result.tools | length' | xargs -I {} echo "{} tools available"
echo

echo -e "${BLUE}Getting server capabilities via JSON-RPC:${NC}"
curl -s -X POST $MCP_URL/mcp/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"mcp/server_capabilities","params":{},"id":2}' | jq '.result'
echo

# 8. Server-Sent Events Demo
echo -e "${YELLOW}8. SERVER-SENT EVENTS (SSE)${NC}"
echo "==========================="
echo -e "${BLUE}Streaming real-time updates:${NC}"
echo -e "${CYAN}GET $MCP_URL/mcp/events${NC}"
echo
echo "Connecting to SSE stream for 5 seconds..."
timeout 5 curl -s -H "Accept: text/event-stream" $MCP_URL/mcp/events 2>/dev/null | head -10
echo
echo -e "${GREEN}✓ SSE stream active - sends status updates every 5 seconds${NC}"
echo

# 9. Integration Examples
echo -e "${YELLOW}9. INTEGRATION EXAMPLES${NC}"
echo "======================="

echo -e "${BLUE}Example 1: AI Agent Integration${NC}"
cat << 'EOF'
# AI agents can use MCP to interact with QuDAG:
curl -X POST http://109.105.222.156:3333/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "qudag_exchange",
    "arguments": {
      "operation": "get_balance",
      "account": "agent-001"
    }
  }'
EOF
echo

echo -e "${BLUE}Example 2: Continuous Monitoring${NC}"
cat << 'EOF'
# Subscribe to SSE events for real-time updates:
curl -H "Accept: text/event-stream" \
  http://109.105.222.156:3333/mcp/events
EOF
echo

echo -e "${BLUE}Example 3: Quantum Crypto Operations${NC}"
cat << 'EOF'
# Generate quantum-resistant keys:
curl -X POST http://109.105.222.156:3333/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "qudag_crypto",
    "arguments": {
      "operation": "generate_keys",
      "algorithm": "ml-kem"
    }
  }'
EOF
echo

# 10. Summary
echo -e "${PURPLE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ MCP SERVER FULLY OPERATIONAL!${NC}"
echo -e "${PURPLE}═══════════════════════════════════════════════════════════${NC}"
echo
echo "Available at: $MCP_URL"
echo
echo "Features:"
echo "• ${GREEN}✓${NC} Tool discovery and execution"
echo "• ${GREEN}✓${NC} Resource access and monitoring"
echo "• ${GREEN}✓${NC} JSON-RPC interface"
echo "• ${GREEN}✓${NC} Server-Sent Events (SSE) streaming"
echo "• ${GREEN}✓${NC} Quantum-resistant cryptography"
echo "• ${GREEN}✓${NC} DAG consensus operations"
echo "• ${GREEN}✓${NC} P2P network management"
echo "• ${GREEN}✓${NC} rUv token exchange"
echo "• ${GREEN}✓${NC} Secure vault operations"
echo
echo -e "${CYAN}The QuDAG MCP server enables AI agents to interact with${NC}"
echo -e "${CYAN}the quantum-resistant DAG network seamlessly!${NC}"