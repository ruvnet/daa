import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import type { 
  QuantumCrypto,
  KeyPair,
  PublicKey,
  SecretKey,
  EncryptedData,
  Signature,
  SharedSecret
} from '@/types';

describe('Quantum-Resistant Cryptographic Operations', () => {
  let crypto: QuantumCrypto;
  let keyPairs: KeyPair[] = [];
  
  beforeEach(async () => {
    const { createQuantumCrypto } = await import('@/crypto');
    crypto = await createQuantumCrypto();
  });
  
  afterEach(() => {
    // Clean up key material
    keyPairs.forEach(kp => {
      kp.secretKey.destroy();
    });
    keyPairs = [];
  });
  
  describe('Key Generation', () => {
    it('should generate ML-KEM-768 key pair', async () => {
      const keyPair = await crypto.generateKeyPair('ML-KEM-768');
      keyPairs.push(keyPair);
      
      expect(keyPair.publicKey).toBeDefined();
      expect(keyPair.secretKey).toBeDefined();
      expect(keyPair.algorithm).toBe('ML-KEM-768');
      
      // Verify key sizes
      expect(keyPair.publicKey.bytes).toHaveLength(1184); // ML-KEM-768 public key size
      expect(keyPair.secretKey.bytes).toHaveLength(2400); // ML-KEM-768 secret key size
    });
    
    it('should generate ML-DSA-65 key pair', async () => {
      const keyPair = await crypto.generateKeyPair('ML-DSA-65');
      keyPairs.push(keyPair);
      
      expect(keyPair.publicKey).toBeDefined();
      expect(keyPair.secretKey).toBeDefined();
      expect(keyPair.algorithm).toBe('ML-DSA-65');
      
      // Verify key sizes
      expect(keyPair.publicKey.bytes).toHaveLength(1952); // ML-DSA-65 public key size
      expect(keyPair.secretKey.bytes).toHaveLength(4032); // ML-DSA-65 secret key size
    });
    
    it('should generate different keys each time', async () => {
      const kp1 = await crypto.generateKeyPair('ML-KEM-768');
      const kp2 = await crypto.generateKeyPair('ML-KEM-768');
      keyPairs.push(kp1, kp2);
      
      expect(kp1.publicKey.bytes).not.toEqual(kp2.publicKey.bytes);
      expect(kp1.secretKey.bytes).not.toEqual(kp2.secretKey.bytes);
    });
    
    it('should support deterministic key generation with seed', async () => {
      const seed = testUtils.generateRandomBytes(32);
      
      const kp1 = await crypto.generateKeyPairFromSeed('ML-KEM-768', seed);
      const kp2 = await crypto.generateKeyPairFromSeed('ML-KEM-768', seed);
      keyPairs.push(kp1, kp2);
      
      expect(kp1.publicKey.bytes).toEqual(kp2.publicKey.bytes);
      expect(kp1.secretKey.bytes).toEqual(kp2.secretKey.bytes);
    });
  });
  
  describe('Encryption/Decryption (ML-KEM)', () => {
    let kemKeyPair: KeyPair;
    
    beforeEach(async () => {
      kemKeyPair = await crypto.generateKeyPair('ML-KEM-768');
      keyPairs.push(kemKeyPair);
    });
    
    it('should encrypt and decrypt data', async () => {
      const plaintext = new TextEncoder().encode('Hello, Quantum World!');
      
      const encrypted = await crypto.encrypt(kemKeyPair.publicKey, plaintext);
      
      expect(encrypted.ciphertext).toBeDefined();
      expect(encrypted.ciphertext).not.toEqual(plaintext);
      expect(encrypted.algorithm).toBe('ML-KEM-768');
      
      const decrypted = await crypto.decrypt(kemKeyPair.secretKey, encrypted);
      
      expect(decrypted).toEqual(plaintext);
    });
    
    it('should handle large payloads', async () => {
      const largeData = testUtils.generateRandomBytes(1024 * 1024); // 1MB
      
      const encrypted = await crypto.encrypt(kemKeyPair.publicKey, largeData);
      const decrypted = await crypto.decrypt(kemKeyPair.secretKey, encrypted);
      
      expect(decrypted).toEqual(largeData);
    });
    
    it('should produce different ciphertexts for same plaintext', async () => {
      const plaintext = new TextEncoder().encode('Repeated message');
      
      const encrypted1 = await crypto.encrypt(kemKeyPair.publicKey, plaintext);
      const encrypted2 = await crypto.encrypt(kemKeyPair.publicKey, plaintext);
      
      expect(encrypted1.ciphertext).not.toEqual(encrypted2.ciphertext);
      
      // But both should decrypt to same plaintext
      const decrypted1 = await crypto.decrypt(kemKeyPair.secretKey, encrypted1);
      const decrypted2 = await crypto.decrypt(kemKeyPair.secretKey, encrypted2);
      
      expect(decrypted1).toEqual(decrypted2);
      expect(decrypted1).toEqual(plaintext);
    });
    
    it('should fail decryption with wrong key', async () => {
      const wrongKeyPair = await crypto.generateKeyPair('ML-KEM-768');
      keyPairs.push(wrongKeyPair);
      
      const plaintext = new TextEncoder().encode('Secret message');
      const encrypted = await crypto.encrypt(kemKeyPair.publicKey, plaintext);
      
      await expect(crypto.decrypt(wrongKeyPair.secretKey, encrypted))
        .rejects.toThrow('Decryption failed');
    });
  });
  
  describe('Signing/Verification (ML-DSA)', () => {
    let dsaKeyPair: KeyPair;
    
    beforeEach(async () => {
      dsaKeyPair = await crypto.generateKeyPair('ML-DSA-65');
      keyPairs.push(dsaKeyPair);
    });
    
    it('should sign and verify messages', async () => {
      const message = new TextEncoder().encode('Sign this message');
      
      const signature = await crypto.sign(dsaKeyPair.secretKey, message);
      
      expect(signature.bytes).toBeDefined();
      expect(signature.algorithm).toBe('ML-DSA-65');
      expect(signature.bytes).toHaveLength(3309); // ML-DSA-65 signature size
      
      const isValid = await crypto.verify(dsaKeyPair.publicKey, message, signature);
      expect(isValid).toBe(true);
    });
    
    it('should reject invalid signatures', async () => {
      const message = new TextEncoder().encode('Original message');
      const signature = await crypto.sign(dsaKeyPair.secretKey, message);
      
      // Tamper with message
      const tamperedMessage = new TextEncoder().encode('Tampered message');
      const isValid = await crypto.verify(dsaKeyPair.publicKey, tamperedMessage, signature);
      expect(isValid).toBe(false);
    });
    
    it('should reject signatures from wrong key', async () => {
      const wrongKeyPair = await crypto.generateKeyPair('ML-DSA-65');
      keyPairs.push(wrongKeyPair);
      
      const message = new TextEncoder().encode('Message');
      const signature = await crypto.sign(dsaKeyPair.secretKey, message);
      
      const isValid = await crypto.verify(wrongKeyPair.publicKey, message, signature);
      expect(isValid).toBe(false);
    });
    
    it('should handle streaming signatures for large data', async () => {
      const signer = await crypto.createStreamingSigner(dsaKeyPair.secretKey);
      
      // Feed data in chunks
      for (let i = 0; i < 1000; i++) {
        const chunk = testUtils.generateRandomBytes(1024);
        await signer.update(chunk);
      }
      
      const signature = await signer.finalize();
      
      // Verify with streaming verifier
      const verifier = await crypto.createStreamingVerifier(dsaKeyPair.publicKey);
      
      // Feed same data
      for (let i = 0; i < 1000; i++) {
        const chunk = testUtils.generateRandomBytes(1024);
        await verifier.update(chunk);
      }
      
      const isValid = await verifier.verify(signature);
      expect(isValid).toBe(true);
    });
  });
  
  describe('Key Derivation and Exchange', () => {
    it('should derive shared secret using ML-KEM', async () => {
      const alice = await crypto.generateKeyPair('ML-KEM-768');
      const bob = await crypto.generateKeyPair('ML-KEM-768');
      keyPairs.push(alice, bob);
      
      // Alice encapsulates for Bob
      const { ciphertext, sharedSecret: aliceSecret } = 
        await crypto.encapsulate(bob.publicKey);
      
      // Bob decapsulates
      const bobSecret = await crypto.decapsulate(bob.secretKey, ciphertext);
      
      // Shared secrets should match
      expect(aliceSecret.bytes).toEqual(bobSecret.bytes);
      expect(aliceSecret.bytes).toHaveLength(32); // 256-bit shared secret
    });
    
    it('should derive multiple keys from shared secret', async () => {
      const alice = await crypto.generateKeyPair('ML-KEM-768');
      const bob = await crypto.generateKeyPair('ML-KEM-768');
      keyPairs.push(alice, bob);
      
      const { ciphertext, sharedSecret } = await crypto.encapsulate(bob.publicKey);
      
      // Derive multiple keys
      const keys = await crypto.deriveKeys(sharedSecret, {
        encryptionKey: 32,
        authenticationKey: 32,
        ivKey: 16
      });
      
      expect(keys.encryptionKey).toHaveLength(32);
      expect(keys.authenticationKey).toHaveLength(32);
      expect(keys.ivKey).toHaveLength(16);
      
      // Keys should be different
      expect(keys.encryptionKey).not.toEqual(keys.authenticationKey);
    });
  });
  
  describe('Hybrid Encryption', () => {
    it('should support hybrid encryption scheme', async () => {
      const recipientKem = await crypto.generateKeyPair('ML-KEM-768');
      const senderDsa = await crypto.generateKeyPair('ML-DSA-65');
      keyPairs.push(recipientKem, senderDsa);
      
      const plaintext = new TextEncoder().encode('Hybrid encrypted message');
      
      // Encrypt with hybrid scheme (KEM + signature)
      const encrypted = await crypto.hybridEncrypt({
        recipientPublicKey: recipientKem.publicKey,
        senderSecretKey: senderDsa.secretKey,
        plaintext
      });
      
      expect(encrypted.kemCiphertext).toBeDefined();
      expect(encrypted.dataEncrypted).toBeDefined();
      expect(encrypted.signature).toBeDefined();
      
      // Decrypt and verify
      const decrypted = await crypto.hybridDecrypt({
        recipientSecretKey: recipientKem.secretKey,
        senderPublicKey: senderDsa.publicKey,
        encrypted
      });
      
      expect(decrypted).toEqual(plaintext);
    });
  });
  
  describe('Performance and Security', () => {
    it('should perform constant-time operations', () => {
      const message1 = new Uint8Array(1024).fill(0);
      const message2 = new Uint8Array(1024).fill(255);
      
      // Test constant-time comparison
      const compare = () => crypto.constantTimeCompare(message1, message2);
      
      expect(compare).toBeConstantTime(1000);
    });
    
    it('should clear sensitive data from memory', async () => {
      const keyPair = await crypto.generateKeyPair('ML-KEM-768');
      const secretKeyBytes = new Uint8Array(keyPair.secretKey.bytes);
      
      // Destroy the key
      keyPair.secretKey.destroy();
      
      // Original bytes should be zeroed
      expect(keyPair.secretKey.bytes.every(b => b === 0)).toBe(true);
      
      // Attempting to use destroyed key should fail
      await expect(crypto.sign(keyPair.secretKey, new Uint8Array([1, 2, 3])))
        .rejects.toThrow('Key has been destroyed');
    });
    
    it('should benchmark cryptographic operations', async () => {
      const kemKeyPair = await crypto.generateKeyPair('ML-KEM-768');
      const dsaKeyPair = await crypto.generateKeyPair('ML-DSA-65');
      keyPairs.push(kemKeyPair, dsaKeyPair);
      
      const message = testUtils.generateRandomBytes(1024);
      
      // Benchmark key generation
      const [, keyGenTime] = await testUtils.measureTime(
        () => crypto.generateKeyPair('ML-KEM-768')
      );
      expect(keyGenTime).toBeLessThan(100); // < 100ms
      
      // Benchmark encryption
      const [, encryptTime] = await testUtils.measureTime(
        () => crypto.encrypt(kemKeyPair.publicKey, message)
      );
      expect(encryptTime).toBeLessThan(10); // < 10ms
      
      // Benchmark signing
      const [, signTime] = await testUtils.measureTime(
        () => crypto.sign(dsaKeyPair.secretKey, message)
      );
      expect(signTime).toBeLessThan(20); // < 20ms
    });
  });
  
  describe('Error Handling', () => {
    it('should validate key algorithm compatibility', async () => {
      const kemKey = await crypto.generateKeyPair('ML-KEM-768');
      const dsaKey = await crypto.generateKeyPair('ML-DSA-65');
      keyPairs.push(kemKey, dsaKey);
      
      // Try to encrypt with DSA key (should fail)
      await expect(crypto.encrypt(dsaKey.publicKey, new Uint8Array([1, 2, 3])))
        .rejects.toThrow('Invalid key type for encryption');
      
      // Try to sign with KEM key (should fail)
      await expect(crypto.sign(kemKey.secretKey, new Uint8Array([1, 2, 3])))
        .rejects.toThrow('Invalid key type for signing');
    });
    
    it('should handle corrupted ciphertext', async () => {
      const keyPair = await crypto.generateKeyPair('ML-KEM-768');
      keyPairs.push(keyPair);
      
      const plaintext = new TextEncoder().encode('Test');
      const encrypted = await crypto.encrypt(keyPair.publicKey, plaintext);
      
      // Corrupt the ciphertext
      encrypted.ciphertext[0] ^= 0xFF;
      
      await expect(crypto.decrypt(keyPair.secretKey, encrypted))
        .rejects.toThrow('Decryption failed');
    });
    
    it('should validate input sizes', async () => {
      const keyPair = await crypto.generateKeyPair('ML-KEM-768');
      keyPairs.push(keyPair);
      
      // Empty plaintext
      await expect(crypto.encrypt(keyPair.publicKey, new Uint8Array(0)))
        .rejects.toThrow('Plaintext cannot be empty');
      
      // Oversized plaintext
      const oversized = new Uint8Array(100 * 1024 * 1024); // 100MB
      await expect(crypto.encrypt(keyPair.publicKey, oversized))
        .rejects.toThrow('Plaintext too large');
    });
  });
});