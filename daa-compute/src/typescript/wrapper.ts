// TypeScript wrapper for DAA Compute WASM module
// Provides convenience functions and better error handling

import * as wasm from '../pkg/daa_compute';
import type {
    BrowserTrainingConfig,
    InferenceConfig,
    ModelMetadata,
    TrainingResult,
    SystemCapabilities,
    BenchmarkResult,
    P2PConfig,
    GradientMessage
} from './index.d';

/**
 * Initialize the WASM module
 */
export async function init(): Promise<void> {
    await wasm.default();
    wasm.init_wasm_training();
}

/**
 * Wrapper for BrowserTrainer with enhanced error handling
 */
export class TrainerWrapper {
    private trainer: wasm.BrowserTrainer;
    
    constructor(config: BrowserTrainingConfig) {
        try {
            this.trainer = new wasm.BrowserTrainer(JSON.stringify(config));
        } catch (error) {
            throw new Error(`Failed to initialize trainer: ${error}`);
        }
    }
    
    async trainBatch(inputs: number[] | Float32Array, targets: number[] | Float32Array): Promise<TrainingResult> {
        const inputArray = inputs instanceof Float32Array ? inputs : new Float32Array(inputs);
        const targetArray = targets instanceof Float32Array ? targets : new Float32Array(targets);
        
        try {
            const resultJson = await this.trainer.train_batch(inputArray, targetArray);
            return JSON.parse(resultJson) as TrainingResult;
        } catch (error) {
            throw new Error(`Training failed: ${error}`);
        }
    }
    
    getGradients(): Uint8Array {
        try {
            return this.trader.get_gradients();
        } catch (error) {
            throw new Error(`Failed to get gradients: ${error}`);
        }
    }
    
    applyGradients(gradients: Uint8Array): void {
        try {
            this.trainer.apply_gradients(gradients);
        } catch (error) {
            throw new Error(`Failed to apply gradients: ${error}`);
        }
    }
}

/**
 * Wrapper for BrowserInference with enhanced error handling
 */
export class InferenceWrapper {
    private inference: wasm.BrowserInference;
    
    constructor(config?: InferenceConfig) {
        try {
            const configJson = config ? JSON.stringify(config) : undefined;
            this.inference = new wasm.BrowserInference(configJson);
        } catch (error) {
            throw new Error(`Failed to initialize inference engine: ${error}`);
        }
    }
    
    async loadModel(modelData: ArrayBuffer | Uint8Array, metadata: ModelMetadata): Promise<string> {
        const dataArray = modelData instanceof ArrayBuffer ? new Uint8Array(modelData) : modelData;
        
        try {
            return await this.inference.load_model(dataArray, JSON.stringify(metadata));
        } catch (error) {
            throw new Error(`Failed to load model: ${error}`);
        }
    }
    
    async predict(modelId: string, input: number[] | Float32Array): Promise<Float32Array> {
        const inputArray = input instanceof Float32Array ? input : new Float32Array(input);
        
        try {
            return await this.inference.infer(modelId, inputArray);
        } catch (error) {
            throw new Error(`Inference failed: ${error}`);
        }
    }
    
    getModelInfo(modelId: string): any {
        try {
            const infoJson = this.inference.get_model_info(modelId);
            return JSON.parse(infoJson);
        } catch (error) {
            throw new Error(`Failed to get model info: ${error}`);
        }
    }
    
    clearCache(): void {
        this.inference.clear_cache();
    }
}

/**
 * P2P Network Manager for browser environments
 */
export class P2PManager {
    private gradients: Map<string, Uint8Array> = new Map();
    private peers: Set<string> = new Set();
    
    constructor(private config: P2PConfig) {}
    
    async connect(): Promise<void> {
        // WebRTC P2P connection logic would go here
        console.log('Connecting to P2P network...');
    }
    
    async broadcastGradients(gradients: Uint8Array): Promise<void> {
        // Broadcast gradients to connected peers
        console.log(`Broadcasting ${gradients.length} bytes of gradients`);
    }
    
    onGradientsReceived(callback: (gradients: Uint8Array, peerId: string) => void): void {
        // Set up callback for receiving gradients from peers
        console.log('Setting up gradient reception callback');
    }
    
    getPeerCount(): number {
        return this.peers.size;
    }
}

/**
 * Utility functions
 */
export class Utils {
    static toFloat32Array(data: number[]): Float32Array {
        return new Float32Array(data);
    }
    
    static toUint8Array(data: number[]): Uint8Array {
        return new Uint8Array(data);
    }
    
