# Model Context Protocol (MCP) Analysis for React Dashboard Integration

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [Protocol Message Formats](#protocol-message-formats)
3. [Transport Layer Options](#transport-layer-options)
4. [Error Handling Patterns](#error-handling-patterns)
5. [Authentication Mechanisms](#authentication-mechanisms)
6. [Real-Time Update Strategies](#real-time-update-strategies)
7. [Implementation Recommendations](#implementation-recommendations)
8. [Code Examples](#code-examples)
9. [Future Considerations](#future-considerations)

## Executive Summary

The Model Context Protocol (MCP) is an open standard developed by Anthropic that enables standardized communication between AI applications and external data sources, tools, and services. This analysis provides comprehensive documentation for implementing an MCP client in the DAA React dashboard.

### Key Protocol Characteristics
- **Foundation**: JSON-RPC 2.0 messaging protocol
- **Architecture**: Client-server with stateful connections
- **Transport Options**: stdio, HTTP+SSE, Streamable HTTP (new), WebSocket (future)
- **Capability Model**: Tools (executable), Resources (read-only), Prompts (templates)
- **Real-time Support**: Server-Sent Events (SSE) for streaming updates
- **Protocol Version**: Currently `2024-11-05` (stable), `2025-03-26` (latest)

## Protocol Message Formats

### JSON-RPC 2.0 Foundation

MCP uses JSON-RPC 2.0 as its messaging foundation with three message types:

#### 1. Request Messages
```json
{
  "jsonrpc": "2.0",
  "id": "unique_request_id",
  "method": "method_name",
  "params": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

**Requirements:**
- `jsonrpc`: Must be "2.0"
- `id`: Must be string or integer, NOT null
- `method`: Target method name
- `params`: Optional parameters object

#### 2. Response Messages

**Success Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "matching_request_id",
  "result": {
    "data": "response_data"
  }
}
```

**Error Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "matching_request_id",
  "error": {
    "code": -32602,
    "message": "Invalid parameters",
    "data": {
      "field": "parameter_name",
      "reason": "Specific error details"
    }
  }
}
```

#### 3. Notification Messages (One-way)
```json
{
  "jsonrpc": "2.0",
  "method": "notification_method",
  "params": {
    "event": "status_update",
    "data": "notification_data"
  }
}
```

### Standard MCP Methods

#### Connection Lifecycle
```json
// Initialize connection
{
  "jsonrpc": "2.0",
  "id": "init_001",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "experimental": {},
      "roots": {
        "listChanged": true
      }
    },
    "clientInfo": {
      "name": "DAA-Dashboard-Client",
      "version": "1.0.0"
    }
  }
}
```

#### Capability Discovery
```json
// List available tools
{
  "jsonrpc": "2.0",
  "id": "tools_001",
  "method": "tools/list"
}

// List available resources
{
  "jsonrpc": "2.0",
  "id": "resources_001",
  "method": "resources/list"
}

// List available prompts
{
  "jsonrpc": "2.0",
  "id": "prompts_001",
  "method": "prompts/list"
}
```

#### Capability Invocation
```json
// Execute tool
{
  "jsonrpc": "2.0",
  "id": "tool_exec_001",
  "method": "tools/call",
  "params": {
    "name": "daa_status",
    "arguments": {
      "detailed": true
    }
  }
}

// Read resource
{
  "jsonrpc": "2.0",
  "id": "resource_read_001",
  "method": "resources/read",
  "params": {
    "uri": "daa://status/orchestrator"
  }
}
```

### Message Batching

MCP supports JSON-RPC batching for efficiency:
```json
[
  {
    "jsonrpc": "2.0",
    "id": "batch_1",
    "method": "tools/list"
  },
  {
    "jsonrpc": "2.0",
    "id": "batch_2",
    "method": "resources/list"
  },
  {
    "jsonrpc": "2.0",
    "id": "batch_3",
    "method": "daa_status"
  }
]
```

## Transport Layer Options

### 1. Standard I/O (stdio) Transport

**Best for:** Local server processes, development, testing

**Characteristics:**
- Newline-delimited JSON messages
- Process-based lifecycle management
- Lowest latency for local operations

**Message Framing:**
```
{"jsonrpc":"2.0","id":"1","method":"initialize","params":{...}}
{"jsonrpc":"2.0","id":"1","result":{...}}
```

**Implementation Pattern:**
```typescript
// Not suitable for React web clients
// Used for local MCP server processes
```

### 2. HTTP + Server-Sent Events (SSE) Transport

**Best for:** Web clients, real-time updates, remote servers

**Connection Flow:**
```
Client                    Server
  |                         |
  |-- POST /mcp/connect --->|
  |<-- 200 + SSE Stream ----|
  |                         |
  |== SSE Messages ========>|
  |<== SSE Responses =======|
```

**Initial Connection (HTTP POST):**
```http
POST /mcp/connect HTTP/1.1
Host: daa-mcp-server.example.com
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": "init",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {}
  }
}
```

**SSE Stream Response:**
```http
HTTP/1.1 200 OK
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive

data: {"jsonrpc":"2.0","id":"init","result":{"protocolVersion":"2024-11-05",...}}

data: {"jsonrpc":"2.0","method":"notification","params":{"status":"ready"}}
```

**Client Implementation:**
```typescript
class MCPSSEClient {
  private eventSource: EventSource | null = null;
  private baseUrl: string;

  async connect(baseUrl: string) {
    this.baseUrl = baseUrl;
    
    // Initialize connection
    const initResponse = await fetch(`${baseUrl}/mcp/connect`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 'init',
        method: 'initialize',
        params: {
          protocolVersion: '2024-11-05',
          capabilities: {}
        }
      })
    });

    if (initResponse.headers.get('content-type')?.includes('text/event-stream')) {
      // SSE stream established
      this.eventSource = new EventSource(`${baseUrl}/mcp/connect`);
      
      this.eventSource.onmessage = (event) => {
        const message = JSON.parse(event.data);
        this.handleMessage(message);
      };
    }
  }

  async sendRequest(method: string, params?: any) {
    const request = {
      jsonrpc: '2.0',
      id: this.generateId(),
      method,
      params
    };

    const response = await fetch(`${this.baseUrl}/mcp/message`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request)
    });

    return response.json();
  }
}
```

### 3. Streamable HTTP Transport (New in 2025-03-26)

**Best for:** Simplified web client integration, stateless connections

**Key Improvements:**
- Single endpoint for all operations
- Supports streaming responses directly from `/messages`
- Optional SSE upgrade
- Simplified connection model

**Implementation:**
```typescript
class MCPStreamableClient {
  async sendMessage(message: any): Promise<Response> {
    return fetch('/mcp/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json, text/event-stream'
      },
      body: JSON.stringify(message)
    });
  }

  async callTool(name: string, args: any) {
    const response = await this.sendMessage({
      jsonrpc: '2.0',
      id: this.generateId(),
      method: 'tools/call',
      params: { name, arguments: args }
    });

    const contentType = response.headers.get('content-type');
    
    if (contentType?.includes('text/event-stream')) {
      // Handle streaming response
      return this.handleSSEResponse(response);
    } else {
      // Handle JSON response
      return response.json();
    }
  }
}
```

### 4. WebSocket Transport (Future)

**Best for:** Real-time bidirectional communication, agent-to-agent interactions

**Connection Setup:**
```typescript
class MCPWebSocketClient {
  private ws: WebSocket | null = null;

