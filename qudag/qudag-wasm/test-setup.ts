import { expect, beforeAll, afterAll, beforeEach, afterEach } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { webcrypto } from 'node:crypto';

// Polyfill crypto for Node.js environment
if (typeof globalThis.crypto === 'undefined') {
  globalThis.crypto = webcrypto as Crypto;
}

// Load WASM module once for all tests
let wasmModule: WebAssembly.Module;

beforeAll(async () => {
  console.log('🚀 Initializing QuDAG WASM test environment...');
  
  // Initialize WASM module (will be built before tests)
  try {
    const wasmPath = resolve(__dirname, './pkg/qudag_wasm_bg.wasm');
    const wasmBytes = readFileSync(wasmPath);
    wasmModule = await WebAssembly.compile(wasmBytes);
  } catch (error) {
    console.warn('WASM module not found. Some tests may be skipped.');
  }
  
  // Set up global test utilities
  globalThis.testUtils = {
    generateRandomBytes: (length: number) => {
      const array = new Uint8Array(length);
      crypto.getRandomValues(array);
      return array;
    },
    
    measureTime: async <T>(fn: () => Promise<T>): Promise<[T, number]> => {
      const start = performance.now();
      const result = await fn();
      const duration = performance.now() - start;
      return [result, duration];
    },
    
    waitForCondition: async (
      condition: () => boolean | Promise<boolean>,
      timeout = 5000,
      interval = 100
    ): Promise<void> => {
      const start = Date.now();
      while (Date.now() - start < timeout) {
        if (await condition()) return;
        await new Promise(resolve => setTimeout(resolve, interval));
      }
      throw new Error('Timeout waiting for condition');
    }
  };
});

afterAll(() => {
  console.log('✅ QuDAG WASM test environment cleanup complete');
});

// Reset test state between tests
beforeEach(() => {
  // Clear any test-specific state
  globalThis.testState = {
    dagInstances: new Set(),
    cryptoKeys: new Map(),
    activeConnections: new Set()
  };
});

afterEach(() => {
  // Cleanup any created resources
  const state = globalThis.testState;
  if (state) {
    // Dispose DAG instances
    for (const dag of state.dagInstances) {
      if (dag && typeof dag.dispose === 'function') {
        dag.dispose();
      }
    }
    
    // Clear crypto keys
    state.cryptoKeys.clear();
    
    // Close connections
    for (const conn of state.activeConnections) {
      if (conn && typeof conn.close === 'function') {
        conn.close();
      }
    }
  }
});

// Custom matchers for QuDAG testing
expect.extend({
  toBeValidVertexId(received: any) {
    const pass = typeof received === 'string' && 
                 /^[0-9a-f]{64}$/.test(received);
    
    return {
      pass,
      message: () => pass
        ? `expected ${received} not to be a valid vertex ID`
        : `expected ${received} to be a valid vertex ID (64 hex chars)`
    };
  },
  
  toBeWithinTolerance(received: number, expected: number, tolerance: number) {
    const diff = Math.abs(received - expected);
    const pass = diff <= tolerance;
    
    return {
      pass,
      message: () => pass
        ? `expected ${received} not to be within ${tolerance} of ${expected}`
        : `expected ${received} to be within ${tolerance} of ${expected} (diff: ${diff})`
    };
  },
  
  toBeConstantTime(fn: Function, iterations = 1000) {
    const times: number[] = [];
    
    for (let i = 0; i < iterations; i++) {
      const start = performance.now();
      fn();
      times.push(performance.now() - start);
    }
    
    // Calculate coefficient of variation
    const mean = times.reduce((a, b) => a + b) / times.length;
    const variance = times.reduce((a, b) => a + Math.pow(b - mean, 2), 0) / times.length;
    const stdDev = Math.sqrt(variance);
    const cv = stdDev / mean;
    
    // Constant time operations should have CV < 0.1 (10%)
    const pass = cv < 0.1;
    
    return {
      pass,
      message: () => pass
        ? `expected function not to be constant time (CV: ${cv.toFixed(3)})`
        : `expected function to be constant time (CV: ${cv.toFixed(3)} > 0.1)`
    };
  }
});

// Type declarations for custom matchers
declare global {
  namespace Vi {
    interface Assertion {
      toBeValidVertexId(): void;
      toBeWithinTolerance(expected: number, tolerance: number): void;
      toBeConstantTime(iterations?: number): void;
    }
  }
  
  var testUtils: {
    generateRandomBytes(length: number): Uint8Array;
    measureTime<T>(fn: () => Promise<T>): Promise<[T, number]>;
    waitForCondition(
      condition: () => boolean | Promise<boolean>,
      timeout?: number,
      interval?: number
    ): Promise<void>;
  };
  
  var testState: {
    dagInstances: Set<any>;
    cryptoKeys: Map<string, any>;
    activeConnections: Set<any>;
  };
}