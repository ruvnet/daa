# MCP React Integration Guide

A comprehensive guide for integrating Model Context Protocol (MCP) with React/TypeScript applications, based on best practices and real-world patterns.

## Table of Contents

1. [Overview](#overview)
2. [React Integration Patterns](#react-integration-patterns)
3. [State Management with MCP Data](#state-management-with-mcp-data)
4. [Error Handling Strategies](#error-handling-strategies)
5. [Performance Optimization Techniques](#performance-optimization-techniques)
6. [Authentication Patterns](#authentication-patterns)
7. [Real-time Update Handling](#real-time-update-handling)
8. [Best Practices](#best-practices)
9. [Code Examples](#code-examples)

## Overview

The Model Context Protocol (MCP) is an open standard that enables secure, standardized connections between LLM applications and external data sources. This guide focuses on React/TypeScript integration patterns for building responsive, real-time dashboards and applications.

### Key Benefits of MCP in React Apps

- **Standardized Communication**: Universal protocol for AI tool integration
- **Real-time Updates**: Live data synchronization with MCP servers
- **Type Safety**: Full TypeScript support for better development experience
- **Performance**: Efficient caching and state management patterns
- **Security**: Built-in OAuth 2.1 authentication flows

## React Integration Patterns

### 1. Hook-Based Architecture

The modern approach uses React hooks to encapsulate MCP client logic:

```typescript
// hooks/use-mcp-client.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { mcpClient } from '@/lib/mcp-client';

export function useMcpStatus() {
  return useQuery({
    queryKey: ['mcp-status'],
    queryFn: () => mcpClient.getStatus(),
    refetchInterval: 30000, // 30 seconds
    staleTime: 20000, // 20 seconds
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

export function useMcpAgents() {
  return useQuery({
    queryKey: ['mcp-agents'],
    queryFn: () => mcpClient.listAgents(),
    refetchInterval: 10000,
    select: (data) => data.filter(agent => agent.status !== 'terminated'),
  });
}
```

### 2. Component Integration Pattern

Clean separation of concerns with dedicated MCP-aware components:

```typescript
// components/McpStatusCard.tsx
import React from 'react';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { useMcpStatus } from '@/hooks/use-mcp-client';
import { Loader2, AlertCircle, CheckCircle } from 'lucide-react';

export function McpStatusCard() {
  const { data: status, isLoading, error, refetch } = useMcpStatus();

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center p-6">
          <Loader2 className="h-6 w-6 animate-spin" />
          <span className="ml-2">Loading MCP status...</span>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="border-red-200">
        <CardContent className="p-6">
          <div className="flex items-center text-red-600">
            <AlertCircle className="h-5 w-5 mr-2" />
            <span>MCP Connection Error</span>
          </div>
          <button 
            onClick={() => refetch()}
            className="mt-2 px-3 py-1 bg-red-100 text-red-700 rounded"
          >
            Retry
          </button>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center">
          <CheckCircle className="h-5 w-5 text-green-500 mr-2" />
          <h3>MCP Status</h3>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          <p>Orchestrator: {status?.orchestrator}</p>
          <p>Active Agents: {status?.agents}</p>
          <p>Network: {status?.network}</p>
          <p>Uptime: {status?.uptime}</p>
        </div>
      </CardContent>
    </Card>
  );
}
```

### 3. Provider Pattern for MCP Context

Global MCP state management using React Context:

```typescript
// contexts/McpContext.tsx
import React, { createContext, useContext, useEffect, useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { mcpClient } from '@/lib/mcp-client';

interface McpContextValue {
  client: typeof mcpClient;
  isConnected: boolean;
  connectionState: 'connecting' | 'connected' | 'disconnected' | 'error';
  error?: string;
}

const McpContext = createContext<McpContextValue | null>(null);

export function McpProvider({ children }: { children: React.ReactNode }) {
  const [connectionState, setConnectionState] = useState<McpContextValue['connectionState']>('connecting');
  const [error, setError] = useState<string>();
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: 3,
        retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
        staleTime: 20000,
      },
    },
  });

  useEffect(() => {
    const checkConnection = async () => {
      try {
        await mcpClient.getStatus();
        setConnectionState('connected');
        setError(undefined);
      } catch (err) {
        setConnectionState('error');
        setError(err instanceof Error ? err.message : 'Connection failed');
      }
    };

    checkConnection();
    const interval = setInterval(checkConnection, 30000);
    return () => clearInterval(interval);
  }, []);

  const value: McpContextValue = {
    client: mcpClient,
    isConnected: connectionState === 'connected',
    connectionState,
    error,
  };

  return (
    <McpContext.Provider value={value}>
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    </McpContext.Provider>
  );
}

export function useMcp() {
  const context = useContext(McpContext);
  if (!context) {
    throw new Error('useMcp must be used within McpProvider');
  }
  return context;
}
```

## State Management with MCP Data

### React Query Integration

React Query is the recommended approach for MCP state management due to its robust caching, synchronization, and error handling capabilities.

#### Key Configuration

```typescript
// lib/query-client.ts
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Cache data for 20 seconds to reduce duplicate requests
      staleTime: 20000,
      // Refetch on window focus for fresh data
      refetchOnWindowFocus: true,
      // Retry failed requests with exponential backoff
      retry: (failureCount, error) => {
        // Don't retry on 4xx errors (client errors)
        if (error instanceof Error && error.message.includes('4')) {
          return false;
        }
        return failureCount < 3;
      },
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    },
    mutations: {
      // Retry mutations only once
      retry: 1,
      onError: (error) => {
        console.error('Mutation error:', error);
        // You could show a toast notification here
      },
    },
  },
});
```

#### Advanced Query Patterns

```typescript
// hooks/use-mcp-advanced.ts
import { useQuery, useMutation, useQueryClient, useInfiniteQuery } from '@tanstack/react-query';

// Dependent queries - only run if prerequisite data exists
export function useMcpAgentDetails(agentId?: string) {
  return useQuery({
    queryKey: ['mcp-agent', agentId],
    queryFn: () => mcpClient.getAgentDetails(agentId!),
    enabled: !!agentId, // Only run if agentId exists
    staleTime: 60000, // Agent details change less frequently
  });
}

// Infinite query for paginated logs
export function useMcpLogs() {
  return useInfiniteQuery({
    queryKey: ['mcp-logs'],
    queryFn: ({ pageParam = 0 }) => 
      mcpClient.getLogs({ offset: pageParam * 50, limit: 50 }),
    getNextPageParam: (lastPage, allPages) => 
      lastPage.hasMore ? allPages.length : undefined,
    refetchInterval: 5000, // Logs update frequently
  });
}

// Optimistic mutations with rollback
export function useCreateAgent() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (agentData) => mcpClient.createAgent(agentData),
    onMutate: async (newAgent) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: ['mcp-agents'] });
      
      // Snapshot the previous value
      const previousAgents = queryClient.getQueryData(['mcp-agents']);
      
      // Optimistically update to the new value
      queryClient.setQueryData(['mcp-agents'], (old) => [
        ...(old || []),
        { ...newAgent, id: 'temp-id', status: 'creating' }
      ]);
      
      return { previousAgents };
    },
    onError: (err, newAgent, context) => {
      // Rollback on error
      queryClient.setQueryData(['mcp-agents'], context?.previousAgents);
    },
    onSettled: () => {
      // Always refetch after error or success
      queryClient.invalidateQueries({ queryKey: ['mcp-agents'] });
    },
  });
}
```

### State Normalization

For complex nested data, implement normalization patterns:

```typescript
// lib/mcp-state.ts
interface NormalizedState {
  agents: Record<string, Agent>;
  agentIds: string[];
  status: SystemStatus;
  lastUpdated: number;
}

export function normalizeAgents(agents: Agent[]): NormalizedState['agents'] {
  return agents.reduce((acc, agent) => ({
    ...acc,
    [agent.id]: agent
  }), {});
}

// Use with React Query transformer
export function useMcpAgentsNormalized() {
  return useQuery({
    queryKey: ['mcp-agents'],
    queryFn: () => mcpClient.listAgents(),
    select: (data) => ({
      agents: normalizeAgents(data),
      agentIds: data.map(agent => agent.id),
      lastUpdated: Date.now(),
    }),
  });
}
```

## Error Handling Strategies

### Comprehensive Error Boundaries

```typescript
// components/McpErrorBoundary.tsx
import React from 'react';
import { ErrorBoundary } from 'react-error-boundary';

function McpErrorFallback({ error, resetErrorBoundary }: any) {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="max-w-md w-full bg-white shadow-lg rounded-lg p-6">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <AlertCircle className="h-8 w-8 text-red-400" />
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-gray-800">
              MCP Connection Error
            </h3>
            <div className="mt-2 text-sm text-gray-500">
              <p>{error.message}</p>
            </div>
          </div>
        </div>
        <div className="mt-4">
          <button
            onClick={resetErrorBoundary}
            className="w-full bg-red-600 text-white py-2 px-4 rounded hover:bg-red-700"
          >
            Try Again
          </button>
        </div>
      </div>
    </div>
  );
}

export function McpErrorBoundary({ children }: { children: React.ReactNode }) {
  return (
    <ErrorBoundary
      FallbackComponent={McpErrorFallback}
      onError={(error, errorInfo) => {
        console.error('MCP Error:', error, errorInfo);
        // Send to error reporting service
      }}
    >
      {children}
    </ErrorBoundary>
  );
}
```

### Query-Level Error Handling

```typescript
// hooks/use-mcp-with-error-handling.ts
export function useMcpWithErrorHandling<T>(
  queryKey: string[],
  queryFn: () => Promise<T>,
  options?: {
    onError?: (error: Error) => void;
    fallbackData?: T;
  }
) {
  const [lastGoodData, setLastGoodData] = useState<T | undefined>(options?.fallbackData);
  
  const query = useQuery({
    queryKey,
    queryFn,
    onSuccess: (data) => {
      setLastGoodData(data);
    },
    onError: (error) => {
      console.error(`Query ${queryKey.join('.')} failed:`, error);
      options?.onError?.(error as Error);
    },
    // Use last good data while retrying
    placeholderData: lastGoodData,
    retry: (failureCount, error) => {
      // Exponential backoff with jitter
      const delay = Math.min(1000 * 2 ** failureCount, 30000) + Math.random() * 1000;
      setTimeout(() => {}, delay);
      return failureCount < 3;
    },
  });

  return {
    ...query,
    // Provide fallback data if current data is stale
    data: query.data || lastGoodData,
    hasStaleData: !query.data && !!lastGoodData,
  };
}

// Usage in component
export function AgentsList() {
  const { 
    data: agents, 
    isLoading, 
    error, 
    hasStaleData,
    refetch 
  } = useMcpWithErrorHandling(
    ['mcp-agents'],
    () => mcpClient.listAgents(),
    {
      onError: (error) => {
        toast.error(`Failed to load agents: ${error.message}`);
      },
      fallbackData: [],
    }
  );

  return (
    <div>
      {hasStaleData && (
        <div className="bg-yellow-50 border border-yellow-200 rounded p-3 mb-4">
          <p className="text-yellow-800">
            Showing cached data. Connection issues detected.
            <button onClick={() => refetch()} className="ml-2 underline">
              Retry
            </button>
          </p>
        </div>
      )}
      {/* Render agents */}
    </div>
  );
}
```

### Circuit Breaker Pattern

```typescript
// lib/circuit-breaker.ts
class CircuitBreaker {
  private failures = 0;
  private lastFailureTime = 0;
  private state: 'closed' | 'open' | 'half-open' = 'closed';
  
  constructor(
    private threshold = 5,
    private timeout = 60000, // 1 minute
    private monitoringPeriod = 10000 // 10 seconds
  ) {}

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === 'open') {
      if (Date.now() - this.lastFailureTime > this.timeout) {
        this.state = 'half-open';
      } else {
        throw new Error('Circuit breaker is OPEN');
      }
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess() {
    this.failures = 0;
    this.state = 'closed';
  }

  private onFailure() {
    this.failures++;
    this.lastFailureTime = Date.now();
    
    if (this.failures >= this.threshold) {
      this.state = 'open';
    }
  }
}

// Usage in MCP client
const circuitBreaker = new CircuitBreaker();

export const mcpClientWithCircuitBreaker = {
  async getStatus() {
    return circuitBreaker.execute(() => mcpClient.getStatus());
  },
  // ... other methods
};
```

## Performance Optimization Techniques

### 1. Smart Caching Strategies

```typescript
// lib/mcp-cache-config.ts
export const mcpCacheConfig = {
  // Static data - cache for longer
  systemInfo: {
    staleTime: 5 * 60 * 1000, // 5 minutes
    cacheTime: 10 * 60 * 1000, // 10 minutes
  },
  
  // Dynamic data - shorter cache times
  agentStatus: {
    staleTime: 10 * 1000, // 10 seconds
    cacheTime: 60 * 1000, // 1 minute
    refetchInterval: 15 * 1000, // 15 seconds
  },
  
  // Real-time data - minimal caching
  logs: {
    staleTime: 0, // Always stale
    cacheTime: 30 * 1000, // 30 seconds
    refetchInterval: 5 * 1000, // 5 seconds
  },
};

// Apply configurations
export function useMcpAgentStatus() {
  return useQuery({
    queryKey: ['mcp-agent-status'],
    queryFn: () => mcpClient.getAgentStatus(),
    ...mcpCacheConfig.agentStatus,
  });
}
```

### 2. Request Deduplication

```typescript
// lib/mcp-client-optimized.ts
class OptimizedMcpClient {
  private pendingRequests = new Map<string, Promise<any>>();
  
  async request<T>(key: string, operation: () => Promise<T>): Promise<T> {
    // Check if request is already pending
    if (this.pendingRequests.has(key)) {
      return this.pendingRequests.get(key);
    }
    
    // Create new request
    const promise = operation().finally(() => {
      this.pendingRequests.delete(key);
    });
    
    this.pendingRequests.set(key, promise);
    return promise;
  }
  
  async getStatus() {
    return this.request('status', () => this.actualGetStatus());
  }
  
  private async actualGetStatus() {
    // Actual MCP call
    return fetch('/mcp/status').then(r => r.json());
  }
}
```

### 3. Virtual Scrolling for Large Datasets

```typescript
// components/VirtualizedLogViewer.tsx
import { FixedSizeList as List } from 'react-window';
import { useMcpLogs } from '@/hooks/use-mcp-client';

interface LogItemProps {
  index: number;
  style: React.CSSProperties;
  data: LogEntry[];
}

const LogItem: React.FC<LogItemProps> = ({ index, style, data }) => (
  <div style={style} className="flex items-center p-2 border-b">
    <span className="text-sm text-gray-500 mr-4">{data[index].timestamp}</span>
    <span className={`px-2 py-1 rounded text-xs mr-4 ${
      data[index].level === 'ERROR' ? 'bg-red-100 text-red-800' :
      data[index].level === 'WARN' ? 'bg-yellow-100 text-yellow-800' :
      'bg-blue-100 text-blue-800'
    }`}>
      {data[index].level}
    </span>
    <span className="flex-1">{data[index].message}</span>
  </div>
);

export function VirtualizedLogViewer() {
  const { data: logs = [], isLoading } = useMcpLogs();
  
  if (isLoading) return <div>Loading logs...</div>;
  
  return (
    <List
      height={400}
      itemCount={logs.length}
      itemSize={60}
      itemData={logs}
    >
      {LogItem}
    </List>
  );
}
```

### 4. Background Prefetching

```typescript
// hooks/use-mcp-prefetch.ts
export function useMcpPrefetch() {
  const queryClient = useQueryClient();
  
  const prefetchAgentDetails = useCallback((agentId: string) => {
    queryClient.prefetchQuery({
      queryKey: ['mcp-agent', agentId],
      queryFn: () => mcpClient.getAgentDetails(agentId),
      staleTime: 60000,
    });
  }, [queryClient]);
  
  // Prefetch on hover
  const handleAgentHover = useCallback((agentId: string) => {
    prefetchAgentDetails(agentId);
  }, [prefetchAgentDetails]);
  
  return { prefetchAgentDetails, handleAgentHover };
}

// Usage in component
export function AgentCard({ agent }: { agent: Agent }) {
  const { handleAgentHover } = useMcpPrefetch();
  
  return (
    <div 
      onMouseEnter={() => handleAgentHover(agent.id)}
      className="cursor-pointer p-4 border rounded hover:bg-gray-50"
    >
      {agent.name}
    </div>
  );
}
```

## Authentication Patterns

### OAuth 2.1 Implementation

```typescript
// lib/mcp-auth.ts
interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
}

class McpAuthManager {
  private tokens: AuthTokens | null = null;
  private refreshPromise: Promise<AuthTokens> | null = null;
  
  constructor(private clientId: string, private redirectUri: string) {
    this.loadTokensFromStorage();
  }
  
  async authenticate(): Promise<void> {
    // Check if we have valid tokens
    if (this.tokens && this.tokens.expiresAt > Date.now()) {
      return;
    }
    
    // Try to refresh if we have refresh token
    if (this.tokens?.refreshToken) {
      try {
        await this.refreshTokens();
        return;
      } catch (error) {
        console.warn('Token refresh failed, starting new auth flow');
      }
    }
    
    // Start OAuth flow
    await this.startOAuthFlow();
  }
  
  private async startOAuthFlow(): Promise<void> {
    const state = this.generateRandomString();
    const codeVerifier = this.generateRandomString();
    const codeChallenge = await this.generateCodeChallenge(codeVerifier);
    
    // Store PKCE parameters
    sessionStorage.setItem('oauth_state', state);
    sessionStorage.setItem('code_verifier', codeVerifier);
    
    const authUrl = new URL('/oauth/authorize', process.env.REACT_APP_MCP_SERVER_URL);
    authUrl.searchParams.set('client_id', this.clientId);
    authUrl.searchParams.set('redirect_uri', this.redirectUri);
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('state', state);
    authUrl.searchParams.set('code_challenge', codeChallenge);
    authUrl.searchParams.set('code_challenge_method', 'S256');
    authUrl.searchParams.set('scope', 'read write');
    
    // Redirect to auth server
    window.location.href = authUrl.toString();
  }
  
  async handleCallback(code: string, state: string): Promise<void> {
    const storedState = sessionStorage.getItem('oauth_state');
    const codeVerifier = sessionStorage.getItem('code_verifier');
    
    if (state !== storedState) {
      throw new Error('Invalid state parameter');
    }
    
    if (!codeVerifier) {
      throw new Error('Missing code verifier');
    }
    
    const tokens = await this.exchangeCodeForTokens(code, codeVerifier);
    this.setTokens(tokens);
    
    // Clean up
    sessionStorage.removeItem('oauth_state');
    sessionStorage.removeItem('code_verifier');
  }
  
  private async refreshTokens(): Promise<AuthTokens> {
    if (this.refreshPromise) {
      return this.refreshPromise;
    }
    
    if (!this.tokens?.refreshToken) {
      throw new Error('No refresh token available');
    }
    
    this.refreshPromise = this.performTokenRefresh(this.tokens.refreshToken);
    
    try {
      const newTokens = await this.refreshPromise;
      this.setTokens(newTokens);
      return newTokens;
    } finally {
      this.refreshPromise = null;
    }
  }
  
  private async performTokenRefresh(refreshToken: string): Promise<AuthTokens> {
    const response = await fetch('/oauth/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: this.clientId,
      }),
    });
    
    if (!response.ok) {
      throw new Error('Token refresh failed');
    }
    
    const data = await response.json();
    return {
      accessToken: data.access_token,
      refreshToken: data.refresh_token || refreshToken,
      expiresAt: Date.now() + (data.expires_in * 1000),
    };
  }
  
  getAccessToken(): string | null {
    return this.tokens?.accessToken || null;
  }
  
  private setTokens(tokens: AuthTokens) {
    this.tokens = tokens;
    localStorage.setItem('mcp_tokens', JSON.stringify(tokens));
  }
  
  private loadTokensFromStorage() {
    const stored = localStorage.getItem('mcp_tokens');
    if (stored) {
      try {
        this.tokens = JSON.parse(stored);
      } catch (error) {
        console.warn('Failed to parse stored tokens');
      }
    }
  }
  
  logout() {
    this.tokens = null;
    localStorage.removeItem('mcp_tokens');
  }
  
  private generateRandomString(): string {
    return btoa(String.fromCharCode(...crypto.getRandomValues(new Uint8Array(32))))
      .replace(/[+/]/g, '')
      .substring(0, 43);
  }
  
  private async generateCodeChallenge(verifier: string): Promise<string> {
    const encoder = new TextEncoder();
    const data = encoder.encode(verifier);
    const digest = await crypto.subtle.digest('SHA-256', data);
    return btoa(String.fromCharCode(...new Uint8Array(digest)))
      .replace(/[+/]/g, '')
      .replace(/=/g, '');
  }
}

// Usage in app
const authManager = new McpAuthManager(
  process.env.REACT_APP_MCP_CLIENT_ID!,
  `${window.location.origin}/auth/callback`
);

export { authManager };
```

### Authenticated MCP Client

```typescript
// lib/authenticated-mcp-client.ts
class AuthenticatedMcpClient {
  constructor(private authManager: McpAuthManager) {}
  
  private async makeRequest<T>(url: string, options: RequestInit = {}): Promise<T> {
    // Ensure we have valid authentication
    await this.authManager.authenticate();
    
    const token = this.authManager.getAccessToken();
    if (!token) {
      throw new Error('No access token available');
    }
    
    const response = await fetch(url, {
      ...options,
      headers: {
        ...options.headers,
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });
    
    if (response.status === 401) {
      // Token might be expired, try to refresh
      try {
        await this.authManager.authenticate();
        const newToken = this.authManager.getAccessToken();
        
        // Retry with new token
        const retryResponse = await fetch(url, {
          ...options,
          headers: {
            ...options.headers,
            'Authorization': `Bearer ${newToken}`,
            'Content-Type': 'application/json',
          },
        });
        
        if (!retryResponse.ok) {
          throw new Error(`HTTP ${retryResponse.status}`);
        }
        
        return retryResponse.json();
      } catch (error) {
        // Refresh failed, need to re-authenticate
        this.authManager.logout();
        throw new Error('Authentication required');
      }
    }
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    return response.json();
  }
  
  async getStatus() {
    return this.makeRequest('/mcp/status');
  }
  
  async listAgents() {
    return this.makeRequest('/mcp/agents');
  }
  
  // ... other MCP methods
}
```

### Auth Context Provider

```typescript
// contexts/AuthContext.tsx
interface AuthContextValue {
  isAuthenticated: boolean;
  isLoading: boolean;
  login: () => void;
  logout: () => void;
  user: User | null;
}

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [user, setUser] = useState<User | null>(null);
  
  useEffect(() => {
    const checkAuth = async () => {
      try {
        await authManager.authenticate();
        const userInfo = await mcpClient.getUserInfo();
        setUser(userInfo);
        setIsAuthenticated(true);
      } catch (error) {
        setIsAuthenticated(false);
      } finally {
        setIsLoading(false);
      }
    };
    
    checkAuth();
  }, []);
  
  const login = useCallback(() => {
    authManager.authenticate();
  }, []);
  
  const logout = useCallback(() => {
    authManager.logout();
    setIsAuthenticated(false);
    setUser(null);
  }, []);
  
  return (
    <AuthContext.Provider value={{ isAuthenticated, isLoading, login, logout, user }}>
      {children}
    </AuthContext.Provider>
  );
}
```

## Real-time Update Handling

### Server-Sent Events Integration

```typescript
// hooks/use-mcp-sse.ts
export function useMcpServerSentEvents() {
  const [events, setEvents] = useState<McpEvent[]>([]);
  const [connectionState, setConnectionState] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');
  const queryClient = useQueryClient();
  
  useEffect(() => {
    let eventSource: EventSource | null = null;
    let reconnectTimeout: NodeJS.Timeout;
    
    const connect = () => {
      setConnectionState('connecting');
      
      eventSource = new EventSource('/mcp/events', {
        withCredentials: true,
      });
      
      eventSource.onopen = () => {
        setConnectionState('connected');
        console.log('SSE connection established');
      };
      
      eventSource.onmessage = (event) => {
        try {
          const mcpEvent: McpEvent = JSON.parse(event.data);
          setEvents(prev => [...prev.slice(-99), mcpEvent]); // Keep last 100 events
          
          // Invalidate relevant queries based on event type
          handleEventQueryInvalidation(mcpEvent, queryClient);
        } catch (error) {
          console.error('Failed to parse SSE event:', error);
        }
      };
      
      eventSource.onerror = (error) => {
        console.error('SSE error:', error);
        setConnectionState('disconnected');
        eventSource?.close();
        
        // Reconnect after delay
        reconnectTimeout = setTimeout(connect, 5000);
      };
      
      // Handle specific MCP event types
      eventSource.addEventListener('agent_status_changed', (event) => {
        const data = JSON.parse(event.data);
        queryClient.invalidateQueries({ queryKey: ['mcp-agents'] });
        queryClient.invalidateQueries({ queryKey: ['mcp-agent', data.agentId] });
      });
      
      eventSource.addEventListener('system_status_changed', () => {
        queryClient.invalidateQueries({ queryKey: ['mcp-status'] });
      });
    };
    
    connect();
    
    return () => {
      eventSource?.close();
      clearTimeout(reconnectTimeout);
    };
  }, [queryClient]);
  
  return { events, connectionState };
}

function handleEventQueryInvalidation(event: McpEvent, queryClient: QueryClient) {
  switch (event.type) {
    case 'agent_created':
    case 'agent_deleted':
      queryClient.invalidateQueries({ queryKey: ['mcp-agents'] });
      break;
      
    case 'agent_status_changed':
      queryClient.invalidateQueries({ queryKey: ['mcp-agents'] });
      queryClient.invalidateQueries({ queryKey: ['mcp-agent', event.data.agentId] });
      break;
      
    case 'config_changed':
      queryClient.invalidateQueries({ queryKey: ['mcp-config'] });
      break;
      
    case 'network_status_changed':
      queryClient.invalidateQueries({ queryKey: ['mcp-network-status'] });
      queryClient.invalidateQueries({ queryKey: ['mcp-network-peers'] });
      break;
      
    default:
      // For unknown events, do a broad invalidation
      queryClient.invalidateQueries({ queryKey: ['mcp'] });
  }
}
```

### WebSocket Integration (Alternative)

```typescript
// hooks/use-mcp-websocket.ts
export function useMcpWebSocket() {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [connectionState, setConnectionState] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');
  const [lastMessage, setLastMessage] = useState<McpMessage | null>(null);
  const queryClient = useQueryClient();
  
  const connect = useCallback(() => {
    if (socket?.readyState === WebSocket.OPEN) return;
    
    setConnectionState('connecting');
    
    const ws = new WebSocket(process.env.REACT_APP_MCP_WS_URL!);
    
    ws.onopen = () => {
      setConnectionState('connected');
      setSocket(ws);
      
      // Send authentication
      ws.send(JSON.stringify({
        type: 'auth',
        token: authManager.getAccessToken(),
      }));
    };
    
    ws.onmessage = (event) => {
      try {
        const message: McpMessage = JSON.parse(event.data);
        setLastMessage(message);
        
        // Handle different message types
        switch (message.type) {
          case 'agent_update':
            queryClient.setQueryData(['mcp-agent', message.data.id], message.data);
            break;
            
          case 'status_update':
            queryClient.setQueryData(['mcp-status'], message.data);
            break;
            
          case 'bulk_update':
            // Handle multiple updates efficiently
            message.data.forEach((update: any) => {
              queryClient.setQueryData(update.queryKey, update.data);
            });
            break;
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };
    
    ws.onclose = () => {
      setConnectionState('disconnected');
      setSocket(null);
      
      // Reconnect after delay
      setTimeout(connect, 5000);
    };
    
    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }, [socket, queryClient]);
  
  const sendMessage = useCallback((message: any) => {
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(message));
    }
  }, [socket]);
  
  useEffect(() => {
    connect();
    
    return () => {
      socket?.close();
    };
  }, [connect]);
  
  return { connectionState, lastMessage, sendMessage, reconnect: connect };
}
```

### Real-time Data Synchronization

```typescript
// hooks/use-mcp-realtime.ts
export function useMcpRealTimeSync() {
  const { events } = useMcpServerSentEvents();
  const queryClient = useQueryClient();
  
  // Sync agent data in real-time
  useEffect(() => {
    const agentEvents = events.filter(e => e.type.startsWith('agent_'));
    
    agentEvents.forEach(event => {
      if (event.type === 'agent_status_changed') {
        // Update specific agent data optimistically
        queryClient.setQueryData(
          ['mcp-agent', event.data.agentId],
          (old: any) => old ? { ...old, status: event.data.status } : undefined
        );
        
        // Update agents list
        queryClient.setQueryData(['mcp-agents'], (old: any[]) => 
          old?.map(agent => 
            agent.id === event.data.agentId 
              ? { ...agent, status: event.data.status }
              : agent
          )
        );
      }
    });
  }, [events, queryClient]);
  
  // Batch updates to avoid excessive re-renders
  const [pendingUpdates, setPendingUpdates] = useState<McpEvent[]>([]);
  
  useEffect(() => {
    const timer = setTimeout(() => {
      if (pendingUpdates.length > 0) {
        // Process batch of updates
        processBatchUpdates(pendingUpdates, queryClient);
        setPendingUpdates([]);
      }
    }, 100); // 100ms batching window
    
    return () => clearTimeout(timer);
  }, [pendingUpdates, queryClient]);
}

function processBatchUpdates(updates: McpEvent[], queryClient: QueryClient) {
  const agentUpdates = new Map<string, Partial<Agent>>();
  let statusUpdate: Partial<SystemStatus> | null = null;
  
  // Collect updates by type
  updates.forEach(event => {
    switch (event.type) {
      case 'agent_status_changed':
        agentUpdates.set(event.data.agentId, { 
          status: event.data.status,
          lastActivity: event.data.timestamp,
        });
        break;
        
      case 'system_status_changed':
        statusUpdate = { ...statusUpdate, ...event.data };
        break;
    }
  });
  
  // Apply batched updates
  if (agentUpdates.size > 0) {
    queryClient.setQueryData(['mcp-agents'], (old: Agent[]) =>
      old?.map(agent => {
        const update = agentUpdates.get(agent.id);
        return update ? { ...agent, ...update } : agent;
      })
    );
  }
  
  if (statusUpdate) {
    queryClient.setQueryData(['mcp-status'], (old: SystemStatus) => ({
      ...old,
      ...statusUpdate,
    }));
  }
}
```

## Best Practices

### 1. Type Safety

```typescript
// types/mcp.ts
export interface McpResponse<T = any> {
  jsonrpc: '2.0';
  id: string | number;
  result?: T;
  error?: McpError;
}

export interface McpError {
  code: number;
  message: string;
  data?: any;
}

export interface Agent {
  id: string;
  name: string;
  type: AgentType;
  status: AgentStatus;
  capabilities: string[];
  createdAt: string;
  lastActivity?: string;
}

export type AgentType = 'treasury' | 'defi' | 'security' | 'analytics';
export type AgentStatus = 'active' | 'idle' | 'error' | 'terminated';

// Use branded types for IDs to prevent mixing
export type AgentId = string & { readonly brand: unique symbol };
export type UserId = string & { readonly brand: unique symbol };

// Helper functions for type safety
export function isValidAgentId(id: string): id is AgentId {
  return /^agent-[a-zA-Z0-9]+$/.test(id);
}
```

### 2. Environment Configuration

```typescript
// config/mcp.ts
interface McpConfig {
  serverUrl: string;
  wsUrl: string;
  clientId: string;
  maxRetries: number;
  retryDelay: number;
  cacheTimeout: number;
  enableRealtime: boolean;
}

const getMcpConfig = (): McpConfig => {
  const config: McpConfig = {
    serverUrl: process.env.REACT_APP_MCP_SERVER_URL || 'http://localhost:3001',
    wsUrl: process.env.REACT_APP_MCP_WS_URL || 'ws://localhost:3001/ws',
    clientId: process.env.REACT_APP_MCP_CLIENT_ID || 'default-client',
    maxRetries: parseInt(process.env.REACT_APP_MCP_MAX_RETRIES || '3'),
    retryDelay: parseInt(process.env.REACT_APP_MCP_RETRY_DELAY || '1000'),
    cacheTimeout: parseInt(process.env.REACT_APP_MCP_CACHE_TIMEOUT || '60000'),
    enableRealtime: process.env.REACT_APP_MCP_ENABLE_REALTIME === 'true',
  };
  
  // Validate configuration
  if (!config.serverUrl || !config.clientId) {
    throw new Error('Missing required MCP configuration');
  }
  
  return config;
};

export const mcpConfig = getMcpConfig();
```

### 3. Testing Strategies

```typescript
// __tests__/mcp-client.test.ts
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useMcpStatus } from '@/hooks/use-mcp-client';
import { mcpClient } from '@/lib/mcp-client';

// Mock the MCP client
jest.mock('@/lib/mcp-client');
const mockMcpClient = mcpClient as jest.Mocked<typeof mcpClient>;

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('useMcpStatus', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });
  
  it('should return status data on successful fetch', async () => {
    const mockStatus = {
      orchestrator: 'running',
      agents: 3,
      network: 'connected',
      uptime: '2h 34m',
    };
    
    mockMcpClient.getStatus.mockResolvedValueOnce(mockStatus);
    
    const { result } = renderHook(() => useMcpStatus(), {
      wrapper: createWrapper(),
    });
    
    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    
    expect(result.current.data).toEqual(mockStatus);
  });
  
  it('should handle errors gracefully', async () => {
    mockMcpClient.getStatus.mockRejectedValueOnce(new Error('Network error'));
    
    const { result } = renderHook(() => useMcpStatus(), {
      wrapper: createWrapper(),
    });
    
    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
    
    expect(result.current.error).toEqual(new Error('Network error'));
  });
});
```

### 4. Monitoring and Debugging

```typescript
// lib/mcp-monitoring.ts
interface McpMetrics {
  requestCount: number;
  errorCount: number;
  averageResponseTime: number;
  cacheHitRate: number;
}

class McpMonitor {
  private metrics: McpMetrics = {
    requestCount: 0,
    errorCount: 0,
    averageResponseTime: 0,
    cacheHitRate: 0,
  };
  
  private responseTimes: number[] = [];
  private cacheHits = 0;
  private cacheMisses = 0;
  
  recordRequest(responseTime: number, fromCache: boolean) {
    this.metrics.requestCount++;
    this.responseTimes.push(responseTime);
    
    if (fromCache) {
      this.cacheHits++;
    } else {
      this.cacheMisses++;
    }
    
    // Calculate rolling average (last 100 requests)
    if (this.responseTimes.length > 100) {
      this.responseTimes.shift();
    }
    
    this.metrics.averageResponseTime = 
      this.responseTimes.reduce((sum, time) => sum + time, 0) / this.responseTimes.length;
    
    this.metrics.cacheHitRate = 
      this.cacheHits / (this.cacheHits + this.cacheMisses);
  }
  
  recordError() {
    this.metrics.errorCount++;
  }
  
  getMetrics(): McpMetrics {
    return { ...this.metrics };
  }
  
  reset() {
    this.metrics = {
      requestCount: 0,
      errorCount: 0,
      averageResponseTime: 0,
      cacheHitRate: 0,
    };
    this.responseTimes = [];
    this.cacheHits = 0;
    this.cacheMisses = 0;
  }
}

export const mcpMonitor = new McpMonitor();

// Development tools component
export function McpDevTools() {
  const [metrics, setMetrics] = useState<McpMetrics>();
  const [isOpen, setIsOpen] = useState(false);
  
  useEffect(() => {
    if (process.env.NODE_ENV === 'development') {
      const interval = setInterval(() => {
        setMetrics(mcpMonitor.getMetrics());
      }, 1000);
      
      return () => clearInterval(interval);
    }
  }, []);
  
  if (process.env.NODE_ENV !== 'development') {
    return null;
  }
  
  return (
    <div className="fixed bottom-4 right-4 z-50">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="bg-blue-600 text-white px-3 py-2 rounded shadow-lg"
      >
        MCP Debug
      </button>
      
      {isOpen && metrics && (
        <div className="absolute bottom-12 right-0 bg-white border shadow-lg rounded p-4 w-64">
          <h3 className="font-bold mb-2">MCP Metrics</h3>
          <div className="space-y-1 text-sm">
            <div>Requests: {metrics.requestCount}</div>
            <div>Errors: {metrics.errorCount}</div>
            <div>Avg Response: {metrics.averageResponseTime.toFixed(2)}ms</div>
            <div>Cache Hit Rate: {(metrics.cacheHitRate * 100).toFixed(1)}%</div>
          </div>
          <button
            onClick={() => mcpMonitor.reset()}
            className="mt-2 px-2 py-1 bg-gray-200 rounded text-xs"
          >
            Reset
          </button>
        </div>
      )}
    </div>
  );
}
```

## Code Examples

### Complete Dashboard Integration

```typescript
// components/McpDashboard.tsx
import React from 'react';
import { McpErrorBoundary } from './McpErrorBoundary';
import { McpProvider } from '@/contexts/McpContext';
import { McpStatusCard } from './McpStatusCard';
import { AgentsList } from './AgentsList';
import { NetworkStatus } from './NetworkStatus';
import { LogViewer } from './LogViewer';
import { McpDevTools } from '@/lib/mcp-monitoring';

export function McpDashboard() {
  return (
    <McpErrorBoundary>
      <McpProvider>
        <div className="p-6 space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <McpStatusCard />
            <NetworkStatus />
            <AgentsList />
          </div>
          
          <LogViewer />
          
          {/* Development tools */}
          <McpDevTools />
        </div>
      </McpProvider>
    </McpErrorBoundary>
  );
}
```

### Production-Ready Hook Implementation

```typescript
// hooks/use-mcp-production.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { mcpClient } from '@/lib/mcp-client';
import { mcpMonitor } from '@/lib/mcp-monitoring';
import { useCallback, useEffect } from 'react';

export function useMcpWithMonitoring<T>(
  queryKey: string[],
  queryFn: () => Promise<T>,
  options: {
    refetchInterval?: number;
    enabled?: boolean;
    onError?: (error: Error) => void;
    onSuccess?: (data: T) => void;
  } = {}
) {
  const startTime = Date.now();
  
  const query = useQuery({
    queryKey,
    queryFn: async () => {
      try {
        const result = await queryFn();
        const responseTime = Date.now() - startTime;
        mcpMonitor.recordRequest(responseTime, false);
        options.onSuccess?.(result);
        return result;
      } catch (error) {
        mcpMonitor.recordError();
        options.onError?.(error as Error);
        throw error;
      }
    },
    refetchInterval: options.refetchInterval,
    enabled: options.enabled,
    retry: (failureCount, error) => {
      // Don't retry on authentication errors
      if (error instanceof Error && error.message.includes('401')) {
        return false;
      }
      return failureCount < 3;
    },
    retryDelay: (attemptIndex) => 
      Math.min(1000 * 2 ** attemptIndex, 30000) + Math.random() * 1000,
  });
  
  // Record cache hits
  useEffect(() => {
    if (query.data && query.isFetched && !query.isFetching) {
      mcpMonitor.recordRequest(0, true);
    }
  }, [query.data, query.isFetched, query.isFetching]);
  
  return query;
}

// Usage
export function useMcpStatus() {
  return useMcpWithMonitoring(
    ['mcp-status'],
    () => mcpClient.getStatus(),
    {
      refetchInterval: 30000,
      onError: (error) => {
        console.error('Status fetch failed:', error);
      },
    }
  );
}
```

This comprehensive guide provides production-ready patterns for integrating MCP with React applications, covering everything from basic hooks to advanced real-time synchronization and monitoring. The patterns demonstrated here are based on current best practices and real-world implementations in the DAA dashboard system.