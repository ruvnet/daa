
import React, { useState } from 'react';
import { Bot, Activity, Zap, Shield, TrendingUp, Users, Plus, Play, Square, RotateCcw, Trash2, Settings, AlertCircle, CheckCircle, Clock, Loader2 } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, PieChart, Pie, Cell, BarChart, Bar } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';
import McpConnectionStatus from '@/components/McpConnectionStatus';
import { useDaaAgents, useCreateAgent, useStopAgent, useRestartAgent } from '@/hooks/use-daa-mcp';

const AgentManagement = () => {
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);
  const [newAgentForm, setNewAgentForm] = useState({
    name: '',
    type: 'treasury',
    capabilities: ''
  });

  // Fetch real MCP data
  const { data: agents = [], isLoading, error, refetch } = useDaaAgents();
  const createAgentMutation = useCreateAgent();
  const stopAgentMutation = useStopAgent();
  const restartAgentMutation = useRestartAgent();

  // Calculate metrics from real data
  const metrics = {
    totalAgents: agents.length,
    activeAgents: agents.filter(agent => agent.status === 'active').length,
    idleAgents: agents.filter(agent => agent.status === 'idle').length,
    errorAgents: agents.filter(agent => agent.status === 'error').length,
  };

  // Generate activity data based on agent statuses
  const agentActivityData = [
    { time: '00:00', active: metrics.activeAgents, idle: metrics.idleAgents },
    { time: '04:00', active: metrics.activeAgents + 2, idle: metrics.idleAgents - 1 },
    { time: '08:00', active: metrics.activeAgents + 5, idle: metrics.idleAgents - 2 },
    { time: '12:00', active: metrics.activeAgents + 3, idle: metrics.idleAgents + 1 },
    { time: '16:00', active: metrics.activeAgents - 1, idle: metrics.idleAgents + 2 },
    { time: '20:00', active: metrics.activeAgents, idle: metrics.idleAgents }
  ];

  const taskCompletionData = [
    { category: 'Treasury', completed: agents.filter(a => a.type === 'treasury' && a.status === 'active').length * 12, pending: 5 },
    { category: 'DeFi', completed: agents.filter(a => a.type === 'defi' && a.status === 'active').length * 15, pending: 3 },
    { category: 'Security', completed: agents.filter(a => a.type === 'security' && a.status === 'active').length * 8, pending: 2 },
    { category: 'Analytics', completed: agents.filter(a => a.type === 'analytics' && a.status === 'active').length * 10, pending: 4 }
  ];

  const securityBreachData = [
    { time: '00:00', breaches: metrics.errorAgents, resolved: Math.max(0, metrics.errorAgents - 1) },
    { time: '04:00', breaches: Math.max(0, metrics.errorAgents - 1), resolved: metrics.errorAgents },
    { time: '08:00', breaches: metrics.errorAgents + 1, resolved: metrics.errorAgents },
    { time: '12:00', breaches: metrics.errorAgents, resolved: metrics.errorAgents },
    { time: '16:00', breaches: Math.max(0, metrics.errorAgents - 1), resolved: metrics.errorAgents },
    { time: '20:00', breaches: metrics.errorAgents, resolved: Math.max(0, metrics.errorAgents - 1) }
  ];

  const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042'];

  const handleCreateAgent = async () => {
    try {
      await createAgentMutation.mutateAsync({
        name: newAgentForm.name,
        type: newAgentForm.type,
        capabilities: newAgentForm.capabilities
      });
      setShowCreateDialog(false);
      setNewAgentForm({ name: '', type: 'treasury', capabilities: '' });
    } catch (error) {
      console.error('Failed to create agent:', error);
    }
  };

  const handleStopAgent = async (agentId: string) => {
    try {
      await stopAgentMutation.mutateAsync({ agentId });
    } catch (error) {
      console.error('Failed to stop agent:', error);
    }
  };

  const handleRestartAgent = async (agentId: string) => {
    try {
      await restartAgentMutation.mutateAsync(agentId);
    } catch (error) {
      console.error('Failed to restart agent:', error);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'active': return <CheckCircle className="h-4 w-4 text-green-400" />;
      case 'idle': return <Clock className="h-4 w-4 text-yellow-400" />;
      case 'error': return <AlertCircle className="h-4 w-4 text-red-400" />;
      default: return <Activity className="h-4 w-4 text-gray-400" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-green-400 bg-green-400/10';
      case 'idle': return 'text-yellow-400 bg-yellow-400/10';
      case 'error': return 'text-red-400 bg-red-400/10';
      default: return 'text-gray-400 bg-gray-400/10';
    }
  };

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <Bot className="h-8 w-8 text-green-400" />
            <h1 className="text-3xl font-bold">Agent Management</h1>
            <div className={`px-3 py-1 rounded-full text-sm ${
              metrics.activeAgents > 0 ? 'bg-green-500/20 text-green-400' : 'bg-gray-500/20 text-gray-400'
            }`}>
              {isLoading ? 'Loading...' : `${metrics.activeAgents} Active`}
            </div>
          </div>
          <div className="flex items-center space-x-3">
            <McpConnectionStatus compact />
            <button
              onClick={() => setShowCreateDialog(true)}
              className="flex items-center space-x-2 bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg transition-colors"
              disabled={createAgentMutation.isPending}
            >
              <Plus className="h-4 w-4" />
              <span>Create Agent</span>
            </button>
          </div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Active Agents</p>
                  <p className="text-2xl font-bold text-green-400">{isLoading ? '...' : metrics.activeAgents}</p>
                  <p className="text-green-400 text-sm">{metrics.totalAgents} total agents</p>
                </div>
                <Activity className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Idle Agents</p>
                  <p className="text-2xl font-bold text-green-400">{isLoading ? '...' : metrics.idleAgents}</p>
                  <p className="text-green-400 text-sm">Standby mode</p>
                </div>
                <Zap className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Error Agents</p>
                  <p className={`text-2xl font-bold ${metrics.errorAgents > 0 ? 'text-red-400' : 'text-green-400'}`}>
                    {isLoading ? '...' : metrics.errorAgents}
                  </p>
                  <p className={`text-sm ${metrics.errorAgents > 0 ? 'text-red-400' : 'text-green-400'}`}>
                    {metrics.errorAgents === 0 ? 'All healthy' : 'Need attention'}
                  </p>
                </div>
                <Shield className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Overall Efficiency</p>
                  <p className="text-2xl font-bold text-green-400">
                    {isLoading ? '...' : `${metrics.totalAgents > 0 ? Math.round((metrics.activeAgents / metrics.totalAgents) * 100) : 0}%`}
                  </p>
                  <p className="text-green-400 text-sm">
                    {metrics.activeAgents > metrics.idleAgents ? 'High activity' : 'Low activity'}
                  </p>
                </div>
                <TrendingUp className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Agent Activity</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ active: { color: '#10b981' }, idle: { color: '#3b82f6' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={agentActivityData}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Area type="monotone" dataKey="active" stackId="1" stroke="#10b981" fill="#10b981" fillOpacity={0.3} />
                    <Area type="monotone" dataKey="idle" stackId="2" stroke="#3b82f6" fill="#3b82f6" fillOpacity={0.3} />
                  </AreaChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Task Completion Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ completed: { color: '#10b981' }, pending: { color: '#6b7280' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={taskCompletionData}>
                    <XAxis dataKey="category" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Bar dataKey="completed" fill="#10b981" />
                    <Bar dataKey="pending" fill="#6b7280" />
                  </BarChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>

        {/* Agent Status Chart */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Agent Status Timeline</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ breaches: { color: '#ef4444' }, resolved: { color: '#10b981' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={securityBreachData}>
                  <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                  <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Line type="monotone" dataKey="breaches" stroke="#ef4444" strokeWidth={2} />
                  <Line type="monotone" dataKey="resolved" stroke="#10b981" strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>

        {/* Agent List */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-green-400">Active Agents</CardTitle>
            <button
              onClick={refetch}
              className="flex items-center space-x-1 text-green-400 hover:text-green-300 transition-colors"
              disabled={isLoading}
            >
              <RotateCcw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
              <span className="text-sm">Refresh</span>
            </button>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-green-400" />
                <span className="ml-2 text-green-400">Loading agents...</span>
              </div>
            ) : error ? (
              <div className="text-center py-8">
                <AlertCircle className="h-8 w-8 text-red-400 mx-auto mb-2" />
                <p className="text-red-400 mb-4">Failed to load agents</p>
                <button
                  onClick={refetch}
                  className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-lg transition-colors"
                >
                  Try Again
                </button>
              </div>
            ) : agents.length === 0 ? (
              <div className="text-center py-8">
                <Bot className="h-8 w-8 text-gray-400 mx-auto mb-2" />
                <p className="text-gray-400 mb-4">No agents found</p>
                <button
                  onClick={() => setShowCreateDialog(true)}
                  className="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg transition-colors"
                >
                  Create First Agent
                </button>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {agents.map((agent) => (
                  <div
                    key={agent.id}
                    className="bg-gray-800/50 border border-gray-700 rounded-lg p-4 hover:border-green-500/40 transition-all duration-200"
                  >
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex items-center space-x-2">
                        <Bot className="h-5 w-5 text-green-400" />
                        <span className="font-medium text-green-400">{agent.name}</span>
                      </div>
                      <div className={`flex items-center space-x-1 px-2 py-1 rounded-full text-xs ${getStatusColor(agent.status)}`}>
                        {getStatusIcon(agent.status)}
                        <span className="capitalize">{agent.status}</span>
                      </div>
                    </div>
                    
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-400">Type:</span>
                        <span className="text-green-400 capitalize">{agent.type}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">ID:</span>
                        <span className="text-green-400 font-mono text-xs">{agent.id}</span>
                      </div>
                      {agent.capabilities && (
                        <div className="flex justify-between">
                          <span className="text-gray-400">Capabilities:</span>
                          <span className="text-green-400 text-xs">{agent.capabilities}</span>
                        </div>
                      )}
                    </div>

                    <div className="flex items-center justify-between mt-4 pt-3 border-t border-gray-700">
                      <div className="flex space-x-1">
                        {agent.status === 'active' ? (
                          <button
                            onClick={() => handleStopAgent(agent.id)}
                            disabled={stopAgentMutation.isPending}
                            className="p-1 text-red-400 hover:text-red-300 hover:bg-red-400/10 rounded transition-colors"
                            title="Stop Agent"
                          >
                            <Square className="h-4 w-4" />
                          </button>
                        ) : (
                          <button
                            onClick={() => handleRestartAgent(agent.id)}
                            disabled={restartAgentMutation.isPending}
                            className="p-1 text-green-400 hover:text-green-300 hover:bg-green-400/10 rounded transition-colors"
                            title="Start Agent"
                          >
                            <Play className="h-4 w-4" />
                          </button>
                        )}
                        <button
                          onClick={() => handleRestartAgent(agent.id)}
                          disabled={restartAgentMutation.isPending}
                          className="p-1 text-blue-400 hover:text-blue-300 hover:bg-blue-400/10 rounded transition-colors"
                          title="Restart Agent"
                        >
                          <RotateCcw className="h-4 w-4" />
                        </button>
                      </div>
                      <button
                        onClick={() => setSelectedAgent(agent.id)}
                        className="p-1 text-gray-400 hover:text-gray-300 hover:bg-gray-400/10 rounded transition-colors"
                        title="Agent Settings"
                      >
                        <Settings className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Create Agent Dialog */}
        {showCreateDialog && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <Card className="bg-gray-900 border-green-500/40 w-full max-w-md">
              <CardHeader>
                <CardTitle className="text-green-400">Create New Agent</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-400 mb-1">
                    Agent Name
                  </label>
                  <input
                    type="text"
                    value={newAgentForm.name}
                    onChange={(e) => setNewAgentForm(prev => ({ ...prev, name: e.target.value }))}
                    className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-green-400 focus:border-green-500 focus:outline-none"
                    placeholder="e.g., treasury_bot_1"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-400 mb-1">
                    Agent Type
                  </label>
                  <select
                    value={newAgentForm.type}
                    onChange={(e) => setNewAgentForm(prev => ({ ...prev, type: e.target.value }))}
                    className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-green-400 focus:border-green-500 focus:outline-none"
                  >
                    <option value="treasury">Treasury</option>
                    <option value="defi">DeFi</option>
                    <option value="security">Security</option>
                    <option value="analytics">Analytics</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-400 mb-1">
                    Capabilities (Optional)
                  </label>
                  <input
                    type="text"
                    value={newAgentForm.capabilities}
                    onChange={(e) => setNewAgentForm(prev => ({ ...prev, capabilities: e.target.value }))}
                    className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-green-400 focus:border-green-500 focus:outline-none"
                    placeholder="e.g., trading, risk_management"
                  />
                </div>
                <div className="flex space-x-3 pt-4">
                  <button
                    onClick={() => setShowCreateDialog(false)}
                    className="flex-1 px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleCreateAgent}
                    disabled={!newAgentForm.name || createAgentMutation.isPending}
                    className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {createAgentMutation.isPending ? (
                      <div className="flex items-center justify-center">
                        <Loader2 className="h-4 w-4 animate-spin mr-1" />
                        Creating...
                      </div>
                    ) : (
                      'Create Agent'
                    )}
                  </button>
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
};

export default AgentManagement;