  connect(url: string) {
    this.ws = new WebSocket(url);
    
    this.ws.onopen = () => {
      this.initialize();
    };

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };
  }

  sendMessage(message: any) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }
}
```

## Error Handling Patterns

### JSON-RPC Error Codes

MCP follows JSON-RPC 2.0 error codes with extensions:

```typescript
enum MCPErrorCodes {
  // JSON-RPC 2.0 standard errors
  PARSE_ERROR = -32700,
  INVALID_REQUEST = -32600,
  METHOD_NOT_FOUND = -32601,
  INVALID_PARAMS = -32602,
  INTERNAL_ERROR = -32603,

  // MCP-specific errors (implementation-defined range)
  AUTHENTICATION_FAILED = -32000,
  AUTHORIZATION_FAILED = -32001,
  RESOURCE_NOT_FOUND = -32002,
  TOOL_EXECUTION_ERROR = -32003,
  TRANSPORT_ERROR = -32004,
  SESSION_EXPIRED = -32005
}
```

### Error Response Structure

```json
{
  "jsonrpc": "2.0",
  "id": "failed_request_id",
  "error": {
    "code": -32002,
    "message": "Resource not found",
    "data": {
      "uri": "daa://invalid/resource",
      "available_resources": [
        "daa://status/orchestrator",
        "daa://agents/list"
      ],
      "suggestion": "Check available resources using resources/list"
    }
  }
}
```

### Client-Side Error Handling

```typescript
class MCPErrorHandler {
  static handleError(error: any): MCPError {
    const { code, message, data } = error;

    switch (code) {
      case MCPErrorCodes.AUTHENTICATION_FAILED:
        return new AuthenticationError(message, data);
      
      case MCPErrorCodes.RESOURCE_NOT_FOUND:
        return new ResourceNotFoundError(message, data);
      
      case MCPErrorCodes.TOOL_EXECUTION_ERROR:
        return new ToolExecutionError(message, data);
      
      default:
        return new GenericMCPError(code, message, data);
    }
  }

