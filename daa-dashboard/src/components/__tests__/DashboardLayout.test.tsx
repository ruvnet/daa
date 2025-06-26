import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@/test/utils/test-utils'
import userEvent from '@testing-library/user-event'
import DashboardLayout from '../DashboardLayout'

// Mock router
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useLocation: () => ({
      pathname: '/'
    })
  }
})

describe('DashboardLayout', () => {
  const user = userEvent.setup()

  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('should render header with logo and title', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    expect(screen.getByText('DAA Global Command')).toBeInTheDocument()
    expect(screen.getByText('Decentralized Autonomous Agents')).toBeInTheDocument()
  })

  it('should render mobile header title', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    expect(screen.getByText('DAA Command')).toBeInTheDocument()
  })

  it('should display current UTC time', () => {
    const mockDate = new Date('2024-01-01T12:00:00Z')
    vi.setSystemTime(mockDate)
    
    render(<DashboardLayout>Content</DashboardLayout>)
    
    expect(screen.getByText('UTC: 2024-01-01T12:00:00Z')).toBeInTheDocument()
  })

  it('should update time every 5 seconds', async () => {
    const initialDate = new Date('2024-01-01T12:00:00Z')
    vi.setSystemTime(initialDate)
    
    render(<DashboardLayout>Content</DashboardLayout>)
    
    expect(screen.getByText('UTC: 2024-01-01T12:00:00Z')).toBeInTheDocument()
    
    // Advance time by 5 seconds
    vi.setSystemTime(new Date('2024-01-01T12:00:05Z'))
    vi.advanceTimersByTime(5000)
    
    await waitFor(() => {
      expect(screen.getByText('UTC: 2024-01-01T12:00:05Z')).toBeInTheDocument()
    })
  })

  it('should render all navigation items', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const navItems = [
      'Dashboard',
      'Agent Management',
      'Economic Management',
      'Network Operations',
      'Governance & Rules',
      'AI & ML Operations',
      'Customer Management',
      'Analytics & Reporting',
      'System Administration',
      'Security & Compliance'
    ]
    
    navItems.forEach(item => {
      expect(screen.getByText(item)).toBeInTheDocument()
    })
  })

  it('should show badges for certain navigation items', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    expect(screen.getByText('1.2K')).toBeInTheDocument() // Agent Management
    expect(screen.getByText('+15%')).toBeInTheDocument() // Economic Management
    expect(screen.getByText('2.8K')).toBeInTheDocument() // Customer Management
    expect(screen.getByText('3')).toBeInTheDocument() // Security & Compliance
  })

  it('should highlight active navigation item', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const dashboardLink = screen.getByText('Dashboard').closest('a')
    expect(dashboardLink).toHaveClass('bg-green-500/20', 'text-green-400', 'border')
  })

  it('should render search button', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const searchButton = screen.getByRole('button', { name: 'Search' })
    expect(searchButton).toBeInTheDocument()
  })

  it('should open search dialog when search button is clicked', async () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const searchButton = screen.getByRole('button', { name: 'Search' })
    await user.click(searchButton)
    
    expect(screen.getByText('Global Search')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Search agents, customers, transactions...')).toBeInTheDocument()
  })

  it('should handle search submission', async () => {
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
    
    render(<DashboardLayout>Content</DashboardLayout>)
    
    // Open search
    await user.click(screen.getByRole('button', { name: 'Search' }))
    
    // Type search query
    const searchInput = screen.getByPlaceholderText('Search agents, customers, transactions...')
    await user.type(searchInput, 'test query')
    
    // Submit search
    const searchSubmitButton = screen.getByRole('button', { name: 'Search' })
    await user.click(searchSubmitButton)
    
    expect(consoleSpy).toHaveBeenCalledWith('Searching for:', 'test query')
    
    consoleSpy.mockRestore()
  })

  it('should render notifications bell with urgent indicator', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const notificationButton = screen.getByRole('button', { name: /Notifications.*urgent/ })
    expect(notificationButton).toBeInTheDocument()
    
    // Check for pulse indicator
    const pulseIndicator = notificationButton.querySelector('.bg-red-500.animate-pulse')
    expect(pulseIndicator).toBeInTheDocument()
  })

  it('should show notification popover when clicked', async () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const notificationButton = screen.getByRole('button', { name: /Notifications.*urgent/ })
    await user.click(notificationButton)
    
    expect(screen.getByText('System Notifications')).toBeInTheDocument()
    expect(screen.getByText('1 Urgent')).toBeInTheDocument()
    expect(screen.getByText('High CPU Usage')).toBeInTheDocument()
    expect(screen.getByText('Node US-EAST-1 at 87%')).toBeInTheDocument()
  })

  it('should mark notification as read when clicked', async () => {
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
    
    render(<DashboardLayout>Content</DashboardLayout>)
    
    // Open notifications
    await user.click(screen.getByRole('button', { name: /Notifications.*urgent/ }))
    
    // Click on a notification
    const notification = screen.getByText('High CPU Usage').closest('div[class*="cursor-pointer"]')
    await user.click(notification!)
    
    expect(consoleSpy).toHaveBeenCalledWith('Marking notification as read:', 1)
    
    consoleSpy.mockRestore()
  })

  it('should render settings button', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const settingsButton = screen.getByRole('button', { name: 'Settings' })
    expect(settingsButton).toBeInTheDocument()
  })

  it('should show settings popover when clicked', async () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const settingsButton = screen.getByRole('button', { name: 'Settings' })
    await user.click(settingsButton)
    
    expect(screen.getByText('Quick Settings')).toBeInTheDocument()
    expect(screen.getByText('Account Settings')).toBeInTheDocument()
    expect(screen.getByText('Security Settings')).toBeInTheDocument()
    expect(screen.getByText('Theme: Hacker Dark')).toBeInTheDocument()
  })

  it('should display admin status', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    expect(screen.getByText('ADMIN')).toBeInTheDocument()
  })

  it('should render logout button', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const logoutButton = screen.getByRole('button').querySelector('[class*="LogOut"]')?.parentElement
    expect(logoutButton).toBeInTheDocument()
  })

  it('should reload page on logout', () => {
    const reloadSpy = vi.fn()
    Object.defineProperty(window, 'location', {
      value: { reload: reloadSpy },
      writable: true
    })
    
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const logoutButton = screen.getAllByRole('button').find(btn => 
      btn.querySelector('[class*="LogOut"]')
    )
    
    fireEvent.click(logoutButton!)
    
    expect(reloadSpy).toHaveBeenCalled()
  })

  it('should render footer with system stats', () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    expect(screen.getByText('Load: 23.4%')).toBeInTheDocument()
    expect(screen.getByText('Memory: 67.8GB/128GB')).toBeInTheDocument()
    expect(screen.getByText('Network: 2.3GB/s')).toBeInTheDocument()
    expect(screen.getByText('DAA Global Command v2.1.0')).toBeInTheDocument()
    expect(screen.getByText('OPERATIONAL')).toBeInTheDocument()
  })

  it('should render children content', () => {
    render(<DashboardLayout><div>Test Content</div></DashboardLayout>)
    
    expect(screen.getByText('Test Content')).toBeInTheDocument()
  })

  it('should toggle mobile sidebar', async () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    const menuButton = screen.getByRole('button').querySelector('[class*="Menu"]')?.parentElement
    
    // Initially sidebar should be closed on mobile
    const sidebar = screen.getByRole('complementary')
    expect(sidebar).toHaveClass('-translate-x-full')
    
    // Open sidebar
    await user.click(menuButton!)
    expect(sidebar).toHaveClass('translate-x-0')
    
    // Close sidebar
    const closeButton = screen.getByRole('button').querySelector('[class*="X"]')?.parentElement
    await user.click(closeButton!)
    expect(sidebar).toHaveClass('-translate-x-full')
  })

  it('should close mobile sidebar when overlay is clicked', async () => {
    render(<DashboardLayout>Content</DashboardLayout>)
    
    // Open sidebar
    const menuButton = screen.getByRole('button').querySelector('[class*="Menu"]')?.parentElement
    await user.click(menuButton!)
    
    // Click overlay
    const overlay = screen.getByRole('generic').querySelector('.fixed.inset-0.z-30')
    await user.click(overlay!)
    
    // Sidebar should be closed
    const sidebar = screen.getByRole('complementary')
    expect(sidebar).toHaveClass('-translate-x-full')
  })

  it('should scroll to top and close sidebar on navigation', async () => {
    const scrollSpy = vi.fn()
    window.scrollTo = scrollSpy
    
    render(<DashboardLayout>Content</DashboardLayout>)
    
    // Open sidebar
    const menuButton = screen.getByRole('button').querySelector('[class*="Menu"]')?.parentElement
    await user.click(menuButton!)
    
    // Click navigation item
    const navItem = screen.getByText('Agent Management')
    await user.click(navItem)
    
    expect(scrollSpy).toHaveBeenCalledWith({ top: 0, behavior: 'smooth' })
    
    // Sidebar should be closed
    const sidebar = screen.getByRole('complementary')
    expect(sidebar).toHaveClass('-translate-x-full')
  })

  it('should cleanup timer on unmount', () => {
    const clearIntervalSpy = vi.spyOn(global, 'clearInterval')
    
    const { unmount } = render(<DashboardLayout>Content</DashboardLayout>)
    
    unmount()
    
    expect(clearIntervalSpy).toHaveBeenCalled()
  })
})