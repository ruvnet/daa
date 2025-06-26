# How to Update Your App.tsx

To integrate the new state management system into your existing `App.tsx`, follow these steps:

## 1. Update App.tsx

Replace your current App.tsx with:

```typescript
import React from 'react';
import { AppProviders } from './contexts/AppProviders';
import { ApiErrorBoundary } from './api/errors/ErrorHandler';
import DashboardLayout from './components/DashboardLayout';
import './App.css';

function App() {
  return (
    <ApiErrorBoundary>
      <AppProviders>
        <DashboardLayout />
      </AppProviders>
    </ApiErrorBoundary>
  );
}

export default App;
```

## 2. Update DashboardLayout.tsx

Add state management hooks to your DashboardLayout:

```typescript
import React from 'react';
import { useAuth } from '../contexts/AuthProvider';
import { useConnectionStatus } from '../stores/websocket.store';
import { useSystemMetrics } from '../api/hooks';

export default function DashboardLayout() {
  const { isAuthenticated, user } = useAuth();
  const { isConnected, status } = useConnectionStatus();
  const { data: metrics } = useSystemMetrics();
  
  // Your existing layout code
  // Now with access to real-time data
}
```

## 3. Update Individual Components

### AgentManagement.tsx
```typescript
import { useAgents, useSpawnAgent, useStopAgent } from '../api/hooks';
import { useAgentUpdates } from '../contexts/WebSocketProvider';

export default function AgentManagement() {
  const { data: agents, isLoading } = useAgents();
  const spawnAgent = useSpawnAgent();
  const stopAgent = useStopAgent();
  const agentUpdates = useAgentUpdates();
  
  // Component implementation
}
```

### NetworkOperations.tsx
```typescript
import { useSwarmStatus, useSystemMetrics } from '../api/hooks';
import { useSystemMetricsUpdates } from '../contexts/WebSocketProvider';

export default function NetworkOperations() {
  const { data: swarmStatus } = useSwarmStatus();
  const metricsUpdate = useSystemMetricsUpdates();
  
  // Component implementation
}
```

## 4. Environment Setup

Create a `.env` file in your project root:

```env
VITE_MCP_SERVER_URL=http://localhost:3001/mcp
VITE_WS_SERVER_URL=ws://localhost:3001/mcp/ws
VITE_USE_MOCK_DATA=false
```

## 5. Remove Old State Management

If you were using any other state management, you can now remove:
- Redux/MobX setup files
- Old context providers
- Manual WebSocket implementations
- Custom API service layers

## 6. TypeScript Configuration

Ensure your tsconfig.json includes:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  }
}
```

## Migration Checklist

- [ ] Install dependencies (zustand is already installed)
- [ ] Create .env file from .env.example
- [ ] Update App.tsx with AppProviders
- [ ] Update components to use new hooks
- [ ] Remove old state management code
- [ ] Test authentication flow
- [ ] Verify WebSocket connections
- [ ] Check real-time updates

## Common Issues

### Authentication Loop
If you get stuck in an authentication loop, set `requireAuth: false` temporarily:

```typescript
<AppProviders config={{ requireAuth: false }}>
```

### WebSocket Connection Failed
Check that your backend is running and the WebSocket URL is correct in .env

### TypeScript Errors
Run `npm run typecheck` to identify and fix any type issues