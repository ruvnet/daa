# WASM Usage Guide

## Overview

QuDAG Exchange provides full WebAssembly (WASM) support, enabling you to run quantum-resistant resource exchange functionality directly in web browsers and Node.js environments. This guide covers building, integrating, and optimizing WASM modules.

## Building WASM Modules

### Prerequisites

Install required tools:

```bash
# Install Rust and wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI
cargo install wasm-bindgen-cli
```

### Building for Different Targets

#### Web Browser Target

```bash
# Build for modern browsers
wasm-pack build --target web --out-dir pkg-web

# Build with optimizations
wasm-pack build --target web --out-dir pkg-web --release -- --features wasm-opt
```

#### Node.js Target

```bash
# Build for Node.js
wasm-pack build --target nodejs --out-dir pkg-node

# With TypeScript definitions
wasm-pack build --target nodejs --out-dir pkg-node --typescript
```

#### Bundler Target (Webpack, Rollup, etc.)

```bash
# Build for bundlers
wasm-pack build --target bundler --out-dir pkg-bundler

# With custom name
wasm-pack build --target bundler --out-name qudag-exchange --out-dir pkg-bundler
```

### Build Configuration

Configure WASM features in `Cargo.toml`:

```toml
[package]
name = "qudag-exchange-wasm"
version = "1.0.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "console",
    "Window",
    "Document",
    "Storage",
    "Crypto",
    "SubtleCrypto"
]}

# Core dependencies with WASM support
qudag-exchange-core = { path = "../core", default-features = false, features = ["wasm"] }
qudag-crypto = { path = "../crypto", default-features = false, features = ["wasm"] }

[features]
default = ["console_error_panic_hook"]
wasm-opt = ["wee_alloc"]

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Single codegen unit for better optimization
```

## Browser Integration

### Basic Setup

Create an HTML file with WASM module:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>QuDAG Exchange WASM</title>
</head>
<body>
    <div id="app">
        <h1>QuDAG Exchange</h1>
        <button id="create-account">Create Account</button>
        <div id="balance"></div>
    </div>
    
    <script type="module">
        import init, { QuDagExchange } from './pkg-web/qudag_exchange_wasm.js';
        
        async function run() {
            // Initialize WASM module
            await init();
            
            // Create exchange instance
            const exchange = new QuDagExchange();
            
            // Set up event handlers
            document.getElementById('create-account').onclick = async () => {
                const account = await exchange.createAccount('user', 'password123');
                console.log('Account created:', account.address);
                
                // Update balance display
                const balance = await exchange.getBalance(account.address);
                document.getElementById('balance').textContent = `Balance: ${balance} rUv`;
            };
        }
        
        run();
    </script>
</body>
</html>
```

### Advanced Browser Integration

Complete browser application with all features:

```javascript
import init, { 
    QuDagExchange, 
    Transaction, 
    ResourceOffer,
    CryptoUtils 
} from './pkg-web/qudag_exchange_wasm.js';

class QuDagExchangeApp {
    constructor() {
        this.exchange = null;
        this.currentAccount = null;
    }
    
    async initialize() {
        // Initialize WASM
        await init();
        
        // Create exchange instance with config
        this.exchange = new QuDagExchange({
            network: 'mainnet',
            storageBackend: 'indexeddb',
            logLevel: 'info'
        });
        
        // Check for existing account in local storage
        await this.loadAccount();
        
        // Set up WebSocket connection for real-time updates
        await this.connectWebSocket();
    }
    
    async createAccount(name, password) {
        try {
            // Generate quantum-resistant keys
            const keyPair = await CryptoUtils.generateKeyPair('ml-dsa');
            
            // Create account with encrypted vault
            const account = await this.exchange.createAccount({
                name,
                password,
                keyType: 'ml-dsa',
                publicKey: keyPair.publicKey
            });
            
            // Store encrypted private key in IndexedDB
            await this.storePrivateKey(account.id, keyPair.privateKey, password);
            
            this.currentAccount = account;
            return account;
            
        } catch (error) {
            console.error('Failed to create account:', error);
            throw error;
        }
    }
    
