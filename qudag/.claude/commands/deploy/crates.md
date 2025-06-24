# /deploy/crates

## Purpose
Publish Rust crates to crates.io registry with proper versioning, dependency management, and workspace coordination. Ensures all QuDAG Rust libraries are published in correct dependency order.

## Parameters
- `<crates>`: Crates to publish - all|crypto|dag|network|protocol|vault|wasm|core (default: all)
- `[version]`: Version increment - patch|minor|major|<specific-version> (default: patch)
- `[dry-run]`: Perform dry run - true|false (default: false)
- `[registry]`: Registry URL - crates-io|staging (default: crates-io)

## Prerequisites
- [ ] Cargo account with publishing permissions
- [ ] Cargo authentication configured (`cargo login`)
- [ ] All workspace builds successful
- [ ] Tests passing across all crates
- [ ] Documentation generated without warnings
- [ ] Clean git working directory

## Execution Steps

### 1. Validation Phase
- Verify Cargo authentication
  ```bash
  cargo login --dry-run
  ```
- Check crate dependency graph
  ```bash
  cd /workspaces/QuDAG
  cargo tree --workspace
  ```
- Validate Cargo.toml files
- Confirm version consistency

### 2. Dependency Order Analysis
- Step 2.1: Determine publishing order
  ```
  Publishing Order (dependencies first):
  1. qudag-crypto (no internal deps)
  2. qudag-dag (may depend on crypto)
  3. qudag-network (depends on crypto)
  4. qudag-protocol (depends on crypto, network)
  5. qudag-vault (depends on crypto, dag)
  6. qudag-wasm (depends on crypto, dag)
  7. qudag (main crate, depends on all)
  ```

### 3. Version Synchronization
- Step 3.1: Update workspace versions
  ```bash
  cd /workspaces/QuDAG
  
  # Update root Cargo.toml
  sed -i 's/version = "[0-9]\+\.[0-9]\+\.[0-9]\+"/version = "NEW_VERSION"/' Cargo.toml
  ```
- Step 3.2: Update dependency versions
  ```bash
  # Update internal dependency versions in each crate
  find . -name "Cargo.toml" -exec sed -i \
    's/qudag-\([a-z]*\) = { version = "[^"]*"/qudag-\1 = { version = "NEW_VERSION"/' {} \;
  ```
- Step 3.3: Generate new Cargo.lock
  ```bash
  cargo update --workspace
  ```

### 4. Pre-publish Validation
- Step 4.1: Build all crates
  ```bash
  cargo build --workspace --release
  ```
- Step 4.2: Run comprehensive tests
  ```bash
  cargo test --workspace --release
  ```
- Step 4.3: Check documentation
  ```bash
  cargo doc --workspace --no-deps
  ```
- Step 4.4: Run clippy
  ```bash
  cargo clippy --workspace -- -D warnings
  ```

### 5. Dry Run Validation
- Step 5.1: Perform dry run for each crate
  ```bash
  cargo publish --dry-run -p qudag-crypto
  cargo publish --dry-run -p qudag-dag
  cargo publish --dry-run -p qudag-network
  cargo publish --dry-run -p qudag-protocol
  cargo publish --dry-run -p qudag-vault
  cargo publish --dry-run -p qudag-wasm
  cargo publish --dry-run -p qudag
  ```

### 6. Sequential Publishing
- Step 6.1: Publish core crypto library
  ```bash
  cd /workspaces/QuDAG/core/crypto
  cargo publish --allow-dirty
  
  # Wait for crates.io indexing
  sleep 30
  ```
- Step 6.2: Publish DAG library
  ```bash
  cd /workspaces/QuDAG/core/dag
  cargo publish --allow-dirty
  sleep 30
  ```
- Step 6.3: Publish network library
  ```bash
  cd /workspaces/QuDAG/core/network
  cargo publish --allow-dirty
  sleep 30
  ```
- Step 6.4: Publish protocol library
  ```bash
  cd /workspaces/QuDAG/core/protocol
  cargo publish --allow-dirty
  sleep 30
  ```
- Step 6.5: Publish vault library
  ```bash
  cd /workspaces/QuDAG/core/vault
  cargo publish --allow-dirty
  sleep 30
  ```
- Step 6.6: Publish WASM bindings
  ```bash
  cd /workspaces/QuDAG/qudag-wasm
  cargo publish --allow-dirty
  sleep 30
  ```
- Step 6.7: Publish main crate
  ```bash
  cd /workspaces/QuDAG/qudag
  cargo publish --allow-dirty
  ```

### 7. Publication Verification
- Step 7.1: Verify all crates published
  ```bash
  for crate in qudag-crypto qudag-dag qudag-network qudag-protocol qudag-vault qudag-wasm qudag; do
    cargo search $crate --limit 1
  done
  ```
- Step 7.2: Test dependency resolution
  ```bash
  mkdir -p /tmp/crates-test
  cd /tmp/crates-test
  cargo init --name test-project
  echo 'qudag = "NEW_VERSION"' >> Cargo.toml
  cargo check
  ```

### 8. Documentation Update
- Step 8.1: Update docs.rs configuration
- Step 8.2: Verify documentation builds
- Step 8.3: Update README with new versions

