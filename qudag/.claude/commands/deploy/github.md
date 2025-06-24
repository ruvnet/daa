# /deploy/github

## Purpose
Create and manage GitHub releases with automated binary artifact generation, cross-platform builds, release notes, and asset distribution. Supports both draft and production releases.

## Parameters
- `<version>`: Release version - v1.2.3|major|minor|patch (required)
- `[type]`: Release type - release|prerelease|draft (default: release)
- `[target]`: Target platforms - all|linux|macos|windows|wasm (default: all)
- `[generate-notes]`: Auto-generate release notes - true|false (default: true)

## Prerequisites
- [ ] GitHub CLI (`gh`) installed and authenticated
- [ ] All tests passing and builds successful
- [ ] Git working directory clean
- [ ] Cross-compilation tools installed
- [ ] Release artifacts built and available

## Execution Steps

### 1. Validation Phase
- Verify GitHub authentication
  ```bash
  gh auth status
  ```
- Check repository status
  ```bash
  git status --porcelain
  gh repo view --json name,owner
  ```
- Validate version format
- Confirm all builds are current

### 2. Pre-release Preparation
- Step 2.1: Update version in all relevant files
  ```bash
  # Update Cargo.toml versions
  find /workspaces/QuDAG -name "Cargo.toml" -exec sed -i \
    's/version = "[0-9]\+\.[0-9]\+\.[0-9]\+"/version = "NEW_VERSION"/' {} \;
  
  # Update package.json versions
  find /workspaces/QuDAG -name "package.json" -exec sed -i \
    's/"version": "[^"]*"/"version": "NEW_VERSION"/' {} \;
  ```
- Step 2.2: Generate changelog entry
  ```bash
  cd /workspaces/QuDAG
  echo "## [NEW_VERSION] - $(date +%Y-%m-%d)" >> CHANGELOG.md
  git log --oneline --pretty=format:"- %s" "$(git describe --tags --abbrev=0)..HEAD" >> CHANGELOG.md
  ```
- Step 2.3: Commit version updates
  ```bash
  git add -A
  git commit -m "chore: bump version to NEW_VERSION"
  git tag -a "vNEW_VERSION" -m "Release NEW_VERSION"
  ```

### 3. Cross-Platform Binary Generation
- Step 3.1: Build Linux x86_64 binaries
  ```bash
  cd /workspaces/QuDAG
  cargo build --release --target x86_64-unknown-linux-gnu --workspace
  
  # Create release archive
  mkdir -p release/linux-x86_64
  cp target/x86_64-unknown-linux-gnu/release/qudag release/linux-x86_64/
  tar -czf qudag-NEW_VERSION-linux-x86_64.tar.gz -C release linux-x86_64
  ```
- Step 3.2: Build Linux ARM64 binaries
  ```bash
  cargo build --release --target aarch64-unknown-linux-gnu --workspace
  
  mkdir -p release/linux-aarch64
  cp target/aarch64-unknown-linux-gnu/release/qudag release/linux-aarch64/
  tar -czf qudag-NEW_VERSION-linux-aarch64.tar.gz -C release linux-aarch64
  ```
- Step 3.3: Build macOS binaries
  ```bash
  # macOS x86_64
  cargo build --release --target x86_64-apple-darwin --workspace
  mkdir -p release/macos-x86_64
  cp target/x86_64-apple-darwin/release/qudag release/macos-x86_64/
  tar -czf qudag-NEW_VERSION-macos-x86_64.tar.gz -C release macos-x86_64
  
  # macOS ARM64
  cargo build --release --target aarch64-apple-darwin --workspace
  mkdir -p release/macos-aarch64
  cp target/aarch64-apple-darwin/release/qudag release/macos-aarch64/
  tar -czf qudag-NEW_VERSION-macos-aarch64.tar.gz -C release macos-aarch64
  ```
- Step 3.4: Build Windows binaries
  ```bash
  cargo build --release --target x86_64-pc-windows-gnu --workspace
  
  mkdir -p release/windows-x86_64
  cp target/x86_64-pc-windows-gnu/release/qudag.exe release/windows-x86_64/
  zip -r qudag-NEW_VERSION-windows-x86_64.zip release/windows-x86_64
  ```

