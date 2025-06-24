import { defineConfig } from 'vitest/config';
import { resolve } from 'path';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [wasm()],
  
  test: {
    globals: true,
    environment: 'node',
    setupFiles: './test-setup.ts',
    coverage: {
      provider: 'c8',
      reporter: ['text', 'json', 'html', 'lcov'],
      exclude: [
        'node_modules/',
        'tests/',
        '**/*.d.ts',
        '**/*.test.ts',
        '**/*.spec.ts',
        '**/mocks/**',
        '**/fixtures/**'
      ],
      thresholds: {
        lines: 95,
        functions: 95,
        branches: 85,
        statements: 95
      }
    },
    testTimeout: 30000,
    hookTimeout: 30000,
    pool: 'threads',
    poolOptions: {
      threads: {
        singleThread: false,
        isolate: true
      }
    },
    reporters: ['verbose', 'html'],
    outputFile: {
      html: './test-results/index.html'
    }
  },
  
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
      '@tests': resolve(__dirname, './tests'),
      '@fixtures': resolve(__dirname, './tests/fixtures'),
      '@helpers': resolve(__dirname, './tests/helpers'),
      '@mocks': resolve(__dirname, './tests/mocks')
    }
  },
  
  build: {
    target: 'esnext',
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'QuDAG',
      formats: ['es', 'cjs']
    },
    rollupOptions: {
      external: ['node:fs', 'node:path', 'node:crypto', 'node:url'],
      output: {
        globals: {
          'node:fs': 'fs',
          'node:path': 'path',
          'node:crypto': 'crypto',
          'node:url': 'url'
        }
      }
    }
  },
  
  server: {
    headers: {
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cross-Origin-Opener-Policy': 'same-origin'
    }
  }
});