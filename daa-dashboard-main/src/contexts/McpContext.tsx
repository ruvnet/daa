import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { daaMcpClient } from '@/lib/mcp-client';
import { useDaaStatus, useDaaAgents, useDaaNetworkStatus } from '@/hooks/use-daa-mcp';

interface McpContextValue {
  client: typeof daaMcpClient;
  isConnected: boolean;
  connectionState: 'connecting' | 'connected' | 'disconnected' | 'error';
  error?: string;
  lastUpdate?: Date;
  retryCount: number;
  retry: () => void;
  reset: () => void;
}

const McpContext = createContext<McpContextValue | null>(null);

interface McpProviderProps {
  children: ReactNode;
  maxRetries?: number;
  retryDelay?: number;
}

export function McpProvider({ 
  children, 
  maxRetries = 3, 
  retryDelay = 5000 
}: McpProviderProps) {
  const [connectionState, setConnectionState] = useState<McpContextValue['connectionState']>('connecting');
  const [error, setError] = useState<string>();
  const [lastUpdate, setLastUpdate] = useState<Date>();
  const [retryCount, setRetryCount] = useState(0);

  // Create a query client with MCP-specific configuration
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: (failureCount, error) => {
          // Don't retry on 4xx errors (client errors)
          if (error instanceof Error && error.message.includes('4')) {
            return false;
          }
          return failureCount < maxRetries;
        },
        retryDelay: (attemptIndex) => 
          Math.min(retryDelay * 2 ** attemptIndex, 30000) + Math.random() * 1000,
        staleTime: 20000, // 20 seconds
        refetchOnWindowFocus: true,
        refetchOnReconnect: true,
      },
      mutations: {
        retry: 1,
        onError: (error) => {
          console.error('MCP Mutation error:', error);
          setError(error instanceof Error ? error.message : 'Mutation failed');
        },
      },
    },
  });

  const checkConnection = async () => {
    try {
      setConnectionState('connecting');
      setError(undefined);
      
      // Test all major MCP endpoints
      const [status, agents, network] = await Promise.all([
        daaMcpClient.getStatus(),
        daaMcpClient.listAgents(),
        daaMcpClient.getNetworkStatus(),
      ]);

      if (status && agents && network) {
        setConnectionState('connected');
        setLastUpdate(new Date());
        setRetryCount(0);
      } else {
        throw new Error('Incomplete MCP response');
      }
    } catch (err) {
      console.error('MCP Connection check failed:', err);
      setConnectionState('error');
      setError(err instanceof Error ? err.message : 'Connection failed');
      
      // Auto-retry with exponential backoff
      if (retryCount < maxRetries) {
        const delay = Math.min(retryDelay * 2 ** retryCount, 30000);
        setTimeout(() => {
          setRetryCount(prev => prev + 1);
          checkConnection();
        }, delay);
      }
    }
  };

  const retry = () => {
    setRetryCount(0);
    checkConnection();
  };

  const reset = () => {
    setConnectionState('connecting');
    setError(undefined);
    setLastUpdate(undefined);
    setRetryCount(0);
    queryClient.clear();
  };

  useEffect(() => {
    checkConnection();
    
    // Set up periodic health checks
    const healthCheckInterval = setInterval(() => {
      if (connectionState === 'connected') {
        checkConnection();
      }
    }, 60000); // Check every minute when connected

    return () => clearInterval(healthCheckInterval);
  }, []);

  // Monitor for network changes
  useEffect(() => {
    const handleOnline = () => {
      if (connectionState === 'error' || connectionState === 'disconnected') {
        retry();
      }
    };

    const handleOffline = () => {
      setConnectionState('disconnected');
      setError('Network connection lost');
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [connectionState]);

  const value: McpContextValue = {
    client: daaMcpClient,
    isConnected: connectionState === 'connected',
    connectionState,
    error,
    lastUpdate,
    retryCount,
    retry,
    reset,
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

// Hook for enhanced MCP queries with connection awareness
export function useMcpQuery<T>(
  queryKey: string[],
  queryFn: () => Promise<T>,
  options?: {
    enabled?: boolean;
    refetchInterval?: number;
    onError?: (error: Error) => void;
    onSuccess?: (data: T) => void;
  }
) {
  const { isConnected, connectionState } = useMcp();
  
  return {
    ...options,
    enabled: (options?.enabled ?? true) && isConnected,
    queryKey: ['mcp', ...queryKey],
    queryFn,
    meta: {
      connectionState,
      timestamp: Date.now(),
    },
  };
}

// Enhanced status hook with connection context
export function useMcpStatusWithContext() {
  const mcpContext = useMcp();
  const statusQuery = useDaaStatus();
  
  return {
    ...statusQuery,
    connectionContext: mcpContext,
    isConnected: mcpContext.isConnected,
    canRetry: mcpContext.connectionState === 'error',
    retry: () => {
      mcpContext.retry();
      statusQuery.refetch();
    },
  };
}

// Enhanced agents hook with connection context
export function useMcpAgentsWithContext() {
  const mcpContext = useMcp();
  const agentsQuery = useDaaAgents();
  
  return {
    ...agentsQuery,
    connectionContext: mcpContext,
    isConnected: mcpContext.isConnected,
    canRetry: mcpContext.connectionState === 'error',
    retry: () => {
      mcpContext.retry();
      agentsQuery.refetch();
    },
  };
}

// Enhanced network hook with connection context
export function useMcpNetworkWithContext() {
  const mcpContext = useMcp();
  const networkQuery = useDaaNetworkStatus();
  
  return {
    ...networkQuery,
    connectionContext: mcpContext,
    isConnected: mcpContext.isConnected,
    canRetry: mcpContext.connectionState === 'error',
    retry: () => {
      mcpContext.retry();
      networkQuery.refetch();
    },
  };
}

export default McpProvider;