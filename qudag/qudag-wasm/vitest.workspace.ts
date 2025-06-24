import { defineWorkspace } from 'vitest/config';

export default defineWorkspace([
  // Unit tests - Node environment
  {
    extends: './vitest.config.ts',
    test: {
      name: 'unit',
      include: ['tests/unit/**/*.test.ts'],
      environment: 'node'
    }
  },
  
  // Integration tests - Node with WASM
  {
    extends: './vitest.config.ts',
    test: {
      name: 'integration',
      include: ['tests/integration/**/*.test.ts'],
      environment: 'node',
      poolOptions: {
        threads: {
          // Single thread for WASM shared memory
          singleThread: true
        }
      }
    }
  },
  
  // Browser tests - Happy-DOM environment
  {
    extends: './vitest.config.ts',
    test: {
      name: 'browser',
      include: ['tests/e2e/browser/**/*.test.ts'],
      environment: 'happy-dom',
      browser: {
        enabled: true,
        name: 'chromium',
        provider: 'playwright',
        headless: true
      }
    }
  },
  
  // Performance benchmarks
  {
    extends: './vitest.config.ts',
    test: {
      name: 'performance',
      include: ['tests/performance/**/*.bench.ts'],
      benchmark: {
        include: ['tests/performance/**/*.bench.ts'],
        outputFile: './test-results/benchmarks.json',
        reporters: ['verbose', 'json']
      }
    }
  },
  
  // E2E CLI tests
  {
    extends: './vitest.config.ts',
    test: {
      name: 'cli',
      include: ['tests/e2e/cli/**/*.test.ts'],
      environment: 'node',
      testTimeout: 60000
    }
  }
]);