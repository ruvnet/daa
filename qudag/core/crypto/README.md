# QuDAG Crypto

Quantum-resistant cryptographic primitives for the QuDAG protocol.

## Features

- **ML-KEM-768**: NIST-standardized key encapsulation mechanism (FIPS 203)
- **ML-DSA (Dilithium)**: Digital signature algorithm (FIPS 204) 
- **HQC**: Hamming Quasi-Cyclic code-based encryption (128/192/256-bit)
- **BLAKE3**: Fast cryptographic hash function
- **Quantum Fingerprinting**: Data fingerprinting using ML-DSA signatures
- **Memory Safety**: Automatic secret zeroization and constant-time operations

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
qudag-crypto = "0.1"
```

## Examples

### ML-KEM Key Encapsulation

```rust
use qudag_crypto::ml_kem::MlKem768;

// Generate keypair
let (public_key, secret_key) = MlKem768::keygen()?;

// Encapsulate to get shared secret
let (ciphertext, shared_secret1) = MlKem768::encapsulate(&public_key)?;

// Decapsulate to recover shared secret
let shared_secret2 = MlKem768::decapsulate(&secret_key, &ciphertext)?;

assert_eq!(shared_secret1.as_bytes(), shared_secret2.as_bytes());
```

### ML-DSA Digital Signatures

```rust
use qudag_crypto::ml_dsa::{MlDsa, MlDsaKeyPair};

// Generate signing keypair
let keypair = MlDsaKeyPair::generate()?;
let message = b"Hello, quantum-resistant world!";

// Sign message
let signature = keypair.sign(message)?;

// Verify signature
assert!(keypair.public_key().verify(message, &signature)?);
```

### BLAKE3 Hashing

```rust
use qudag_crypto::hash::HashFunction;

let data = b"Data to hash";
let hash = HashFunction::hash(data);
println!("Hash: {}", hex::encode(hash.as_bytes()));
```

### Quantum Fingerprinting

```rust
use qudag_crypto::fingerprint::Fingerprint;

let data = b"Important data";
let fingerprint = Fingerprint::generate(data)?;

// Verify data integrity
assert!(fingerprint.verify(data)?);
```

## Security Features

- **Post-Quantum Security**: Resistant to attacks from quantum computers
- **Memory Safety**: All secret data is automatically zeroized
- **Side-Channel Resistance**: Constant-time implementations
- **NIST Compliance**: Implements FIPS 203 and FIPS 204 standards

## Supported Algorithms

| Algorithm | Type | Security Level | Standard |
|-----------|------|----------------|----------|
| ML-KEM-768 | Key Encapsulation | NIST Level 3 | FIPS 203 |
| ML-DSA | Digital Signatures | NIST Level 3 | FIPS 204 |
| HQC-128/192/256 | Encryption | 128/192/256-bit | NIST Round 4 |
| BLAKE3 | Hash Function | 256-bit | RFC Draft |

## Performance

QuDAG Crypto is optimized for performance with:
- SIMD acceleration where available
- Efficient memory management
- Minimal allocation overhead
- Hardware feature detection

## Documentation

- [API Documentation](https://docs.rs/qudag-crypto)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)

## License

Licensed under either MIT or Apache-2.0 at your option.