## Success Criteria
- [ ] All crates published successfully to crates.io
- [ ] Dependency resolution works correctly
- [ ] Documentation builds without errors on docs.rs
- [ ] Version numbers are consistent across all crates
- [ ] No breaking changes introduced without major version bump
- [ ] All CI/CD pipelines pass with new versions

## Error Handling
- **Authentication failures**: Run `cargo login` with valid token
- **Version conflicts**: Check existing versions with `cargo search <crate>`
- **Dependency resolution**: Ensure internal dependencies use correct versions
- **Build failures**: Fix compilation errors before publishing
- **Documentation errors**: Fix rustdoc warnings and missing docs
- **Network timeouts**: Retry after waiting for crates.io indexing

## Output
- **Success**: Published crate URLs and version information
- **Failure**: Specific error messages and suggested fixes
- **Reports**: 
  - Publication log with timestamps
  - Dependency resolution verification
  - Documentation build status

## Example Usage
```bash
# Publish all crates with patch version bump
/deploy/crates all patch false crates-io

# Dry run for crypto crate only
/deploy/crates crypto minor true crates-io

# Publish specific version
/deploy/crates all 1.3.0 false crates-io
```

### Example Output
```
Publishing QuDAG crates to crates.io...

✓ Version updated: 1.2.1 → 1.2.2
✓ Dependency versions synchronized
✓ Pre-publish validation passed

Publishing in dependency order:
✓ qudag-crypto@1.2.2 published (30s)
✓ qudag-dag@1.2.2 published (30s) 
✓ qudag-network@1.2.2 published (30s)
✓ qudag-protocol@1.2.2 published (30s)
✓ qudag-vault@1.2.2 published (30s)
✓ qudag-wasm@1.2.2 published (30s)
✓ qudag@1.2.2 published (30s)

Post-publish verification:
✓ All crates searchable on crates.io
✓ Dependency resolution: SUCCESS
✓ Documentation: Building on docs.rs

Crate URLs:
- https://crates.io/crates/qudag-crypto
- https://crates.io/crates/qudag-dag
- https://crates.io/crates/qudag-network
- https://crates.io/crates/qudag-protocol
- https://crates.io/crates/qudag-vault
- https://crates.io/crates/qudag-wasm
- https://crates.io/crates/qudag
```

## Related Commands
- `/build/cargo`: Build crates before publishing
- `/test/cargo`: Validate crates before publishing
- `/deploy/github`: Create GitHub release with crate versions

## Workflow Integration
This command is part of the Release Deployment workflow and:
- Follows: `/build/cargo` and comprehensive testing
- Precedes: `/deploy/github` for GitHub release creation
- Can be run in parallel with: `/deploy/npm` for NPM publishing

## Agent Coordination
- **Primary Agent**: Crates Deployment Agent
- **Supporting Agents**: 
  - Build Agent: Ensures all crates build successfully
  - Test Agent: Validates crate functionality
  - Documentation Agent: Ensures docs.rs compatibility

## Notes
- crates.io publishing is irreversible - cannot unpublish versions
- Dependency order is critical for successful publication
- Wait for crates.io indexing between dependent crate publications
- Use semantic versioning strictly for API compatibility
- Monitor docs.rs build status after publishing

---

## Advanced Publishing Scenarios

### Workspace Version Management
```toml
# Cargo.toml workspace configuration
[workspace]
members = ["core/*", "qudag", "qudag-wasm"]

[workspace.package]
version = "1.2.2"
authors = ["QuDAG Team"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/ruvnet/QuDAG"

[workspace.dependencies]
qudag-crypto = { version = "1.2.2", path = "core/crypto" }
qudag-dag = { version = "1.2.2", path = "core/dag" }
```

### Conditional Publishing
```bash
# Only publish if tests pass
publish_if_tests_pass() {
  local crate=$1
  
  echo "Testing $crate before publishing..."
  if cargo test -p $crate --release; then
    echo "✓ Tests passed, publishing $crate"
    cargo publish -p $crate --allow-dirty
  else
    echo "✗ Tests failed, skipping $crate"
    return 1
  fi
}
```

### Version Validation
```bash
# Validate version consistency
validate_versions() {
  local expected_version=$1
  
  echo "Validating version consistency..."
  
  # Check all Cargo.toml files
  find . -name "Cargo.toml" -exec grep -H "version.*$expected_version" {} \; || {
    echo "Version mismatch found!"
    return 1
  }
  
  echo "✓ All versions consistent: $expected_version"
}
```

### Documentation Metadata
```toml
# Cargo.toml documentation configuration
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### Custom Registry Publishing
```bash
# Publish to custom registry
cargo publish --registry my-registry --allow-dirty
```

### Automated Changelog Generation
```bash
# Generate changelog from git commits
generate_changelog() {
  local prev_version=$1
  local new_version=$2
  
  echo "## [$new_version] - $(date +%Y-%m-%d)" >> CHANGELOG.md
  git log --oneline --pretty=format:"- %s" "v$prev_version..HEAD" >> CHANGELOG.md
  echo "" >> CHANGELOG.md
}
```