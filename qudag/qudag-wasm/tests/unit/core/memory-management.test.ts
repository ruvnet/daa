import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import type { MemoryManager, Buffer, MemoryStats } from '@/types';

describe('Memory Management', () => {
  let memoryManager: MemoryManager;
  let allocatedBuffers: Buffer[] = [];
  
  beforeEach(async () => {
    const { createMemoryManager } = await import('@/core/memory');
    memoryManager = createMemoryManager({
      initialPages: 16,
      maxPages: 256,
      enableGrowth: true
    });
  });
  
  afterEach(() => {
    // Clean up all allocated buffers
    allocatedBuffers.forEach(buffer => {
      if (!buffer.isFreed()) {
        memoryManager.free(buffer);
      }
    });
    allocatedBuffers = [];
    memoryManager.destroy();
  });
  
  describe('Buffer Allocation', () => {
    it('should allocate buffer of requested size', () => {
      const size = 1024;
      const buffer = memoryManager.allocate(size);
      allocatedBuffers.push(buffer);
      
      expect(buffer).toBeDefined();
      expect(buffer.size).toBe(size);
      expect(buffer.ptr).toBeGreaterThan(0);
      expect(buffer.isFreed()).toBe(false);
    });
    
    it('should allocate aligned buffers', () => {
      const buffer16 = memoryManager.allocateAligned(1024, 16);
      const buffer32 = memoryManager.allocateAligned(2048, 32);
      const buffer64 = memoryManager.allocateAligned(4096, 64);
      
      allocatedBuffers.push(buffer16, buffer32, buffer64);
      
      expect(buffer16.ptr % 16).toBe(0);
      expect(buffer32.ptr % 32).toBe(0);
      expect(buffer64.ptr % 64).toBe(0);
    });
    
    it('should handle zero-size allocations', () => {
      const buffer = memoryManager.allocate(0);
      allocatedBuffers.push(buffer);
      
      expect(buffer.size).toBe(0);
      expect(buffer.ptr).toBe(0);
      expect(buffer.isNull()).toBe(true);
    });
    
    it('should throw on oversized allocations', () => {
      const maxSize = memoryManager.getMaxAllocationSize();
      
      expect(() => {
        memoryManager.allocate(maxSize + 1);
      }).toThrow('Allocation size exceeds maximum');
    });
  });
  
  describe('Memory Pooling', () => {
    it('should reuse freed buffers of same size', () => {
      const size = 1024;
      const buffer1 = memoryManager.allocate(size);
      const ptr1 = buffer1.ptr;
      
      memoryManager.free(buffer1);
      
      const buffer2 = memoryManager.allocate(size);
      allocatedBuffers.push(buffer2);
      
      // Should reuse the same memory location
      expect(buffer2.ptr).toBe(ptr1);
    });
    
    it('should maintain separate pools for different sizes', () => {
      const small = memoryManager.allocate(256);
      const medium = memoryManager.allocate(1024);
      const large = memoryManager.allocate(4096);
      
      memoryManager.free(small);
      memoryManager.free(medium);
      memoryManager.free(large);
      
      const stats = memoryManager.getPoolStats();
      expect(stats.pools).toHaveLength(3);
      expect(stats.pools[0].size).toBe(256);
      expect(stats.pools[1].size).toBe(1024);
      expect(stats.pools[2].size).toBe(4096);
    });
    
    it('should limit pool size to prevent memory bloat', () => {
      const buffers = [];
      for (let i = 0; i < 100; i++) {
        buffers.push(memoryManager.allocate(1024));
      }
      
      // Free all buffers
      buffers.forEach(buf => memoryManager.free(buf));
      
      const stats = memoryManager.getPoolStats();
      const pool1024 = stats.pools.find(p => p.size === 1024);
      
      // Pool should have a maximum size limit
      expect(pool1024.count).toBeLessThanOrEqual(10);
    });
  });
  
  describe('Arena Allocation', () => {
    it('should support arena allocation for bulk operations', () => {
      const arena = memoryManager.createArena(1024 * 1024); // 1MB arena
      
      const allocs = [];
      for (let i = 0; i < 100; i++) {
        allocs.push(arena.allocate(1024));
      }
      
      expect(allocs).toHaveLength(100);
      expect(arena.getUsed()).toBe(100 * 1024);
      
      // Reset arena frees all allocations at once
      arena.reset();
      expect(arena.getUsed()).toBe(0);
      
      arena.destroy();
    });
    
    it('should grow arena when needed', () => {
      const arena = memoryManager.createArena(1024, { growable: true });
      
      // Allocate more than initial size
      const large = arena.allocate(2048);
      expect(large).toBeDefined();
      expect(arena.getCapacity()).toBeGreaterThanOrEqual(2048);
      
      arena.destroy();
    });
    
    it('should support nested arenas', () => {
      const parent = memoryManager.createArena(10240);
      const child = parent.createSubArena(1024);
      
      const parentAlloc = parent.allocate(512);
      const childAlloc = child.allocate(256);
      
      expect(parentAlloc).toBeDefined();
      expect(childAlloc).toBeDefined();
      
      // Resetting child doesn't affect parent
      child.reset();
      expect(parent.getUsed()).toBe(512 + 1024); // Parent alloc + child arena
      
      parent.destroy();
    });
  });
  
  describe('Memory Statistics', () => {
    it('should track allocation statistics', () => {
      const stats1 = memoryManager.getStats();
      
      const buffers = [
        memoryManager.allocate(1024),
        memoryManager.allocate(2048),
        memoryManager.allocate(4096)
      ];
      allocatedBuffers.push(...buffers);
      
      const stats2 = memoryManager.getStats();
      
      expect(stats2.totalAllocations).toBe(stats1.totalAllocations + 3);
      expect(stats2.currentlyAllocated).toBe(stats1.currentlyAllocated + 7168);
      expect(stats2.peakAllocated).toBeGreaterThanOrEqual(stats2.currentlyAllocated);
    });
    
    it('should track fragmentation', () => {
      // Create fragmentation by alternating allocations and frees
      const buffers = [];
      for (let i = 0; i < 20; i++) {
        buffers.push(memoryManager.allocate(1024));
      }
      
      // Free every other buffer
      for (let i = 0; i < 20; i += 2) {
        memoryManager.free(buffers[i]);
      }
      
      const stats = memoryManager.getStats();
      expect(stats.fragmentation).toBeGreaterThan(0);
      expect(stats.fragmentation).toBeLessThan(1);
    });
    
    it('should provide detailed memory map', () => {
      const buffer1 = memoryManager.allocate(1024);
      const buffer2 = memoryManager.allocate(2048);
      allocatedBuffers.push(buffer1, buffer2);
      
      const memoryMap = memoryManager.getMemoryMap();
      
      expect(memoryMap.regions).toContainEqual(
        expect.objectContaining({
          start: buffer1.ptr,
          size: 1024,
          type: 'allocated'
        })
      );
      
      expect(memoryMap.regions).toContainEqual(
        expect.objectContaining({
          start: buffer2.ptr,
          size: 2048,
          type: 'allocated'
        })
      );
    });
  });
  
  describe('Memory Safety', () => {
    it('should prevent use-after-free', () => {
      const buffer = memoryManager.allocate(1024);
      memoryManager.free(buffer);
      
      expect(() => {
        buffer.write(new Uint8Array(10));
      }).toThrow('Buffer has been freed');
      
      expect(() => {
        buffer.read();
      }).toThrow('Buffer has been freed');
    });
    
    it('should detect double-free', () => {
      const buffer = memoryManager.allocate(1024);
      memoryManager.free(buffer);
      
      expect(() => {
        memoryManager.free(buffer);
      }).toThrow('Double free detected');
    });
    
    it('should validate buffer bounds', () => {
      const buffer = memoryManager.allocate(1024);
      allocatedBuffers.push(buffer);
      
      expect(() => {
        buffer.writeAt(1024, new Uint8Array(1)); // Write at boundary
      }).toThrow('Write out of bounds');
      
      expect(() => {
        buffer.readAt(1020, 10); // Read past boundary
      }).toThrow('Read out of bounds');
    });
    
    it('should zero memory on allocation when requested', () => {
      const buffer = memoryManager.allocateZeroed(1024);
      allocatedBuffers.push(buffer);
      
      const data = buffer.read();
      expect(data.every(byte => byte === 0)).toBe(true);
    });
  });
  
  describe('Memory Pressure Handling', () => {
    it('should trigger callbacks on memory pressure', async () => {
      const lowPressure = vi.fn();
      const highPressure = vi.fn();
      
      memoryManager.onLowMemory(lowPressure);
      memoryManager.onHighMemory(highPressure);
      
      // Allocate until pressure
      const buffers = [];
      try {
        while (true) {
          buffers.push(memoryManager.allocate(1024 * 1024)); // 1MB chunks
        }
      } catch (e) {
        // Expected to fail eventually
      }
      
      expect(lowPressure).toHaveBeenCalled();
      expect(highPressure).toHaveBeenCalled();
      
      // Cleanup
      buffers.forEach(buf => {
        try { memoryManager.free(buf); } catch {}
      });
    });
    
    it('should attempt garbage collection on pressure', () => {
      const gcSpy = vi.spyOn(memoryManager, 'gc');
      
      // Fill memory
      const buffers = [];
      try {
        while (true) {
          buffers.push(memoryManager.allocate(1024 * 1024));
        }
      } catch {}
      
      expect(gcSpy).toHaveBeenCalled();
      
      // Cleanup
      buffers.forEach(buf => {
        try { memoryManager.free(buf); } catch {}
      });
    });
    
    it('should compact memory when fragmented', () => {
      // Create fragmentation
      const buffers = [];
      for (let i = 0; i < 100; i++) {
        buffers.push(memoryManager.allocate(1024));
      }
      
      // Free every other buffer
      for (let i = 0; i < 100; i += 2) {
        memoryManager.free(buffers[i]);
      }
      
      const beforeStats = memoryManager.getStats();
      memoryManager.compact();
      const afterStats = memoryManager.getStats();
      
      expect(afterStats.fragmentation).toBeLessThan(beforeStats.fragmentation);
      
      // Cleanup remaining buffers
      for (let i = 1; i < 100; i += 2) {
        memoryManager.free(buffers[i]);
      }
    });
  });
  
  describe('Buffer Operations', () => {
    it('should support buffer slicing', () => {
      const buffer = memoryManager.allocate(1024);
      allocatedBuffers.push(buffer);
      
      const data = new Uint8Array(1024);
      for (let i = 0; i < 1024; i++) {
        data[i] = i % 256;
      }
      buffer.write(data);
      
      const slice = buffer.slice(100, 200);
      expect(slice.size).toBe(100);
      
      const sliceData = slice.read();
      for (let i = 0; i < 100; i++) {
        expect(sliceData[i]).toBe((i + 100) % 256);
      }
    });
    
    it('should support buffer copying', () => {
      const src = memoryManager.allocate(1024);
      const dst = memoryManager.allocate(1024);
      allocatedBuffers.push(src, dst);
      
      const data = testUtils.generateRandomBytes(1024);
      src.write(data);
      
      memoryManager.copy(src, dst);
      
      const copied = dst.read();
      expect(copied).toEqual(data);
    });
    
    it('should support buffer comparison', () => {
      const buffer1 = memoryManager.allocate(1024);
      const buffer2 = memoryManager.allocate(1024);
      allocatedBuffers.push(buffer1, buffer2);
      
      const data = testUtils.generateRandomBytes(1024);
      buffer1.write(data);
      buffer2.write(data);
      
      expect(memoryManager.compare(buffer1, buffer2)).toBe(0);
      
      // Modify one byte
      const modified = new Uint8Array(data);
      modified[0] = (modified[0] + 1) % 256;
      buffer2.write(modified);
      
      expect(memoryManager.compare(buffer1, buffer2)).not.toBe(0);
    });
  });
});