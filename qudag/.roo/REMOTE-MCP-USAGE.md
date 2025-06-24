# Remote MCP Server Usage

## QuDAG Testnet MCP Server

The QuDAG testnet now has a live MCP (Model Context Protocol) server running on the Toronto node.

### Connection Details

- **Working URL**: http://109.105.222.156:3333/mcp
- **Transport**: HTTP (via stdio proxy)
- **Server**: QuDAG Toronto Node (qudag-testnet-node1)

### Known Issues

- **HTTPS Access**: The HTTPS endpoint `https://qudag-testnet-node1.fly.dev/mcp` times out due to Fly.io proxy issues
- **Workaround**: Use the direct HTTP endpoint on port 3333 instead
- **Intro Page**: The main site at `https://qudag-testnet-node1.fly.dev/` works fine

### Configuration Added to .roo/mcp.json

```json
"qudag-testnet": {
  "command": "node",
  "args": [
    "/workspaces/QuDAG/.roo/mcp-http-proxy.js",
    "http://109.105.222.156:3333"
  ],
  "alwaysAllow": [
    "qudag_crypto",
    "qudag_vault",
    "qudag_dag",
    "qudag_network",
    "qudag_exchange"
  ],
  "description": "QuDAG Testnet MCP Server (Toronto Node)",
  "timeout": 600
}
```

### Why Use a Proxy?

The MCP HTTP transport expects Server-Sent Events (SSE) for certain operations, but the QuDAG MCP server provides a standard HTTP/JSON API. The proxy script (`mcp-http-proxy.js`) bridges this gap by:

1. Converting stdio transport (which MCP clients prefer) to HTTP requests
2. Handling the JSON-RPC protocol correctly
3. Mapping MCP methods to the appropriate HTTP endpoints

This ensures compatibility with MCP clients that expect SSE or have specific transport requirements.

### Available Tools

1. **qudag_crypto** - Quantum-resistant cryptography operations
   - generate_keys (ml-dsa, ml-kem, hqc)
   - sign, verify, encrypt, decrypt

2. **qudag_vault** - Secure vault operations
   - create, unlock, store, retrieve, list

3. **qudag_dag** - DAG consensus operations
   - get_status, add_vertex, get_tips, validate

4. **qudag_network** - P2P network operations
   - list_peers, connect, disconnect, broadcast

5. **qudag_exchange** - rUv token exchange operations
   - create_account, get_balance, transfer, get_fee_info

### Available Resources

1. **dag_status** - Current DAG state and sync status
2. **network_peers** - Connected peers and bootstrap nodes
3. **crypto_keys** - Available cryptographic keys
4. **vault_status** - Vault state and configuration
5. **exchange_info** - Exchange configuration and fee model

### Testing the Connection

```bash
# Test MCP discovery
curl -s http://109.105.222.156:3333/mcp | jq '.'

# List available tools
curl -s http://109.105.222.156:3333/mcp/tools | jq '.tools[].name'

# List available resources
curl -s http://109.105.222.156:3333/mcp/resources | jq '.resources[].name'

# Execute a tool (example: get DAG status)
curl -X POST http://109.105.222.156:3333/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "qudag_dag",
    "arguments": {
      "operation": "get_status"
    }
  }' | jq '.'

# Subscribe to Server-Sent Events
curl -H "Accept: text/event-stream" http://109.105.222.156:3333/mcp/events
```

### Additional Endpoints

- **JSON-RPC**: http://109.105.222.156:3333/mcp/rpc
- **Server Info**: http://109.105.222.156:3333/mcp/info
- **Well-known**: http://109.105.222.156:3333/.well-known/mcp

### Usage in AI Applications

AI agents and applications can now connect to this remote MCP server to:
- Interact with the QuDAG quantum-resistant network
- Perform cryptographic operations
- Manage vaults and secrets
- Monitor network status
- Execute rUv token transfers

The server supports standard MCP protocols including:
- Tool discovery and execution
- Resource monitoring
- Server-Sent Events for real-time updates
- JSON-RPC interface

### Node Information

- **Node Name**: toronto-node
- **Network**: qudag-testnet
- **Version**: 1.0.0-mcp-v2
- **Features**: MCP enabled, HTTP integration, SSE support