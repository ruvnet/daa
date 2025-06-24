# QuDAG MCP Server Access Guide

## Current Status

The MCP server is fully operational on the QuDAG testnet node1.

### ✅ Working Access Methods

**Direct IP + Port Access (Recommended)**
- Base URL: `http://109.105.222.156:3333`
- Example: `http://109.105.222.156:3333/mcp`
- All MCP endpoints work perfectly via this method

### ⚠️ Domain Access Status

**HTTPS Domain Access**
- URL: `https://qudag-testnet-node1.fly.dev/mcp` 
- Status: Currently experiencing routing issues
- Alternative: Use direct IP access above

## Available Endpoints

All endpoints are accessible at `http://109.105.222.156:3333`:

| Endpoint | Method | Description |
|----------|---------|-------------|
| `/mcp` | GET | MCP discovery and capabilities |
| `/mcp/info` | GET | Server information |
| `/mcp/tools` | GET | List available tools |
| `/mcp/tools/call` | POST | Execute a tool |
| `/mcp/resources` | GET | List available resources |
| `/mcp/resources/:name` | GET | Get specific resource |
| `/mcp/events` | GET | Server-Sent Events stream |
| `/mcp/rpc` | POST | JSON-RPC interface |
| `/.well-known/mcp` | GET | Well-known MCP endpoint |

## Quick Examples

### Discovery
```bash
curl http://109.105.222.156:3333/mcp | jq
```

### List Tools
```bash
curl http://109.105.222.156:3333/mcp/tools | jq
```

### Execute Tool
```bash
curl -X POST http://109.105.222.156:3333/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "qudag_dag",
    "arguments": {
      "operation": "get_status"
    }
  }' | jq
```

### JSON-RPC
```bash
curl -X POST http://109.105.222.156:3333/mcp/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"mcp/list_tools","params":{},"id":1}' | jq
```

### SSE Stream
```bash
curl -H "Accept: text/event-stream" http://109.105.222.156:3333/mcp/events
```

## Technical Details

- **MCP Port**: 3333 (dedicated MCP server)
- **HTTP Port**: 8080 (main HTTP API - MCP integration pending)
- **Protocol Version**: 2024-11-05
- **Server Version**: 1.0.0

## Future Improvements

We attempted to make MCP available through the main HTTP port (8080) and Fly.io's HTTPS proxy, but encountered routing issues. For now, please use the direct IP + port access method above.

The MCP server is fully functional and ready for AI agent integration!