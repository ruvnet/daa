# QuDAG Vault

[![Crates.io](https://img.shields.io/crates/v/qudag-vault-core.svg)](https://crates.io/crates/qudag-vault-core)
[![Documentation](https://docs.rs/qudag-vault-core/badge.svg)](https://docs.rs/qudag-vault-core)
[![License](https://img.shields.io/crates/l/qudag-vault-core.svg)](https://github.com/ruvnet/QuDAG/blob/main/LICENSE)
[![Downloads](https://img.shields.io/crates/d/qudag-vault-core.svg)](https://crates.io/crates/qudag-vault-core)

A quantum-resistant password vault library with post-quantum cryptography for secure password management.

## Features

- 🔐 **Quantum-Resistant**: Uses ML-KEM (Kyber) and ML-DSA (Dilithium) for post-quantum security
- 🛡️ **Strong Encryption**: AES-256-GCM with Argon2id key derivation
- 🔒 **Memory Safety**: Automatic zeroization of sensitive data in memory
- 📊 **DAG Storage**: Organize secrets in a directed acyclic graph structure
- 🔑 **Password Generation**: Secure random password generation
- 📦 **Backup/Restore**: Encrypted vault export and import functionality
- ⚡ **Fast Performance**: Optimized cryptographic operations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qudag-vault-core = "0.1.0"
```

Or install via cargo:

```bash
cargo add qudag-vault-core
```

## Architecture

The vault library is designed as a separate crate within the QuDAG workspace, ensuring clean separation of concerns while leveraging QuDAG's powerful cryptographic and DAG modules.

### Key Components

- **`vault.rs`** - Main vault implementation providing high-level API
- **`crypto.rs`** - Integration with QuDAG's quantum-resistant crypto
- **`dag_storage.rs`** - DAG-based secret storage using QuDAG's DAG module
- **`kdf.rs`** - Argon2id key derivation for master password
- **`secret.rs`** - Secret entry types with automatic memory zeroization
- **`error.rs`** - Error handling with proper error chain mapping
- **`utils.rs`** - Password generation and utility functions

## Quick Start

```rust
use qudag_vault_core::Vault;

// Create a new vault
let mut vault = Vault::create("vault.qdag", "master_password")?;

// Add a secret with auto-generated password
vault.add_secret("github", "myusername", None)?;

// Add a secret with specific password
vault.add_secret("email", "user@example.com", Some("my_password"))?;

// Retrieve a secret
let secret = vault.get_secret("github")?;
println!("Username: {}", secret.username);
println!("Password: {}", secret.password.as_str());

// List all secrets
let secrets = vault.list_secrets(None)?;
for label in secrets {
    println!("- {}", label);
}

// Export vault for backup
vault.export("backup.qdag")?;
```

## Advanced Usage

### Password Generation

```rust
use qudag_vault_core::utils::generate_password;

// Generate a secure password
let password = generate_password(16, true, true, true)?;
```

### Hierarchical Organization

```rust
// Organize secrets in categories
vault.add_secret("work/email", "user@company.com", None)?;
vault.add_secret("work/github", "work_account", None)?;
vault.add_secret("personal/email", "user@personal.com", None)?;

// List secrets in a category
let work_secrets = vault.list_secrets(Some("work"))?;
```

### CLI Usage

QuDAG Vault is also available through the QuDAG CLI tool:

```bash
# Install QuDAG CLI
cargo install qudag-cli

# Generate secure passwords
qudag vault generate --length 16 --count 3

# Initialize a vault
qudag vault init

# Add passwords
qudag vault add github --username myuser

# Retrieve passwords
qudag vault get github

# List all entries
qudag vault list --format json

# Export vault for backup
qudag vault export ~/vault-backup.qdag

# Show vault configuration
qudag vault config show
```


## Security Features

### Cryptographic Foundation
- **Post-Quantum Security**: ML-KEM-768 (Kyber) and ML-DSA (Dilithium-3) implementations
- **Authenticated Encryption**: AES-256-GCM with 96-bit nonces
- **Key Derivation**: Argon2id with 64MB memory, 3 iterations, and random salt
- **Hashing**: BLAKE3 for fast, quantum-resistant hashing

### Memory Protection
- **Automatic Zeroization**: All sensitive data cleared from memory using `zeroize` crate
- **Secure String Handling**: Password strings implement `ZeroizeOnDrop`
- **Stack Protection**: Cryptographic operations use stack-allocated buffers when possible

### Storage Security
- **File Encryption**: Vault files encrypted with derived master key
- **DAG Integrity**: Each secret stored as a verified DAG node with cryptographic links
- **Backup Encryption**: Export files use independent encryption with key verification

### Performance & Safety
- **Constant-Time Operations**: Timing attack resistant password comparison
- **Memory-Safe**: Zero unsafe code, leveraging Rust's ownership system
- **Concurrent Safety**: Thread-safe operations with proper locking

## Integration with QuDAG

The vault library seamlessly integrates with QuDAG's infrastructure:

- Uses `qudag-crypto` for all cryptographic operations
- Leverages `qudag-dag` for DAG-based storage
- Compatible with QuDAG's quantum-resistant security model
- Follows QuDAG's coding standards and patterns

## Building

```bash
# Build the vault library
cargo build -p qudag-vault-core

# Run tests
cargo test -p qudag-vault-core

# Run benchmarks
cargo bench -p qudag-vault-core
```

## Future Enhancements

- [ ] Node.js bindings via napi-rs
- [ ] Python bindings via PyO3
- [ ] Biometric authentication
- [ ] Role-based access control (RBAC)
- [ ] Audit logging with Dilithium signatures
- [ ] Distributed vault with DAG consensus
- [ ] Hardware security module (HSM) support
- [ ] Browser extension

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Acknowledgments

This project is part of the QuDAG ecosystem, building quantum-resistant distributed systems for the future.