    async transfer(to, amount, memo) {
        if (!this.currentAccount) {
            throw new Error('No account loaded');
        }
        
        // Create transaction
        const tx = Transaction.new({
            from: this.currentAccount.address,
            to,
            amount,
            memo,
            timestamp: Date.now(),
            nonce: await this.exchange.getNonce(this.currentAccount.address)
        });
        
        // Sign transaction
        const privateKey = await this.loadPrivateKey(this.currentAccount.id);
        const signature = await CryptoUtils.sign(tx.hash(), privateKey);
        tx.setSignature(signature);
        
        // Submit transaction
        const result = await this.exchange.submitTransaction(tx);
        
        // Wait for confirmation
        await this.waitForConfirmation(result.transactionId);
        
        return result;
    }
    
    async createResourceOffer(specs) {
        const offer = ResourceOffer.new({
            provider: this.currentAccount.address,
            resourceType: specs.type,
            specifications: {
                cpu: specs.cpu,
                memory: specs.memory,
                gpu: specs.gpu
            },
            pricePerHour: specs.price,
            minDuration: specs.minDuration || '1h',
            maxDuration: specs.maxDuration || '24h'
        });
        
        // Sign offer
        const privateKey = await this.loadPrivateKey(this.currentAccount.id);
        const signature = await CryptoUtils.sign(offer.hash(), privateKey);
        offer.setSignature(signature);
        
        // Submit offer
        return await this.exchange.createOffer(offer);
    }
    
    async searchResources(query) {
        return await this.exchange.searchOffers({
            resourceType: query.type,
            minCpu: query.cpu,
            minMemory: query.memory,
            maxPrice: query.maxPrice,
            sortBy: 'price'
        });
    }
    
    // Storage helpers
    async storePrivateKey(accountId, privateKey, password) {
        // Encrypt private key with password
        const encrypted = await CryptoUtils.encrypt(privateKey, password);
        
        // Store in IndexedDB
        const db = await this.openDB();
        const tx = db.transaction(['keys'], 'readwrite');
        await tx.objectStore('keys').put({
            accountId,
            encryptedKey: encrypted,
            timestamp: Date.now()
        });
    }
    
    async loadPrivateKey(accountId) {
        const password = await this.promptPassword();
        
        const db = await this.openDB();
        const tx = db.transaction(['keys'], 'readonly');
        const data = await tx.objectStore('keys').get(accountId);
        
        if (!data) {
            throw new Error('Private key not found');
        }
        
        // Decrypt private key
        return await CryptoUtils.decrypt(data.encryptedKey, password);
    }
    
    async openDB() {
        return new Promise((resolve, reject) => {
            const request = indexedDB.open('QuDagExchange', 1);
            
            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve(request.result);
            
            request.onupgradeneeded = (event) => {
                const db = event.target.result;
                if (!db.objectStoreNames.contains('keys')) {
                    db.createObjectStore('keys', { keyPath: 'accountId' });
                }
            };
        });
    }
    
    // WebSocket for real-time updates
    async connectWebSocket() {
        const wsUrl = 'wss://api.qudag.io/v1/ws';
        this.ws = new WebSocket(wsUrl);
        
        this.ws.onopen = () => {
            console.log('WebSocket connected');
            
            // Subscribe to account updates
            if (this.currentAccount) {
                this.ws.send(JSON.stringify({
                    type: 'subscribe',
                    channel: 'account',
                    accountId: this.currentAccount.id
                }));
            }
        };
        
        this.ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            this.handleWebSocketMessage(message);
        };
    }
    
    handleWebSocketMessage(message) {
        switch (message.type) {
            case 'balance_update':
                this.onBalanceUpdate(message.data);
                break;
            case 'transaction_confirmed':
                this.onTransactionConfirmed(message.data);
                break;
            case 'offer_accepted':
                this.onOfferAccepted(message.data);
                break;
        }
    }
}

// Initialize app
const app = new QuDagExchangeApp();
await app.initialize();
```

### React Integration

Using QuDAG Exchange in React:

```jsx
import React, { useState, useEffect } from 'react';
import init, { QuDagExchange } from '@qudag/exchange-wasm';

// Custom hook for QuDAG Exchange
function useQuDagExchange() {
    const [exchange, setExchange] = useState(null);
    const [loading, setLoading] = useState(true);
    
    useEffect(() => {
        async function initializeWasm() {
            await init();
            const exchangeInstance = new QuDagExchange();
            setExchange(exchangeInstance);
            setLoading(false);
        }
        
        initializeWasm();
    }, []);
    
    return { exchange, loading };
}

