import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@/test/utils/test-utils'
import AlertsPanel from '../AlertsPanel'

describe('AlertsPanel', () => {
  it('should render alerts panel header', () => {
    render(<AlertsPanel />)
    
    expect(screen.getByText('System Alerts')).toBeInTheDocument()
  })

  it('should display active alerts count', () => {
    render(<AlertsPanel />)
    
    expect(screen.getByText('3 Active')).toBeInTheDocument()
  })

  it('should render all initial alerts', () => {
    render(<AlertsPanel />)
    
    expect(screen.getByText('High CPU Usage')).toBeInTheDocument()
    expect(screen.getByText('Security Scan Complete')).toBeInTheDocument()
    expect(screen.getByText('ML Training Delayed')).toBeInTheDocument()
    expect(screen.getByText('Unusual Network Activity')).toBeInTheDocument()
  })

  it('should render alert descriptions', () => {
    render(<AlertsPanel />)
    
    expect(screen.getByText('Node US-EAST-1 CPU utilization at 87%')).toBeInTheDocument()
    expect(screen.getByText('Weekly vulnerability scan found 0 critical issues')).toBeInTheDocument()
    expect(screen.getByText('Federated learning round 247 delayed due to low participation')).toBeInTheDocument()
    expect(screen.getByText('Increased P2P traffic detected in ASIA-PACIFIC region')).toBeInTheDocument()
  })

  it('should apply correct severity colors', () => {
    render(<AlertsPanel />)
    
    const warningAlert = screen.getByText('High CPU Usage').closest('div')?.parentElement
    expect(warningAlert).toHaveClass('border-yellow-500', 'bg-yellow-500/10', 'text-yellow-400')
    
    const successAlert = screen.getByText('Security Scan Complete').closest('div')?.parentElement
    expect(successAlert).toHaveClass('border-green-500', 'bg-green-500/10', 'text-green-400')
    
    const infoAlert = screen.getByText('Unusual Network Activity').closest('div')?.parentElement
    expect(infoAlert).toHaveClass('border-blue-500', 'bg-blue-500/10', 'text-blue-400')
  })

  it('should show acknowledged alerts separately', () => {
    render(<AlertsPanel />)
    
    // Check acknowledged section exists
    expect(screen.getByText('Acknowledged (1)')).toBeInTheDocument()
    
    // ML Training Delayed should be in acknowledged section
    const acknowledgedAlert = screen.getByText('ML Training Delayed').closest('div')?.parentElement
    expect(acknowledgedAlert).toHaveClass('opacity-60')
  })

  it('should acknowledge an alert when acknowledge button is clicked', () => {
    render(<AlertsPanel />)
    
    // Initially 3 active alerts
    expect(screen.getByText('3 Active')).toBeInTheDocument()
    
    // Find and click acknowledge button for first active alert
    const cpuAlert = screen.getByText('High CPU Usage').closest('div')?.parentElement
    const acknowledgeButton = cpuAlert?.querySelector('button[title="Acknowledge"]')
    
    fireEvent.click(acknowledgeButton!)
    
    // Should now be 2 active alerts
    expect(screen.getByText('2 Active')).toBeInTheDocument()
    
    // Should now show in acknowledged section
    expect(screen.getByText('Acknowledged (2)')).toBeInTheDocument()
  })

  it('should dismiss an alert when dismiss button is clicked', () => {
    render(<AlertsPanel />)
    
    // Find and click dismiss button for an alert
    const cpuAlert = screen.getByText('High CPU Usage').closest('div')?.parentElement
    const dismissButton = cpuAlert?.querySelector('button[title="Dismiss"]')
    
    fireEvent.click(dismissButton!)
    
    // Alert should be removed
    expect(screen.queryByText('High CPU Usage')).not.toBeInTheDocument()
    
    // Active count should decrease
    expect(screen.getByText('2 Active')).toBeInTheDocument()
  })

  it('should dismiss acknowledged alerts', () => {
    render(<AlertsPanel />)
    
    // Find the acknowledged alert
    const mlAlert = screen.getByText('ML Training Delayed').closest('div')?.parentElement
    const dismissButton = mlAlert?.querySelector('button[title="Dismiss"]')
    
    fireEvent.click(dismissButton!)
    
    // Alert should be removed
    expect(screen.queryByText('ML Training Delayed')).not.toBeInTheDocument()
    
    // Acknowledged section should be gone
    expect(screen.queryByText('Acknowledged (1)')).not.toBeInTheDocument()
  })

  it('should format timestamps correctly', () => {
    render(<AlertsPanel />)
    
    expect(screen.getByText('3m ago')).toBeInTheDocument()
    expect(screen.getByText('15m ago')).toBeInTheDocument()
    expect(screen.getByText('25m ago')).toBeInTheDocument()
    expect(screen.getByText('45m ago')).toBeInTheDocument()
  })

  it('should show empty state when all alerts are dismissed', () => {
    render(<AlertsPanel />)
    
    // Dismiss all alerts
    const dismissButtons = screen.getAllByTitle('Dismiss')
    dismissButtons.forEach(button => fireEvent.click(button))
    
    // Should show empty state
    expect(screen.getByText('All systems operational')).toBeInTheDocument()
  })

  it('should have scrollable content area', () => {
    render(<AlertsPanel />)
    
    const scrollableArea = screen.getByRole('generic').querySelector('.max-h-96.overflow-y-auto')
    
    expect(scrollableArea).toBeInTheDocument()
    expect(scrollableArea).toHaveClass('scrollbar-thin')
    expect(scrollableArea).toHaveClass('scrollbar-track-gray-800')
    expect(scrollableArea).toHaveClass('scrollbar-thumb-green-500')
  })

  it('should maintain alert order with active alerts first', () => {
    render(<AlertsPanel />)
    
    const allAlertTitles = screen.getAllByRole('heading', { level: 4 }).map(el => el.textContent)
    
    // Active alerts should come first
    expect(allAlertTitles[0]).toBe('High CPU Usage')
    expect(allAlertTitles[1]).toBe('Security Scan Complete')
    expect(allAlertTitles[2]).toBe('Unusual Network Activity')
    
    // Acknowledged alert should come last
    expect(allAlertTitles[3]).toBe('ML Training Delayed')
  })

  it('should render correct icons for each alert', () => {
    render(<AlertsPanel />)
    
    // Check that icons are rendered (they're SVG elements)
    const icons = screen.getAllByRole('img', { hidden: true })
    expect(icons.length).toBeGreaterThan(0)
  })

  it('should handle multiple acknowledgements', () => {
    render(<AlertsPanel />)
    
    // Acknowledge multiple alerts
    const acknowledgeButtons = screen.getAllByTitle('Acknowledge')
    acknowledgeButtons[0].click()
    acknowledgeButtons[1].click()
    
    // Should now have 1 active and 3 acknowledged
    expect(screen.getByText('1 Active')).toBeInTheDocument()
    expect(screen.getByText('Acknowledged (3)')).toBeInTheDocument()
  })
})