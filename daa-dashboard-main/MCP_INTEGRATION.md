# MCP Integration Guide

## Overview

The DAA Dashboard has been updated to use the official Model Context Protocol (MCP) SDK for connecting to the DAA MCP server. This replaces the previous mock implementation with a real, production-ready MCP client.

## Architecture

### Key Components

1. **MCP Client** (`/src/lib/mcp-client.ts`)
   - Official `@modelcontextprotocol/sdk` implementation
   - Support for multiple transport mechanisms (HTTP, SSE, WebSocket)
   - Automatic retry logic and connection management
   - Type-safe tool calling with full TypeScript support

2. **Type Definitions** (`/src/lib/types.ts`)
   - Complete TypeScript interfaces for all DAA MCP tools
   - Type-safe arguments and return values
   - Connection state and event handling types

3. **Configuration** (`/src/lib/config.ts`)
   - Environment-based configuration management
   - Development and production settings
   - Transport and authentication options

4. **Environment Variables** (`.env.local`, `.env.example`)
   - Server connection settings
   - Transport configuration
   - Debug and authentication options

## Configuration

### Environment Variables

Copy `.env.example` to `.env.local` and configure:

```bash
# MCP Server Connection
VITE_MCP_SERVER_URL=http://localhost:3000
VITE_MCP_SERVER_NAME=daa-mcp-server
VITE_MCP_PROTOCOL_VERSION=2024-11-05

# MCP Client Configuration
VITE_MCP_CLIENT_NAME=daa-dashboard
VITE_MCP_CLIENT_VERSION=1.0.0

# Connection Settings
VITE_MCP_CONNECT_TIMEOUT=10000
VITE_MCP_REQUEST_TIMEOUT=30000
VITE_MCP_MAX_RETRIES=3
VITE_MCP_RETRY_DELAY=1000

# Transport Configuration
VITE_MCP_TRANSPORT_TYPE=http
VITE_MCP_ENABLE_WEBSOCKET=false
VITE_MCP_WEBSOCKET_URL=ws://localhost:3001

# Development/Debug Settings
VITE_MCP_DEBUG_MODE=true
VITE_MCP_LOG_LEVEL=debug
```

### Transport Options

1. **HTTP (Streamable)** - Recommended for production
   - Modern transport with streaming support
   - Automatic fallback to SSE if unavailable
   - Best performance and reliability

2. **Server-Sent Events (SSE)** - Legacy fallback
   - One-way server-to-client communication
   - Good browser compatibility
   - Fallback option for older servers

3. **WebSocket** - Real-time bidirectional
   - Full-duplex communication
   - Ideal for real-time updates
   - Requires WebSocket server support

## Usage

### Basic Connection

```typescript
import { daaMcpClient } from '@/lib/mcp-client';

// Connect to the MCP server
await daaMcpClient.connect();

// Check connection status
const isConnected = daaMcpClient.isConnected();
```

### Tool Calling

```typescript
// Get DAA system status
const status = await daaMcpClient.getStatus();

// List all agents
const agents = await daaMcpClient.listAgents();

// Get specific agent details
const agent = await daaMcpClient.getAgentDetails('agent-001');

// Create a new agent
const newAgent = await daaMcpClient.createAgent('trading_bot', 'trading', 'crypto_trading');
```

### Event Handling

```typescript
// Listen for connection events
daaMcpClient.on('connected', () => {
  console.log('MCP client connected');
});

daaMcpClient.on('disconnected', () => {
  console.log('MCP client disconnected');
});

daaMcpClient.on('error', (error) => {
  console.error('MCP client error:', error);
});
```

### Health Checking

```typescript
// Perform a health check
const health = await daaMcpClient.healthCheck();
console.log('Health status:', health.status);
console.log('Details:', health.details);
```

## React Integration

The existing React hooks (`/src/hooks/use-daa-mcp.ts`) continue to work without changes. They now use the real MCP client instead of mock responses:

```typescript
import { useDaaStatus, useDaaAgents } from '@/hooks/use-daa-mcp';

function MyComponent() {
  const { data: status, isLoading, error } = useDaaStatus();
  const { data: agents } = useDaaAgents();
  
  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  
  return (
    <div>
      <h1>System Status: {status.orchestrator}</h1>
      <p>Active Agents: {agents.length}</p>
    </div>
  );
}
```

## Development and Testing

### NPM Scripts

```bash
# Run type checking
npm run typecheck

# Test MCP connection
npm run mcp:test

# List available MCP tools
npm run mcp:tools
```

### Development Mode

When `VITE_MCP_DEBUG_MODE=true`, the client will:
- Log detailed connection information
- Display transport negotiation details
- Show tool call arguments and responses
- Provide verbose error messages

### Testing Connection

To test the MCP connection manually:

```bash
npm run mcp:test
```

This will attempt to connect to the configured MCP server and report the health status.

## Error Handling

The MCP client includes comprehensive error handling:

1. **Connection Errors**: Automatic retry with exponential backoff
2. **Network Errors**: Graceful degradation and reconnection attempts
3. **Tool Errors**: Proper error propagation with context
4. **Transport Fallback**: Automatic fallback from HTTP to SSE

### Error Recovery

- **Automatic Reconnection**: Failed connections trigger automatic retry
- **Exponential Backoff**: Retry delays increase exponentially to prevent server overload
- **Transport Fallback**: If HTTP fails, automatically tries SSE transport
- **Connection Pooling**: Maintains connection state across tool calls

## Migration from Mock Implementation

The migration is backwards compatible:

1. **API Compatibility**: All existing method signatures remain the same
2. **Hook Compatibility**: React hooks continue to work without changes
3. **Error Handling**: Enhanced error handling with graceful fallbacks
4. **Type Safety**: Improved TypeScript support with strict typing

### What Changed

- **Real MCP Calls**: Tool calls now go to the actual DAA MCP server
- **Connection Management**: Proper connection lifecycle management
- **Error Handling**: Robust error handling and retry logic
- **Type Safety**: Full TypeScript support for all tool arguments and responses
- **Configuration**: Environment-based configuration system

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Ensure the DAA MCP server is running
   - Check the server URL in `.env.local`
   - Verify firewall settings

2. **Protocol Version Mismatch**
   - Use `2024-11-05` for maximum compatibility
   - Check server and client protocol versions

3. **Transport Errors**
   - Try different transport types (http, sse, websocket)
   - Check CORS settings for browser-based clients
   - Verify WebSocket support if using WebSocket transport

4. **Authentication Errors**
   - Set `VITE_MCP_AUTH_ENABLED=true` if server requires auth
   - Provide valid `VITE_MCP_AUTH_TOKEN`

### Debug Information

Enable debug mode for detailed logging:

```bash
VITE_MCP_DEBUG_MODE=true
VITE_MCP_LOG_LEVEL=debug
```

This will provide:
- Connection establishment details
- Transport negotiation information
- Tool call request/response data
- Error stack traces

## Future Enhancements

Planned improvements:

1. **Real-time Updates**: WebSocket-based live data streaming
2. **Connection Pooling**: Multiple server connections
3. **Caching Layer**: Intelligent response caching
4. **Batch Operations**: Batch tool calls for better performance
5. **Monitoring**: Connection health monitoring and metrics

## Security Considerations

1. **Authentication**: OAuth 2.1 support for secure connections
2. **CORS**: Proper CORS configuration for browser clients
3. **Input Validation**: Tool argument validation
4. **Error Sanitization**: Secure error message handling

For questions or issues with the MCP integration, refer to the official documentation at https://modelcontextprotocol.io/