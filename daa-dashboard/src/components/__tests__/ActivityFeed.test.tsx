import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@/test/utils/test-utils'
import ActivityFeed from '../ActivityFeed'

describe('ActivityFeed', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('should render activity feed header', () => {
    render(<ActivityFeed />)
    
    expect(screen.getByText('Real-Time Activity Feed')).toBeInTheDocument()
  })

  it('should render initial activities', () => {
    render(<ActivityFeed />)
    
    expect(screen.getByText('New Treasury Agent deployed in US-EAST-1')).toBeInTheDocument()
    expect(screen.getByText('Anomalous trading pattern detected')).toBeInTheDocument()
    expect(screen.getByText('Monthly revenue milestone reached')).toBeInTheDocument()
    expect(screen.getByText('Federated learning round completed')).toBeInTheDocument()
    expect(screen.getByText('Node connectivity restored in EU-WEST-1')).toBeInTheDocument()
    expect(screen.getByText('New enterprise customer onboarded')).toBeInTheDocument()
  })

  it('should display activity details', () => {
    render(<ActivityFeed />)
    
    expect(screen.getByText('Agent ID: TRE-4472, Customer: ACME Corp')).toBeInTheDocument()
    expect(screen.getByText('Agent ID: DEF-8821, Threshold exceeded by 15%')).toBeInTheDocument()
    expect(screen.getByText('$8.95M achieved, 15.3% above target')).toBeInTheDocument()
  })

  it('should format timestamps correctly', () => {
    render(<ActivityFeed />)
    
    // Should show relative times
    expect(screen.getByText('2m ago')).toBeInTheDocument()
    expect(screen.getByText('5m ago')).toBeInTheDocument()
    expect(screen.getByText('8m ago')).toBeInTheDocument()
    expect(screen.getByText('12m ago')).toBeInTheDocument()
    expect(screen.getByText('15m ago')).toBeInTheDocument()
    expect(screen.getByText('20m ago')).toBeInTheDocument()
  })

  it('should apply correct severity colors', () => {
    render(<ActivityFeed />)
    
    const successActivity = screen.getByText('Monthly revenue milestone reached').closest('div')
    expect(successActivity).toHaveClass('text-green-400')
    
    const warningActivity = screen.getByText('Anomalous trading pattern detected').closest('div')
    expect(warningActivity).toHaveClass('text-yellow-400')
  })

  it('should add new activities periodically', async () => {
    render(<ActivityFeed />)
    
    // Initial count
    const initialActivities = screen.getAllByRole('generic').filter(el => 
      el.className.includes('flex items-start space-x-3')
    )
    expect(initialActivities).toHaveLength(6)
    
    // Fast-forward 15 seconds
    vi.advanceTimersByTime(15000)
    
    await waitFor(() => {
      const activities = screen.getAllByRole('generic').filter(el => 
        el.className.includes('flex items-start space-x-3')
      )
      expect(activities).toHaveLength(7)
    })
    
    // Check that new activity contains expected text
    expect(screen.getByText(/Agent performance update: \d+ agents processed/)).toBeInTheDocument()
    expect(screen.getByText(/Processing efficiency: \d+\.\d%/)).toBeInTheDocument()
  })

  it('should limit activities to 10 items', async () => {
    render(<ActivityFeed />)
    
    // Fast-forward multiple intervals
    for (let i = 0; i < 5; i++) {
      vi.advanceTimersByTime(15000)
    }
    
    await waitFor(() => {
      const activities = screen.getAllByRole('generic').filter(el => 
        el.className.includes('flex items-start space-x-3')
      )
      expect(activities).toHaveLength(10)
    })
  })

  it('should show "Just now" for very recent activities', () => {
    // Mock Date.now to control timestamp
    const now = Date.now()
    vi.setSystemTime(now)
    
    render(<ActivityFeed />)
    
    // Fast-forward to trigger new activity
    vi.advanceTimersByTime(15000)
    
    // Reset system time to just after the activity was added
    vi.setSystemTime(now + 15500)
    
    waitFor(() => {
      expect(screen.getByText('Just now')).toBeInTheDocument()
    })
  })

  it('should have scrollable content area', () => {
    render(<ActivityFeed />)
    
    const scrollableArea = screen.getByRole('generic', { 
      hidden: true 
    }).querySelector('.max-h-96.overflow-y-auto')
    
    expect(scrollableArea).toBeInTheDocument()
    expect(scrollableArea).toHaveClass('scrollbar-thin')
    expect(scrollableArea).toHaveClass('scrollbar-track-gray-800')
    expect(scrollableArea).toHaveClass('scrollbar-thumb-green-500')
  })

  it('should render correct icons for activity types', () => {
    render(<ActivityFeed />)
    
    // Check that icons are rendered (they're SVG elements)
    const icons = screen.getAllByRole('img', { hidden: true })
    expect(icons.length).toBeGreaterThan(0)
  })

  it('should clean up interval on unmount', () => {
    const { unmount } = render(<ActivityFeed />)
    
    const clearIntervalSpy = vi.spyOn(global, 'clearInterval')
    
    unmount()
    
    expect(clearIntervalSpy).toHaveBeenCalled()
  })
})