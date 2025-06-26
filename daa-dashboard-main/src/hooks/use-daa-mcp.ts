// React hooks for DAA MCP integration
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { daaMcpClient } from '@/lib/mcp-client';
import { DaaStatus, DaaAgent, DaaAgentDetails, NetworkStatus } from '@/lib/types';

// Hook for DAA system status
export function useDaaStatus() {
  return useQuery<DaaStatus>({
    queryKey: ['daa-status'],
    queryFn: () => daaMcpClient.callTool('daa_status'),
    refetchInterval: 30000, // Refresh every 30 seconds
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

// Hook for agent list
export function useDaaAgents() {
  return useQuery<DaaAgent[]>({
    queryKey: ['daa-agents'],
    queryFn: () => daaMcpClient.callTool('daa_agent_list'),
    refetchInterval: 10000, // Refresh every 10 seconds
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

// Hook for specific agent details
export function useDaaAgent(agentId: string) {
  return useQuery<DaaAgentDetails>({
    queryKey: ['daa-agent', agentId],
    queryFn: () => daaMcpClient.callTool('daa_agent_show', { agent_id: agentId }),
    enabled: !!agentId,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

// Hook for network status
export function useDaaNetworkStatus() {
  return useQuery<NetworkStatus>({
    queryKey: ['daa-network-status'],
    queryFn: () => daaMcpClient.callTool('daa_network_status'),
    refetchInterval: 15000, // Refresh every 15 seconds
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

// Hook for network peers
export function useDaaNetworkPeers() {
  return useQuery({
    queryKey: ['daa-network-peers'],
    queryFn: () => daaMcpClient.getNetworkPeers(),
    refetchInterval: 20000, // Refresh every 20 seconds
  });
}

// Hook for configuration
export function useDaaConfig() {
  return useQuery({
    queryKey: ['daa-config'],
    queryFn: () => daaMcpClient.getConfig(),
  });
}

// Hook for logs
export function useDaaLogs(lines = 100, level?: string, component?: string) {
  return useQuery({
    queryKey: ['daa-logs', lines, level, component],
    queryFn: () => daaMcpClient.getLogs(lines, level, component),
    refetchInterval: 5000, // Refresh every 5 seconds
  });
}

// Mutation hooks for actions

// Hook for creating agents
export function useCreateAgent() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ name, type, capabilities }: { name: string; type: string; capabilities?: string }) =>
      daaMcpClient.createAgent(name, type, capabilities),
    onSuccess: () => {
      // Invalidate and refetch agents list
      queryClient.invalidateQueries({ queryKey: ['daa-agents'] });
      queryClient.invalidateQueries({ queryKey: ['daa-status'] });
    },
  });
}

// Hook for stopping agents
export function useStopAgent() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ agentId, force }: { agentId: string; force?: boolean }) =>
      daaMcpClient.stopAgent(agentId, force),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daa-agents'] });
      queryClient.invalidateQueries({ queryKey: ['daa-status'] });
    },
  });
}

// Hook for restarting agents
export function useRestartAgent() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (agentId: string) => daaMcpClient.restartAgent(agentId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daa-agents'] });
      queryClient.invalidateQueries({ queryKey: ['daa-status'] });
    },
  });
}

// Hook for setting config values
export function useSetConfig() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ key, value }: { key: string; value: any }) =>
      daaMcpClient.setConfigValue(key, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daa-config'] });
    },
  });
}

// Hook for adding rules
export function useAddRule() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ name, ruleType, params, description }: { 
      name: string; 
      ruleType: string; 
      params?: string; 
      description?: string;
    }) => daaMcpClient.addRule(name, ruleType, params, description),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daa-status'] });
    },
  });
}

// Hook for starting orchestrator
export function useStartOrchestrator() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (daemon = false) => daaMcpClient.startOrchestrator(daemon),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daa-status'] });
    },
  });
}

// Hook for stopping orchestrator
export function useStopOrchestrator() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (force = false) => daaMcpClient.stopOrchestrator(force),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daa-status'] });
    },
  });
}

// Hook for connecting to network
export function useConnectToNetwork() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (node?: string) => daaMcpClient.connectToNetwork(node),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daa-network-status'] });
      queryClient.invalidateQueries({ queryKey: ['daa-network-peers'] });
    },
  });
}

// Hook for initializing DAA
export function useInitializeDAA() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ directory, template, force }: { 
      directory?: string; 
      template?: string; 
      force?: boolean;
    }) => daaMcpClient.initializeDAA(directory, template, force),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daa-status'] });
      queryClient.invalidateQueries({ queryKey: ['daa-config'] });
    },
  });
}