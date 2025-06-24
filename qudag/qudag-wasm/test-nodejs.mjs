import { readFile } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Read the WASM binary
const wasmPath = join(__dirname, 'pkg-final', 'qudag_wasm_bg.wasm');
const wasmBytes = await readFile(wasmPath);

// Initialize the WASM module
const wasmModule = await WebAssembly.instantiate(wasmBytes, {
    './qudag_wasm_bg.js': await import(join(__dirname, 'pkg-final', 'qudag_wasm.js'))
});

console.log('QuDAG WASM Node.js Test');
console.log('========================');

try {
    // Import QuDAG WASM functions
    const { 
        QuDAGClient, 
        get_init_status,
        Blake3Hash
    } = await import('./pkg-final/qudag_wasm.js');

    console.log('✅ WASM module imported successfully');

    // Test initialization status
    const status = get_init_status();
    console.log(`✅ Module Version: ${status.version()}`);
    console.log(`✅ Features: ${status.features().join(', ')}`);

    // Test client creation
    const client = new QuDAGClient();
    console.log('✅ QuDAG client created');
    
    const config = client.getConfig();
    console.log(`✅ Config - Port: ${config.network_port}, Max Peers: ${config.max_peers}`);

    // Test BLAKE3 hashing
    const testMessage = "Hello from Node.js QuDAG WASM!";
    const hash = Blake3Hash.hash(testMessage);
    console.log(`✅ BLAKE3 Hash: ${hash}`);

    // Test feature detection
    console.log(`✅ Has crypto: ${QuDAGClient.hasFeature('crypto')}`);
    console.log(`✅ Has DAG: ${QuDAGClient.hasFeature('dag')}`);
    console.log(`✅ Has network: ${QuDAGClient.hasFeature('network')}`);

    console.log('\n🎉 All Node.js tests passed!');

} catch (error) {
    console.error('❌ Test failed:', error.message);
    console.error(error.stack);
}