### 4. WASM Artifacts Preparation
- Step 4.1: Build WASM packages
  ```bash
  cd /workspaces/QuDAG/qudag-wasm
  ./build.sh
  ```
- Step 4.2: Create WASM distribution
  ```bash
  tar -czf qudag-wasm-NEW_VERSION.tar.gz pkg pkg-node pkg-optimized
  ```

### 5. Release Notes Generation
- Step 5.1: Generate automated release notes
  ```bash
  gh release create "vNEW_VERSION" --generate-notes --draft
  ```
- Step 5.2: Enhance with custom content
  ```bash
  cat > release-notes.md << EOF
  ## QuDAG NEW_VERSION Release
  
  ### 🚀 New Features
  $(git log --oneline --grep="feat:" "$(git describe --tags --abbrev=0)..HEAD" | sed 's/^[a-f0-9]* feat: /- /')
  
  ### 🐛 Bug Fixes  
  $(git log --oneline --grep="fix:" "$(git describe --tags --abbrev=0)..HEAD" | sed 's/^[a-f0-9]* fix: /- /')
  
  ### 📈 Performance Improvements
  $(git log --oneline --grep="perf:" "$(git describe --tags --abbrev=0)..HEAD" | sed 's/^[a-f0-9]* perf: /- /')
  
  ### 📚 Documentation
  $(git log --oneline --grep="docs:" "$(git describe --tags --abbrev=0)..HEAD" | sed 's/^[a-f0-9]* docs: /- /')
  
  ### Installation
  
  #### Binary Downloads
  Download the appropriate binary for your platform from the assets below.
  
  #### Package Managers
  \`\`\`bash
  # NPM
  npm install -g qudag@NEW_VERSION
  
  # Cargo  
  cargo install qudag --version NEW_VERSION
  \`\`\`
  
  #### WASM
  \`\`\`bash
  npm install qudag-wasm@NEW_VERSION
  \`\`\`
  
  ### Checksums
  \`\`\`
  $(find . -name "qudag-NEW_VERSION-*" -exec sha256sum {} \;)
  \`\`\`
  EOF
  ```

### 6. Release Creation and Asset Upload
- Step 6.1: Create release with assets
  ```bash
  gh release create "vNEW_VERSION" \
    --title "QuDAG NEW_VERSION" \
    --notes-file release-notes.md \
    qudag-NEW_VERSION-*.tar.gz \
    qudag-NEW_VERSION-*.zip \
    qudag-wasm-NEW_VERSION.tar.gz
  ```
- Step 6.2: Upload additional assets
  ```bash
  # Upload checksums
  sha256sum qudag-NEW_VERSION-* > SHA256SUMS
  gh release upload "vNEW_VERSION" SHA256SUMS
  
  # Upload source code
  git archive --format=tar.gz --prefix=qudag-NEW_VERSION/ vNEW_VERSION > qudag-NEW_VERSION-source.tar.gz
  gh release upload "vNEW_VERSION" qudag-NEW_VERSION-source.tar.gz
  ```

### 7. Post-Release Tasks
- Step 7.1: Push tags and commits
  ```bash
  git push origin main
  git push origin "vNEW_VERSION"
  ```
- Step 7.2: Update repository metadata
  ```bash
  # Update GitHub topics/tags
  gh repo edit --add-topic "version-NEW_VERSION"
  ```
- Step 7.3: Notify deployment systems
  ```bash
  # Trigger deployment workflows
  gh workflow run deploy-production.yml --field version=NEW_VERSION
  ```

## Success Criteria
- [ ] GitHub release created successfully with all assets
- [ ] All cross-platform binaries included and functional
- [ ] Release notes are comprehensive and accurate
- [ ] Checksums provided for all binary assets
- [ ] Repository tags and version metadata updated
- [ ] Download links work correctly

## Error Handling
- **Authentication failures**: Run `gh auth login` and verify permissions
- **Build failures**: Fix compilation errors before creating release
- **Asset upload failures**: Verify file paths and retry upload
- **Tag conflicts**: Use different version or delete existing tag
- **Network issues**: Retry operations or use different network
- **Size limits**: Compress large assets or use external storage

## Output
- **Success**: GitHub release URL with download statistics
- **Failure**: Specific error messages and suggested remediation
- **Reports**: 
  - Release creation log with asset inventory
  - Download statistics and platform distribution
  - Post-release validation results

