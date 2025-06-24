import { describe, it, expect, beforeEach } from 'vitest';
import type { 
  QuDAGError, 
  ErrorCode, 
  ErrorContext,
  ErrorHandler 
} from '@/types';

describe('Error Handling', () => {
  let errorHandler: ErrorHandler;
  
  beforeEach(async () => {
    const { createErrorHandler } = await import('@/core/errors');
    errorHandler = createErrorHandler();
  });
  
  describe('Error Creation', () => {
    it('should create typed errors with proper structure', () => {
      const error = errorHandler.create({
        code: 'INVALID_VERTEX',
        message: 'Vertex validation failed',
        details: {
          vertexId: '123',
          reason: 'Missing parents'
        }
      });
      
      expect(error).toBeInstanceOf(Error);
      expect(error.code).toBe('INVALID_VERTEX');
      expect(error.message).toBe('Vertex validation failed');
      expect(error.details).toEqual({
        vertexId: '123',
        reason: 'Missing parents'
      });
      expect(error.timestamp).toBeDefined();
      expect(error.stack).toBeDefined();
    });
    
    it('should create errors with cause chain', () => {
      const rootCause = new Error('Database connection failed');
      const midError = errorHandler.create({
        code: 'STORAGE_ERROR',
        message: 'Failed to persist vertex',
        cause: rootCause
      });
      
      const topError = errorHandler.create({
        code: 'DAG_ERROR',
        message: 'Failed to add vertex to DAG',
        cause: midError
      });
      
      expect(topError.cause).toBe(midError);
      expect(midError.cause).toBe(rootCause);
      
      const chain = errorHandler.getCauseChain(topError);
      expect(chain).toHaveLength(3);
      expect(chain[0]).toBe(topError);
      expect(chain[1]).toBe(midError);
      expect(chain[2]).toBe(rootCause);
    });
    
    it('should include context information', () => {
      const error = errorHandler.create({
        code: 'CRYPTO_ERROR',
        message: 'Encryption failed',
        context: {
          algorithm: 'ML-KEM-768',
          keySize: 768,
          operation: 'encrypt'
        }
      });
      
      expect(error.context).toBeDefined();
      expect(error.context.algorithm).toBe('ML-KEM-768');
      expect(error.context.keySize).toBe(768);
      expect(error.context.operation).toBe('encrypt');
    });
  });
  
  describe('Error Codes', () => {
    it('should validate error codes', () => {
      expect(() => {
        errorHandler.create({
          code: 'INVALID_CODE_123' as ErrorCode,
          message: 'Test'
        });
      }).toThrow('Invalid error code');
    });
    
    it('should provide error code descriptions', () => {
      const description = errorHandler.getCodeDescription('INVALID_VERTEX');
      expect(description).toContain('vertex');
      expect(description).toContain('validation');
    });
    
    it('should categorize error codes', () => {
      expect(errorHandler.getCodeCategory('INVALID_VERTEX')).toBe('validation');
      expect(errorHandler.getCodeCategory('CRYPTO_ERROR')).toBe('crypto');
      expect(errorHandler.getCodeCategory('NETWORK_ERROR')).toBe('network');
      expect(errorHandler.getCodeCategory('STORAGE_ERROR')).toBe('storage');
    });
  });
  
  describe('Error Recovery', () => {
    it('should suggest recovery actions', () => {
      const error = errorHandler.create({
        code: 'NETWORK_ERROR',
        message: 'Connection timeout',
        details: { timeout: 5000 }
      });
      
      const recovery = errorHandler.getRecoveryActions(error);
      expect(recovery).toContain('retry');
      expect(recovery).toContain('check_connectivity');
      expect(recovery).toContain('increase_timeout');
    });
    
    it('should provide retry strategies', () => {
      const error = errorHandler.create({
        code: 'TEMPORARY_ERROR',
        message: 'Resource temporarily unavailable'
      });
      
      const strategy = errorHandler.getRetryStrategy(error);
      expect(strategy.shouldRetry).toBe(true);
      expect(strategy.maxAttempts).toBe(3);
      expect(strategy.backoff).toBe('exponential');
      expect(strategy.initialDelay).toBe(100);
    });
    
    it('should identify non-recoverable errors', () => {
      const error = errorHandler.create({
        code: 'INVALID_CONFIGURATION',
        message: 'Invalid cryptographic parameters'
      });
      
      const strategy = errorHandler.getRetryStrategy(error);
      expect(strategy.shouldRetry).toBe(false);
      expect(strategy.reason).toContain('configuration');
    });
  });
  
  describe('Error Transformation', () => {
    it('should transform WASM errors to JavaScript errors', () => {
      // Simulate WASM error (number code)
      const wasmError = { code: 42, message: 'WASM panic' };
      
      const jsError = errorHandler.fromWasm(wasmError);
      expect(jsError).toBeInstanceOf(Error);
      expect(jsError.code).toBe('WASM_ERROR');
      expect(jsError.details.wasmCode).toBe(42);
      expect(jsError.message).toContain('WASM panic');
    });
    
    it('should serialize errors for cross-boundary communication', () => {
      const error = errorHandler.create({
        code: 'DAG_ERROR',
        message: 'Test error',
        details: { foo: 'bar' },
        context: { baz: 'qux' }
      });
      
      const serialized = errorHandler.serialize(error);
      expect(typeof serialized).toBe('string');
      
      const deserialized = errorHandler.deserialize(serialized);
      expect(deserialized.code).toBe(error.code);
      expect(deserialized.message).toBe(error.message);
      expect(deserialized.details).toEqual(error.details);
      expect(deserialized.context).toEqual(error.context);
    });
    
    it('should handle circular references in serialization', () => {
      const obj: any = { a: 1 };
      obj.circular = obj;
      
      const error = errorHandler.create({
        code: 'TEST_ERROR',
        message: 'Circular reference',
        details: obj
      });
      
      expect(() => {
        errorHandler.serialize(error);
      }).not.toThrow();
    });
  });
  
  describe('Error Aggregation', () => {
    it('should aggregate multiple errors', () => {
      const errors = [
        errorHandler.create({ code: 'ERROR_1', message: 'First error' }),
        errorHandler.create({ code: 'ERROR_2', message: 'Second error' }),
        errorHandler.create({ code: 'ERROR_3', message: 'Third error' })
      ];
      
      const aggregated = errorHandler.aggregate(errors, 'Multiple failures occurred');
      
      expect(aggregated.code).toBe('AGGREGATE_ERROR');
      expect(aggregated.message).toBe('Multiple failures occurred');
      expect(aggregated.errors).toHaveLength(3);
      expect(aggregated.errors[0]).toBe(errors[0]);
    });
    
    it('should flatten nested aggregate errors', () => {
      const error1 = errorHandler.create({ code: 'ERROR_1', message: 'Error 1' });
      const error2 = errorHandler.create({ code: 'ERROR_2', message: 'Error 2' });
      const aggregate1 = errorHandler.aggregate([error1, error2], 'Group 1');
      
      const error3 = errorHandler.create({ code: 'ERROR_3', message: 'Error 3' });
      const aggregate2 = errorHandler.aggregate([aggregate1, error3], 'Group 2');
      
      expect(aggregate2.errors).toHaveLength(3);
      expect(aggregate2.errors).toContain(error1);
      expect(aggregate2.errors).toContain(error2);
      expect(aggregate2.errors).toContain(error3);
    });
  });
  
  describe('Error Handlers and Listeners', () => {
    it('should register global error handlers', () => {
      const handler = vi.fn();
      errorHandler.onError(handler);
      
      const error = errorHandler.create({
        code: 'TEST_ERROR',
        message: 'Test'
      });
      
      errorHandler.emit(error);
      
      expect(handler).toHaveBeenCalledWith(error);
    });
    
    it('should support error filters', () => {
      const cryptoHandler = vi.fn();
      const networkHandler = vi.fn();
      
      errorHandler.onError(cryptoHandler, { 
        filter: (err) => err.code.startsWith('CRYPTO_') 
      });
      
      errorHandler.onError(networkHandler, { 
        filter: (err) => err.code.startsWith('NETWORK_') 
      });
      
      const cryptoError = errorHandler.create({
        code: 'CRYPTO_ERROR',
        message: 'Crypto failure'
      });
      
      const networkError = errorHandler.create({
        code: 'NETWORK_ERROR',
        message: 'Network failure'
      });
      
      errorHandler.emit(cryptoError);
      errorHandler.emit(networkError);
      
      expect(cryptoHandler).toHaveBeenCalledWith(cryptoError);
      expect(cryptoHandler).not.toHaveBeenCalledWith(networkError);
      expect(networkHandler).toHaveBeenCalledWith(networkError);
      expect(networkHandler).not.toHaveBeenCalledWith(cryptoError);
    });
    
    it('should handle errors in error handlers', () => {
      const badHandler = vi.fn(() => {
        throw new Error('Handler error');
      });
      
      const fallbackHandler = vi.fn();
      errorHandler.onHandlerError(fallbackHandler);
      
      errorHandler.onError(badHandler);
      
      const error = errorHandler.create({
        code: 'TEST_ERROR',
        message: 'Test'
      });
      
      errorHandler.emit(error);
      
      expect(badHandler).toHaveBeenCalled();
      expect(fallbackHandler).toHaveBeenCalled();
    });
  });
  
  describe('Error Reporting', () => {
    it('should generate error reports', () => {
      const error = errorHandler.create({
        code: 'COMPLEX_ERROR',
        message: 'Complex failure',
        details: {
          component: 'DAG',
          operation: 'consensus',
          vertexCount: 1000
        },
        context: {
          user: 'test-user',
          timestamp: Date.now()
        }
      });
      
      const report = errorHandler.generateReport(error);
      
      expect(report).toContain('COMPLEX_ERROR');
      expect(report).toContain('Complex failure');
      expect(report).toContain('DAG');
      expect(report).toContain('consensus');
      expect(report).toContain('test-user');
      expect(report).toContain('Stack trace:');
    });
    
    it('should collect error metrics', () => {
      // Generate various errors
      for (let i = 0; i < 10; i++) {
        errorHandler.emit(errorHandler.create({
          code: 'CRYPTO_ERROR',
          message: 'Crypto failure'
        }));
      }
      
      for (let i = 0; i < 5; i++) {
        errorHandler.emit(errorHandler.create({
          code: 'NETWORK_ERROR',
          message: 'Network failure'
        }));
      }
      
      const metrics = errorHandler.getMetrics();
      
      expect(metrics.total).toBe(15);
      expect(metrics.byCode['CRYPTO_ERROR']).toBe(10);
      expect(metrics.byCode['NETWORK_ERROR']).toBe(5);
      expect(metrics.byCategory['crypto']).toBe(10);
      expect(metrics.byCategory['network']).toBe(5);
    });
    
    it('should track error rates over time', async () => {
      const startTime = Date.now();
      
      // Generate errors over time
      for (let i = 0; i < 20; i++) {
        errorHandler.emit(errorHandler.create({
          code: 'TEST_ERROR',
          message: 'Test'
        }));
        await new Promise(resolve => setTimeout(resolve, 50));
      }
      
      const rates = errorHandler.getErrorRates();
      
      expect(rates.perSecond).toBeGreaterThan(0);
      expect(rates.perMinute).toBeGreaterThan(0);
      expect(rates.trend).toBeDefined();
    });
  });
  
  describe('Error Boundaries', () => {
    it('should catch and handle errors in async operations', async () => {
      const operation = async () => {
        throw new Error('Async operation failed');
      };
      
      const result = await errorHandler.tryAsync(operation, {
        fallback: 'default-value'
      });
      
      expect(result).toBe('default-value');
    });
    
    it('should propagate errors when no fallback provided', async () => {
      const operation = async () => {
        throw new Error('Async operation failed');
      };
      
      await expect(errorHandler.tryAsync(operation)).rejects.toThrow('Async operation failed');
    });
    
    it('should transform errors in boundaries', async () => {
      const operation = async () => {
        throw new Error('Raw error');
      };
      
      try {
        await errorHandler.tryAsync(operation, {
          transform: (err) => errorHandler.create({
            code: 'WRAPPED_ERROR',
            message: 'Operation failed',
            cause: err
          })
        });
      } catch (error) {
        expect(error.code).toBe('WRAPPED_ERROR');
        expect(error.cause.message).toBe('Raw error');
      }
    });
  });
});