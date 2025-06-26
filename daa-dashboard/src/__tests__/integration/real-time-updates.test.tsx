import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, act } from '@/test/utils/test-utils'
import React from 'react'
import { DaaApiService } from '@/api/services/DaaApiService'
import { AgentStatus, TaskStatus } from '@/api/types'

// Mock component that displays real-time agent updates
function AgentMonitor({ api }: { api: DaaApiService }) {
  const [agents, setAgents] = React.useState<any[]>([])
  const [lastUpdate, setLastUpdate] = React.useState<Date | null>(null)

  React.useEffect(() => {
    // Load initial agents
    api.getAgents().then(setAgents)

    // Listen for real-time updates
    const handleAgentUpdate = (update: any) => {
      setAgents(prev => 
        prev.map(agent => 
          agent.id === update.agent_id 
            ? { ...agent, status: update.status, last_seen: update.last_seen }
            : agent
        )
      )
      setLastUpdate(new Date())
    }

    api.on('agent_update', handleAgentUpdate)

    return () => {
      api.off('agent_update', handleAgentUpdate)
    }
  }, [api])

  return (
    <div>
      <h2>Agent Monitor</h2>
      {lastUpdate && <div>Last update: {lastUpdate.toISOString()}</div>}
      <ul>
        {agents.map(agent => (
          <li key={agent.id} data-testid={`agent-${agent.id}`}>
            <span>{agent.name}</span>
            <span data-testid={`status-${agent.id}`}> - {agent.status}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}

// Mock component for task updates
function TaskMonitor({ api }: { api: DaaApiService }) {
  const [tasks, setTasks] = React.useState<any[]>([])
  const [updateCount, setUpdateCount] = React.useState(0)

  React.useEffect(() => {
    // Load initial tasks
    api.getTasks().then(setTasks)

    // Listen for task updates
    const handleTaskUpdate = (update: any) => {
      setTasks(prev => 
        prev.map(item => 
          item.task.id === update.task_id 
            ? { ...item, status: update.status }
            : item
        )
      )
      setUpdateCount(c => c + 1)
    }

    api.on('task_update', handleTaskUpdate)

    return () => {
      api.off('task_update', handleTaskUpdate)
    }
  }, [api])

  return (
    <div>
      <h2>Task Monitor</h2>
      <div>Updates received: {updateCount}</div>
      <ul>
        {tasks.map(({ task, status }) => (
          <li key={task.id} data-testid={`task-${task.id}`}>
            <span>{task.description}</span>
            <span data-testid={`task-status-${task.id}`}> - {status}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}

// Mock component for swarm messages
function SwarmMessageViewer({ api }: { api: DaaApiService }) {
  const [messages, setMessages] = React.useState<any[]>([])

  React.useEffect(() => {
    const handleSwarmMessage = (message: any) => {
      setMessages(prev => [...prev, message].slice(-10)) // Keep last 10
    }

    api.on('swarm_message', handleSwarmMessage)

    return () => {
      api.off('swarm_message', handleSwarmMessage)
    }
  }, [api])

  return (
    <div>
      <h2>Swarm Messages</h2>
      <div data-testid="message-count">Total: {messages.length}</div>
      <ul>
        {messages.map((msg, idx) => (
          <li key={idx}>
            From: {msg.from_agent} - Type: {msg.message_type}
          </li>
        ))}
      </ul>
    </div>
  )
}

describe('Real-time Updates Integration', () => {
  let api: DaaApiService

  beforeEach(() => {
    api = new DaaApiService({
      mcpServerUrl: 'http://localhost:3001/mcp',
      wsServerUrl: 'ws://localhost:3001/mcp/ws',
      useMockData: false // Use real WebSocket for these tests
    })
  })

  afterEach(() => {
    api.disconnect()
  })

  describe('Agent Status Updates', () => {
    it('should update agent status in real-time', async () => {
      await api.initialize()

      render(<AgentMonitor api={api} />)

      // Wait for initial load
      await waitFor(() => {
        expect(screen.getByText('Agent Monitor')).toBeInTheDocument()
      })

      // Get WebSocket handler
      const wsHandler = (api as any).wsHandler

      // Simulate agent update via WebSocket
      act(() => {
        wsHandler.emit('agent_update', {
          agent_id: 'agent-1',
          status: AgentStatus.Running,
          last_seen: new Date().toISOString()
        })
      })

      // Check that update was received
      await waitFor(() => {
        expect(screen.getByText(/Last update:/)).toBeInTheDocument()
      })
    })

    it('should handle multiple agent updates', async () => {
      await api.initialize()

      render(<AgentMonitor api={api} />)

      await waitFor(() => {
        expect(screen.getByText('Agent Monitor')).toBeInTheDocument()
      })

      const wsHandler = (api as any).wsHandler

      // Simulate multiple updates
      act(() => {
        wsHandler.emit('agent_update', {
          agent_id: 'agent-1',
          status: AgentStatus.Paused,
          last_seen: new Date().toISOString()
        })

        wsHandler.emit('agent_update', {
          agent_id: 'agent-2',
          status: AgentStatus.Stopped,
          last_seen: new Date().toISOString()
        })

        wsHandler.emit('agent_update', {
          agent_id: 'agent-3',
          status: AgentStatus.Error,
          last_seen: new Date().toISOString()
        })
      })

      // Verify updates were applied
      await waitFor(() => {
        const lastUpdate = screen.getByText(/Last update:/)
        expect(lastUpdate).toBeInTheDocument()
      })
    })
  })

  describe('Task Status Updates', () => {
    it('should update task status in real-time', async () => {
      await api.initialize()

      render(<TaskMonitor api={api} />)

      await waitFor(() => {
        expect(screen.getByText('Task Monitor')).toBeInTheDocument()
      })

      const wsHandler = (api as any).wsHandler

      // Simulate task update
      act(() => {
        wsHandler.emit('task_update', {
          task_id: 'task-1',
          status: TaskStatus.Completed
        })
      })

      await waitFor(() => {
        expect(screen.getByText('Updates received: 1')).toBeInTheDocument()
      })
    })

    it('should count multiple task updates', async () => {
      await api.initialize()

      render(<TaskMonitor api={api} />)

      await waitFor(() => {
        expect(screen.getByText('Task Monitor')).toBeInTheDocument()
      })

      const wsHandler = (api as any).wsHandler

      // Simulate multiple task updates
      act(() => {
        for (let i = 0; i < 5; i++) {
          wsHandler.emit('task_update', {
            task_id: `task-${i}`,
            status: TaskStatus.Running
          })
        }
      })

      await waitFor(() => {
        expect(screen.getByText('Updates received: 5')).toBeInTheDocument()
      })
    })
  })

  describe('Swarm Message Handling', () => {
    it('should receive and display swarm messages', async () => {
      await api.initialize()

      render(<SwarmMessageViewer api={api} />)

      await waitFor(() => {
        expect(screen.getByText('Swarm Messages')).toBeInTheDocument()
      })

      const wsHandler = (api as any).wsHandler

      // Simulate swarm message
      act(() => {
        wsHandler.emit('swarm_message', {
          from_agent: 'agent-123',
          to_agents: ['agent-456'],
          message_type: 'coordination',
          payload: { action: 'sync' }
        })
      })

      await waitFor(() => {
        expect(screen.getByText('Total: 1')).toBeInTheDocument()
        expect(screen.getByText('From: agent-123 - Type: coordination')).toBeInTheDocument()
      })
    })

    it('should limit message history to last 10', async () => {
      await api.initialize()

      render(<SwarmMessageViewer api={api} />)

      await waitFor(() => {
        expect(screen.getByText('Swarm Messages')).toBeInTheDocument()
      })

      const wsHandler = (api as any).wsHandler

      // Send 15 messages
      act(() => {
        for (let i = 0; i < 15; i++) {
          wsHandler.emit('swarm_message', {
            from_agent: `agent-${i}`,
            to_agents: [],
            message_type: 'heartbeat',
            payload: {}
          })
        }
      })

      await waitFor(() => {
        expect(screen.getByText('Total: 10')).toBeInTheDocument()
      })

      // Should only show last 10 messages
      expect(screen.queryByText('From: agent-0 - Type: heartbeat')).not.toBeInTheDocument()
      expect(screen.getByText('From: agent-14 - Type: heartbeat')).toBeInTheDocument()
    })
  })

  describe('Connection State Management', () => {
    it('should handle connection and disconnection events', async () => {
      let connectionState = 'unknown'
      
      api.on('connected', () => { connectionState = 'connected' })
      api.on('disconnected', () => { connectionState = 'disconnected' })

      await api.initialize()
      
      expect(connectionState).toBe('connected')

      // Simulate disconnection
      const wsHandler = (api as any).wsHandler
      act(() => {
        wsHandler.emit('disconnected')
      })

      expect(connectionState).toBe('disconnected')
    })
  })

  describe('Error Handling in Real-time Updates', () => {
    it('should handle errors in update callbacks gracefully', async () => {
      await api.initialize()

      const errorCallback = vi.fn().mockImplementation(() => {
        throw new Error('Callback error')
      })

      api.on('agent_update', errorCallback)

      const wsHandler = (api as any).wsHandler

      // This should not throw even though callback errors
      expect(() => {
        act(() => {
          wsHandler.emit('agent_update', { agent_id: 'test' })
        })
      }).not.toThrow()

      expect(errorCallback).toHaveBeenCalled()
    })
  })

  describe('Resource Subscription Updates', () => {
    it('should receive updates for subscribed resources', async () => {
      await api.initialize()

      const updates: any[] = []
      api.on('resource_changed', (data) => {
        updates.push(data)
      })

      // Subscribe to a resource
      await api.subscribeToResource('daa://agents')

      const wsHandler = (api as any).wsHandler

      // Simulate resource update notification
      act(() => {
        wsHandler.emit('resource_changed', {
          uri: 'daa://agents',
          action: 'update',
          timestamp: new Date().toISOString()
        })
      })

      await waitFor(() => {
        expect(updates).toHaveLength(1)
        expect(updates[0]).toEqual({
          uri: 'daa://agents',
          action: 'update',
          timestamp: expect.any(String)
        })
      })
    })
  })

  describe('System Metrics Updates', () => {
    it('should stream system metrics', async () => {
      await api.initialize()

      const metrics: any[] = []
      api.on('system_metric', (data) => {
        metrics.push(data)
      })

      const wsHandler = (api as any).wsHandler

      // Simulate metric updates
      act(() => {
        wsHandler.emit('system_metric', {
          cpu: 45.2,
          memory: 67.8,
          timestamp: new Date().toISOString()
        })

        wsHandler.emit('system_metric', {
          cpu: 46.1,
          memory: 68.2,
          timestamp: new Date().toISOString()
        })
      })

      await waitFor(() => {
        expect(metrics).toHaveLength(2)
        expect(metrics[0].cpu).toBe(45.2)
        expect(metrics[1].cpu).toBe(46.1)
      })
    })
  })
})