  static isRetryableError(error: MCPError): boolean {
    return [
      MCPErrorCodes.INTERNAL_ERROR,
      MCPErrorCodes.TRANSPORT_ERROR
    ].includes(error.code);
  }
}
```

### Transport-Specific Error Handling

#### SSE Connection Errors
```typescript
class SSEErrorHandler {
  setupErrorHandling(eventSource: EventSource) {
    eventSource.onerror = (event) => {
      console.error('SSE connection error:', event);
      
      // Implement exponential backoff retry
      this.retryConnection();
    };
  }

  private async retryConnection(attempt = 1) {
    const delay = Math.min(1000 * Math.pow(2, attempt), 30000);
    
    setTimeout(() => {
      if (attempt < 5) {
        this.connect().catch(() => this.retryConnection(attempt + 1));
      }
    }, delay);
  }
}
```

#### HTTP Request Errors
```typescript
async function robustMCPRequest(url: string, request: any) {
  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
      timeout: 10000
    });

    if (!response.ok) {
      throw new TransportError(`HTTP ${response.status}: ${response.statusText}`);
    }

    return await response.json();
  } catch (error) {
    if (error instanceof TypeError) {
      throw new NetworkError('Network connection failed');
    }
    throw error;
  }
}
```

## Authentication Mechanisms

### Current State

MCP v2024-11-05 does not specify a standardized authentication mechanism, leaving implementation to developers. The upcoming v2025-03-26 introduces OAuth 2.1 framework.

### OAuth 2.1 Framework (v2025-03-26)

#### Authorization Code Flow with PKCE
```typescript
class MCPOAuthClient {
  private clientId: string;
  private redirectUri: string;

  async initiateAuth(): Promise<string> {
    // Generate PKCE parameters
    const codeVerifier = this.generateCodeVerifier();
    const codeChallenge = await this.generateCodeChallenge(codeVerifier);
    
    // Store for later use
    sessionStorage.setItem('code_verifier', codeVerifier);

    const authUrl = new URL('/oauth/authorize', this.baseUrl);
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('client_id', this.clientId);
    authUrl.searchParams.set('redirect_uri', this.redirectUri);
    authUrl.searchParams.set('code_challenge', codeChallenge);
    authUrl.searchParams.set('code_challenge_method', 'S256');
    authUrl.searchParams.set('scope', 'mcp:tools mcp:resources');

    return authUrl.toString();
  }

  async exchangeCodeForToken(code: string): Promise<AuthTokens> {
    const codeVerifier = sessionStorage.getItem('code_verifier');
    
    const response = await fetch('/oauth/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code,
        redirect_uri: this.redirectUri,
        client_id: this.clientId,
        code_verifier: codeVerifier!
      })
    });

    return response.json();
  }
}
```

#### Token-Based Authentication
```typescript
class AuthenticatedMCPClient {
  private accessToken: string | null = null;
  private refreshToken: string | null = null;

  setTokens(tokens: AuthTokens) {
    this.accessToken = tokens.access_token;
    this.refreshToken = tokens.refresh_token;
  }

  async authenticatedRequest(request: any) {
    if (!this.accessToken) {
      throw new AuthenticationError('No access token available');
    }

    const response = await fetch('/mcp/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.accessToken}`
      },
      body: JSON.stringify(request)
    });

    if (response.status === 401) {
      await this.refreshAccessToken();
      return this.authenticatedRequest(request);
    }

    return response.json();
  }

  private async refreshAccessToken() {
    if (!this.refreshToken) {
      throw new AuthenticationError('No refresh token available');
    }

    const response = await fetch('/oauth/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: this.refreshToken
      })
    });

    if (!response.ok) {
      throw new AuthenticationError('Token refresh failed');
    }

    const tokens = await response.json();
    this.setTokens(tokens);
  }
}
```

### Custom Authentication Patterns

For current implementations, consider these patterns:

#### API Key Authentication
```typescript
class APIKeyMCPClient {
  constructor(private apiKey: string) {}

  async sendRequest(request: any) {
    return fetch('/mcp/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-API-Key': this.apiKey
      },
      body: JSON.stringify(request)
    });
  }
}
```

#### Session-Based Authentication
```typescript
class SessionMCPClient {
  async authenticate(credentials: LoginCredentials) {
    const response = await fetch('/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(credentials),
      credentials: 'include' // Include cookies
    });

    if (!response.ok) {
      throw new AuthenticationError('Login failed');
    }

