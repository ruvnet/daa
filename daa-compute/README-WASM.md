# DAA Compute WebAssembly

Browser-compatible AI training and inference using WebAssembly with P2P gradient sharing.

## Features

- **Browser-based training**: Lightweight neural network training optimized for browsers
- **WebRTC P2P networking**: Direct browser-to-browser communication for gradient sharing
- **WebGL/WebGPU acceleration**: Hardware acceleration when available
- **Lightweight inference**: Efficient model inference with automatic optimization
- **TypeScript support**: Full type definitions and wrapper library
- **Framework integration**: Compatible with TensorFlow.js and ONNX.js

## Installation

```bash
npm install @daa/compute-wasm
```

## Quick Start

### Training Example

```typescript
import { init, TrainerWrapper } from '@daa/compute-wasm';

async function trainModel() {
    // Initialize WASM module
    await init();
    
    // Configure training
    const config = {
        max_train_time_ms: 100,
        batch_size: 32,
        use_simd: true,
        memory_limit_mb: 256,
    };
    
    // Create trainer
    const trainer = new TrainerWrapper(config);
    
    // Training data (replace with your data)
    const inputs = new Float32Array([/* your input data */]);
    const targets = new Float32Array([/* your target data */]);
    
    // Train on batch
    const result = await trainer.trainBatch(inputs, targets);
    console.log('Training result:', result);
    
    // Get gradients for P2P sharing
    const gradients = trainer.getGradients();
    
    // Apply gradients from other peers
    trainer.applyGradients(gradients);
}
```

### Inference Example

```typescript
import { init, InferenceWrapper } from '@daa/compute-wasm';

async function runInference() {
    await init();
    
    const config = {
        max_batch_size: 64,
        use_webgl: true,
        use_webgpu: true,
        cache_in_indexeddb: true,
        max_inference_time_ms: 100,
    };
    
    const inference = new InferenceWrapper(config);
    
    // Load model
    const modelData = new Uint8Array(/* your model data */);
    const metadata = {
        name: 'my-model',
        version: '1.0.0',
        input_shape: [784], // e.g., MNIST
        output_shape: [10],
        quantization: 'None',
    };
    
    const modelId = await inference.loadModel(modelData, metadata);
    
    // Run inference
    const input = new Float32Array([/* your input data */]);
    const output = await inference.predict(modelId, input);
    
    console.log('Prediction:', output);
}
```

### P2P Networking

```typescript
import { P2PManager } from '@daa/compute-wasm';

const p2pConfig = {
    listen_addresses: ['/ip4/0.0.0.0/tcp/0/ws'],
    bootstrap_nodes: [],
    enable_mdns: true,
    enable_nat_traversal: true,
    enable_relay: true,
    compression_level: 3,
};

const p2p = new P2PManager(p2pConfig);

// Connect to network
await p2p.connect();

// Broadcast gradients
await p2p.broadcastGradients(gradients);

// Receive gradients from peers
p2p.onGradientsReceived((gradients, peerId) => {
    trainer.applyGradients(gradients);
});
```

## API Reference

### TrainerWrapper

Browser-based neural network training with automatic optimization.

#### Methods

- `constructor(config: BrowserTrainingConfig)`: Create trainer instance
- `trainBatch(inputs: Float32Array, targets: Float32Array)`: Train on data batch
- `getGradients()`: Get compressed gradients for sharing
- `applyGradients(gradients: Uint8Array)`: Apply peer gradients

### InferenceWrapper

Lightweight model inference with hardware acceleration.

#### Methods

- `constructor(config?: InferenceConfig)`: Create inference engine
- `loadModel(data: Uint8Array, metadata: ModelMetadata)`: Load model for inference
- `predict(modelId: string, input: Float32Array)`: Run model inference
- `getModelInfo(modelId: string)`: Get model information
- `clearCache()`: Clear model cache

### Configuration Types

#### BrowserTrainingConfig