// Account component
function AccountManager() {
    const { exchange, loading } = useQuDagExchange();
    const [account, setAccount] = useState(null);
    const [balance, setBalance] = useState(0);
    
    const createAccount = async () => {
        if (!exchange) return;
        
        const name = prompt('Enter account name:');
        const password = prompt('Enter password:');
        
        try {
            const newAccount = await exchange.createAccount(name, password);
            setAccount(newAccount);
            
            // Fetch initial balance
            const bal = await exchange.getBalance(newAccount.address);
            setBalance(bal);
        } catch (error) {
            console.error('Failed to create account:', error);
        }
    };
    
    const refreshBalance = async () => {
        if (!account || !exchange) return;
        
        const bal = await exchange.getBalance(account.address);
        setBalance(bal);
    };
    
    if (loading) {
        return <div>Loading WASM module...</div>;
    }
    
    return (
        <div className="account-manager">
            {!account ? (
                <button onClick={createAccount}>Create Account</button>
            ) : (
                <div>
                    <h3>Account: {account.name}</h3>
                    <p>Address: {account.address}</p>
                    <p>Balance: {balance} rUv</p>
                    <button onClick={refreshBalance}>Refresh Balance</button>
                </div>
            )}
        </div>
    );
}

// Transfer component
function TransferForm({ exchange, account }) {
    const [to, setTo] = useState('');
    const [amount, setAmount] = useState('');
    const [memo, setMemo] = useState('');
    const [status, setStatus] = useState('');
    
    const handleTransfer = async (e) => {
        e.preventDefault();
        
        if (!exchange || !account) return;
        
        try {
            setStatus('Signing transaction...');
            
            const tx = await exchange.createTransaction({
                from: account.address,
                to,
                amount: parseFloat(amount),
                memo
            });
            
            setStatus('Submitting transaction...');
            const result = await exchange.submitTransaction(tx);
            
            setStatus('Waiting for confirmation...');
            await exchange.waitForConfirmation(result.transactionId);
            
            setStatus('Transfer completed!');
            
            // Reset form
            setTo('');
            setAmount('');
            setMemo('');
            
        } catch (error) {
            setStatus(`Error: ${error.message}`);
        }
    };
    
    return (
        <form onSubmit={handleTransfer}>
            <input
                type="text"
                placeholder="Recipient address"
                value={to}
                onChange={(e) => setTo(e.target.value)}
                required
            />
            <input
                type="number"
                placeholder="Amount"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                required
            />
            <input
                type="text"
                placeholder="Memo (optional)"
                value={memo}
                onChange={(e) => setMemo(e.target.value)}
            />
            <button type="submit">Transfer</button>
            {status && <p>{status}</p>}
        </form>
    );
}
```

## Node.js Integration

### Basic Setup

```javascript
// Import WASM module for Node.js
const { QuDagExchange, CryptoUtils } = require('./pkg-node/qudag_exchange_wasm.js');

async function main() {
    // Create exchange instance
    const exchange = new QuDagExchange({
        network: 'testnet',
        dataDir: './data'
    });
    
    // Create account
    const account = await exchange.createAccount('alice', 'securepassword');
    console.log('Account created:', account.address);
    
    // Check balance
    const balance = await exchange.getBalance(account.address);
    console.log('Balance:', balance, 'rUv');
}

main().catch(console.error);
```

### Advanced Node.js Usage

Complete Node.js application:

```javascript
const fs = require('fs').promises;
const path = require('path');
const { QuDagExchange, Transaction, CryptoUtils } = require('@qudag/exchange-wasm');

class QuDagExchangeNode {
    constructor(config) {
        this.config = config;
        this.exchange = null;
        this.accounts = new Map();
    }
    
    async initialize() {
        // Initialize exchange
        this.exchange = new QuDagExchange({
            network: this.config.network || 'mainnet',
            dataDir: this.config.dataDir || './qudag-data',
            logLevel: this.config.logLevel || 'info'
        });
        
        // Load existing accounts
        await this.loadAccounts();
        
        // Start background tasks
        this.startBackgroundTasks();
    }
    
    async createAccount(name, password) {
        // Generate keys
        const keyPair = await CryptoUtils.generateKeyPair('ml-dsa');
        
        // Create account
        const account = await this.exchange.createAccount({
            name,
            password,
            publicKey: keyPair.publicKey
        });
        
        // Store account data
        const accountData = {
            id: account.id,
            name: account.name,
            address: account.address,
            publicKey: keyPair.publicKey,
            encryptedPrivateKey: await this.encryptKey(keyPair.privateKey, password)
        };
        
        await this.saveAccount(accountData);
        this.accounts.set(account.id, accountData);
        
        return account;
    }
    
