// Simple Node.js test for QuDAG WASM
import pkg from './pkg-nodejs/qudag_wasm.js';

const { 
    QuDAGClient, 
    getInitStatus,
    Blake3Hash 
} = pkg;

console.log('🚀 QuDAG WASM Node.js Test');
console.log('===========================');

try {
    // Test 1: Module status
    console.log('\n1. Testing module initialization...');
    const status = getInitStatus();
    console.log(`✅ Version: ${status.version()}`);
    console.log(`✅ Features: [${status.features().join(', ')}]`);
    console.log(`✅ Initialized: ${status.is_initialized()}`);

    // Test 2: Client creation  
    console.log('\n2. Testing client creation...');
    const client = new QuDAGClient();
    console.log('✅ QuDAG client created successfully');

    const config = client.getConfig();
    console.log(`✅ Network port: ${config.network_port}`);
    console.log(`✅ Max peers: ${config.max_peers}`);

    // Test 3: Feature detection
    console.log('\n3. Testing feature detection...');
    console.log(`✅ Crypto support: ${QuDAGClient.hasFeature('crypto')}`);
    console.log(`✅ DAG support: ${QuDAGClient.hasFeature('dag')}`);
    console.log(`✅ Network support: ${QuDAGClient.hasFeature('network')}`);
    console.log(`✅ WASM support: ${QuDAGClient.hasFeature('wasm')}`);

    // Test 4: BLAKE3 hashing
    console.log('\n4. Testing BLAKE3 cryptography...');
    const testData = 'Hello QuDAG WASM from Node.js!';
    const hash = Blake3Hash.hash(testData);
    console.log(`✅ Input: "${testData}"`);
    console.log(`✅ Hash: ${hash}`);

    // Test 5: Version check
    console.log('\n5. Testing version information...');
    const version = QuDAGClient.getVersion();
    console.log(`✅ Library version: ${version}`);

    console.log('\n🎉 All tests passed! QuDAG WASM is working correctly in Node.js');

} catch (error) {
    console.error('\n❌ Test failed:', error.message);
    console.error('Stack trace:', error.stack);
    process.exit(1);
}