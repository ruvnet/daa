
import React, { useState, useEffect } from 'react';
import { 
  Activity, 
  Globe, 
  TrendingUp, 
  Users, 
  Server, 
  DollarSign,
  Bot,
  Network,
  Shield,
  AlertCircle,
  CheckCircle,
  Loader2
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import GlobalMap from '@/components/GlobalMap';
import MetricsChart from '@/components/MetricsChart';
import ActivityFeed from '@/components/ActivityFeed';
import NetworkTopology from '@/components/NetworkTopology';
import AlertsPanel from '@/components/AlertsPanel';
import DashboardLayout from '@/components/DashboardLayout';
import { useDaaStatus, useDaaAgents, useDaaNetworkStatus } from '@/hooks/use-daa-mcp';

const Index = () => {
  // Fetch real MCP data
  const { data: daaStatus, isLoading: statusLoading, error: statusError } = useDaaStatus();
  const { data: agents, isLoading: agentsLoading, error: agentsError } = useDaaAgents();
  const { data: networkStatus, isLoading: networkLoading, error: networkError } = useDaaNetworkStatus();

  // Calculate derived metrics from MCP data
  const metrics = {
    totalAgents: agents?.length || 0,
    activeAgents: agents?.filter(agent => agent.status === 'active').length || 0,
    idleAgents: agents?.filter(agent => agent.status === 'idle').length || 0,
    errorAgents: agents?.filter(agent => agent.status === 'error').length || 0,
    orchestratorStatus: daaStatus?.orchestrator || 'unknown',
    networkConnected: networkStatus?.status === 'connected',
    networkPeers: networkStatus?.peers || 0,
    systemUptime: daaStatus?.uptime || 'unknown',
    rulesCount: daaStatus?.rules || 0
  };

  const hasErrors = statusError || agentsError || networkError;
  const isLoading = statusLoading || agentsLoading || networkLoading;

  const MetricCard = ({ title, value, change, trend, icon: Icon, suffix = '', prefix = '', isLoading = false, error = null }) => (
    <Card className={`bg-gray-900/50 transition-all duration-300 ${
      error ? 'border-red-500/40 hover:border-red-500/60' : 
      'border-green-500/20 hover:border-green-500/40'
    }`}>
      <CardContent className="p-4 sm:p-6">
        <div className="flex items-center justify-between">
          <div className="flex-1 min-w-0">
            <p className={`text-xs sm:text-sm font-mono uppercase tracking-wide truncate ${
              error ? 'text-red-400/70' : 'text-green-400/70'
            }`}>{title}</p>
            {isLoading ? (
              <div className="flex items-center mt-1">
                <Loader2 className="h-4 w-4 animate-spin text-green-400 mr-1" />
                <p className="text-lg sm:text-2xl font-bold text-green-400 font-mono">Loading...</p>
              </div>
            ) : (
              <p className={`text-lg sm:text-2xl font-bold font-mono mt-1 truncate ${
                error ? 'text-red-400' : 'text-green-400'
              }`}>
                {prefix}{typeof value === 'number' ? value.toLocaleString() : value}{suffix}
              </p>
            )}
            {change && !isLoading && (
              <div className={`flex items-center mt-1 sm:mt-2 text-xs sm:text-sm font-mono ${
                error ? 'text-red-400' :
                trend === 'up' ? 'text-green-400' : 
                trend === 'down' ? 'text-red-400' : 'text-gray-400'
              }`}>
                <span className="truncate">{change}</span>
              </div>
            )}
          </div>
          <div className={`p-2 sm:p-3 rounded-lg ml-2 flex-shrink-0 ${
            error ? 'bg-red-500/10' : 'bg-green-500/10'
          }`}>
            <Icon className={`h-5 w-5 sm:h-8 sm:w-8 ${
              error ? 'text-red-400' : 'text-green-400'
            }`} />
          </div>
        </div>
      </CardContent>
    </Card>
  );

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        {/* Hero Metrics */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-3 sm:gap-6">
          <MetricCard 
            title="Total Agents" 
            value={isLoading ? '...' : metrics.totalAgents} 
            change={`Active: ${metrics.activeAgents}`} 
            trend={metrics.activeAgents > 0 ? "up" : "down"} 
            icon={Bot}
            isLoading={agentsLoading}
            error={agentsError} 
          />
          <MetricCard 
            title="Orchestrator" 
            value={isLoading ? '...' : metrics.orchestratorStatus.toUpperCase()} 
            change={daaStatus?.message || 'Checking status...'} 
            trend={metrics.orchestratorStatus === 'running' ? "up" : "down"} 
            icon={Server}
            isLoading={statusLoading}
            error={statusError} 
          />
          <MetricCard 
            title="Network Status" 
            value={isLoading ? '...' : (metrics.networkConnected ? 'CONNECTED' : 'DISCONNECTED')} 
            change={`${metrics.networkPeers} peers`} 
            trend={metrics.networkConnected ? "up" : "down"} 
            icon={Network}
            isLoading={networkLoading}
            error={networkError} 
          />
          <MetricCard 
            title="System Uptime" 
            value={isLoading ? '...' : metrics.systemUptime} 
            change={`${metrics.rulesCount} rules active`} 
            trend="up"
            icon={Activity}
            isLoading={statusLoading}
            error={statusError} 
          />
          <MetricCard 
            title="Agent Status" 
            value={isLoading ? '...' : `${metrics.activeAgents}/${metrics.totalAgents}`} 
            change={metrics.errorAgents > 0 ? `${metrics.errorAgents} errors` : 'All healthy'} 
            trend={metrics.errorAgents === 0 ? "up" : "down"} 
            icon={Shield}
            isLoading={agentsLoading}
            error={agentsError} 
          />
          <MetricCard 
            title="MCP Connection" 
            value={hasErrors ? 'ERROR' : 'CONNECTED'} 
            change={hasErrors ? 'Connection issues detected' : 'All systems operational'} 
            trend={hasErrors ? "down" : "up"} 
            icon={hasErrors ? AlertCircle : CheckCircle}
            isLoading={isLoading}
            error={hasErrors} 
          />
        </div>

        {/* Global Infrastructure Map */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader className="pb-3 sm:pb-6">
            <CardTitle className="text-green-400 flex items-center text-sm sm:text-base">
              <Globe className="h-4 w-4 sm:h-5 sm:w-5 mr-2" />
              Global Infrastructure Status
            </CardTitle>
          </CardHeader>
          <CardContent className="p-3 sm:p-6 pt-0">
            <GlobalMap />
          </CardContent>
        </Card>

        {/* Charts and Analytics */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 sm:gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader className="pb-3 sm:pb-6">
              <CardTitle className="text-green-400 flex items-center text-sm sm:text-base">
                <TrendingUp className="h-4 w-4 sm:h-5 sm:w-5 mr-2" />
                Performance Metrics
              </CardTitle>
            </CardHeader>
            <CardContent className="p-3 sm:p-6 pt-0">
              <MetricsChart />
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader className="pb-3 sm:pb-6">
              <CardTitle className="text-green-400 flex items-center text-sm sm:text-base">
                <Network className="h-4 w-4 sm:h-5 sm:w-5 mr-2" />
                Network Topology
              </CardTitle>
            </CardHeader>
            <CardContent className="p-3 sm:p-6 pt-0">
              <NetworkTopology />
            </CardContent>
          </Card>
        </div>

        {/* Activity Feed and Alerts */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 sm:gap-6">
          <div className="lg:col-span-2">
            <ActivityFeed />
          </div>
          <div>
            <AlertsPanel />
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
};

export default Index;