```typescript
interface BrowserTrainingConfig {
    max_train_time_ms: number;    // Max training time per batch
    batch_size: number;           // Training batch size
    use_simd: boolean;           // Enable SIMD acceleration
    memory_limit_mb: number;     // Memory limit in MB
}
```

#### InferenceConfig

```typescript
interface InferenceConfig {
    max_batch_size: number;           // Max inference batch size
    use_webgl: boolean;              // Enable WebGL acceleration
    use_webgpu: boolean;             // Enable WebGPU acceleration
    cache_in_indexeddb: boolean;     // Cache models in IndexedDB
    max_inference_time_ms: number;   // Max inference time limit
}
```

## Performance Optimization

### Browser Compatibility

- **Modern browsers**: Full feature support including WebGL and WebRTC
- **Safari**: Limited WebRTC support, falls back to WebSocket
- **Mobile browsers**: Reduced memory limits and CPU-only training

### Memory Management

```typescript
// Check available memory
const capabilities = await getSystemCapabilities();
console.log('Available memory:', capabilities.device_memory);

// Adjust batch size based on memory
const config = {
    batch_size: Math.min(64, capabilities.device_memory * 8),
    memory_limit_mb: capabilities.device_memory * 0.5,
};
```

### Hardware Acceleration

```typescript
// Check acceleration capabilities
const caps = await getInferenceCapabilities();

const config = {
    use_webgl: caps.webgl,
    use_webgpu: caps.webgpu,
    use_simd: caps.wasm_simd,
};
```

## Framework Integration

### TensorFlow.js

```typescript
import * as tf from '@tensorflow/tfjs';
import { Integrations } from '@daa/compute-wasm';

// Convert TensorFlow tensor to WASM format
const tfTensor = tf.tensor2d([[1, 2], [3, 4]]);
const data = Integrations.tensorFlow.fromTensor(tfTensor);

// Convert WASM result back to tensor
const result = await inference.predict(modelId, data);
const outputTensor = Integrations.tensorFlow.toTensor(result, [2, 2]);
```

### ONNX.js

```typescript
import { InferenceSession } from 'onnxjs';
import { Integrations } from '@daa/compute-wasm';

// Convert ONNX model
const session = new InferenceSession();
await session.loadModel('./model.onnx');

const modelData = await Integrations.onnx.convertModel(session);
const metadata = Integrations.onnx.extractMetadata(session);
```

## Building from Source

### Prerequisites

- Rust 1.70+
- wasm-pack
- Node.js 16+

### Build Commands

```bash
# Install dependencies
npm install

# Build for web
npm run build

# Build for Node.js
npm run build:nodejs

# Build for bundlers
npm run build:bundler

# Build all targets
npm run build:all

# Run tests
npm test
```

### Development Workflow

```bash
# Watch mode for development
wasm-pack build --dev --target web

# Test in browser
npm run test

# Test in Node.js
npm run test:node
```

## Examples

See the `examples/` directory for complete working examples:

- `basic-training.html`: Simple browser training
- `federated-learning.html`: Multi-peer training
- `model-inference.html`: Model serving in browser
- `tensorflow-integration.js`: TensorFlow.js integration
- `node-example.js`: Node.js usage

## Troubleshooting

### Common Issues

1. **Memory errors**: Reduce batch size or memory limit
2. **WebRTC connection failures**: Check STUN/TURN configuration
3. **Slow performance**: Enable hardware acceleration
4. **Model loading errors**: Verify model format and metadata

### Performance Tips

- Use SIMD-enabled browsers for faster computation
- Enable WebGL/WebGPU for hardware acceleration
- Adjust batch sizes based on available memory
- Cache frequently used models in IndexedDB

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes to Rust code in `src/`
4. Add TypeScript types to `src/typescript/`
5. Update tests and documentation
6. Submit a pull request

## License

MIT License - see LICENSE file for details

## Changelog

### v0.1.0

- Initial WebAssembly implementation
- Browser training and inference
- WebRTC P2P networking
- TypeScript wrapper and definitions
- Hardware acceleration support