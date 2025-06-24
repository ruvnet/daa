import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import type { QuDAGModule, InitOptions } from '@/types';

describe('WASM Module Initialization', () => {
  let QuDAG: QuDAGModule;
  
  beforeAll(async () => {
    // This will fail until the module is implemented
    const { default: initQuDAG } = await import('@/index');
    QuDAG = await initQuDAG();
  });
  
  afterAll(() => {
    if (QuDAG && QuDAG.cleanup) {
      QuDAG.cleanup();
    }
  });
  
  describe('Module Loading', () => {
    it('should load WASM module successfully', () => {
      expect(QuDAG).toBeDefined();
      expect(QuDAG.version).toBeDefined();
      expect(QuDAG.features).toBeDefined();
    });
    
    it('should report correct version', () => {
      expect(QuDAG.version).toMatch(/^\d+\.\d+\.\d+$/);
    });
    
    it('should expose core features', () => {
      const features = QuDAG.features;
      expect(features).toContain('dag');
      expect(features).toContain('crypto');
      expect(features).toContain('consensus');
      expect(features).toContain('network');
    });
    
    it('should initialize with custom options', async () => {
      const options: InitOptions = {
        memory: {
          initial: 16, // 16 pages = 1MB
          maximum: 256, // 256 pages = 16MB
          shared: false
        },
        threading: {
          enabled: false,
          workers: 0
        },
        crypto: {
          provider: 'wasm',
          algorithms: ['ML-KEM-768', 'ML-DSA-65']
        }
      };
      
      const customQuDAG = await QuDAG.initWithOptions(options);
      expect(customQuDAG).toBeDefined();
      expect(customQuDAG.getMemoryUsage().maximum).toBe(256 * 64 * 1024);
    });
  });
  
  describe('Memory Management', () => {
    it('should track memory usage', () => {
      const usage = QuDAG.getMemoryUsage();
      expect(usage).toHaveProperty('used');
      expect(usage).toHaveProperty('total');
      expect(usage).toHaveProperty('peak');
      expect(usage.used).toBeGreaterThanOrEqual(0);
      expect(usage.used).toBeLessThanOrEqual(usage.total);
    });
    
    it('should grow memory when needed', async () => {
      const initialUsage = QuDAG.getMemoryUsage();
      
      // Allocate a large buffer
      const largeBuffer = QuDAG.allocateBuffer(1024 * 1024); // 1MB
      
      const afterUsage = QuDAG.getMemoryUsage();
      expect(afterUsage.used).toBeGreaterThan(initialUsage.used);
      
      // Free the buffer
      QuDAG.freeBuffer(largeBuffer);
      
      const finalUsage = QuDAG.getMemoryUsage();
      expect(finalUsage.used).toBeLessThan(afterUsage.used);
    });
    
    it('should handle memory pressure gracefully', async () => {
      const pressureHandler = vi.fn();
      QuDAG.onMemoryPressure(pressureHandler);
      
      // Allocate memory until pressure
      const buffers = [];
      try {
        for (let i = 0; i < 1000; i++) {
          buffers.push(QuDAG.allocateBuffer(1024 * 1024)); // 1MB each
        }
      } catch (error) {
        expect(error.message).toContain('memory');
      }
      
      expect(pressureHandler).toHaveBeenCalled();
      
      // Cleanup
      buffers.forEach(buf => QuDAG.freeBuffer(buf));
    });
  });
  
  describe('Error Handling', () => {
    it('should throw on invalid initialization', async () => {
      await expect(QuDAG.initWithOptions({
        memory: { initial: -1 }
      })).rejects.toThrow('Invalid memory configuration');
    });
    
    it('should handle WASM instantiation errors', async () => {
      // Force an error by providing invalid WASM bytes
      const { initFromBytes } = await import('@/index');
      const invalidBytes = new Uint8Array([0, 1, 2, 3]);
      
      await expect(initFromBytes(invalidBytes)).rejects.toThrow('Invalid WASM module');
    });
    
    it('should provide detailed error information', async () => {
      try {
        await QuDAG.performInvalidOperation();
      } catch (error) {
        expect(error).toHaveProperty('code');
        expect(error).toHaveProperty('details');
        expect(error.code).toBe('INVALID_OPERATION');
      }
    });
  });
  
  describe('Thread Pool Management', () => {
    it('should initialize thread pool', async () => {
      const threadedQuDAG = await QuDAG.initWithOptions({
        threading: {
          enabled: true,
          workers: 4
        }
      });
      
      const threadInfo = threadedQuDAG.getThreadingInfo();
      expect(threadInfo.enabled).toBe(true);
      expect(threadInfo.activeWorkers).toBe(4);
      expect(threadInfo.taskQueue).toBeDefined();
    });
    
    it('should distribute work across threads', async () => {
      const threadedQuDAG = await QuDAG.initWithOptions({
        threading: { enabled: true, workers: 4 }
      });
      
      const tasks = Array(100).fill(0).map((_, i) => ({
        id: i,
        type: 'hash',
        data: new Uint8Array(1024)
      }));
      
      const results = await threadedQuDAG.executeBatch(tasks);
      expect(results).toHaveLength(100);
      
      // Check that work was distributed
      const workerStats = threadedQuDAG.getWorkerStats();
      expect(Object.keys(workerStats).length).toBe(4);
      Object.values(workerStats).forEach(stats => {
        expect(stats.tasksCompleted).toBeGreaterThan(0);
      });
    });
  });
  
  describe('Feature Detection', () => {
    it('should detect SIMD support', () => {
      const capabilities = QuDAG.getCapabilities();
      expect(capabilities).toHaveProperty('simd');
      expect(typeof capabilities.simd).toBe('boolean');
    });
    
    it('should detect SharedArrayBuffer support', () => {
      const capabilities = QuDAG.getCapabilities();
      expect(capabilities).toHaveProperty('sharedMemory');
      expect(typeof capabilities.sharedMemory).toBe('boolean');
    });
    
    it('should detect crypto extensions', () => {
      const capabilities = QuDAG.getCapabilities();
      expect(capabilities).toHaveProperty('cryptoExtensions');
      expect(Array.isArray(capabilities.cryptoExtensions)).toBe(true);
    });
    
    it('should adapt features based on environment', async () => {
      // Simulate restricted environment
      const restrictedQuDAG = await QuDAG.initWithOptions({
        environment: 'restricted'
      });
      
      const features = restrictedQuDAG.getEnabledFeatures();
      expect(features.threading).toBe(false);
      expect(features.simd).toBe(false);
    });
  });
  
  describe('Module Lifecycle', () => {
    it('should support hot reload', async () => {
      const instance1 = await QuDAG.createInstance();
      const id1 = instance1.getId();
      
      // Simulate module update
      await QuDAG.hotReload();
      
      const instance2 = await QuDAG.createInstance();
      const id2 = instance2.getId();
      
      expect(id1).not.toBe(id2);
      expect(instance1.isValid()).toBe(false);
      expect(instance2.isValid()).toBe(true);
    });
    
    it('should cleanup resources on dispose', async () => {
      const instance = await QuDAG.createInstance();
      const beforeStats = QuDAG.getResourceStats();
      
      instance.dispose();
      
      const afterStats = QuDAG.getResourceStats();
      expect(afterStats.instances).toBe(beforeStats.instances - 1);
      expect(afterStats.memoryUsed).toBeLessThan(beforeStats.memoryUsed);
    });
    
    it('should handle multiple instances', async () => {
      const instances = await Promise.all([
        QuDAG.createInstance(),
        QuDAG.createInstance(),
        QuDAG.createInstance()
      ]);
      
      expect(instances).toHaveLength(3);
      instances.forEach((instance, i) => {
        expect(instance.getId()).toBeDefined();
        expect(instance.getId()).not.toBe(instances[(i + 1) % 3].getId());
      });
      
      // Cleanup
      instances.forEach(instance => instance.dispose());
    });
  });
});