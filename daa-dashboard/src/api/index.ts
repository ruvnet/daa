// Export all API-related modules

// Types
export * from './types';

// Services
export { DaaApiService } from './services/DaaApiService';
export type { DaaApiConfig } from './services/DaaApiService';

// MCP Client
export { McpClient, DaaTools } from './mcp/McpClient';
export type { McpClientConfig } from './mcp/McpClient';

// WebSocket
export { WebSocketHandler, WebSocketEvent } from './websocket/WebSocketHandler';
export type { WebSocketConfig } from './websocket/WebSocketHandler';

// Error handling
export { 
  ErrorHandler, 
  DaaError, 
  ErrorCode,
  ApiErrorBoundary,
  useErrorHandler
} from './errors/ErrorHandler';

// Hooks
export * from './hooks';