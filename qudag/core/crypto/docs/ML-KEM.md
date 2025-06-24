# ML-KEM-768 Implementation

This document describes the implementation of ML-KEM-768 (formerly Kyber-768) in the QuDAG protocol. ML-KEM-768 provides NIST security level 3 post-quantum key encapsulation.

## Overview

The implementation provides three main operations:
- Key Generation (keygen)
- Encapsulation (encapsulate)
- Decapsulation (decapsulate)

## Key Types and Sizes

All key material and cryptographic objects implement secure memory management:
- Automatic zeroization on drop using `ZeroizeOnDrop`
- Constant-time comparison operations
- Protected against memory reads after free

Key sizes:
- Public Key: 1184 bytes
- Secret Key: 2400 bytes
- Ciphertext: 1088 bytes
- Shared Secret: 32 bytes

## Security Features

### Constant-Time Operations

All cryptographic operations are implemented to be constant-time to prevent timing attacks:

1. Key comparison:
   - Uses `subtle::ConstantTimeEq` for comparing keys and secrets
   - Custom `constant_time_compare` function ensures length-independent timing
   - No early returns or branches based on secret data

2. Memory Management:
   - All sensitive types implement `Zeroize` and `ZeroizeOnDrop`
   - Secret keys are immediately zeroized when dropped
   - Temporary buffers are zeroized using `defer!` blocks
   - Public keys do not require zeroization (clearly marked with `#[zeroize(skip)]`)

### Error Handling

The implementation provides comprehensive error handling:

- Size validation for all inputs
- Distinct error types for different failure modes
- No error messages containing sensitive information
- All errors implement `std::error::Error`

## API Usage

### Key Generation

```rust
use qudag_crypto::ml_kem::MlKem768;
use qudag_crypto::kem::KeyEncapsulation;

// Generate new keypair
let (public_key, secret_key) = MlKem768::keygen()?;
```

### Encapsulation

```rust
// Encapsulate to generate shared secret and ciphertext
let (ciphertext, shared_secret) = MlKem768::encapsulate(&public_key)?;
```

### Decapsulation

```rust
// Decapsulate to recover shared secret
let shared_secret = MlKem768::decapsulate(&secret_key, &ciphertext)?;
```

## Testing

The implementation includes:

1. Unit Tests:
   - Basic functionality testing
   - NIST test vectors validation
   - Error case handling
   - Memory zeroization verification

2. Property-Based Tests:
   - Random input handling
   - Edge case exploration
   - Fuzzing of public inputs

3. Constant-Time Tests:
   - Validation of timing independence
   - Memory access pattern verification

## Security Considerations

1. Random Number Generation:
   - Uses OS-provided cryptographic RNG
   - Seed values are immediately zeroized after use
   - No static/fixed seeds in production code

2. Side-Channel Protection:
   - No branching on secret data
   - No variable-time operations on secrets
   - Protected against cache-timing attacks

3. Memory Safety:
   - No unsafe code blocks
   - Secure memory zeroization
   - Protected against memory leaks

## Performance Characteristics

The implementation is optimized for:
- Constant-time operation over performance
- Minimal memory allocations
- Efficient error handling
- Zero-copy operations where possible

Note: Performance benchmarks are available in the `benchmarks/` directory.