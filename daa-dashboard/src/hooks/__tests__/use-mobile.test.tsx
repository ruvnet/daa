import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useIsMobile } from '../use-mobile'

describe('useIsMobile', () => {
  let originalMatchMedia: typeof window.matchMedia

  beforeEach(() => {
    // Save original matchMedia
    originalMatchMedia = window.matchMedia
  })

  afterEach(() => {
    // Restore original matchMedia
    window.matchMedia = originalMatchMedia
  })

  const createMatchMediaMock = (matches: boolean) => {
    return vi.fn().mockImplementation((query: string) => ({
      matches,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(), // deprecated
      removeListener: vi.fn(), // deprecated
      dispatchEvent: vi.fn(),
    }))
  }

  it('should return true when screen width is less than 768px', () => {
    window.matchMedia = createMatchMediaMock(true)
    
    const { result } = renderHook(() => useIsMobile())
    
    expect(result.current).toBe(true)
  })

  it('should return false when screen width is 768px or more', () => {
    window.matchMedia = createMatchMediaMock(false)
    
    const { result } = renderHook(() => useIsMobile())
    
    expect(result.current).toBe(false)
  })

  it('should update when media query changes', () => {
    let mediaQueryListeners: Array<(e: any) => void> = []
    
    const mockMatchMedia = vi.fn().mockImplementation((query: string) => {
      const mediaQueryList = {
        matches: false,
        media: query,
        onchange: null,
        addEventListener: vi.fn((event: string, listener: (e: any) => void) => {
          if (event === 'change') {
            mediaQueryListeners.push(listener)
          }
        }),
        removeEventListener: vi.fn((event: string, listener: (e: any) => void) => {
          if (event === 'change') {
            mediaQueryListeners = mediaQueryListeners.filter(l => l !== listener)
          }
        }),
        addListener: vi.fn(),
        removeListener: vi.fn(),
        dispatchEvent: vi.fn(),
      }
      return mediaQueryList
    })
    
    window.matchMedia = mockMatchMedia
    
    const { result } = renderHook(() => useIsMobile())
    
    // Initially false (desktop)
    expect(result.current).toBe(false)
    
    // Simulate media query change to mobile
    act(() => {
      mediaQueryListeners.forEach(listener => {
        listener({ matches: true, media: '(max-width: 767px)' })
      })
    })
    
    expect(result.current).toBe(true)
    
    // Simulate media query change back to desktop
    act(() => {
      mediaQueryListeners.forEach(listener => {
        listener({ matches: false, media: '(max-width: 767px)' })
      })
    })
    
    expect(result.current).toBe(false)
  })

  it('should clean up event listeners on unmount', () => {
    const removeEventListenerSpy = vi.fn()
    
    const mockMatchMedia = vi.fn().mockImplementation(() => ({
      matches: false,
      media: '(max-width: 767px)',
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: removeEventListenerSpy,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))
    
    window.matchMedia = mockMatchMedia
    
    const { unmount } = renderHook(() => useIsMobile())
    
    unmount()
    
    expect(removeEventListenerSpy).toHaveBeenCalledWith('change', expect.any(Function))
  })

  it('should use the correct media query', () => {
    const mockMatchMedia = createMatchMediaMock(false)
    window.matchMedia = mockMatchMedia
    
    renderHook(() => useIsMobile())
    
    expect(mockMatchMedia).toHaveBeenCalledWith('(max-width: 767px)')
  })

  it('should handle multiple hook instances independently', () => {
    window.matchMedia = createMatchMediaMock(true)
    
    const { result: result1 } = renderHook(() => useIsMobile())
    const { result: result2 } = renderHook(() => useIsMobile())
    
    expect(result1.current).toBe(true)
    expect(result2.current).toBe(true)
  })

  it('should work with SSR (no window.matchMedia)', () => {
    // Simulate SSR by removing matchMedia
    // @ts-ignore
    delete window.matchMedia
    
    // Should not throw
    const { result } = renderHook(() => useIsMobile())
    
    // Default value should be false when matchMedia is not available
    expect(result.current).toBe(false)
  })

  it('should handle matchMedia returning null', () => {
    // @ts-ignore
    window.matchMedia = vi.fn().mockReturnValue(null)
    
    // Should not throw
    const { result } = renderHook(() => useIsMobile())
    
    expect(result.current).toBe(false)
  })

  it('should update immediately when hook is first called', () => {
    let capturedListener: ((e: any) => void) | null = null
    
    const mockMatchMedia = vi.fn().mockImplementation(() => ({
      matches: true,
      media: '(max-width: 767px)',
      onchange: null,
      addEventListener: vi.fn((event, listener) => {
        if (event === 'change') {
          capturedListener = listener
        }
      }),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))
    
    window.matchMedia = mockMatchMedia
    
    const { result } = renderHook(() => useIsMobile())
    
    // Should immediately reflect the matches value
    expect(result.current).toBe(true)
    
    // Should have registered a listener
    expect(capturedListener).not.toBeNull()
  })
})