    async loadAccounts() {
        const accountsPath = path.join(this.config.dataDir, 'accounts.json');
        
        try {
            const data = await fs.readFile(accountsPath, 'utf8');
            const accounts = JSON.parse(data);
            
            for (const account of accounts) {
                this.accounts.set(account.id, account);
            }
        } catch (error) {
            // No accounts file yet
            console.log('No existing accounts found');
        }
    }
    
    async saveAccount(accountData) {
        const accountsPath = path.join(this.config.dataDir, 'accounts.json');
        const accounts = Array.from(this.accounts.values());
        accounts.push(accountData);
        
        await fs.mkdir(this.config.dataDir, { recursive: true });
        await fs.writeFile(accountsPath, JSON.stringify(accounts, null, 2));
    }
    
    async batchTransfer(accountId, transfers, password) {
        const account = this.accounts.get(accountId);
        if (!account) {
            throw new Error('Account not found');
        }
        
        // Decrypt private key
        const privateKey = await this.decryptKey(account.encryptedPrivateKey, password);
        
        // Create batch transaction
        const transactions = [];
        let nonce = await this.exchange.getNonce(account.address);
        
        for (const transfer of transfers) {
            const tx = Transaction.new({
                from: account.address,
                to: transfer.to,
                amount: transfer.amount,
                memo: transfer.memo || '',
                nonce: nonce++,
                timestamp: Date.now()
            });
            
            // Sign transaction
            const signature = await CryptoUtils.sign(tx.hash(), privateKey);
            tx.setSignature(signature);
            
            transactions.push(tx);
        }
        
        // Submit batch
        const results = await this.exchange.submitBatch(transactions);
        
        // Wait for all confirmations
        await Promise.all(results.map(r => 
            this.exchange.waitForConfirmation(r.transactionId)
        ));
        
        return results;
    }
    
    async provideResources(accountId, specs, password) {
        const account = this.accounts.get(accountId);
        if (!account) {
            throw new Error('Account not found');
        }
        
        // Start resource provider
        const provider = await this.exchange.startProvider({
            account: account.address,
            resources: {
                cpu: specs.cpu || 0,
                memory: specs.memory || 0,
                storage: specs.storage || 0,
                gpu: specs.gpu || []
            },
            pricing: specs.pricing
        });
        
        // Monitor earnings
        provider.on('job_completed', async (job) => {
            console.log(`Job completed: ${job.id}, earned: ${job.payment} rUv`);
        });
        
        provider.on('payment_received', async (payment) => {
            console.log(`Payment received: ${payment.amount} rUv`);
            const balance = await this.exchange.getBalance(account.address);
            console.log(`New balance: ${balance} rUv`);
        });
        
        return provider;
    }
    
    async monitorMarket() {
        // Subscribe to market updates
        const marketStream = await this.exchange.subscribeToMarket({
            resourceTypes: ['cpu', 'gpu', 'storage'],
            priceUpdates: true,
            interval: 60000 // 1 minute
        });
        
        marketStream.on('price_update', (data) => {
            console.log('Market prices:', data);
            
            // Adjust our pricing based on market
            this.adjustPricing(data);
        });
        
        marketStream.on('demand_spike', (data) => {
            console.log('High demand for:', data.resourceType);
            
            // Consider offering more resources
            this.scaleResources(data.resourceType);
        });
    }
    
    // Utility functions
    async encryptKey(privateKey, password) {
        return await CryptoUtils.encrypt(privateKey, password);
    }
    
    async decryptKey(encryptedKey, password) {
        return await CryptoUtils.decrypt(encryptedKey, password);
    }
    
    startBackgroundTasks() {
        // Periodic balance check
        setInterval(async () => {
            for (const account of this.accounts.values()) {
                const balance = await this.exchange.getBalance(account.address);
                console.log(`${account.name}: ${balance} rUv`);
            }
        }, 300000); // Every 5 minutes
        
        // Market monitoring
        this.monitorMarket();
    }
}

// CLI interface
const readline = require('readline');
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

