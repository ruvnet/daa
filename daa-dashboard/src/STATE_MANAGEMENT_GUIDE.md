# DAA Dashboard State Management Guide

## Overview

The DAA Dashboard uses a comprehensive state management architecture that combines:
- **Zustand** for local state management
- **React Query** for server state and caching
- **WebSocket** for real-time updates
- **Context API** for dependency injection

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        App Providers                          │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                 QueryClientProvider                   │    │
│  │  ┌────────────────────────────────────────────┐     │    │
│  │  │              DaaApiProvider                 │     │    │
│  │  │  ┌─────────────────────────────────────┐   │     │    │
│  │  │  │          AuthProvider               │   │     │    │
│  │  │  │  ┌──────────────────────────┐      │   │     │    │
│  │  │  │  │   WebSocketProvider      │      │   │     │    │
│  │  │  │  │                          │      │   │     │    │
│  │  │  │  └──────────────────────────┘      │   │     │    │
│  │  │  └─────────────────────────────────────┘   │     │    │
│  │  └────────────────────────────────────────────┘     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## State Management Layers

### 1. Zustand Stores (Local State)

Located in `src/stores/`:

- **auth.store.ts** - Authentication state and user management
- **agents.store.ts** - Agent entities and filtering
- **dashboard.store.ts** - UI state, layouts, and preferences
- **websocket.store.ts** - WebSocket connection state

### 2. React Query (Server State)

Located in `src/api/hooks/`:

- **useAgents.ts** - Agent CRUD operations
- **useTasks.ts** - Task management
- **useSystem.ts** - System metrics and health
- **useSwarm.ts** - Swarm coordination

### 3. Real-time Updates

WebSocket integration automatically updates:
- Zustand stores
- React Query cache
- Component state via hooks

## Usage Examples

### Basic Agent Management

```typescript
import { useAgents, useSpawnAgent } from '@/api/hooks';
import { useAgentUpdates } from '@/contexts';

function AgentManager() {
  // Fetch agents with React Query
  const { data: agents, isLoading } = useAgents();
  
  // Mutation for creating agents
  const spawnAgent = useSpawnAgent();
  
  // Real-time updates
  const agentUpdate = useAgentUpdates();
  
  const handleCreateAgent = async () => {
    await spawnAgent.mutateAsync({
      name: 'New Agent',
      agent_type: 'treasury',
      capabilities: ['trading', 'analytics']
    });
  };
  
  return (
    <div>
      {/* UI implementation */}
    </div>
  );
}
```

### Dashboard State Management

```typescript
import { useDashboardStore } from '@/stores';

function Dashboard() {
  // Direct store access
  const { 
    metrics, 
    preferences, 
    addAlert,
    setPreferences 
  } = useDashboardStore();
  
  // Or use specific hooks
  const alerts = useAlerts();
  const currentLayout = useCurrentLayout();
  
  return (
    <div>
      {/* Dashboard UI */}
    </div>
  );
}
```

### Real-time Task Updates

```typescript
import { useTaskStatus } from '@/api/hooks';
import { useTaskUpdates } from '@/contexts';

function TaskMonitor({ taskId }: { taskId: string }) {
  // Initial data and polling
  const { data: task } = useTaskStatus(taskId);
  
  // Real-time updates
  const update = useTaskUpdates(taskId);
  
  // Merged state
  const currentStatus = update?.status || task?.result?.status;
  
  return (
    <div>
      Task Status: {currentStatus}
    </div>
  );
}
```

### Authentication Flow

```typescript
import { useAuth } from '@/contexts';

function LoginPage() {
  const { login, isAuthenticated } = useAuth();
  
  const handleLogin = async (credentials) => {
    try {
      await login(credentials);
      // Redirect on success
    } catch (error) {
      // Handle error
    }
  };
  
  if (isAuthenticated) {
    return <Navigate to="/dashboard" />;
  }
  
  return <LoginForm onSubmit={handleLogin} />;
}
```

## Best Practices

### 1. Use the Right Tool

- **Zustand**: UI state, user preferences, local data
- **React Query**: Server data, API calls, caching
- **WebSocket**: Real-time updates, live data
- **Context**: Dependency injection, cross-cutting concerns

### 2. Optimistic Updates

```typescript
const stopAgent = useStopAgent();

// Optimistic update happens automatically
await stopAgent.mutateAsync(agentId);
```

### 3. Error Handling

```typescript
function MyComponent() {
  const { data, error, isError } = useAgents();
  
  if (isError) {
    return <ErrorDisplay error={error} />;
  }
  
  // Normal rendering
}
```

### 4. Real-time Synchronization

```typescript
// Automatic synchronization between stores and cache
useEffect(() => {
  // WebSocket updates automatically sync to:
  // 1. Zustand stores
  // 2. React Query cache
  // 3. Component state
}, []);
```

### 5. Performance Optimization

```typescript
// Use selectors to prevent unnecessary re-renders
const runningAgents = useAgentsStore(
  state => state.agents.filter(a => a.status === 'running')
);

// Use React Query's select option
const { data: activeCount } = useAgents({
  select: (agents) => agents.filter(a => a.status === 'running').length
});
```

## Environment Configuration

Create a `.env` file based on `.env.example`:

```env
VITE_MCP_SERVER_URL=http://localhost:3001/mcp
VITE_WS_SERVER_URL=ws://localhost:3001/mcp/ws
VITE_USE_MOCK_DATA=false
```

## Testing

### Unit Testing Stores

```typescript
import { renderHook, act } from '@testing-library/react-hooks';
import { useAgentsStore } from '@/stores';

test('should update agent', () => {
  const { result } = renderHook(() => useAgentsStore());
  
  act(() => {
    result.current.addAgent(mockAgent);
    result.current.updateAgent(mockAgent.id, { status: 'paused' });
  });
  
  expect(result.current.getAgentById(mockAgent.id)?.status).toBe('paused');
});
```

### Testing with Providers

```typescript
import { render } from '@testing-library/react';
import { AppProviders } from '@/contexts';

function renderWithProviders(ui: React.ReactElement) {
  return render(
    <AppProviders config={{ useMockData: true }}>
      {ui}
    </AppProviders>
  );
}
```

## Troubleshooting

### WebSocket Connection Issues

```typescript
// Check connection status
const { status, error } = useConnectionStatus();

// Manual reconnection
const wsStore = useWebSocketStore.getState();
wsStore.setStatus(ConnectionStatus.Reconnecting);
```

### Query Cache Issues

```typescript
// Manual cache invalidation
queryClient.invalidateQueries({ queryKey: queryKeys.agents.all });

// Clear all cache
queryClient.clear();
```

### State Synchronization Issues

```typescript
// Force refresh all data
const refreshAll = () => {
  queryClient.refetchQueries();
  wsStore.clearMessageQueue();
  agentsStore.setAgents([]);
};
```