    return response.json();
  }

  async sendRequest(request: any) {
    return fetch('/mcp/messages', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include', // Include session cookie
      body: JSON.stringify(request)
    });
  }
}
```

## Real-Time Update Strategies

### Server-Sent Events (SSE) Implementation

#### Client-Side SSE Handler
```typescript
class MCPSSEHandler {
  private eventSource: EventSource | null = null;
  private listeners: Map<string, Function[]> = new Map();

  async initializeSSE(baseUrl: string) {
    // Resume capability with Last-Event-ID
    const lastEventId = localStorage.getItem('mcp_last_event_id');
    const headers: Record<string, string> = {};
    
    if (lastEventId) {
      headers['Last-Event-ID'] = lastEventId;
    }

    this.eventSource = new EventSource(`${baseUrl}/mcp/stream`, {
      withCredentials: true
    });

    this.eventSource.onmessage = (event) => {
      // Store event ID for resumption
      if (event.lastEventId) {
        localStorage.setItem('mcp_last_event_id', event.lastEventId);
      }

      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };

    this.eventSource.onerror = (error) => {
      console.error('SSE Error:', error);
      this.handleReconnection();
    };
  }

  private handleMessage(message: any) {
    if (message.method) {
      // Handle notification
      this.emitNotification(message.method, message.params);
    } else if (message.result || message.error) {
      // Handle response
      this.handleResponse(message);
    }
  }

  subscribe(event: string, callback: Function) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(callback);
  }

  private emitNotification(method: string, params: any) {
    const callbacks = this.listeners.get(method) || [];
    callbacks.forEach(callback => callback(params));
  }
}
```

#### Progress Notifications
```typescript
interface ProgressNotification {
  operation: string;
  progress: number; // 0.0 to 1.0
  message?: string;
  total?: number;
  current?: number;
}

class ProgressTracker {
  trackOperation(operationId: string, callback: (progress: ProgressNotification) => void) {
    mcpClient.subscribe('progress', (params: any) => {
      if (params.operation === operationId) {
        callback({
          operation: params.operation,
          progress: params.progress,
          message: params.message,
          total: params.total,
          current: params.current
        });
      }
    });
  }
}

// Usage in React component
function useOperationProgress(operationId: string) {
  const [progress, setProgress] = useState<ProgressNotification | null>(null);

  useEffect(() => {
    const tracker = new ProgressTracker();
    tracker.trackOperation(operationId, setProgress);
  }, [operationId]);

  return progress;
}
```

### Resource Change Subscriptions

```typescript
interface ResourceSubscription {
  uri: string;
  changeTypes: ('created' | 'modified' | 'deleted')[];
}

class ResourceWatcher {
  async subscribe(subscription: ResourceSubscription) {
    await mcpClient.sendRequest('resources/subscribe', {
      uri: subscription.uri,
      changeTypes: subscription.changeTypes
    });

    mcpClient.subscribe('resource_changed', (params: any) => {
      if (this.matchesSubscription(params.uri, subscription.uri)) {
        this.handleResourceChange(params);
      }
    });
  }

  private matchesSubscription(resourceUri: string, pattern: string): boolean {
    // Support glob patterns like "daa://agents/*"
    const regex = pattern.replace(/\*/g, '.*');
    return new RegExp(`^${regex}$`).test(resourceUri);
  }

  private handleResourceChange(change: any) {
    // Emit change event to interested components
    this.emit('change', {
      uri: change.uri,
      changeType: change.type,
      data: change.data
    });
  }
}
```

### React Integration Patterns

#### Real-time Data Hook
```typescript
function useRealtimeMCPData<T>(
  method: string,
  params?: any,
  subscriptions?: string[]
) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;

    // Initial data fetch
    const fetchData = async () => {
      try {
        const result = await mcpClient.sendRequest(method, params);
        if (mounted) {
          setData(result);
          setLoading(false);
        }
      } catch (err) {
        if (mounted) {
          setError(err as Error);
          setLoading(false);
        }
      }
    };

    fetchData();

    // Subscribe to real-time updates
    const unsubscribers: (() => void)[] = [];
    
    subscriptions?.forEach(subscription => {
      const unsubscribe = mcpClient.subscribe(subscription, (update: any) => {
        if (mounted) {
          setData(prevData => ({ ...prevData, ...update }));
        }
      });
      unsubscribers.push(unsubscribe);
    });

    return () => {
      mounted = false;
      unsubscribers.forEach(unsub => unsub());
    };
  }, [method, JSON.stringify(params)]);

  return { data, loading, error };
}

