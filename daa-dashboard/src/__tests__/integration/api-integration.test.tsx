import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@/test/utils/test-utils'
import { QueryClient } from '@tanstack/react-query'
import userEvent from '@testing-library/user-event'
import { DaaApiService } from '@/api/services/DaaApiService'
import React from 'react'

// Mock API context
const ApiContext = React.createContext<DaaApiService | null>(null)

// Test component that uses API
function TestAgentList() {
  const api = React.useContext(ApiContext)
  const [agents, setAgents] = React.useState<any[]>([])
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  const loadAgents = async () => {
    if (!api) return
    
    setLoading(true)
    setError(null)
    
    try {
      const data = await api.getAgents()
      setAgents(data)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }

  React.useEffect(() => {
    loadAgents()
  }, [])

  const handleStopAgent = async (agentId: string) => {
    if (!api) return
    
    try {
      await api.stopAgent(agentId)
      await loadAgents()
    } catch (err: any) {
      setError(err.message)
    }
  }

  if (loading) return <div>Loading agents...</div>
  if (error) return <div>Error: {error}</div>

  return (
    <div>
      <h2>Agent List</h2>
      <button onClick={loadAgents}>Refresh</button>
      <ul>
        {agents.map((agent) => (
          <li key={agent.id}>
            <span>{agent.name}</span>
            <span> - {agent.status}</span>
            <button onClick={() => handleStopAgent(agent.id)}>Stop</button>
          </li>
        ))}
      </ul>
    </div>
  )
}

// Test component for real-time updates
function TestRealTimeMetrics() {
  const api = React.useContext(ApiContext)
  const [metrics, setMetrics] = React.useState<any>(null)
  const [connected, setConnected] = React.useState(false)

  React.useEffect(() => {
    if (!api) return

    const handleMetricUpdate = (data: any) => {
      setMetrics(data)
    }

    const handleConnected = () => {
      setConnected(true)
    }

    const handleDisconnected = () => {
      setConnected(false)
    }

    api.on('system_metric', handleMetricUpdate)
    api.on('connected', handleConnected)
    api.on('disconnected', handleDisconnected)

    // Subscribe to metrics
    api.subscribeToResource('daa://system/metrics')

    return () => {
      api.off('system_metric', handleMetricUpdate)
      api.off('connected', handleConnected)
      api.off('disconnected', handleDisconnected)
      api.unsubscribeFromResource('daa://system/metrics')
    }
  }, [api])

  return (
    <div>
      <h2>Real-Time Metrics</h2>
      <div>Connection: {connected ? 'Connected' : 'Disconnected'}</div>
      {metrics && (
        <div>
          <div>CPU: {metrics.cpu_usage}%</div>
          <div>Memory: {metrics.memory_usage}%</div>
        </div>
      )}
    </div>
  )
}

describe('API Integration Tests', () => {
  let api: DaaApiService
  const user = userEvent.setup()

  beforeEach(() => {
    api = new DaaApiService({
      mcpServerUrl: 'http://localhost:3001/mcp',
      wsServerUrl: 'ws://localhost:3001/mcp/ws',
      useMockData: true,
      mockDelay: 0
    })
  })

  afterEach(() => {
    api.disconnect()
  })

  describe('Agent Management Integration', () => {
    it('should load and display agents from API', async () => {
      await api.initialize()

      render(
        <ApiContext.Provider value={api}>
          <TestAgentList />
        </ApiContext.Provider>
      )

      expect(screen.getByText('Loading agents...')).toBeInTheDocument()

      await waitFor(() => {
        expect(screen.getByText('Agent List')).toBeInTheDocument()
      })

      // Should display agents
      const agents = await screen.findAllByRole('listitem')
      expect(agents.length).toBe(10) // Mock factory generates 10 agents

      // Each agent should have name and status
      agents.forEach((agent) => {
        expect(agent.textContent).toMatch(/\w+ \w+ - (running|paused|stopped)/)
      })
    })

    it('should handle refresh action', async () => {
      await api.initialize()

      render(
        <ApiContext.Provider value={api}>
          <TestAgentList />
        </ApiContext.Provider>
      )

      await waitFor(() => {
        expect(screen.getByText('Agent List')).toBeInTheDocument()
      })

      const refreshButton = screen.getByText('Refresh')
      await user.click(refreshButton)

      // Should show loading state
      expect(screen.getByText('Loading agents...')).toBeInTheDocument()

      // Should reload agents
      await waitFor(() => {
        expect(screen.getByText('Agent List')).toBeInTheDocument()
      })
    })

    it('should handle stop agent action', async () => {
      await api.initialize()

      render(
        <ApiContext.Provider value={api}>
          <TestAgentList />
        </ApiContext.Provider>
      )

      await waitFor(() => {
        expect(screen.getByText('Agent List')).toBeInTheDocument()
      })

      const stopButtons = await screen.findAllByText('Stop')
      await user.click(stopButtons[0])

      // Should reload the list after stopping
      await waitFor(() => {
        expect(screen.getAllByRole('listitem')).toHaveLength(10)
      })
    })
  })

  describe('Real-time Updates Integration', () => {
    beforeEach(() => {
      vi.useFakeTimers()
    })

    afterEach(() => {
      vi.useRealTimers()
    })

    it('should receive and display real-time metrics', async () => {
      await api.initialize()

      render(
        <ApiContext.Provider value={api}>
          <TestRealTimeMetrics />
        </ApiContext.Provider>
      )

      expect(screen.getByText('Real-Time Metrics')).toBeInTheDocument()
      expect(screen.getByText('Connection: Disconnected')).toBeInTheDocument()

      // Fast-forward to trigger mock update
      vi.advanceTimersByTime(5000)

      await waitFor(() => {
        expect(screen.getByText(/CPU: \d+(\.\d+)?%/)).toBeInTheDocument()
        expect(screen.getByText(/Memory: \d+(\.\d+)?%/)).toBeInTheDocument()
      })
    })
  })

  describe('Error Handling Integration', () => {
    it('should display error when API call fails', async () => {
      // Create API instance that will fail
      const failingApi = new DaaApiService({
        mcpServerUrl: 'http://localhost:3001/mcp',
        wsServerUrl: 'ws://localhost:3001/mcp/ws',
        useMockData: false // Use real API which will fail in test
      })

      render(
        <ApiContext.Provider value={failingApi}>
          <TestAgentList />
        </ApiContext.Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Error:/)).toBeInTheDocument()
      })

      failingApi.disconnect()
    })
  })

  describe('State Management Integration', () => {
    it('should integrate with React Query for caching', async () => {
      const queryClient = new QueryClient({
        defaultOptions: {
          queries: {
            retry: false,
            staleTime: 1000,
          },
        },
      })

      // Component using React Query
      function TestWithQuery() {
        const api = React.useContext(ApiContext)
        const { data: agents, isLoading, error } = React.useSuspenseQuery({
          queryKey: ['agents'],
          queryFn: () => api!.getAgents(),
        })

        if (isLoading) return <div>Loading...</div>
        if (error) return <div>Error: {(error as Error).message}</div>

        return (
          <div>
            <h2>Agents with Query</h2>
            <div>Total: {agents?.length || 0}</div>
          </div>
        )
      }

      await api.initialize()

      render(
        <ApiContext.Provider value={api}>
          <TestWithQuery />
        </ApiContext.Provider>,
        { queryClient }
      )

      await waitFor(() => {
        expect(screen.getByText('Agents with Query')).toBeInTheDocument()
        expect(screen.getByText('Total: 10')).toBeInTheDocument()
      })
    })
  })

  describe('Authentication Flow Integration', () => {
    it('should handle authentication and API access', async () => {
      // Mock auth manager
      const authManager = {
        isAuthenticated: false,
        authenticate: vi.fn().mockResolvedValue(true),
        getToken: vi.fn().mockResolvedValue('mock-token'),
      }

      function TestAuthFlow() {
        const api = React.useContext(ApiContext)
        const [authenticated, setAuthenticated] = React.useState(false)
        const [data, setData] = React.useState<any>(null)

        const handleLogin = async () => {
          await authManager.authenticate()
          setAuthenticated(true)
          
          // Now make authenticated API call
          const agents = await api!.getAgents()
          setData(agents)
        }

        return (
          <div>
            {!authenticated ? (
              <button onClick={handleLogin}>Login</button>
            ) : (
              <div>
                <div>Authenticated</div>
                {data && <div>Agents loaded: {data.length}</div>}
              </div>
            )}
          </div>
        )
      }

      await api.initialize()

      render(
        <ApiContext.Provider value={api}>
          <TestAuthFlow />
        </ApiContext.Provider>
      )

      const loginButton = screen.getByText('Login')
      await user.click(loginButton)

      await waitFor(() => {
        expect(screen.getByText('Authenticated')).toBeInTheDocument()
        expect(screen.getByText('Agents loaded: 10')).toBeInTheDocument()
      })

      expect(authManager.authenticate).toHaveBeenCalled()
    })
  })

  describe('Batch Operations Integration', () => {
    it('should handle multiple concurrent API calls', async () => {
      await api.initialize()

      const results = await Promise.all([
        api.getAgents(),
        api.getTasks(),
        api.getSystemMetrics(),
        api.performHealthcheck(),
      ])

      expect(results[0]).toHaveLength(10) // agents
      expect(results[1]).toHaveLength(15) // tasks
      expect(results[2]).toHaveProperty('agents')
      expect(results[3]).toHaveProperty('overall_status')
    })
  })

  describe('Resource Subscription Integration', () => {
    it('should handle multiple resource subscriptions', async () => {
      await api.initialize()

      const updates: any[] = []
      
      api.on('resource_update', (data) => {
        updates.push(data)
      })

      // Subscribe to multiple resources
      await api.subscribeToResource('daa://agents')
      await api.subscribeToResource('daa://tasks')
      await api.subscribeToResource('daa://system/metrics')

      // Fast-forward to get updates
      vi.useFakeTimers()
      vi.advanceTimersByTime(5000)
      vi.useRealTimers()

      await waitFor(() => {
        expect(updates.length).toBeGreaterThanOrEqual(3)
      })

      // Check that we got updates for each resource
      const uris = updates.map(u => u.uri)
      expect(uris).toContain('daa://agents')
      expect(uris).toContain('daa://tasks')
      expect(uris).toContain('daa://system/metrics')
    })
  })
})