async function cli() {
    const node = new QuDagExchangeNode({
        network: 'testnet',
        dataDir: './qudag-node-data'
    });
    
    await node.initialize();
    
    console.log('QuDAG Exchange Node initialized');
    console.log('Commands: create-account, transfer, provide-resources, balance, exit');
    
    const prompt = () => {
        rl.question('> ', async (command) => {
            const [cmd, ...args] = command.split(' ');
            
            try {
                switch (cmd) {
                    case 'create-account':
                        const name = await question('Account name: ');
                        const password = await question('Password: ', true);
                        const account = await node.createAccount(name, password);
                        console.log('Account created:', account.address);
                        break;
                        
                    case 'transfer':
                        // Implementation...
                        break;
                        
                    case 'balance':
                        // Implementation...
                        break;
                        
                    case 'exit':
                        process.exit(0);
                        
                    default:
                        console.log('Unknown command');
                }
            } catch (error) {
                console.error('Error:', error.message);
            }
            
            prompt();
        });
    };
    
    prompt();
}

// Run CLI
if (require.main === module) {
    cli().catch(console.error);
}

module.exports = { QuDagExchangeNode };
```

## Performance Optimization

### WASM Module Size Optimization

Reduce module size for faster loading:

```rust
// Use wee_alloc for smaller binary size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Disable panic strings in release
#[cfg(not(debug_assertions))]
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    unsafe { core::arch::wasm32::unreachable() }
}

// Use size-optimized features
#[wasm_bindgen]
pub struct QuDagExchange {
    #[cfg(feature = "minimal")]
    core: MinimalCore,
    
    #[cfg(not(feature = "minimal"))]
    core: FullCore,
}
```

### Lazy Loading

Load WASM modules on demand:

```javascript
// Lazy load heavy features
class QuDagExchangeLazy {
    constructor() {
        this.modules = new Map();
    }
    
    async loadModule(name) {
        if (this.modules.has(name)) {
            return this.modules.get(name);
        }
        
        let module;
        switch (name) {
            case 'crypto':
                module = await import('./pkg-web/crypto.js');
                break;
            case 'consensus':
                module = await import('./pkg-web/consensus.js');
                break;
            case 'market':
                module = await import('./pkg-web/market.js');
                break;
        }
        
        this.modules.set(name, module);
        return module;
    }
    
    async generateKeys(algorithm) {
        const crypto = await this.loadModule('crypto');
        return crypto.generateKeyPair(algorithm);
    }
}
```

### Memory Management

Efficient memory usage in WASM:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MemoryEfficientExchange {
    // Use compact data structures
    accounts: BTreeMap<u32, Account>,
    
    // Pool commonly used objects
    tx_pool: ObjectPool<Transaction>,
}

#[wasm_bindgen]
impl MemoryEfficientExchange {
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
            tx_pool: ObjectPool::new(100),
        }
    }
    
    pub fn create_transaction(&mut self) -> Transaction {
        // Reuse pooled objects
        self.tx_pool.get_or_create()
    }
    
    pub fn return_transaction(&mut self, tx: Transaction) {
        // Return to pool for reuse
        self.tx_pool.return_object(tx);
    }
    
    // Manual memory management
    pub fn clear_cache(&mut self) {
        self.tx_pool.clear();
        self.accounts.shrink_to_fit();
    }
}
```

### Web Workers

Use Web Workers for CPU-intensive tasks:

```javascript
// Main thread
const worker = new Worker('qudag-worker.js');

async function performHeavyComputation(data) {
    return new Promise((resolve, reject) => {
        worker.postMessage({
            type: 'compute',
            data
        });
        
        worker.onmessage = (event) => {
            if (event.data.type === 'result') {
                resolve(event.data.result);
            } else if (event.data.type === 'error') {
                reject(new Error(event.data.error));
            }
        };
    });
}

// Worker thread (qudag-worker.js)
importScripts('./pkg-web/qudag_exchange_wasm.js');

let exchange;

self.onmessage = async (event) => {
    if (!exchange) {
        await wasm_bindgen('./pkg-web/qudag_exchange_wasm_bg.wasm');
        exchange = new wasm_bindgen.QuDagExchange();
    }
    
    switch (event.data.type) {
        case 'compute':
            try {
                const result = await exchange.heavyComputation(event.data.data);
                self.postMessage({ type: 'result', result });
            } catch (error) {
                self.postMessage({ type: 'error', error: error.message });
            }
            break;
    }
};
```

## Testing WASM Modules

### Browser Testing

Test WASM in headless browser:

```bash
# Install test dependencies
npm install --save-dev @web/test-runner @web/test-runner-playwright

# Run tests
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
```

Test file example:

```javascript
// tests/browser.test.js
import { expect } from '@esm-bundle/chai';
import init, { QuDagExchange } from '../pkg-web/qudag_exchange_wasm.js';

describe('QuDAG Exchange WASM Browser Tests', () => {
    let exchange;
    
    before(async () => {
        await init();
        exchange = new QuDagExchange();
    });
    
    it('should create account', async () => {
        const account = await exchange.createAccount('test', 'password');
        expect(account).to.have.property('address');
        expect(account.address).to.match(/^qd1[a-z0-9]{39}$/);
    });
    
    it('should handle transactions', async () => {
        const tx = await exchange.createTransaction({
            from: 'qd1sender...',
            to: 'qd1receiver...',
            amount: 100
        });
        
        expect(tx).to.have.property('hash');
        expect(tx.hash).to.have.lengthOf(64);
    });
});
```

### Node.js Testing

```javascript
// tests/node.test.js
const test = require('tape');
const { QuDagExchange } = require('../pkg-node/qudag_exchange_wasm.js');

test('QuDAG Exchange WASM Node.js Tests', async (t) => {
    const exchange = new QuDagExchange();
    
    t.test('account creation', async (st) => {
        const account = await exchange.createAccount('test', 'password');
        st.ok(account.address, 'should have address');
        st.equal(account.address.length, 42, 'address should be 42 chars');
        st.end();
    });
    
    t.test('balance operations', async (st) => {
        const balance = await exchange.getBalance('qd1test...');
        st.equal(typeof balance, 'string', 'balance should be string');
        st.ok(parseFloat(balance) >= 0, 'balance should be non-negative');
        st.end();
    });
    
    t.end();
});
```

## Troubleshooting

### Common Issues

#### 1. Module Loading Errors

```javascript
// Error: Failed to load WASM module
// Solution: Ensure correct MIME type
// Add to server config:
app.use((req, res, next) => {
    if (req.url.endsWith('.wasm')) {
        res.setHeader('Content-Type', 'application/wasm');
    }
    next();
});
```

#### 2. Memory Issues

```javascript
// Monitor memory usage
const memoryUsage = () => {
    if (performance.memory) {
        const used = performance.memory.usedJSHeapSize / 1048576;
        const total = performance.memory.totalJSHeapSize / 1048576;
        console.log(`Memory: ${used.toFixed(2)}MB / ${total.toFixed(2)}MB`);
    }
};

// Clean up when done
exchange.free();
```

#### 3. Browser Compatibility

```javascript
// Check for WASM support
if (typeof WebAssembly === 'undefined') {
    console.error('WebAssembly not supported');
    // Fall back to REST API
    return new QuDagExchangeREST();
}

// Check for required features
const checkFeatures = async () => {
    const features = {
        bigInt: typeof BigInt !== 'undefined',
        sharedMemory: typeof SharedArrayBuffer !== 'undefined',
        simd: await wasmFeatureDetect.simd(),
        threads: await wasmFeatureDetect.threads()
    };
    
    return features;
};
```

## Best Practices

### 1. Error Handling

Always handle WASM errors properly:

```javascript
try {
    const result = await exchange.operation();
} catch (error) {
    if (error instanceof WebAssembly.RuntimeError) {
        console.error('WASM runtime error:', error);
    } else if (error.message.includes('unreachable')) {
        console.error('WASM panic:', error);
    } else {
        console.error('Operation failed:', error);
    }
}
```

### 2. Resource Cleanup

Free WASM resources when done:

```javascript
class ManagedExchange {
    constructor() {
        this.exchange = new QuDagExchange();
        
        // Auto-cleanup on page unload
        window.addEventListener('beforeunload', () => {
            this.cleanup();
        });
    }
    
    cleanup() {
        if (this.exchange) {
            this.exchange.free();
            this.exchange = null;
        }
    }
}
```

### 3. Progressive Enhancement

Provide fallbacks for non-WASM environments:

```javascript
const createExchange = async () => {
    try {
        // Try WASM first
        await init();
        return new QuDagExchangeWASM();
    } catch (error) {
        console.warn('WASM not available, using REST API');
        return new QuDagExchangeREST();
    }
};
```

## Conclusion

QuDAG Exchange WASM support enables powerful quantum-resistant cryptography and resource exchange functionality in any JavaScript environment. By following this guide, you can integrate QuDAG Exchange into web applications, Node.js services, and hybrid applications with optimal performance and security.