// Usage
function AgentList() {
  const { data: agents, loading, error } = useRealtimeMCPData<Agent[]>(
    'daa_agent_list',
    {},
    ['agent_created', 'agent_updated', 'agent_deleted']
  );

  if (loading) return <div>Loading agents...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return (
    <div>
      {agents?.map(agent => (
        <AgentCard key={agent.id} agent={agent} />
      ))}
    </div>
  );
}
```

## Implementation Recommendations

### React Dashboard MCP Client Architecture

```typescript
// Core MCP Client
class DAAMCPClient {
  private transport: MCPTransport;
  private auth: MCPAuthenticator;
  private eventBus: EventEmitter;

  constructor(config: MCPClientConfig) {
    this.transport = this.createTransport(config.transport);
    this.auth = new MCPAuthenticator(config.auth);
    this.eventBus = new EventEmitter();
  }

  async connect() {
    await this.auth.authenticate();
    await this.transport.connect();
    this.setupEventHandlers();
  }

  async callTool(name: string, args: any) {
    const request = {
      jsonrpc: '2.0',
      id: this.generateId(),
      method: 'tools/call',
      params: { name, arguments: args }
    };

    return this.transport.sendRequest(request);
  }

  subscribe(event: string, callback: Function) {
    this.eventBus.on(event, callback);
  }
}

// React Context Provider
const MCPContext = createContext<DAAMCPClient | null>(null);

export function MCPProvider({ children }: { children: ReactNode }) {
  const [client] = useState(() => new DAAMCPClient({
    transport: {
      type: 'sse',
      baseUrl: process.env.REACT_APP_MCP_BASE_URL || 'http://localhost:3001'
    },
    auth: {
      type: 'oauth2',
      clientId: process.env.REACT_APP_MCP_CLIENT_ID
    }
  }));

  useEffect(() => {
    client.connect().catch(console.error);
    return () => client.disconnect();
  }, [client]);

  return (
    <MCPContext.Provider value={client}>
      {children}
    </MCPContext.Provider>
  );
}

export const useMCP = () => {
  const client = useContext(MCPContext);
  if (!client) {
    throw new Error('useMCP must be used within MCPProvider');
  }
  return client;
};
```

### Configuration Management

```typescript
interface MCPClientConfig {
  transport: {
    type: 'stdio' | 'sse' | 'streamable-http' | 'websocket';
    baseUrl?: string;
    reconnectOptions?: {
      maxAttempts: number;
      initialDelay: number;
      maxDelay: number;
    };
  };
  auth: {
    type: 'none' | 'api-key' | 'oauth2' | 'session';
    apiKey?: string;
    clientId?: string;
    scopes?: string[];
  };
  capabilities: {
    experimental?: Record<string, any>;
    roots?: { listChanged: boolean };
  };
}

const defaultConfig: MCPClientConfig = {
  transport: {
    type: 'sse',
    reconnectOptions: {
      maxAttempts: 5,
      initialDelay: 1000,
      maxDelay: 30000
    }
  },
  auth: { type: 'none' },
  capabilities: {
    roots: { listChanged: true }
  }
};
```

### Error Boundary Integration

```typescript
class MCPErrorBoundary extends Component<
  { children: ReactNode; fallback: ComponentType<{ error: Error }> },
  { error: Error | null }
> {
  constructor(props: any) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error) {
    if (error instanceof MCPError) {
      return { error };
    }
    return null;
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    if (error instanceof MCPError) {
      console.error('MCP Error:', error, errorInfo);
      // Report to error tracking service
      this.reportError(error, errorInfo);
    }
  }

  render() {
    if (this.state.error) {
      const FallbackComponent = this.props.fallback;
      return <FallbackComponent error={this.state.error} />;
    }

    return this.props.children;
  }
}
```

## Code Examples

### Complete MCP Client Implementation

```typescript
// mcp-client.ts
import { EventSource } from 'eventsource'; // For Node.js environments

export interface MCPMessage {
  jsonrpc: '2.0';
  id?: string | number;
  method?: string;
  params?: any;
  result?: any;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
}

