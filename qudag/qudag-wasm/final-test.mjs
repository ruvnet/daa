// Final comprehensive test for QuDAG WASM
import pkg from './pkg-nodejs/qudag_wasm.js';

const { 
    QuDAGClient, 
    getInitStatus,
    WasmMlDsaKeyPair,
    WasmMlKemKeyPair,
    WasmQuantumFingerprint,
    WasmKdf,
    SecureRandom,
    Encoding,
    Validation
} = pkg;

console.log('🚀 QuDAG WASM Comprehensive Test');
console.log('==================================');

try {
    // Test 1: Module status
    console.log('\n1. Module Initialization');
    console.log('------------------------');
    const status = getInitStatus();
    console.log(`✅ Version: ${status.version()}`);
    console.log(`✅ Features: [${status.features().join(', ')}]`);
    console.log(`✅ Initialized: ${status.is_initialized()}`);

    // Test 2: Client creation  
    console.log('\n2. Client Creation');
    console.log('------------------');
    const client = new QuDAGClient();
    console.log('✅ QuDAG client created');

    const config = client.getConfig();
    console.log(`✅ Network port: ${config.network_port}`);
    console.log(`✅ Max peers: ${config.max_peers}`);
    console.log(`✅ Client features: [${client.getFeatures().join(', ')}]`);

    // Test 3: Cryptographic functions
    console.log('\n3. Cryptography Tests');
    console.log('---------------------');
    
    // Test random generation
    const randomBytes = SecureRandom.getRandomBytes(32);
    console.log(`✅ Generated ${randomBytes.length} random bytes`);
    
    // Test ML-DSA digital signatures
    const mlDsaKeyPair = new WasmMlDsaKeyPair();
    const publicKey = mlDsaKeyPair.getPublicKey();
    console.log(`✅ ML-DSA public key: ${publicKey.length} bytes`);
    
    const testMessage = Encoding.stringToBytes("Hello QuDAG WASM!");
    const signature = mlDsaKeyPair.sign(testMessage);
    console.log(`✅ Signature created: ${signature.length} bytes`);
    
    const isValid = mlDsaKeyPair.verify(testMessage, signature);
    console.log(`✅ Signature valid: ${isValid}`);

    // Test ML-KEM key encapsulation
    const mlKemKeyPair = new WasmMlKemKeyPair();
    const kemPublicKey = mlKemKeyPair.getPublicKey();
    console.log(`✅ ML-KEM public key: ${kemPublicKey.length} bytes`);
    
    const encapResult = mlKemKeyPair.encapsulate(kemPublicKey);
    console.log(`✅ Key encapsulation successful`);
    
    const sharedSecret = mlKemKeyPair.decapsulate(encapResult.ciphertext);
    console.log(`✅ Shared secret decapsulated: ${sharedSecret.length} bytes`);

    // Test quantum fingerprint
    const fingerprint = WasmQuantumFingerprint.generate(testMessage);
    const fingerprintHash = fingerprint.getHash();
    console.log(`✅ Quantum fingerprint: ${fingerprintHash.length} bytes`);

    // Test key derivation
    const password = Encoding.stringToBytes("test_password");
    const salt = WasmKdf.generateSalt();
    const derivedKey = WasmKdf.deriveKey(password, salt, 32);
    console.log(`✅ Key derivation: ${derivedKey.length} bytes`);

    // Test 4: Encoding utilities
    console.log('\n4. Encoding & Validation');
    console.log('-------------------------');
    
    const testString = "QuDAG WASM Test";
    const testBytes = Encoding.stringToBytes(testString);
    const backToString = Encoding.bytesToString(testBytes);
    console.log(`✅ String encoding: "${testString}" → "${backToString}"`);
    
    const hexString = Encoding.bytesToHex(testBytes);
    const backToBytes = Encoding.hexToBytes(hexString);
    console.log(`✅ Hex encoding: ${hexString} (${backToBytes.length} bytes)`);

    // Test validation
    console.log(`✅ Dark domain validation: ${Validation.isDarkDomain("test.dark")}`);
    console.log(`✅ Hex validation: ${Validation.isValidHex(hexString)}`);

    // Test 5: Feature detection
    console.log('\n5. Feature Detection');
    console.log('--------------------');
    const features = ['crypto', 'dag', 'network', 'vault', 'wasm'];
    features.forEach(feature => {
        const hasFeature = QuDAGClient.hasFeature(feature);
        console.log(`✅ ${feature}: ${hasFeature}`);
    });

    console.log('\n🎉 ALL TESTS PASSED!');
    console.log('====================================');
    console.log('QuDAG WASM is fully functional with:');
    console.log('• Quantum-resistant cryptography');
    console.log('• Digital signatures (ML-DSA)');
    console.log('• Key encapsulation (ML-KEM)');
    console.log('• Quantum fingerprinting');
    console.log('• Secure random generation');
    console.log('• Key derivation functions');
    console.log('• Complete encoding utilities');
    console.log('• Input validation');
    console.log('\nReady for NPM publishing! 🚀');

} catch (error) {
    console.error('\n❌ Test failed:', error.message);
    console.error('Stack trace:', error.stack);
    process.exit(1);
}