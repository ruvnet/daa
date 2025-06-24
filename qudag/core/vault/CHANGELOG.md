# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-21

### Added

- Initial release of qudag-vault
- Quantum-resistant password vault using ML-KEM (Kyber) and ML-DSA (Dilithium)
- AES-256-GCM encryption with Argon2id key derivation
- DAG-based secret storage for hierarchical organization
- Automatic memory zeroization for sensitive data
- Secure password generation utilities
- Vault export/import functionality
- Comprehensive test suite with unit and integration tests
- Performance benchmarks for cryptographic operations

### Security

- Post-quantum cryptography implementation
- Secure key derivation with Argon2id
- Memory protection with automatic zeroization
- Authenticated encryption with AES-GCM

[0.1.0]: https://github.com/ruvnet/QuDAG/releases/tag/qudag-vault-v0.1.0