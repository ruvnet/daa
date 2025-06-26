import React from 'react';
import { AlertCircle, CheckCircle, Loader2, WifiOff, Wifi } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { useDaaStatus, useDaaAgents, useDaaNetworkStatus } from '@/hooks/use-daa-mcp';

interface McpConnectionStatusProps {
  showDetails?: boolean;
  compact?: boolean;
}

export function McpConnectionStatus({ showDetails = false, compact = false }: McpConnectionStatusProps) {
  const { data: daaStatus, isLoading: statusLoading, error: statusError, refetch: refetchStatus } = useDaaStatus();
  const { data: agents, isLoading: agentsLoading, error: agentsError, refetch: refetchAgents } = useDaaAgents();
  const { data: networkStatus, isLoading: networkLoading, error: networkError, refetch: refetchNetwork } = useDaaNetworkStatus();

  const isLoading = statusLoading || agentsLoading || networkLoading;
  const hasErrors = statusError || agentsError || networkError;
  const isConnected = !hasErrors && !isLoading && daaStatus && agents && networkStatus;

  const getConnectionStatus = () => {
    if (isLoading) return { status: 'connecting', label: 'Connecting...', color: 'text-yellow-400' };
    if (hasErrors) return { status: 'error', label: 'Connection Error', color: 'text-red-400' };
    if (isConnected) return { status: 'connected', label: 'Connected', color: 'text-green-400' };
    return { status: 'disconnected', label: 'Disconnected', color: 'text-gray-400' };
  };

  const connectionStatus = getConnectionStatus();

  const handleRetry = () => {
    refetchStatus();
    refetchAgents();
    refetchNetwork();
  };

  const getStatusIcon = () => {
    switch (connectionStatus.status) {
      case 'connecting':
        return <Loader2 className="h-5 w-5 animate-spin" />;
      case 'connected':
        return <CheckCircle className="h-5 w-5" />;
      case 'error':
        return <AlertCircle className="h-5 w-5" />;
      default:
        return <WifiOff className="h-5 w-5" />;
    }
  };

  if (compact) {
    return (
      <div className="flex items-center space-x-2">
        <div className={connectionStatus.color}>
          {getStatusIcon()}
        </div>
        <span className={`text-sm font-medium ${connectionStatus.color}`}>
          {connectionStatus.label}
        </span>
      </div>
    );
  }

  return (
    <Card className={`border-2 transition-all duration-300 ${
      hasErrors ? 'border-red-500/40 bg-red-900/10' : 
      isConnected ? 'border-green-500/40 bg-green-900/10' :
      'border-gray-500/40 bg-gray-900/10'
    }`}>
      <CardContent className="p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className={connectionStatus.color}>
              {getStatusIcon()}
            </div>
            <div>
              <h3 className={`font-semibold ${connectionStatus.color}`}>
                MCP Connection
              </h3>
              <p className="text-sm text-gray-500">
                {connectionStatus.label}
              </p>
            </div>
          </div>
          
          {hasErrors && (
            <button
              onClick={handleRetry}
              className="px-3 py-1 bg-red-600 hover:bg-red-700 text-white text-sm rounded transition-colors"
            >
              Retry
            </button>
          )}
        </div>

        {showDetails && (
          <div className="mt-4 space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Status Service:</span>
              <span className={statusError ? 'text-red-400' : 'text-green-400'}>
                {statusLoading ? 'Loading...' : statusError ? 'Error' : 'Connected'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Agent Service:</span>
              <span className={agentsError ? 'text-red-400' : 'text-green-400'}>
                {agentsLoading ? 'Loading...' : agentsError ? 'Error' : 'Connected'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Network Service:</span>
              <span className={networkError ? 'text-red-400' : 'text-green-400'}>
                {networkLoading ? 'Loading...' : networkError ? 'Error' : 'Connected'}
              </span>
            </div>
            {isConnected && (
              <>
                <div className="flex justify-between">
                  <span className="text-gray-400">Orchestrator:</span>
                  <span className="text-green-400">{daaStatus?.orchestrator}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Active Agents:</span>
                  <span className="text-green-400">{agents?.length || 0}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Network Peers:</span>
                  <span className="text-green-400">{networkStatus?.peers || 0}</span>
                </div>
              </>
            )}
          </div>
        )}

        {hasErrors && (
          <div className="mt-4 p-3 bg-red-900/20 border border-red-500/40 rounded">
            <h4 className="text-red-400 font-medium mb-2">Connection Errors:</h4>
            <div className="space-y-1 text-sm">
              {statusError && (
                <div className="text-red-300">
                  Status: {statusError.message || 'Connection failed'}
                </div>
              )}
              {agentsError && (
                <div className="text-red-300">
                  Agents: {agentsError.message || 'Connection failed'}
                </div>
              )}
              {networkError && (
                <div className="text-red-300">
                  Network: {networkError.message || 'Connection failed'}
                </div>
              )}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default McpConnectionStatus;