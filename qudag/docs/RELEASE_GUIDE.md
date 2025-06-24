# QuDAG Release Guide

This guide explains how to publish updated versions of QuDAG to crates.io.

## 🚀 Quick Release (Automated)

Use the automated release script for version bumping and publishing:

```bash
./release.sh
```

The script will:
1. Show current version (0.1.0)
2. Suggest patch (0.1.1) or minor (0.2.0) releases
3. Update all Cargo.toml files and READMEs
4. Build and test the workspace
5. Publish all crates in dependency order
6. Create a git tag

## 📋 Manual Release Process

If you prefer manual control:

### 1. Update Version

```bash
# Current version: 0.1.0
# For documentation fixes (patch): 0.1.1
# For new features (minor): 0.2.0

# Update workspace version
sed -i 's/version = "0.1.0"/version = "0.1.1"/' Cargo.toml

# Update version references in READMEs
sed -i 's/"0.1.0"/"0.1.1"/g' README.md
sed -i 's/"0.1.0"/"0.1.1"/g' core/*/README.md
sed -i 's/"0.1.0"/"0.1.1"/g' tools/*/README.md
sed -i 's/"0.1.0"/"0.1.1"/g' qudag/README.md
```

### 2. Build and Test

```bash
# Verify everything builds
cargo check --workspace --no-default-features

# Run tests
cargo test --workspace --no-default-features
```

### 3. Publish in Order

```bash
# Publish in dependency order
cd core/crypto && cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty && sleep 45
cd ../dag && cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty && sleep 45
cd ../network && cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty && sleep 45
cd ../protocol && cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty && sleep 45
cd ../../qudag && cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty && sleep 45
cd ../tools/cli && cargo publish --token $CRATES_API_KEY --no-default-features --allow-dirty
```

## 🔄 For README-Only Updates

Since you just updated the README files and documentation, this would be a **patch release (0.1.1)**:

```bash
# Run the automated script
./release.sh

# Choose option 1 (patch release) when prompted
# This will create version 0.1.1
```

## 📦 What Gets Updated

When you release a new version, these get updated:
- All crate versions in workspace
- Version references in README files
- Documentation links on crates.io
- Installation instructions show new version

## 🌐 After Publishing

Users can update to your new version:

```bash
# Update CLI tool
cargo install qudag-cli --force

# Update library in existing projects
cargo update qudag
```

## ⚠️ Important Notes

1. **Version Order Matters**: Always publish dependencies first (crypto, dag, network, protocol) then main crates (qudag, qudag-cli)

2. **Wait Between Publishes**: The 45-second delays allow crates.io to update indexes

3. **Patch vs Minor**: 
   - **Patch (0.1.1)**: Documentation, README updates, bug fixes
   - **Minor (0.2.0)**: New features, API additions (backward compatible)
   - **Major (1.0.0)**: Breaking changes

4. **Git Tags**: The automated script creates git tags (v0.1.1) for version tracking

## 🔍 Verify Release

After publishing, verify the updates:

```bash
# Check that new version is live
curl -s https://crates.io/api/v1/crates/qudag | jq '.crate.max_version'

# Test installation
cargo install qudag-cli --force
qudag-cli --version
```

## 🎯 For Your Current Situation

Since you updated READMEs and documentation:

```bash
# Run this to publish 0.1.1 with updated documentation
./release.sh

# Choose option 1 (patch release)
# This will publish all crates as v0.1.1 with the new READMEs
```

This will make all the README improvements visible on crates.io! 🎉