## Example Usage
```bash
# Create full release with all platforms
/deploy/github v1.2.3 release all true

# Create draft release for testing
/deploy/github v1.3.0-beta prerelease all true

# Linux-only release
/deploy/github v1.2.4 release linux false
```

### Example Output
```
Creating GitHub release v1.2.3...

✓ Version updated in all files
✓ Changelog generated from 15 commits
✓ Git tag created: v1.2.3

Building cross-platform binaries:
✓ Linux x86_64: qudag (12.4MB)
✓ Linux ARM64: qudag (13.1MB)  
✓ macOS x86_64: qudag (11.8MB)
✓ macOS ARM64: qudag (11.2MB)
✓ Windows x86_64: qudag.exe (14.2MB)
✓ WASM package: 2.1MB

✓ Release created: https://github.com/ruvnet/QuDAG/releases/tag/v1.2.3
✓ Assets uploaded: 7 files, 64.8MB total
✓ Checksums generated: SHA256SUMS
✓ Repository tags updated

Release Statistics:
- Binary downloads available for 5 platforms
- WASM package for web/Node.js deployment
- Source code archive included
- Auto-generated and custom release notes
```

## Related Commands
- `/build/cargo`: Build binaries before release
- `/deploy/npm`: Coordinate NPM package releases
- `/deploy/crates`: Coordinate crates.io releases

## Workflow Integration
This command is part of the Release Management workflow and:
- Follows: `/build/cargo`, `/build/wasm`, and all testing
- Precedes: Production deployment and announcement
- Coordinates with: `/deploy/npm` and `/deploy/crates` for package releases

## Agent Coordination
- **Primary Agent**: GitHub Release Agent
- **Supporting Agents**: 
  - Build Agent: Provides cross-platform binaries
  - Documentation Agent: Generates release notes
  - Deployment Agent: Manages post-release tasks

## Notes
- GitHub has file size limits (100MB per file, 2GB per release)
- Use semantic versioning for consistent version management
- Consider using GitHub Actions for automated releases
- Monitor download statistics to understand platform usage
- Coordinate timing with package registry releases

---

## Advanced Release Scenarios

### Automated Release Pipeline
```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Create Release
        run: /deploy/github ${{ github.ref_name }} release all true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Multi-Repository Coordination
```bash
# Coordinate releases across related repositories
coordinate_releases() {
  local version=$1
  
  # Release main repository
  /deploy/github $version release all true
  
  # Update dependent repositories
  for repo in qudag-docs qudag-examples qudag-docker; do
    gh repo clone ruvnet/$repo
    cd $repo
    # Update version references
    find . -name "*.md" -exec sed -i "s/qudag@[0-9.]*/qudag@$version/g" {} \;
    git commit -am "chore: update QuDAG version to $version"
    git tag -a "v$version" -m "Update for QuDAG $version"
    git push origin main --tags
    cd ..
  done
}
```

### Release Rollback
```bash
# Rollback release if issues found
rollback_release() {
  local version=$1
  
  echo "Rolling back release $version"
  
  # Delete GitHub release
  gh release delete $version --yes
  
  # Delete git tag
  git tag -d $version
  git push origin :refs/tags/$version
  
  # Revert version commits
  git revert HEAD --no-edit
  git push origin main
}
```

### Custom Asset Generation
```bash
# Generate additional release assets
generate_custom_assets() {
  local version=$1
  
  # Generate API documentation
  cargo doc --workspace --no-deps
  tar -czf qudag-$version-docs.tar.gz -C target/doc .
  
  # Generate usage examples
  tar -czf qudag-$version-examples.tar.gz examples/
  
  # Generate configuration templates
  tar -czf qudag-$version-configs.tar.gz configs/
}
```

### Release Metrics Collection
```bash
# Collect release metrics
collect_metrics() {
  local version=$1
  
  # Download statistics
  gh api repos/ruvnet/QuDAG/releases/tags/$version \
    --jq '.assets[] | {name: .name, downloads: .download_count}' > metrics.json
  
  # Platform distribution
  echo "Platform distribution for $version:" > platform-stats.txt
  gh api repos/ruvnet/QuDAG/releases/tags/$version \
    --jq '.assets[] | select(.name | contains("linux")) | .download_count' >> platform-stats.txt
}
```