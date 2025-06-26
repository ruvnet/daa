import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import { AppProviders } from './contexts/AppProviders';
import { ApiErrorBoundary } from './api/errors/ErrorHandler';
import { DashboardLayout } from './components/DashboardLayout';
import { useAgents, useSystemMetrics } from './api/hooks';
import { useAuth } from './contexts/AuthProvider';
import { useWebSocket, useAgentUpdates } from './contexts/WebSocketProvider';
import { useDashboardStore, usePreferences } from './stores/dashboard.store';
import { useConnectionStatus } from './stores/websocket.store';

// Example component showing how to use the state management
function ExampleDashboard() {
  // Auth state
  const { isAuthenticated, user } = useAuth();
  
  // API hooks with React Query
  const { data: agents, isLoading: agentsLoading } = useAgents();
  const { data: metrics } = useSystemMetrics();
  
  // WebSocket state
  const { isConnected, status } = useConnectionStatus();
  const agentUpdates = useAgentUpdates(); // Real-time agent updates
  
  // Zustand stores
  const preferences = usePreferences();
  const { addAlert } = useDashboardStore();
  
  // Example of handling real-time updates
  React.useEffect(() => {
    if (agentUpdates) {
      console.log('Real-time agent update:', agentUpdates);
      addAlert({
        type: 'info',
        title: 'Agent Updated',
        message: `Agent ${agentUpdates.agent_id} status changed`
      });
    }
  }, [agentUpdates, addAlert]);
  
  return (
    <div className="min-h-screen bg-gray-100">
      {/* Connection Status Bar */}
      <div className={`h-1 ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
      
      {/* Main Content */}
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">DAA Dashboard Example</h1>
        
        {/* User Info */}
        <div className="mb-4 p-4 bg-white rounded shadow">
          <h2 className="text-lg font-semibold mb-2">User Information</h2>
          <p>Authenticated: {isAuthenticated ? 'Yes' : 'No'}</p>
          <p>Username: {user?.username || 'Not logged in'}</p>
          <p>WebSocket: {status}</p>
        </div>
        
        {/* System Metrics */}
        {metrics && (
          <div className="mb-4 p-4 bg-white rounded shadow">
            <h2 className="text-lg font-semibold mb-2">System Metrics</h2>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-sm text-gray-600">Total Agents</p>
                <p className="text-2xl font-bold">{metrics.agents.total}</p>
              </div>
              <div>
                <p className="text-sm text-gray-600">Running Tasks</p>
                <p className="text-2xl font-bold">{metrics.tasks.running}</p>
              </div>
            </div>
          </div>
        )}
        
        {/* Agents List */}
        <div className="p-4 bg-white rounded shadow">
          <h2 className="text-lg font-semibold mb-2">Active Agents</h2>
          {agentsLoading ? (
            <p>Loading agents...</p>
          ) : (
            <ul className="space-y-2">
              {agents?.map(agent => (
                <li key={agent.id} className="p-2 border rounded">
                  <span className="font-medium">{agent.name}</span>
                  <span className={`ml-2 px-2 py-1 text-xs rounded ${
                    agent.status === 'running' ? 'bg-green-100 text-green-800' : 'bg-gray-100'
                  }`}>
                    {agent.status}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}

// Main App component with all providers
export default function App() {
  return (
    <ApiErrorBoundary>
      <AppProviders
        config={{
          // These can be overridden by environment variables
          mcpServerUrl: 'http://localhost:3001/mcp',
          wsServerUrl: 'ws://localhost:3001/mcp/ws',
          useMockData: false,
          requireAuth: true
        }}
      >
        <BrowserRouter>
          <ExampleDashboard />
        </BrowserRouter>
      </AppProviders>
    </ApiErrorBoundary>
  );
}