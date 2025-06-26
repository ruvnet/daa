import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@/test/utils/test-utils'
import userEvent from '@testing-library/user-event'
import LoginForm from '../LoginForm'

describe('LoginForm', () => {
  const mockOnSwitchToRegister = vi.fn()
  const mockOnSwitchToForgotPassword = vi.fn()
  const mockOnLogin = vi.fn()
  const user = userEvent.setup()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  const renderLoginForm = () => {
    return render(
      <LoginForm
        onSwitchToRegister={mockOnSwitchToRegister}
        onSwitchToForgotPassword={mockOnSwitchToForgotPassword}
        onLogin={mockOnLogin}
      />
    )
  }

  it('should render login form header', () => {
    renderLoginForm()
    
    expect(screen.getByText('DAA Global Command')).toBeInTheDocument()
    expect(screen.getByText('Secure Access Portal')).toBeInTheDocument()
  })

  it('should render email input field', () => {
    renderLoginForm()
    
    const emailLabel = screen.getByText('Email Address')
    const emailInput = screen.getByPlaceholderText('demo@daa.dark')
    
    expect(emailLabel).toBeInTheDocument()
    expect(emailInput).toBeInTheDocument()
    expect(emailInput).toHaveAttribute('type', 'email')
    expect(emailInput).toBeRequired()
  })

  it('should render password input field', () => {
    renderLoginForm()
    
    const passwordLabel = screen.getByText('Password')
    const passwordInput = screen.getByPlaceholderText('••••••••')
    
    expect(passwordLabel).toBeInTheDocument()
    expect(passwordInput).toBeInTheDocument()
    expect(passwordInput).toHaveAttribute('type', 'password')
    expect(passwordInput).toBeRequired()
  })

  it('should toggle password visibility', async () => {
    renderLoginForm()
    
    const passwordInput = screen.getByPlaceholderText('••••••••')
    const toggleButton = passwordInput.parentElement?.querySelector('button[type="button"]')
    
    expect(passwordInput).toHaveAttribute('type', 'password')
    
    await user.click(toggleButton!)
    expect(passwordInput).toHaveAttribute('type', 'text')
    
    await user.click(toggleButton!)
    expect(passwordInput).toHaveAttribute('type', 'password')
  })

  it('should update email input value', async () => {
    renderLoginForm()
    
    const emailInput = screen.getByPlaceholderText('demo@daa.dark')
    
    await user.type(emailInput, 'test@example.com')
    
    expect(emailInput).toHaveValue('test@example.com')
  })

  it('should update password input value', async () => {
    renderLoginForm()
    
    const passwordInput = screen.getByPlaceholderText('••••••••')
    
    await user.type(passwordInput, 'testpassword123')
    
    expect(passwordInput).toHaveValue('testpassword123')
  })

  it('should show login button with correct text', () => {
    renderLoginForm()
    
    const loginButton = screen.getByRole('button', { name: /ACCESS SYSTEM/i })
    
    expect(loginButton).toBeInTheDocument()
    expect(loginButton).toHaveClass('bg-green-500')
  })

  it('should handle form submission', async () => {
    vi.useFakeTimers()
    renderLoginForm()
    
    const emailInput = screen.getByPlaceholderText('demo@daa.dark')
    const passwordInput = screen.getByPlaceholderText('••••••••')
    const loginButton = screen.getByRole('button', { name: /ACCESS SYSTEM/i })
    
    await user.type(emailInput, 'test@example.com')
    await user.type(passwordInput, 'password123')
    
    fireEvent.click(loginButton)
    
    // Should show loading state
    expect(screen.getByText('AUTHENTICATING...')).toBeInTheDocument()
    expect(loginButton).toBeDisabled()
    
    // Fast-forward the timeout
    vi.advanceTimersByTime(1000)
    
    await waitFor(() => {
      expect(mockOnLogin).toHaveBeenCalledWith('test@example.com', 'password123')
    })
    
    vi.useRealTimers()
  })

  it('should prevent form submission with empty fields', () => {
    renderLoginForm()
    
    const form = screen.getByRole('generic').querySelector('form')
    const submitEvent = new Event('submit', { bubbles: true, cancelable: true })
    
    fireEvent(form!, submitEvent)
    
    expect(mockOnLogin).not.toHaveBeenCalled()
  })

  it('should render forgot password link', () => {
    renderLoginForm()
    
    const forgotPasswordLink = screen.getByText('Forgot Password?')
    
    expect(forgotPasswordLink).toBeInTheDocument()
  })

  it('should call onSwitchToForgotPassword when forgot password is clicked', async () => {
    renderLoginForm()
    
    const forgotPasswordLink = screen.getByText('Forgot Password?')
    
    await user.click(forgotPasswordLink)
    
    expect(mockOnSwitchToForgotPassword).toHaveBeenCalledTimes(1)
  })

  it('should render register link', () => {
    renderLoginForm()
    
    const registerLink = screen.getByText('Create New Account')
    
    expect(registerLink).toBeInTheDocument()
  })

  it('should call onSwitchToRegister when register link is clicked', async () => {
    renderLoginForm()
    
    const registerLink = screen.getByText('Create New Account')
    
    await user.click(registerLink)
    
    expect(mockOnSwitchToRegister).toHaveBeenCalledTimes(1)
  })

  it('should render separator', () => {
    renderLoginForm()
    
    expect(screen.getByText('──────── OR ────────')).toBeInTheDocument()
  })

  it('should render copyright notice', () => {
    renderLoginForm()
    
    expect(screen.getByText('© 2025 DAA Technologies • Quantum-Secured Platform')).toBeInTheDocument()
  })

  it('should show loading spinner during authentication', async () => {
    vi.useFakeTimers()
    renderLoginForm()
    
    const emailInput = screen.getByPlaceholderText('demo@daa.dark')
    const passwordInput = screen.getByPlaceholderText('••••••••')
    const loginButton = screen.getByRole('button', { name: /ACCESS SYSTEM/i })
    
    await user.type(emailInput, 'test@example.com')
    await user.type(passwordInput, 'password123')
    
    fireEvent.click(loginButton)
    
    // Check for spinner
    const spinner = screen.getByRole('generic').querySelector('.animate-spin')
    expect(spinner).toBeInTheDocument()
    
    vi.useRealTimers()
  })

  it('should maintain button width during loading state', async () => {
    vi.useFakeTimers()
    renderLoginForm()
    
    const loginButton = screen.getByRole('button', { name: /ACCESS SYSTEM/i })
    const initialClasses = loginButton.className
    
    const emailInput = screen.getByPlaceholderText('demo@daa.dark')
    const passwordInput = screen.getByPlaceholderText('••••••••')
    
    await user.type(emailInput, 'test@example.com')
    await user.type(passwordInput, 'password123')
    
    fireEvent.click(loginButton)
    
    // Button should still have w-full class
    expect(loginButton).toHaveClass('w-full')
    
    vi.useRealTimers()
  })

  it('should have proper styling for dark theme', () => {
    renderLoginForm()
    
    // Check background colors
    const container = screen.getByRole('generic').querySelector('.bg-gray-900\\/50')
    expect(container).toBeInTheDocument()
    
    // Check text colors
    const title = screen.getByText('DAA Global Command')
    expect(title).toHaveClass('text-green-400')
    
    // Check input styling
    const emailInput = screen.getByPlaceholderText('demo@daa.dark')
    expect(emailInput).toHaveClass('bg-black/50', 'border-green-500/30', 'text-green-400')
  })

  it('should have proper focus states', async () => {
    renderLoginForm()
    
    const emailInput = screen.getByPlaceholderText('demo@daa.dark')
    const passwordInput = screen.getByPlaceholderText('••••••••')
    
    await user.click(emailInput)
    expect(emailInput).toHaveClass('focus:border-green-500')
    
    await user.click(passwordInput)
    expect(passwordInput).toHaveClass('focus:border-green-500')
  })
})