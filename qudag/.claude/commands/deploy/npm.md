# /deploy/npm

## Purpose
Publish QuDAG WASM and Node.js packages to NPM registry with proper versioning, documentation, and multi-target support. Handles both automated and manual publishing workflows.

## Parameters
- `<package>`: Package to publish - wasm|node|both (default: both)
- `[version]`: Version increment - patch|minor|major|prerelease|<specific-version> (default: patch)
- `[tag]`: NPM tag - latest|beta|alpha|next (default: latest)
- `[registry]`: NPM registry - npmjs|github|custom (default: npmjs)

## Prerequisites
- [ ] NPM account with publishing permissions
- [ ] NPM authentication configured (`npm login`)
- [ ] WASM builds completed in `/workspaces/QuDAG/qudag-wasm/pkg*/`
- [ ] Node.js package built in `/workspaces/QuDAG/qudag-npm/dist/`
- [ ] All tests passing
- [ ] Clean git working directory

## Execution Steps

### 1. Validation Phase
- Verify NPM authentication
  ```bash
  npm whoami
  ```
- Check package build status
  ```bash
  ls -la /workspaces/QuDAG/qudag-wasm/pkg*/package.json
  ls -la /workspaces/QuDAG/qudag-npm/package.json
  ```
- Validate package.json integrity
- Confirm version consistency across packages

### 2. Pre-publish Preparation
- Step 2.1: Update version numbers
  ```bash
  cd /workspaces/QuDAG/qudag-wasm
  npm version patch --no-git-tag-version
  
  cd /workspaces/QuDAG/qudag-npm
  npm version patch --no-git-tag-version
  ```
- Step 2.2: Generate/update README files
  ```bash
  # Copy main README to packages
  cp /workspaces/QuDAG/README.md /workspaces/QuDAG/qudag-wasm/pkg/
  cp /workspaces/QuDAG/README.md /workspaces/QuDAG/qudag-npm/
  ```
- Step 2.3: Validate package contents
  ```bash
  cd /workspaces/QuDAG/qudag-wasm
  npm pack --dry-run
  
  cd /workspaces/QuDAG/qudag-npm
  npm pack --dry-run
  ```

### 3. WASM Package Publishing
- Step 3.1: Publish web build
  ```bash
  cd /workspaces/QuDAG/qudag-wasm/pkg
  npm publish --access public --tag latest
  ```
- Step 3.2: Publish Node.js build
  ```bash
  cd /workspaces/QuDAG/qudag-wasm/pkg-node
  # Update package.json name for Node.js specific build
  sed -i 's/"qudag-wasm"/"qudag-wasm-node"/' package.json
  npm publish --access public --tag latest
  ```

### 4. Node.js Package Publishing
- Step 4.1: Final build verification
  ```bash
  cd /workspaces/QuDAG/qudag-npm
  npm run build
  npm run test
  ```
- Step 4.2: Publish Node.js package
  ```bash
  npm publish --access public --tag latest
  ```

### 5. Package Verification
- Step 5.1: Verify published packages
  ```bash
  npm view qudag-wasm
  npm view qudag-wasm-node
  npm view qudag
  ```
- Step 5.2: Test installation
  ```bash
  mkdir -p /tmp/npm-test
  cd /tmp/npm-test
  npm init -y
  npm install qudag-wasm qudag
  node -e "console.log(require('qudag-wasm'))"
  ```

### 6. Documentation Update
- Step 6.1: Update installation instructions
- Step 6.2: Generate API documentation
- Step 6.3: Update changelog
- Step 6.4: Create release notes

### 7. Post-publish Validation
- Step 7.1: Automated NPX testing
  ```bash
  npx qudag --version
  npx qudag --help
  ```
- Step 7.2: CDN availability check
  ```bash
  curl -I https://cdn.jsdelivr.net/npm/qudag-wasm@latest/qudag_wasm.js
  curl -I https://unpkg.com/qudag-wasm@latest/qudag_wasm.js
  ```

## Success Criteria
- [ ] All packages published successfully to NPM
- [ ] Package versions are consistent across all builds
- [ ] NPX installation works correctly
- [ ] CDN links resolve within 5 minutes
- [ ] No breaking changes for existing users
- [ ] Documentation is updated and accessible

## Error Handling
- **Authentication failures**: Run `npm login` and verify credentials
- **Version conflicts**: Check for existing versions with `npm view <package> versions`
- **Build missing**: Run `/build/wasm` and `/build/cargo` first
- **File not found**: Verify WASM builds exist in correct directories
- **Permission denied**: Ensure NPM account has publishing rights
- **Network timeouts**: Retry publishing or use different registry