    static parseTrainingResult(json: string): TrainingResult {
        return JSON.parse(json) as TrainingResult;
    }
    
    static parseCapabilities(json: string): SystemCapabilities {
        return JSON.parse(json) as SystemCapabilities;
    }
    
    static parseBenchmark(json: string): BenchmarkResult {
        return JSON.parse(json) as BenchmarkResult;
    }
    
    static validateTrainingConfig(config: BrowserTrainingConfig): boolean {
        return (
            typeof config.max_train_time_ms === 'number' &&
            typeof config.batch_size === 'number' &&
            typeof config.use_simd === 'boolean' &&
            typeof config.memory_limit_mb === 'number' &&
            config.max_train_time_ms > 0 &&
            config.batch_size > 0 &&
            config.memory_limit_mb > 0
        );
    }
    
    static validateInferenceConfig(config: InferenceConfig): boolean {
        return (
            typeof config.max_batch_size === 'number' &&
            typeof config.use_webgl === 'boolean' &&
            typeof config.use_webgpu === 'boolean' &&
            typeof config.cache_in_indexeddb === 'boolean' &&
            typeof config.max_inference_time_ms === 'number' &&
            config.max_batch_size > 0 &&
            config.max_inference_time_ms > 0
        );
    }
}

/**
 * Framework integrations
 */
export class Integrations {
    static tensorFlow = {
        fromTensor(tensor: any): Float32Array {
            return tensor.dataSync() as Float32Array;
        },
        
        toTensor(data: Float32Array, shape: number[]): any {
            // Assuming tf is available globally
            if (typeof window !== 'undefined' && (window as any).tf) {
                return (window as any).tf.tensor(data, shape);
            }
            throw new Error('TensorFlow.js not available');
        }
    };
    
    static onnx = {
        async convertModel(model: any): Promise<Uint8Array> {
            // Convert ONNX model to our format
            console.warn('ONNX conversion not yet implemented');
            return new Uint8Array();
        },
        
        extractMetadata(model: any): ModelMetadata {
            // Extract metadata from ONNX model
            return {
                name: model.name || 'unknown',
                version: '1.0.0',
                input_shape: [1, 224, 224, 3], // placeholder
                output_shape: [1, 1000], // placeholder
                quantization: 'None'
            };
        }
    };
}

/**
 * System information and capabilities
 */
export async function getSystemCapabilities(): Promise<SystemCapabilities> {
    try {
        const capsJson = wasm.get_browser_capabilities();
        return JSON.parse(capsJson) as SystemCapabilities;
    } catch (error) {
        throw new Error(`Failed to get system capabilities: ${error}`);
    }
}

/**
 * Performance benchmarking
 */
export async function runBenchmark(inputSize: number = 1000): Promise<BenchmarkResult> {
    try {
        const resultJson = await wasm.benchmark_inference(inputSize);
        return JSON.parse(resultJson) as BenchmarkResult;
    } catch (error) {
        throw new Error(`Benchmark failed: ${error}`);
    }
}

/**
 * Example usage and demos
 */
export class Examples {
    static async basicTraining(): Promise<void> {
        await init();
        
        const config: BrowserTrainingConfig = {
            max_train_time_ms: 100,
            batch_size: 32,
            use_simd: true,
            memory_limit_mb: 256,
        };
        
        const trainer = new TrainerWrapper(config);
        
        // Generate dummy data
        const inputs = Array.from({ length: 320 }, () => Math.random());
        const targets = Array.from({ length: 128 }, () => Math.random());
        
        const result = await trainer.trainBatch(inputs, targets);
        console.log('Training result:', result);
    }
    
    static async basicInference(): Promise<void> {
        await init();
        
        const config: InferenceConfig = {
            max_batch_size: 64,
            use_webgl: true,
            use_webgpu: true,
            cache_in_indexeddb: true,
            max_inference_time_ms: 100,
        };
        
        const inference = new InferenceWrapper(config);
        
        // Load dummy model
        const modelData = new Uint8Array(1000); // placeholder
        const metadata: ModelMetadata = {
            name: 'test-model',
            version: '1.0.0',
            input_shape: [10],
            output_shape: [4],
            quantization: 'None',
        };
        
        const modelId = await inference.loadModel(modelData, metadata);
        
        // Run inference
        const input = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        const output = await inference.predict(modelId, input);
        console.log('Inference output:', output);
    }
}

// Default export with all functionality
export default {
    init,
    TrainerWrapper,
    InferenceWrapper,
    P2PManager,
    Utils,
    Integrations,
    getSystemCapabilities,
    runBenchmark,
    Examples,
};