export class DAAMCPClient {
  private baseUrl: string;
  private eventSource: EventSource | null = null;
  private pendingRequests: Map<string | number, {
    resolve: (value: any) => void;
    reject: (error: any) => void;
  }> = new Map();
  private requestIdCounter = 1;
  private listeners: Map<string, Function[]> = new Map();

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
  }

  async initialize(): Promise<any> {
    try {
      // Try SSE connection first
      await this.initializeSSE();
    } catch (error) {
      console.warn('SSE connection failed, falling back to HTTP:', error);
      // Fallback to HTTP-only mode
    }

    // Send initialize request
    return this.sendRequest('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {
        roots: { listChanged: true }
      },
      clientInfo: {
        name: 'DAA-Dashboard-Client',
        version: '1.0.0'
      }
    });
  }

  private async initializeSSE(): Promise<void> {
    return new Promise((resolve, reject) => {
      const eventSourceUrl = `${this.baseUrl}/mcp/stream`;
      
      this.eventSource = new EventSource(eventSourceUrl, {
        withCredentials: true
      });

      this.eventSource.onopen = () => {
        console.log('SSE connection established');
        resolve();
      };

      this.eventSource.onmessage = (event) => {
        try {
          const message: MCPMessage = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          console.error('Failed to parse SSE message:', error);
        }
      };

      this.eventSource.onerror = (error) => {
        console.error('SSE connection error:', error);
        reject(error);
      };

      // Timeout after 5 seconds
      setTimeout(() => reject(new Error('SSE connection timeout')), 5000);
    });
  }

  private handleMessage(message: MCPMessage): void {
    if (message.id && this.pendingRequests.has(message.id)) {
      // Handle response
      const { resolve, reject } = this.pendingRequests.get(message.id)!;
      this.pendingRequests.delete(message.id);

      if (message.error) {
        reject(new MCPError(message.error.code, message.error.message, message.error.data));
      } else {
        resolve(message.result);
      }
    } else if (message.method) {
      // Handle notification
      this.emitNotification(message.method, message.params);
    }
  }

  async sendRequest(method: string, params?: any): Promise<any> {
    const id = this.requestIdCounter++;
    const request: MCPMessage = {
      jsonrpc: '2.0',
      id,
      method,
      params
    };

    return new Promise(async (resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });

      try {
        if (this.eventSource && this.eventSource.readyState === EventSource.OPEN) {
          // Send via SSE connection
          await this.sendViaSSE(request);
        } else {
          // Send via HTTP
          const result = await this.sendViaHTTP(request);
          resolve(result);
        }
      } catch (error) {
        this.pendingRequests.delete(id);
        reject(error);
      }

      // Timeout after 30 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error('Request timeout'));
        }
      }, 30000);
    });
  }

  private async sendViaSSE(request: MCPMessage): Promise<void> {
    // For SSE, we typically send the request via HTTP POST
    // and receive the response via the SSE stream
    const response = await fetch(`${this.baseUrl}/mcp/request`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      credentials: 'include',
      body: JSON.stringify(request)
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
  }

  private async sendViaHTTP(request: MCPMessage): Promise<any> {
    const response = await fetch(`${this.baseUrl}/mcp/messages`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      credentials: 'include',
      body: JSON.stringify(request)
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();
    
    if (result.error) {
      throw new MCPError(result.error.code, result.error.message, result.error.data);
    }

    return result.result;
  }

  subscribe(event: string, callback: Function): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    
    const callbacks = this.listeners.get(event)!;
    callbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = callbacks.indexOf(callback);
      if (index > -1) {
        callbacks.splice(index, 1);
      }
    };
  }

  private emitNotification(method: string, params: any): void {
    const callbacks = this.listeners.get(method) || [];
    callbacks.forEach(callback => {
      try {
        callback(params);
      } catch (error) {
        console.error('Error in notification callback:', error);
      }
    });
  }

  disconnect(): void {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    // Reject all pending requests
    this.pendingRequests.forEach(({ reject }) => {
      reject(new Error('Connection closed'));
    });
    this.pendingRequests.clear();
  }

  // High-level API methods
  async getStatus(): Promise<any> {
    return this.sendRequest('tools/call', {
      name: 'daa_status',
      arguments: {}
    });
  }

  async listAgents(): Promise<any[]> {
    const result = await this.sendRequest('tools/call', {
      name: 'daa_agent_list',
      arguments: {}
    });
    return result.content?.[0]?.text ? JSON.parse(result.content[0].text) : [];
  }

  async createAgent(name: string, type: string, capabilities?: string): Promise<any> {
    return this.sendRequest('tools/call', {
      name: 'daa_agent_create',
      arguments: { name, agent_type: type, capabilities }
    });
  }

  async subscribeToResourceChanges(uri: string): Promise<void> {
    return this.sendRequest('resources/subscribe', {
      uri,
      changeTypes: ['created', 'modified', 'deleted']
    });
  }
}

export class MCPError extends Error {
  constructor(
    public code: number,
    message: string,
    public data?: any
  ) {
    super(message);
    this.name = 'MCPError';
  }
}
```

### React Hook Implementation

```typescript
// use-mcp.ts
import { useState, useEffect, useCallback } from 'react';
import { DAAMCPClient, MCPError } from './mcp-client';

// Singleton client instance
let mcpClient: DAAMCPClient | null = null;

