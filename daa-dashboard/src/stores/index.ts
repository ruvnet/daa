// Export all stores and their hooks

// Auth store
export { 
  useAuthStore,
  useAuth as useAuthState,
  useAuthActions 
} from './auth.store';

// Agents store
export {
  useAgentsStore,
  useAgent,
  useAgentsByType,
  useAgentsByStatus,
  useFilteredAgents,
  subscribeToAgentUpdates,
  subscribeToAgentStatus,
  AgentStatus
} from './agents.store';

// Dashboard store
export {
  useDashboardStore,
  useCurrentLayout,
  useMetrics,
  useAlerts,
  usePreferences,
  useDashboardActions
} from './dashboard.store';

// WebSocket store
export {
  useWebSocketStore,
  useConnectionStatus,
  useWebSocketStats,
  useLastMessage,
  useMessageHistory,
  subscribeToConnectionStatus,
  subscribeToMessages,
  ConnectionStatus
} from './websocket.store';

// Re-export types
export type { 
  User,
  AuthState 
} from './auth.store';

export type {
  DaaAgentInfo,
  AgentConfig,
  AgentsState
} from './agents.store';

export type {
  DashboardWidget,
  DashboardLayout,
  SystemMetrics,
  DashboardState
} from './dashboard.store';

export type {
  WebSocketMessage,
  WebSocketState
} from './websocket.store';