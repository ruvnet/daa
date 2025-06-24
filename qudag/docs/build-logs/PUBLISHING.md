# Publishing QuDAG to crates.io

This guide explains how to publish QuDAG to crates.io.

## Prerequisites

1. **Verified crates.io account**
   - Go to https://crates.io/settings/profile
   - Verify your email address
   - Generate an API token

2. **Environment setup**
   - Set `CRATES_API_KEY` in your environment or `.env` file
   - Ensure all packages compile: `cargo check --workspace --no-default-features`

## Quick Publishing

### Option 1: Automated Script (Recommended)
```bash
./publish-manual.sh
```

This interactive script will:
- Verify prerequisites
- Build each package
- Ask for confirmation before each publish
- Handle dependency order automatically
- Wait between publishes for crates.io to update

### Option 2: Manual Step-by-Step

Publish in this exact order (due to dependencies):

```bash
# 1. Crypto (no dependencies)
cd core/crypto
cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty
sleep 45

# 2. DAG (depends on crypto)
cd ../dag
cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty
sleep 45

# 3. Network (depends on crypto)  
cd ../network
cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty
sleep 45

# 4. Protocol (depends on crypto, dag, network)
cd ../protocol
cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty
sleep 45

# 5. Main QuDAG crate (depends on all)
cd ../../qudag
cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty
sleep 45

# 6. CLI tool (depends on all)
cd ../tools/cli
cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty
```

## Published Crates

After publishing, the following crates will be available:

| Crate | Purpose | Installation | Documentation |
|-------|---------|--------------|---------------|
| `qudag` | Main library | `cargo add qudag` | [docs.rs/qudag](https://docs.rs/qudag) |
| `qudag-cli` | CLI tool | `cargo install qudag-cli` | [docs.rs/qudag-cli](https://docs.rs/qudag-cli) |
| `qudag-crypto` | Cryptography | `cargo add qudag-crypto` | [docs.rs/qudag-crypto](https://docs.rs/qudag-crypto) |
| `qudag-dag` | DAG consensus | `cargo add qudag-dag` | [docs.rs/qudag-dag](https://docs.rs/qudag-dag) |
| `qudag-network` | P2P networking | `cargo add qudag-network` | [docs.rs/qudag-network](https://docs.rs/qudag-network) |
| `qudag-protocol` | Protocol coordination | `cargo add qudag-protocol` | [docs.rs/qudag-protocol](https://docs.rs/qudag-protocol) |

### ✅ Complete Documentation

Each crate now includes:
- Comprehensive README with examples
- API documentation on docs.rs
- Installation instructions
- Feature explanations
- Usage examples

## Troubleshooting

### Email Verification Required
```
error: A verified email address is required to publish crates to crates.io
```
**Solution:** Go to https://crates.io/settings/profile and verify your email.

### Dependency Not Found
```
error: no matching package named `qudag-crypto` found
```
**Solution:** Wait 30-60 seconds after publishing each crate for crates.io to update.

### Invalid Token
```
error: failed to authenticate to registry
```
**Solution:** Generate a new API token at https://crates.io/settings/tokens

### Version Already Exists
```
error: crate version `0.1.0` is already uploaded
```
**Solution:** This is expected if the crate was already published. Skip this crate.

### Build Failures
All packages are configured to build without default features to avoid optimization module compilation issues.

## Version Management

- All crates use version `0.1.0` for the initial release
- Version is managed centrally in the workspace `Cargo.toml`
- For future releases, update the version in one place

## Package Features

### Default Features (Disabled for Publishing)
We disable default features during publishing to avoid compilation issues with optimization modules.

Users can enable features when using the crates:
```toml
[dependencies]
qudag = { version = "0.1.0", features = ["optimizations"] }
```

## Post-Publishing Verification

After publishing, verify the packages work:

```bash
# Test library installation
cargo new test-qudag
cd test-qudag
cargo add qudag
echo 'fn main() { println!("QuDAG installed!"); }' > src/main.rs
cargo run

# Test CLI installation
cargo install qudag-cli
qudag-cli --help
```

## Links

- [Main QuDAG crate](https://crates.io/crates/qudag)
- [QuDAG CLI](https://crates.io/crates/qudag-cli)
- [Documentation](https://docs.rs/qudag)
- [Repository](https://github.com/ruvnet/QuDAG)