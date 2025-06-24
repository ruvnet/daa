# QuDAG Cryptographic API Documentation

The `qudag_crypto` module provides production-ready quantum-resistant cryptographic primitives for the QuDAG protocol. This comprehensive suite includes ML-KEM-768, ML-DSA, HQC, BLAKE3 hashing, and quantum fingerprinting implementations that are fully compliant with NIST post-quantum standards.

## Table of Contents

- [Overview](#overview)
- [ML-KEM (Key Encapsulation Mechanism)](#ml-kem-key-encapsulation-mechanism)
- [ML-DSA (Digital Signature Algorithm)](#ml-dsa-digital-signature-algorithm)
- [HQC (Hamming Quasi-Cyclic)](#hqc-hamming-quasi-cyclic)
- [Quantum Fingerprinting](#quantum-fingerprinting)
- [BLAKE3 Hashing](#blake3-hashing)
- [Migration Guide](#migration-guide)
- [Performance Characteristics](#performance-characteristics)
- [Security Properties](#security-properties)

## Overview

QuDAG implements a comprehensive suite of post-quantum cryptographic algorithms standardized by NIST and other leading cryptographic organizations. All implementations prioritize:

- **Quantum Resistance**: Protection against both classical and quantum attacks
- **Memory Safety**: Automatic secret zeroization and constant-time operations
- **Performance**: Optimized implementations with hardware acceleration where available
- **Compatibility**: Standards-compliant implementations following NIST specifications

### Supported Algorithms

| Algorithm | Type | Security Level | Standard | Status |
|-----------|------|----------------|----------|---------|
| **ML-KEM-768** | Key Encapsulation | NIST Level 3 | FIPS 203 | ✅ Production Ready |
| **ML-DSA** | Digital Signatures | NIST Level 3 | FIPS 204 | ✅ Production Ready |
| **HQC** | Public Key Encryption | 128/192/256-bit | NIST Round 4 | ✅ Production Ready |
| **BLAKE3** | Cryptographic Hash | 256-bit | RFC Draft | ✅ Production Ready |

### Dependencies

The implementation uses the following production cryptographic libraries:

```toml
[dependencies]
ml-kem = "0.2"              # NIST ML-KEM implementation
pqcrypto-dilithium = "0.5"  # ML-DSA (Dilithium) implementation
pqcrypto-traits = "0.3"     # Post-quantum crypto traits
blake3 = "1.3"              # BLAKE3 hashing
sha3 = "0.10"               # SHA3/SHAKE for ML-DSA
zeroize = "1.5"             # Memory zeroization
subtle = "2.4"              # Constant-time operations
rand = "0.8"                # Cryptographic randomness
```

## ML-KEM (Key Encapsulation Mechanism)

ML-KEM-768 provides quantum-resistant key encapsulation based on the Module-LWE problem. This implementation is fully compliant with NIST FIPS 203 and provides NIST Level 3 security.

### Basic Usage

```rust
use qudag_crypto::{ml_kem::MlKem768, kem::{KeyEncapsulation, KEMError}};

fn basic_kem_example() -> Result<(), KEMError> {
    // Generate a key pair
    let (public_key, secret_key) = MlKem768::keygen()?;
    
    // Encapsulate a shared secret
    let (ciphertext, shared_secret1) = MlKem768::encapsulate(&public_key)?;
    
    // Decapsulate the shared secret
    let shared_secret2 = MlKem768::decapsulate(&secret_key, &ciphertext)?;
    
    // Verify both parties have the same secret
    assert_eq!(shared_secret1.as_bytes(), shared_secret2.as_bytes());
    Ok(())
}
```

### MlKem768 Implementation

The main ML-KEM-768 implementation with NIST Level 3 security.

```rust
pub struct MlKem768;

impl MlKem768 {
    pub const PUBLIC_KEY_SIZE: usize = 1184;
    pub const SECRET_KEY_SIZE: usize = 2400;
    pub const CIPHERTEXT_SIZE: usize = 1088;
    pub const SHARED_SECRET_SIZE: usize = 32;
    pub const SECURITY_LEVEL: u8 = 3;
    pub const CACHE_SIZE: usize = 1024;
}
```

### Methods

#### `keygen() -> Result<(PublicKey, SecretKey), KEMError>`

Generates a new ML-KEM-768 key pair using secure randomness.

```rust
let (pk, sk) = MlKem768::keygen()?;
println!("Public key: {} bytes", pk.as_bytes().len());
println!("Secret key: {} bytes", sk.as_bytes().len());
```

#### `encapsulate(public_key: &PublicKey) -> Result<(Ciphertext, SharedSecret), KEMError>`

Encapsulates a random shared secret using the provided public key.

```rust
let (ciphertext, shared_secret) = MlKem768::encapsulate(&public_key)?;
println!("Ciphertext: {} bytes", ciphertext.as_bytes().len());
println!("Shared secret: {} bytes", shared_secret.as_bytes().len());
```

#### `decapsulate(secret_key: &SecretKey, ciphertext: &Ciphertext) -> Result<SharedSecret, KEMError>`

Decapsulates the shared secret using the secret key and ciphertext.

```rust
let shared_secret = MlKem768::decapsulate(&secret_key, &ciphertext)?;
assert_eq!(shared_secret.as_bytes().len(), MlKem768::SHARED_SECRET_SIZE);
```

### Performance Metrics

The implementation provides performance metrics for monitoring:

```rust
pub struct Metrics {
    pub key_cache_misses: u64,
    pub key_cache_hits: u64,
    pub avg_decap_time_ns: u64,
}

let metrics = MlKem768::get_metrics();
println!("Cache hits: {}", metrics.key_cache_hits);
```

### ML-KEM Key Types

```rust
pub struct PublicKey([u8; MlKem768::PUBLIC_KEY_SIZE]);
pub struct SecretKey([u8; MlKem768::SECRET_KEY_SIZE]);
pub struct Ciphertext([u8; MlKem768::CIPHERTEXT_SIZE]);
pub struct SharedSecret([u8; MlKem768::SHARED_SECRET_SIZE]);
```

### KeyEncapsulation Trait

```rust
pub trait KeyEncapsulation {
    type PublicKey;
    type SecretKey;
    type Ciphertext;
    type SharedSecret;
    type Error;

    fn keygen(&self) -> Result<(Self::PublicKey, Self::SecretKey), Self::Error>;
    fn encap(&self, pk: &Self::PublicKey) -> Result<(Self::Ciphertext, Self::SharedSecret), Self::Error>;
    fn decap(&self, sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::SharedSecret, Self::Error>;
}
```

### Performance Metrics

```rust
pub struct MlKemMetrics {
    pub avg_decap_time_ns: u64,
    pub key_cache_hits: u64,
    pub key_cache_misses: u64,
}
```

## Digital Signatures (ML-DSA)

### MlDsaKeyPair

A key pair for quantum-resistant digital signatures using ML-DSA.

```rust
pub struct MlDsaKeyPair {
    // private fields
}

impl MlDsaKeyPair {
    pub fn generate() -> Result<Self, MlDsaError>;
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, MlDsaError>;
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), MlDsaError>;
}
```

### MlDsaPublicKey

The public portion of an ML-DSA key pair used for signature verification.

```rust
pub struct MlDsaPublicKey {
    // private fields
}
```

## Hash Functions

### BLAKE3 Implementation

```rust
pub trait HashFunction {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
    fn hash_with_key(&self, data: &[u8], key: &[u8]) -> Vec<u8>;
    fn verify_hash(&self, data: &[u8], hash: &[u8]) -> bool;
}
```

## Quantum Fingerprinting

### QuantumFingerprint

Data fingerprinting using ML-DSA for authentication.

```rust
pub struct QuantumFingerprint {
    // Implementation details
}

impl QuantumFingerprint {
    pub fn new() -> Self;
    pub fn fingerprint(&self, data: &[u8]) -> Result<Vec<u8>, FingerprintError>;
    pub fn verify(&self, data: &[u8], fingerprint: &[u8]) -> Result<bool, FingerprintError>;
}
```

## Error Types

### KEMError

```rust
pub enum KEMError {
    KeyGenError,
    EncapsulationError,
    DecapsulationError,
    InvalidKey,
    InvalidParameters,
    OperationFailed,
    InternalError,
}
```

### MlDsaError

```rust
pub enum MlDsaError {
    InvalidKeyFormat(String),
    SigningFailed(String),
    VerificationFailed(String),
}
```

### CryptoError

Main error type for the crypto module:

```rust
pub enum CryptoError {
    KemError(KEMError),
    SignatureError(SignatureError),
    HashError(String),
    FingerprintError(String),
}
```

## Example Usage

### ML-KEM Key Encapsulation

```rust
use qudag_crypto::{MlKem768, KeyEncapsulation};

// Initialize ML-KEM
let kem = MlKem768;

// Generate key pair
let (public_key, secret_key) = kem.keygen()?;

// Encapsulation (sender side)
let (ciphertext, shared_secret1) = kem.encap(&public_key)?;

// Decapsulation (receiver side)
let shared_secret2 = kem.decap(&secret_key, &ciphertext)?;

// Both sides now have the same shared secret
assert_eq!(shared_secret1, shared_secret2);
```

### ML-DSA Digital Signatures

```rust
use qudag_crypto::{MlDsaKeyPair, MlDsaError};

// Generate a new key pair
let keypair = MlDsaKeyPair::generate()?;

// Sign a message
let message = b"Hello, quantum-resistant world!";
let signature = keypair.sign(message)?;

// Verify the signature
keypair.verify(message, &signature)?;
```

### Quantum Fingerprinting

```rust
use qudag_crypto::QuantumFingerprint;

// Create fingerprint generator
let fingerprinter = QuantumFingerprint::new();

// Generate fingerprint for data
let data = b"Important protocol data";
let fingerprint = fingerprinter.fingerprint(data)?;

// Verify fingerprint later
let is_valid = fingerprinter.verify(data, &fingerprint)?;
assert!(is_valid);
```

### BLAKE3 Hashing

```rust
use qudag_crypto::HashFunction;

// Create BLAKE3 hasher
let hasher = Blake3::new();

// Hash data
let data = b"Data to hash";
let hash = hasher.hash(data);

// Hash with key for authentication
let key = b"secret key for authenticated hashing";
let auth_hash = hasher.hash_with_key(data, key);

// Verify hash
let is_valid = hasher.verify_hash(data, &hash);
assert!(is_valid);
```

### Error Handling

```rust
use qudag_crypto::{CryptoError, KEMError, MlDsaError};

fn handle_crypto_operations() -> Result<(), CryptoError> {
    let kem = MlKem768;
    
    match kem.keygen() {
        Ok((pk, sk)) => {
            // Use keys
            let (ct, ss) = kem.encap(&pk)?;
            let ss2 = kem.decap(&sk, &ct)?;
            assert_eq!(ss, ss2);
        }
        Err(KEMError::KeyGenError) => {
            eprintln!("Key generation failed - check entropy source");
            return Err(CryptoError::KemError(KEMError::KeyGenError));
        }
        Err(e) => return Err(CryptoError::KemError(e)),
    }
    
    Ok(())
}
```

### Performance Monitoring

```rust
use qudag_crypto::{MlKem768, MlKemMetrics};

fn monitor_kem_performance() {
    let kem = MlKem768;
    let (pk, sk) = kem.keygen().unwrap();
    
    // Perform operations and collect metrics
    let start = std::time::Instant::now();
    let (ct, _) = kem.encap(&pk).unwrap();
    let _ = kem.decap(&sk, &ct).unwrap();
    let elapsed = start.elapsed();
    
    println!("KEM operation took: {:?}", elapsed);
    
    // Access metrics if available
    let metrics = kem.get_metrics(); // hypothetical method
    println!("Cache hits: {}", metrics.key_cache_hits);
    println!("Average decap time: {}ns", metrics.avg_decap_time_ns);
}
```

## Security Considerations

### 1. Memory Management

All cryptographic types implement `ZeroizeOnDrop` to ensure sensitive data is securely cleared:

- **Secret Keys**: Automatically zeroized when dropped
- **Shared Secrets**: Memory cleared after use
- **Intermediate Values**: Temporary crypto values are zeroized
- **Error Handling**: No sensitive data leaked in error messages

```rust
use zeroize::Zeroize;

// Secret data is automatically cleared
{
    let secret_key = kem.keygen()?.1;
    // ... use secret key
} // secret_key memory is zeroized here
```

### 2. Constant-Time Operations

All implementations use constant-time algorithms to prevent timing attacks:

- **ML-KEM Operations**: Decapsulation runs in constant time
- **ML-DSA Signatures**: Signing and verification are constant-time
- **Comparisons**: All secret comparisons use `subtle::ConstantTimeEq`
- **Side-Channel Resistance**: No conditional branches on secret data

### 3. Post-Quantum Security

The implemented algorithms provide security against both classical and quantum attacks:

- **ML-KEM-768**: NIST security level 3 (equivalent to AES-256)
- **ML-DSA**: Quantum-resistant digital signatures
- **BLAKE3**: Quantum-resistant cryptographic hashing
- **Forward Secrecy**: Each session uses fresh key material

### 4. Key Management

Best practices for key handling:

```rust
// Good: Generate fresh keys for each session
let (pk, sk) = kem.keygen()?;

// Bad: Reusing keys across contexts
// Don't do this - generate fresh keys instead

// Good: Clear sensitive data explicitly if needed
let mut sensitive_data = get_sensitive_data();
sensitive_data.zeroize();
```

### 5. Random Number Generation

All key generation uses cryptographically secure randomness:

- **Entropy Source**: Uses system entropy for key generation
- **CSPRNG**: Cryptographically secure pseudo-random generation
- **Seed Security**: No predictable or weak entropy sources

### 6. Algorithm Parameters

Current cryptographic parameters provide quantum security:

```rust
// ML-KEM-768 parameters (security level 3)
MlKem768::PUBLIC_KEY_SIZE   // 1184 bytes
MlKem768::SECRET_KEY_SIZE   // 2400 bytes  
MlKem768::CIPHERTEXT_SIZE   // 1088 bytes
MlKem768::SHARED_SECRET_SIZE // 32 bytes

// Security equivalent to AES-256 against quantum attacks
```

### 7. Error Handling Security

Error messages are designed to prevent information leakage:

```rust
// Good: Generic error messages
match crypto_operation() {
    Err(CryptoError::KemError(_)) => {
        // Log internally but don't expose details
        log::warn!("Cryptographic operation failed");
        return Err("Operation failed");
    }
}
```

### 8. Performance vs Security

The implementation prioritizes security over performance:

- **Constant-Time**: All operations run in constant time
- **Memory Clearing**: Extra cycles spent clearing sensitive memory
- **Side-Channel Resistance**: Additional protections against attacks
- **Quantum Security**: Larger key sizes for post-quantum resistance

## Configuration

### Default Security Parameters

The cryptographic parameters are preset for maximum security:

```rust
// Default configuration provides:
// - NIST security level 3 (equivalent to AES-256)
// - Quantum resistance for both encryption and signatures
// - Side-channel attack resistance
// - Forward secrecy for session keys
```

### Performance Tuning

For performance-critical applications, monitor metrics:

```rust
// Enable performance monitoring
let metrics = kem.get_metrics();
if metrics.avg_decap_time_ns > PERFORMANCE_THRESHOLD {
    // Consider caching or optimization
}
```

### Integration Guidelines

When integrating with the QuDAG protocol:

1. **Use ML-KEM** for key establishment between nodes
2. **Use ML-DSA** for message authentication and non-repudiation  
3. **Use BLAKE3** for general hashing and data integrity
4. **Use Quantum Fingerprinting** for data authentication in DAG
5. **Generate fresh keys** for each session or communication channel