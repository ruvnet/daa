# QuDAG WASM Bindings

WebAssembly bindings for the QuDAG protocol, enabling browser and Node.js applications to interact with the quantum-resistant DAG network.

## Features

- 🔐 **Quantum-Resistant Cryptography**: ML-KEM-768, ML-DSA, BLAKE3 hashing
- 📊 **DAG Operations**: Vertex management, consensus queries, DAG statistics
- 🌐 **P2P Networking**: Peer management, network statistics, onion routing
- 🌑 **Dark Addressing**: .dark domain registration/resolution, shadow addresses
- 🔑 **Password Vault**: Secure password management with AES-256-GCM
- ⚡ **Optimized for Size**: ~200KB gzipped with wee_alloc

## Installation

```bash
npm install qudag-wasm
# or
yarn add qudag-wasm
```

## Usage

### Browser

```javascript
import init, { QuDAGClient, WasmMlDsaKeyPair, WasmDarkResolver } from 'qudag-wasm';

async function main() {
  // Initialize the WASM module
  await init();
  
  // Create a client
  const client = new QuDAGClient();
  console.log('QuDAG version:', QuDAGClient.getVersion());
  
  // Generate quantum-resistant keys
  const keypair = new WasmMlDsaKeyPair();
  const publicKey = keypair.getPublicKey();
  
  // Register a dark domain
  const resolver = new WasmDarkResolver();
  const domain = await resolver.registerDomain('myservice.dark');
  console.log('Registered:', domain);
}

main();
```

### Node.js

```javascript
const { QuDAGClient, WasmHasher, WasmVault } = require('qudag-wasm');

// Hash data with BLAKE3
const data = Buffer.from('Hello, QuDAG!');
const hash = WasmHasher.hashBlake3(data);
console.log('Hash:', Buffer.from(hash).toString('hex'));

// Password vault operations
const vault = new WasmVault();
await vault.init('master-password');

// Generate secure password
const password = WasmVault.generatePassword(16, true, true);
await vault.addEntry('github', 'user@example.com', password);
```

## API Reference

### Core Client

```typescript
class QuDAGClient {
  constructor();
  static getVersion(): string;
  static hasFeature(feature: string): boolean;
  getConfig(): object;
}
```

### Cryptography

```typescript
// ML-DSA Digital Signatures
class WasmMlDsaKeyPair {
  constructor();
  getPublicKey(): Uint8Array;
  getSecretKey(): Uint8Array;
  sign(message: Uint8Array): Uint8Array;
  toJson(): object;
}

// ML-KEM-768 Key Encapsulation
class WasmMlKem768 {
  constructor();
  generateKeyPair(): { publicKey: string, secretKey: string };
  encapsulate(publicKey: Uint8Array): { ciphertext: string, sharedSecret: string };
  decapsulate(secretKey: Uint8Array, ciphertext: Uint8Array): Uint8Array;
}

// Hashing
class WasmHasher {
  static hashBlake3(data: Uint8Array): Uint8Array;
  static hashBlake3Hex(data: Uint8Array): string;
}
```

### DAG Operations

```typescript
class WasmDag {
  constructor();
  addVertex(vertexData: object): string;
  getVertex(vertexId: string): object;
  getStats(): { vertexCount: number, edgeCount: number, tipCount: number };
  getTips(): object[];
  validate(): boolean;
}

class WasmConsensus {
  constructor();
  async queryVertex(vertexId: string): object;
  getMetrics(): object;
}
```

### Networking

```typescript
class WasmNetworkManager {
  constructor();
  listPeers(): object[];
  async addPeer(address: string): string;
  removePeer(peerId: string): boolean;
  banPeer(peerId: string, reason?: string): boolean;
  getNetworkStats(): object;
  async testConnectivity(): object;
}
```

### Dark Addressing

```typescript
class WasmDarkResolver {
  constructor();
  async registerDomain(domain: string): object;
  async resolveDomain(domain: string): object;
  generateShadowAddress(ttlSeconds: number): object;
  createFingerprint(data: Uint8Array): object;
  listDomains(): string[];
  isDomainAvailable(domain: string): boolean;
}
```

### Password Vault

```typescript
class WasmVault {
  constructor();
  async init(masterPassword: string): void;
  addEntry(label: string, username: string, password: string, category?: string): string;
  getEntry(label: string): object;
  getPassword(label: string): string;
  listEntries(category?: string): object[];
  removeEntry(label: string): boolean;
  static generatePassword(length: number, includeSymbols: boolean, includeNumbers: boolean): string;
  getStats(): object;
}
```

### Utilities

```typescript
// Logging
function log(message: string): void;
function logError(message: string): void;
function logWarn(message: string): void;

// Performance
class Performance {
  static now(): number;
  static measure(name: string, start: number): number;
}

// Encoding
class Encoding {
  static bytesToHex(bytes: Uint8Array): string;
  static hexToBytes(hex: string): Uint8Array;
  static stringToBytes(s: string): Uint8Array;
  static bytesToString(bytes: Uint8Array): string;
}

// Validation
class Validation {
  static isDarkDomain(domain: string): boolean;
  static isPeerAddress(address: string): boolean;
  static isValidHex(hex: string): boolean;
}
```

## Building from Source

### Prerequisites

- Rust 1.75+ with `wasm32-unknown-unknown` target
- wasm-pack
- Node.js 16+

### Build Steps

```bash
# Install dependencies
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Build the WASM module
wasm-pack build --target web --out-dir pkg

# Build for Node.js
wasm-pack build --target nodejs --out-dir pkg-node

# Build with all optimizations
wasm-pack build --target web --release -- --features wee_alloc
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Run WASM tests in browser
wasm-pack test --chrome --firefox

# Run WASM tests in Node.js
wasm-pack test --node
```

## Performance

The WASM module is optimized for size and performance:

- **Size**: ~200KB gzipped (with wee_alloc)
- **ML-DSA signing**: ~5ms per operation
- **BLAKE3 hashing**: <1ms for 1KB data
- **Memory usage**: Minimal with wee_alloc allocator

## Security Considerations

1. **Key Management**: Secret keys are exposed through the API. Ensure proper key storage in your application.
2. **Master Password**: The vault's master password should be stored securely and never in plain text.
3. **Browser Security**: Use Content Security Policy (CSP) headers to protect against XSS attacks.
4. **CORS**: Configure appropriate CORS policies when loading the WASM module.

## Browser Compatibility

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

Node.js 14+ is required for server-side usage.

## License

MIT OR Apache-2.0