function getMCPClient(): DAAMCPClient {
  if (!mcpClient) {
    const baseUrl = process.env.REACT_APP_MCP_BASE_URL || 'http://localhost:3001';
    mcpClient = new DAAMCPClient(baseUrl);
  }
  return mcpClient;
}

export function useMCPStatus() {
  const [status, setStatus] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const refreshStatus = useCallback(async () => {
    try {
      setLoading(true);
      const client = getMCPClient();
      const result = await client.getStatus();
      setStatus(result);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refreshStatus();

    // Subscribe to status updates if available
    const client = getMCPClient();
    const unsubscribe = client.subscribe('status_changed', (newStatus: any) => {
      setStatus(newStatus);
    });

    // Refresh every 30 seconds as fallback
    const interval = setInterval(refreshStatus, 30000);

    return () => {
      unsubscribe();
      clearInterval(interval);
    };
  }, [refreshStatus]);

  return { status, loading, error, refresh: refreshStatus };
}

export function useMCPAgents() {
  const [agents, setAgents] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const refreshAgents = useCallback(async () => {
    try {
      setLoading(true);
      const client = getMCPClient();
      const result = await client.listAgents();
      setAgents(result);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  const createAgent = useCallback(async (name: string, type: string, capabilities?: string) => {
    try {
      const client = getMCPClient();
      await client.createAgent(name, type, capabilities);
      await refreshAgents(); // Refresh the list
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  }, [refreshAgents]);

  useEffect(() => {
    refreshAgents();

    // Subscribe to agent updates
    const client = getMCPClient();
    const unsubscribeCreated = client.subscribe('agent_created', refreshAgents);
    const unsubscribeUpdated = client.subscribe('agent_updated', refreshAgents);
    const unsubscribeDeleted = client.subscribe('agent_deleted', refreshAgents);

    return () => {
      unsubscribeCreated();
      unsubscribeUpdated();
      unsubscribeDeleted();
    };
  }, [refreshAgents]);

  return { 
    agents, 
    loading, 
    error, 
    refresh: refreshAgents,
    createAgent
  };
}

export function useMCPTool<T = any>(toolName: string, args: any = {}) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const execute = useCallback(async (overrideArgs?: any) => {
    try {
      setLoading(true);
      setError(null);
      
      const client = getMCPClient();
      const result = await client.sendRequest('tools/call', {
        name: toolName,
        arguments: overrideArgs || args
      });
      
      setData(result);
      return result;
    } catch (err) {
      const error = err as Error;
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [toolName, args]);

  return { data, loading, error, execute };
}

// Initialize MCP client on app startup
export async function initializeMCP(): Promise<void> {
  try {
    const client = getMCPClient();
    await client.initialize();
    console.log('MCP client initialized successfully');
  } catch (error) {
    console.error('Failed to initialize MCP client:', error);
    throw error;
  }
}
```

### Component Integration Example

```typescript
// AgentManagement.tsx
import React, { useState } from 'react';
import { useMCPAgents, useMCPStatus } from '../hooks/use-mcp';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { toast } from '../components/ui/use-toast';

export function AgentManagement() {
  const { agents, loading: agentsLoading, error: agentsError, createAgent } = useMCPAgents();
  const { status, loading: statusLoading } = useMCPStatus();
  const [creating, setCreating] = useState(false);

  const handleCreateAgent = async () => {
    try {
      setCreating(true);
      await createAgent('new_agent', 'trader', 'portfolio_management,risk_analysis');
      toast({
        title: 'Success',
        description: 'Agent created successfully'
      });
    } catch (error) {
      toast({
        title: 'Error',
        description: (error as Error).message,
        variant: 'destructive'
      });
    } finally {
      setCreating(false);
    }
  };

  if (agentsLoading || statusLoading) {
    return <div>Loading...</div>;
  }

  if (agentsError) {
    return <div>Error: {agentsError.message}</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold">Agent Management</h1>
        <Button onClick={handleCreateAgent} disabled={creating}>
          {creating ? 'Creating...' : 'Create Agent'}
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>System Status</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Orchestrator</p>
              <Badge variant={status?.orchestrator === 'running' ? 'default' : 'destructive'}>
                {status?.orchestrator || 'Unknown'}
              </Badge>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Active Agents</p>
              <p className="text-lg font-semibold">{status?.agents || 0}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Rules</p>
              <p className="text-lg font-semibold">{status?.rules || 0}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Network</p>
              <Badge variant={status?.network === 'connected' ? 'default' : 'destructive'}>
                {status?.network || 'Unknown'}
              </Badge>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-4">
        {agents.map((agent: any) => (
          <Card key={agent.id}>
            <CardContent className="pt-6">
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="font-semibold">{agent.name}</h3>
                  <p className="text-sm text-muted-foreground">ID: {agent.id}</p>
                  <p className="text-sm text-muted-foreground">Type: {agent.type}</p>
                </div>
                <Badge variant={agent.status === 'active' ? 'default' : 'secondary'}>
                  {agent.status}
                </Badge>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
```

## Future Considerations

### Protocol Evolution

1. **Version 2025-03-26 Adoption**
   - OAuth 2.1 authentication framework
   - Streamable HTTP transport
   - JSON-RPC batching support
   - Enhanced security features

2. **WebSocket Transport**
   - Real-time bidirectional communication
   - Agent-to-agent interactions
   - Enhanced streaming capabilities

3. **Protocol Extensions**
   - Custom capability types
   - Advanced subscription mechanisms
   - Enhanced error reporting

### Performance Optimizations

1. **Connection Pooling**
   ```typescript
   class MCPConnectionPool {
     private connections: Map<string, DAAMCPClient> = new Map();
     
     getConnection(server: string): DAAMCPClient {
       if (!this.connections.has(server)) {
         this.connections.set(server, new DAAMCPClient(server));
       }
       return this.connections.get(server)!;
     }
   }
   ```

2. **Request Batching**
   ```typescript
   class BatchingMCPClient {
     private batchQueue: MCPMessage[] = [];
     private batchTimeout: NodeJS.Timeout | null = null;

     async sendRequest(method: string, params?: any): Promise<any> {
       return new Promise((resolve, reject) => {
         const request = {
           jsonrpc: '2.0' as const,
           id: this.generateId(),
           method,
           params
         };

         this.batchQueue.push(request);
         this.pendingRequests.set(request.id, { resolve, reject });

         if (!this.batchTimeout) {
           this.batchTimeout = setTimeout(() => this.flushBatch(), 100);
         }
       });
     }

     private async flushBatch() {
       if (this.batchQueue.length === 0) return;

       const batch = [...this.batchQueue];
       this.batchQueue.length = 0;
       this.batchTimeout = null;

       try {
         await this.sendBatch(batch);
       } catch (error) {
         // Handle batch error
         batch.forEach(request => {
           const { reject } = this.pendingRequests.get(request.id!)!;
           reject(error);
         });
       }
     }
   }
   ```

3. **Caching Strategies**
   ```typescript
   interface CacheConfig {
     ttl: number; // Time to live in milliseconds
     maxSize: number;
   }

   class MCPCache {
     private cache = new Map<string, { data: any; timestamp: number }>();
     
     get(key: string, ttl: number): any | null {
       const entry = this.cache.get(key);
       if (!entry) return null;
       
       if (Date.now() - entry.timestamp > ttl) {
         this.cache.delete(key);
         return null;
       }
       
       return entry.data;
     }

     set(key: string, data: any): void {
       this.cache.set(key, { data, timestamp: Date.now() });
     }
   }
   ```

### Security Enhancements

1. **Content Security Policy**
   ```html
   <meta http-equiv="Content-Security-Policy" 
         content="default-src 'self'; 
                  connect-src 'self' https://daa-mcp-server.example.com;
                  script-src 'self' 'unsafe-inline';
                  style-src 'self' 'unsafe-inline';">
   ```

2. **Request Validation**
   ```typescript
   class SecureMCPClient extends DAAMCPClient {
     private validateRequest(request: MCPMessage): void {
       // Validate request structure
       if (!request.jsonrpc || request.jsonrpc !== '2.0') {
         throw new Error('Invalid JSON-RPC version');
       }

       // Validate method names (whitelist approach)
       const allowedMethods = [
         'initialize', 'tools/list', 'tools/call',
         'resources/list', 'resources/read'
       ];

       if (request.method && !allowedMethods.includes(request.method)) {
         throw new Error(`Method not allowed: ${request.method}`);
       }

       // Sanitize parameters
       if (request.params) {
         this.sanitizeParams(request.params);
       }
     }

     private sanitizeParams(params: any): void {
       // Remove potentially dangerous properties
       const dangerousKeys = ['__proto__', 'constructor', 'prototype'];
       
       function sanitizeObject(obj: any): any {
         if (typeof obj !== 'object' || obj === null) return obj;
         
         const sanitized: any = {};
         for (const [key, value] of Object.entries(obj)) {
           if (!dangerousKeys.includes(key)) {
             sanitized[key] = typeof value === 'object' ? sanitizeObject(value) : value;
           }
         }
         return sanitized;
       }

       return sanitizeObject(params);
     }
   }
   ```

This comprehensive analysis provides the foundation for implementing a robust MCP client in the DAA React dashboard, with support for real-time updates, proper error handling, and future protocol evolution.