## Output
- **Success**: Published package URLs and version information
- **Failure**: Specific error messages and remediation steps
- **Reports**: 
  - Publication confirmation with package URLs
  - Download statistics and CDN status
  - Installation test results

## Example Usage
```bash
# Publish both packages with patch version bump
/deploy/npm both patch latest npmjs

# Publish WASM package only with beta tag
/deploy/npm wasm minor beta npmjs

# Publish specific version
/deploy/npm both 1.2.3 latest npmjs
```

### Example Output
```
Publishing QuDAG packages to NPM...

✓ Version updated: 1.2.1 → 1.2.2
✓ WASM package validated: qudag-wasm@1.2.2
✓ Node.js package validated: qudag@1.2.2

Publishing packages:
✓ qudag-wasm@1.2.2 published to https://www.npmjs.com/package/qudag-wasm
✓ qudag-wasm-node@1.2.2 published to https://www.npmjs.com/package/qudag-wasm-node
✓ qudag@1.2.2 published to https://www.npmjs.com/package/qudag

Post-publish verification:
✓ NPX installation: qudag@1.2.2 works
✓ CDN availability: jsdelivr.net (✓) unpkg.com (✓)
✓ API documentation: Updated

Package URLs:
- Web: https://www.npmjs.com/package/qudag-wasm
- Node.js: https://www.npmjs.com/package/qudag
- Docs: https://docs.rs/qudag-wasm
```

## Related Commands
- `/build/wasm`: Build WASM packages before publishing
- `/test/wasm`: Validate packages before publishing
- `/deploy/github`: Create GitHub release with NPM packages

## Workflow Integration
This command is part of the Release Deployment workflow and:
- Follows: `/build/wasm` and `/test/wasm` for package preparation
- Precedes: `/deploy/github` for release management
- Can be run in parallel with: `/deploy/crates` for Rust crate publishing

## Agent Coordination
- **Primary Agent**: NPM Deployment Agent
- **Supporting Agents**: 
  - Build Agent: Ensures packages are built correctly
  - Test Agent: Validates package functionality
  - Documentation Agent: Updates package documentation

## Notes
- NPM publishing is irreversible - unpublishing is restricted
- Semantic versioning is critical for dependency management
- Consider using NPM 2FA for security
- Test packages in isolated environments before publishing
- Monitor download statistics after publishing

---

## Advanced Publishing Scenarios

### Multi-Registry Publishing
```bash
# Publish to NPM and GitHub Packages
npm publish --registry https://registry.npmjs.org/
npm publish --registry https://npm.pkg.github.com/
```

### Scoped Package Publishing
```json
{
  "name": "@qudag/wasm",
  "publishConfig": {
    "registry": "https://registry.npmjs.org/",
    "access": "public"
  }
}
```

### Prerelease Publishing
```bash
# Publish beta version
npm version prerelease --preid=beta
npm publish --tag beta

# Promote beta to latest
npm dist-tag add qudag-wasm@1.2.2-beta.1 latest
```

### Automated Publishing Pipeline
```yaml
# .github/workflows/npm-publish.yml
name: NPM Publish
on:
  push:
    tags: ['v*']

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'
      - name: Publish packages
        run: /deploy/npm both patch latest npmjs
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### Package Validation Scripts
```javascript
// scripts/validate-package.js
const fs = require('fs');
const path = require('path');

const validatePackage = (packagePath) => {
  const pkg = JSON.parse(fs.readFileSync(path.join(packagePath, 'package.json')));
  
  // Check required fields
  const required = ['name', 'version', 'description', 'main', 'types'];
  const missing = required.filter(field => !pkg[field]);
  
  if (missing.length > 0) {
    throw new Error(`Missing required fields: ${missing.join(', ')}`);
  }
  
  // Check WASM files exist
  if (pkg.name.includes('wasm')) {
    const wasmFile = path.join(packagePath, pkg.main.replace('.js', '_bg.wasm'));
    if (!fs.existsSync(wasmFile)) {
      throw new Error(`WASM file not found: ${wasmFile}`);
    }
  }
  
  return true;
};
```

### CDN Testing
```bash
# Test CDN availability
test_cdn() {
  local package=$1
  local version=$2
  
  echo "Testing CDN availability for $package@$version"
  
  # Test jsdelivr
  curl -f -s https://cdn.jsdelivr.net/npm/$package@$version/package.json > /dev/null
  echo "✓ jsdelivr.net"
  
  # Test unpkg
  curl -f -s https://unpkg.com/$package@$version/package.json > /dev/null
  echo "✓ unpkg.com"
}
```