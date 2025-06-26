import React from 'react';
import { useDaaApi, useAgents, useTasks, useSystemMonitoring } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Loader2 } from 'lucide-react';

export function ApiExample() {
  const { api, isInitialized, error: apiError } = useDaaApi();
  const { agents, loading: agentsLoading, fetchAgents, spawnAgent, stopAgent } = useAgents(api);
  const { tasks, loading: tasksLoading, createTask } = useTasks(api);
  const { metrics, health, fetchHealth } = useSystemMonitoring(api, 10000); // Refresh every 10 seconds

  const handleSpawnAgent = async () => {
    try {
      await spawnAgent({
        name: 'Test Agent',
        agent_type: 'analytics',
        capabilities: ['data_analysis', 'reporting'],
        rules: {},
        metadata: { purpose: 'demo' }
      });
    } catch (error) {
      console.error('Failed to spawn agent:', error);
    }
  };

  const handleCreateTask = async () => {
    try {
      await createTask({
        task_type: 'analysis',
        description: 'Analyze market trends',
        priority: 'medium'
      });
    } catch (error) {
      console.error('Failed to create task:', error);
    }
  };

  const handleHealthCheck = async () => {
    await fetchHealth(true); // Deep health check
  };

  if (!isInitialized) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin" />
        <span className="ml-2">Initializing API...</span>
      </div>
    );
  }

  if (apiError) {
    return (
      <div className="p-8 text-red-600">
        Error initializing API: {apiError.message}
      </div>
    );
  }

  return (
    <div className="container mx-auto p-6 space-y-6">
      {/* System Status */}
      <Card>
        <CardHeader>
          <CardTitle>System Status</CardTitle>
          <CardDescription>Overall system health and metrics</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <p className="text-sm font-medium">Health Status</p>
              <Badge variant={health?.overall_status === 'healthy' ? 'default' : 'destructive'}>
                {health?.overall_status || 'Unknown'}
              </Badge>
            </div>
            {metrics && (
              <>
                <div>
                  <p className="text-sm font-medium">Active Agents</p>
                  <p className="text-2xl font-bold">{metrics.agents.running}</p>
                </div>
                <div>
                  <p className="text-sm font-medium">Running Tasks</p>
                  <p className="text-2xl font-bold">{metrics.tasks.running}</p>
                </div>
              </>
            )}
          </div>
          <Button onClick={handleHealthCheck} className="mt-4" size="sm">
            Run Deep Health Check
          </Button>
        </CardContent>
      </Card>

      {/* Agents Section */}
      <Card>
        <CardHeader>
          <CardTitle>Agents</CardTitle>
          <CardDescription>Manage DAA agents</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex gap-2">
              <Button onClick={() => fetchAgents()} disabled={agentsLoading}>
                {agentsLoading ? <Loader2 className="h-4 w-4 animate-spin mr-2" /> : null}
                Refresh Agents
              </Button>
              <Button onClick={handleSpawnAgent} variant="outline">
                Spawn New Agent
              </Button>
            </div>
            
            <div className="space-y-2">
              {agents.map((agent) => (
                <div key={agent.id} className="flex items-center justify-between p-3 border rounded">
                  <div>
                    <p className="font-medium">{agent.name}</p>
                    <p className="text-sm text-muted-foreground">{agent.agent_type}</p>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant={agent.status === 'running' ? 'default' : 'secondary'}>
                      {agent.status}
                    </Badge>
                    <Button 
                      onClick={() => stopAgent(agent.id)} 
                      size="sm" 
                      variant="destructive"
                      disabled={agent.status !== 'running'}
                    >
                      Stop
                    </Button>
                  </div>
                </div>
              ))}
              {agents.length === 0 && (
                <p className="text-muted-foreground text-center py-4">No agents found</p>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Tasks Section */}
      <Card>
        <CardHeader>
          <CardTitle>Tasks</CardTitle>
          <CardDescription>View and manage tasks</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <Button onClick={handleCreateTask} disabled={tasksLoading}>
              Create New Task
            </Button>
            
            <div className="space-y-2">
              {tasks.map((item) => (
                <div key={item.task.id} className="flex items-center justify-between p-3 border rounded">
                  <div>
                    <p className="font-medium">{item.task.description}</p>
                    <p className="text-sm text-muted-foreground">{item.task.task_type}</p>
                  </div>
                  <Badge variant={
                    item.status === 'completed' ? 'default' :
                    item.status === 'failed' ? 'destructive' :
                    item.status === 'running' ? 'secondary' : 'outline'
                  }>
                    {item.status}
                  </Badge>
                </div>
              ))}
              {tasks.length === 0 && (
                <p className="text-muted-foreground text-center py-4">No tasks found</p>
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}