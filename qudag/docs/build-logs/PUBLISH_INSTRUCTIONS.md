# QuDAG Vault Publishing Instructions

## Prerequisites

1. **Get a crates.io API token**:
   - Visit https://crates.io/me
   - Generate a new API token
   - Set it as an environment variable:
   ```bash
   export CARGO_REGISTRY_TOKEN=your_token_here
   ```

2. **Verify your setup**:
   ```bash
   cargo login
   # Or manually set the token
   ```

## Version Updates Applied

- ✅ Workspace version updated to `0.2.0`
- ✅ Vault library version updated to `0.2.0` 
- ✅ CLI dependencies updated to use `0.2.0`
- ✅ Made QuDAG dependencies optional with `qudag-integration` feature

## Publishing Steps

### Option 1: Automated Publishing (Recommended)

```bash
# Set your token
export CARGO_REGISTRY_TOKEN=your_crates_io_token

# Run the automated script
./publish-vault-simple.sh
```

### Option 2: Manual Publishing

#### Step 1: Publish Vault Library

```bash
cd core/vault

# Build and test without QuDAG dependencies
cargo build --no-default-features
cargo test --no-default-features --lib

# Generate docs
cargo doc --no-deps --no-default-features

# Dry run
cargo publish --dry-run --no-default-features

# Publish
cargo publish --no-default-features
```

#### Step 2: Publish CLI (Optional)

```bash
cd ../../tools/cli

# Build CLI 
cargo build

# Test CLI
cargo test

# Publish CLI (requires vault to be published first)
cargo publish
```

## Features Available

### Vault Library (`qudag-vault-core`)

- **Default**: Standalone mode with basic AES-256-GCM encryption
- **qudag-integration**: Full QuDAG integration with ML-KEM/ML-DSA
- **enterprise**: Advanced features (RBAC, audit logging, MFA)

### CLI Tool (`qudag-cli`)

- Full vault integration with all commands
- Password generation and management
- Export/import functionality
- Configuration management

## Usage After Publishing

### For Rust Developers

```bash
# Add to Cargo.toml
cargo add qudag-vault-core

# With QuDAG integration
cargo add qudag-vault-core --features qudag-integration
```

### For End Users

```bash
# Install CLI tool
cargo install qudag-cli

# Use vault commands
qudag vault generate --length 16
qudag vault config show
```

## Troubleshooting

### Common Issues

1. **Token Issues**: Ensure `CARGO_REGISTRY_TOKEN` is set correctly
2. **Version Conflicts**: All workspace versions should be `0.2.0`
3. **Feature Compilation**: Use `--no-default-features` for standalone publishing

### Publishing Dependencies

- The vault library can be published independently in standalone mode
- CLI requires vault library to be published first
- QuDAG integration features require core libraries to be published

## Post-Publishing

1. **Update README badges** with new version numbers
2. **Create GitHub release** with changelog
3. **Update documentation** on docs.rs
4. **Announce release** to the community

## Security Notes

- Never commit API tokens to the repository
- Use `.env` file for local development (gitignored)
- Vault provides quantum-resistant security when `qudag-integration` feature is enabled