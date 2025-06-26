import { describe, it, expect, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useToast } from '../use-toast'

describe('useToast', () => {
  it('should return toast function and toasts array', () => {
    const { result } = renderHook(() => useToast())
    
    expect(result.current).toHaveProperty('toast')
    expect(result.current).toHaveProperty('toasts')
    expect(result.current).toHaveProperty('dismiss')
    expect(typeof result.current.toast).toBe('function')
    expect(Array.isArray(result.current.toasts)).toBe(true)
    expect(result.current.toasts).toHaveLength(0)
  })

  it('should add a toast when toast function is called', () => {
    const { result } = renderHook(() => useToast())
    
    act(() => {
      result.current.toast({
        title: 'Test Toast',
        description: 'This is a test toast message',
      })
    })
    
    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0]).toMatchObject({
      title: 'Test Toast',
      description: 'This is a test toast message',
    })
  })

  it('should add multiple toasts', () => {
    const { result } = renderHook(() => useToast())
    
    act(() => {
      result.current.toast({ title: 'Toast 1' })
      result.current.toast({ title: 'Toast 2' })
      result.current.toast({ title: 'Toast 3' })
    })
    
    expect(result.current.toasts).toHaveLength(3)
    expect(result.current.toasts[0].title).toBe('Toast 1')
    expect(result.current.toasts[1].title).toBe('Toast 2')
    expect(result.current.toasts[2].title).toBe('Toast 3')
  })

  it('should generate unique IDs for toasts', () => {
    const { result } = renderHook(() => useToast())
    
    act(() => {
      result.current.toast({ title: 'Toast 1' })
      result.current.toast({ title: 'Toast 2' })
    })
    
    const ids = result.current.toasts.map(toast => toast.id)
    expect(new Set(ids).size).toBe(ids.length) // All IDs should be unique
  })

  it('should dismiss a specific toast', () => {
    const { result } = renderHook(() => useToast())
    
    let toastId: string
    
    act(() => {
      const { id } = result.current.toast({ title: 'Toast to dismiss' })
      toastId = id
      result.current.toast({ title: 'Toast to keep' })
    })
    
    expect(result.current.toasts).toHaveLength(2)
    
    act(() => {
      result.current.dismiss(toastId!)
    })
    
    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0].title).toBe('Toast to keep')
  })

  it('should dismiss all toasts when dismiss is called without ID', () => {
    const { result } = renderHook(() => useToast())
    
    act(() => {
      result.current.toast({ title: 'Toast 1' })
      result.current.toast({ title: 'Toast 2' })
      result.current.toast({ title: 'Toast 3' })
    })
    
    expect(result.current.toasts).toHaveLength(3)
    
    act(() => {
      result.current.dismiss()
    })
    
    expect(result.current.toasts).toHaveLength(0)
  })

  it('should handle different toast variants', () => {
    const { result } = renderHook(() => useToast())
    
    act(() => {
      result.current.toast({
        title: 'Default Toast',
      })
      result.current.toast({
        title: 'Destructive Toast',
        variant: 'destructive',
      })
    })
    
    expect(result.current.toasts[0].variant).toBeUndefined()
    expect(result.current.toasts[1].variant).toBe('destructive')
  })

  it('should return the created toast object', () => {
    const { result } = renderHook(() => useToast())
    
    let createdToast: any
    
    act(() => {
      createdToast = result.current.toast({
        title: 'Test Toast',
        description: 'Test description',
      })
    })
    
    expect(createdToast).toHaveProperty('id')
    expect(createdToast).toHaveProperty('title', 'Test Toast')
    expect(createdToast).toHaveProperty('description', 'Test description')
    expect(createdToast).toHaveProperty('dismiss')
    expect(typeof createdToast.dismiss).toBe('function')
  })

  it('should dismiss toast using the returned dismiss function', () => {
    const { result } = renderHook(() => useToast())
    
    let createdToast: any
    
    act(() => {
      createdToast = result.current.toast({
        title: 'Toast with dismiss',
      })
    })
    
    expect(result.current.toasts).toHaveLength(1)
    
    act(() => {
      createdToast.dismiss()
    })
    
    expect(result.current.toasts).toHaveLength(0)
  })

  it('should update existing toast with same ID', () => {
    const { result } = renderHook(() => useToast())
    
    const customId = 'custom-toast-id'
    
    act(() => {
      result.current.toast({
        id: customId,
        title: 'Original Title',
      })
    })
    
    expect(result.current.toasts[0].title).toBe('Original Title')
    
    act(() => {
      result.current.toast({
        id: customId,
        title: 'Updated Title',
      })
    })
    
    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0].title).toBe('Updated Title')
  })

  it('should preserve other properties when updating toast', () => {
    const { result } = renderHook(() => useToast())
    
    const customId = 'update-test'
    
    act(() => {
      result.current.toast({
        id: customId,
        title: 'Original',
        description: 'Original description',
        variant: 'default',
      })
    })
    
    act(() => {
      result.current.toast({
        id: customId,
        title: 'Updated',
      })
    })
    
    expect(result.current.toasts[0]).toMatchObject({
      id: customId,
      title: 'Updated',
      description: 'Original description',
      variant: 'default',
    })
  })

  it('should handle action in toast', () => {
    const { result } = renderHook(() => useToast())
    
    const actionHandler = vi.fn()
    
    act(() => {
      result.current.toast({
        title: 'Toast with action',
        action: {
          label: 'Undo',
          onClick: actionHandler,
        },
      })
    })
    
    expect(result.current.toasts[0].action).toBeDefined()
    expect(result.current.toasts[0].action?.label).toBe('Undo')
    
    // Simulate clicking the action
    result.current.toasts[0].action?.onClick?.()
    expect(actionHandler).toHaveBeenCalledTimes(1)
  })

  it('should limit number of toasts to TOAST_LIMIT', () => {
    const { result } = renderHook(() => useToast())
    
    // Assuming TOAST_LIMIT is 3 (you may need to adjust based on actual implementation)
    const TOAST_LIMIT = 3
    
    act(() => {
      for (let i = 0; i < TOAST_LIMIT + 2; i++) {
        result.current.toast({ title: `Toast ${i}` })
      }
    })
    
    expect(result.current.toasts.length).toBeLessThanOrEqual(TOAST_LIMIT)
  })
})