import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@/test/utils/test-utils'
import MetricsChart from '../MetricsChart'

// Mock recharts to avoid rendering issues in tests
vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: any) => <div data-testid="responsive-container">{children}</div>,
  AreaChart: ({ children, data }: any) => (
    <div data-testid="area-chart" data-data={JSON.stringify(data)}>
      {children}
    </div>
  ),
  BarChart: ({ children, data }: any) => (
    <div data-testid="bar-chart" data-data={JSON.stringify(data)}>
      {children}
    </div>
  ),
  LineChart: ({ children }: any) => <div data-testid="line-chart">{children}</div>,
  Line: () => <div data-testid="line" />,
  Area: ({ dataKey }: any) => <div data-testid={`area-${dataKey}`} />,
  Bar: ({ dataKey }: any) => <div data-testid={`bar-${dataKey}`} />,
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
}))

describe('MetricsChart', () => {
  it('should render chart headers', () => {
    render(<MetricsChart />)
    
    expect(screen.getByText('System Performance (24h)')).toBeInTheDocument()
    expect(screen.getByText('Network Latency (ms)')).toBeInTheDocument()
  })

  it('should render responsive containers', () => {
    render(<MetricsChart />)
    
    const containers = screen.getAllByTestId('responsive-container')
    expect(containers).toHaveLength(2)
  })

  it('should render area chart for system performance', () => {
    render(<MetricsChart />)
    
    const areaChart = screen.getByTestId('area-chart')
    expect(areaChart).toBeInTheDocument()
  })

  it('should render bar chart for network latency', () => {
    render(<MetricsChart />)
    
    const barChart = screen.getByTestId('bar-chart')
    expect(barChart).toBeInTheDocument()
  })

  it('should pass performance data to area chart', () => {
    render(<MetricsChart />)
    
    const areaChart = screen.getByTestId('area-chart')
    const data = JSON.parse(areaChart.getAttribute('data-data') || '[]')
    
    expect(data).toHaveLength(7)
    expect(data[0]).toEqual({
      time: '00:00',
      cpu: 23,
      memory: 45,
      network: 67,
      agents: 1420
    })
    expect(data[6]).toEqual({
      time: '23:59',
      cpu: 25,
      memory: 48,
      network: 74,
      agents: 1430
    })
  })

  it('should pass network latency data to bar chart', () => {
    render(<MetricsChart />)
    
    const barChart = screen.getByTestId('bar-chart')
    const data = JSON.parse(barChart.getAttribute('data-data') || '[]')
    
    expect(data).toHaveLength(7)
    expect(data[0]).toEqual({ region: 'US-E', latency: 12 })
    expect(data[6]).toEqual({ region: 'OCE', latency: 52 })
  })

  it('should render CPU and memory areas in performance chart', () => {
    render(<MetricsChart />)
    
    expect(screen.getByTestId('area-cpu')).toBeInTheDocument()
    expect(screen.getByTestId('area-memory')).toBeInTheDocument()
  })

  it('should render latency bars in network chart', () => {
    render(<MetricsChart />)
    
    expect(screen.getByTestId('bar-latency')).toBeInTheDocument()
  })

  it('should render axes for both charts', () => {
    render(<MetricsChart />)
    
    const xAxes = screen.getAllByTestId('x-axis')
    const yAxes = screen.getAllByTestId('y-axis')
    
    expect(xAxes).toHaveLength(2)
    expect(yAxes).toHaveLength(2)
  })

  it('should render grids for both charts', () => {
    render(<MetricsChart />)
    
    const grids = screen.getAllByTestId('cartesian-grid')
    expect(grids).toHaveLength(2)
  })

  it('should have proper chart container heights', () => {
    const { container } = render(<MetricsChart />)
    
    const performanceContainer = container.querySelector('.h-64')
    const latencyContainer = container.querySelector('.h-48')
    
    expect(performanceContainer).toBeInTheDocument()
    expect(latencyContainer).toBeInTheDocument()
  })

  it('should apply correct styling to headers', () => {
    const { container } = render(<MetricsChart />)
    
    const headers = container.querySelectorAll('h4')
    headers.forEach(header => {
      expect(header).toHaveClass('text-sm', 'text-green-400/70', 'mb-3', 'font-mono', 'uppercase', 'tracking-wide')
    })
  })

  it('should have time-based data points for performance', () => {
    render(<MetricsChart />)
    
    const areaChart = screen.getByTestId('area-chart')
    const data = JSON.parse(areaChart.getAttribute('data-data') || '[]')
    
    const times = data.map((d: any) => d.time)
    expect(times).toEqual(['00:00', '04:00', '08:00', '12:00', '16:00', '20:00', '23:59'])
  })

  it('should have region-based data points for latency', () => {
    render(<MetricsChart />)
    
    const barChart = screen.getByTestId('bar-chart')
    const data = JSON.parse(barChart.getAttribute('data-data') || '[]')
    
    const regions = data.map((d: any) => d.region)
    expect(regions).toEqual(['US-E', 'US-W', 'EU-C', 'ASIA', 'EU-W', 'INDIA', 'OCE'])
  })

  it('should show increasing latency across regions', () => {
    render(<MetricsChart />)
    
    const barChart = screen.getByTestId('bar-chart')
    const data = JSON.parse(barChart.getAttribute('data-data') || '[]')
    
    // US-E should have lowest latency
    expect(data[0].latency).toBe(12)
    
    // OCE should have highest latency
    expect(data[6].latency).toBe(52)
  })
})