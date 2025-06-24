# /debug-security

## Purpose
Debug and validate security configurations across the QuDAG system, checking cryptographic parameters, memory protection settings, network security, and build configurations to ensure proper security posture.

## Parameters
- `[scope]`: Security scope to check (crypto, network, memory, config, dependencies, all) - defaults to "all"
- `[verbose]`: Enable verbose logging for detailed diagnostics (true/false) - defaults to false
- `[fix]`: Attempt to auto-fix configuration issues (true/false) - defaults to false
- `[report_format]`: Output format (console, json, markdown) - defaults to "console"

## Prerequisites
- [ ] System builds successfully with current configuration
- [ ] Security tools available (cargo-audit, cargo-deny)
- [ ] Access to configuration files and test vectors
- [ ] Appropriate permissions for system diagnostics

## Execution Steps

### 1. Validation Phase
- Validate scope parameter is one of: crypto, network, memory, config, dependencies, all
- Check availability of diagnostic tools and permissions
- Verify output directory exists at `/workspaces/QuDAG/reports/`
- Load security workflow from `/workspaces/QuDAG/.claude/commands/workflow/security_workflow.md`

### 2. Planning Phase
- Determine which security components to check based on scope
- Create diagnostic plan with specific verification steps
- Identify auto-fixable vs manual intervention issues
- Prepare reporting structure based on format parameter

### 3. Implementation Phase
- Step 3.1: Cryptographic Configuration Check
  ```bash
  # Verify ML-KEM-768 parameters
  echo "Checking ML-KEM-768 (FIPS 203) configuration..."
  # Expected: k=3, n=256, q=3329, security level 3 (192-bit)
  
  # Verify ML-DSA-65 parameters
  echo "Checking ML-DSA-65 (FIPS 204) configuration..."
  # Expected: security level 3, deterministic signing
  
  # Verify HQC parameters
  echo "Checking HQC Round 4 configuration..."
  # Expected: security level 1 (128-bit), proper error correction
  
  # Check RNG configuration
  echo "Verifying ChaCha20Rng CSPRNG setup..."
  ```

- Step 3.2: Network Security Diagnostics
  ```bash
  # Check TLS configuration
  rg "rustls|native-tls" /workspaces/QuDAG/Cargo.toml
  # Verify minimum TLS version (should be 1.2+)
  
  # Check exposed ports and endpoints
  rg "bind|listen" /workspaces/QuDAG/core/network --type rust
  
  # Verify authentication mechanisms
  rg "authenticate|verify_peer" /workspaces/QuDAG/core/network --type rust
  ```

- Step 3.3: Memory Protection Verification
  - Check for secure allocator usage
  - Verify stack protection flags in build
  - Validate ASLR and DEP/NX settings
  - Test zeroization of sensitive memory
  ```bash
  # Check for zeroize usage
  rg "zeroize|clear_on_drop" /workspaces/QuDAG/core --type rust
  
  # Verify no sensitive data in debug prints
  rg "println!|dbg!|debug!" /workspaces/QuDAG/core --type rust | rg -i "key|secret|password"
  ```

- Step 3.4: Build Configuration Security
  ```bash
  # Check Cargo.toml for security features
  cat /workspaces/QuDAG/Cargo.toml | grep -E "lto|opt-level|overflow-checks"
  
  # Verify security-related feature flags
  rg "\[features\]" /workspaces/QuDAG/core/*/Cargo.toml -A 10
  
  # Check for debug symbols in release
  cargo tree --features security-tests
  ```

- Step 3.5: Dependency Security Scan
  ```bash
  # Run cargo audit for known vulnerabilities
  cargo audit
  
  # Check dependency licenses
  cargo deny check licenses
  
  # Verify no banned dependencies
  cargo deny check bans
  
  # Check for outdated crypto libraries
  cargo outdated -R --features quantum-resistant
  ```

### 4. Verification Phase
- Run security configuration tests
  ```bash
  cargo test --features security-config-tests
  ```
- Validate all cryptographic test vectors pass
- Check that security policies are enforced
- Verify audit logging is functional
- Test secure communication channels

### 5. Documentation Phase
- Generate security configuration report based on format parameter
- If console format: Display findings with color coding (red=critical, yellow=warning, green=ok)
- If json format: Save to `/workspaces/QuDAG/reports/security_config.json`
- If markdown format: Save to `/workspaces/QuDAG/reports/security_config_debug.md`
- Update security context with current configuration status
- Log all diagnostic activities to audit trail

## Success Criteria
- [ ] All critical security configurations properly set
- [ ] No outdated dependencies with known vulnerabilities
- [ ] Cryptographic parameters match NIST specifications
- [ ] Memory protection features enabled
- [ ] Network security properly configured
- [ ] Build flags optimize for security
- [ ] Audit logging functional

## Error Handling
- **Invalid Scope**: Display valid scopes: crypto, network, memory, config, dependencies, all
- **Check Failure**: Run with `--verbose` flag for detailed error information and stack traces
- **Critical Issue**: Display "CRITICAL: Security misconfiguration detected" and block deployment
- **Permission Denied**: Request appropriate permissions or run with elevated privileges
- **Tool Missing**: Provide installation commands for missing security tools

## Output
- **Success**: "Security configuration validated. Risk level: LOW. All {count} checks passed."
- **Failure**: "Security issues detected. Risk level: {CRITICAL|HIGH|MEDIUM}. {failed}/{total} checks failed. See report for details."
- **Reports**:
  - Console: Color-coded output with immediate action items
  - JSON: Structured findings for programmatic processing
  - Markdown: Detailed report with sections for each scope area

## Example Usage
```
/debug-security
/debug-security --scope crypto --verbose
/debug-security --scope network --fix
/debug-security --report_format json > security_config.json
```

### Example Scenario
Debugging cryptographic configuration with auto-fix:
```
/debug-security --scope crypto --fix --verbose

Security Configuration Debug - Crypto Scope
==========================================

Checking ML-KEM-768 configuration...
✓ Parameter set verified: k=3, n=256, q=3329
✓ Security level: 3 (192-bit)
✓ Constant-time operations: ENABLED

Checking ML-DSA-65 configuration...
✓ Security level: 3 (192-bit)
⚠ Deterministic nonce generation: NEEDS UPDATE
  → Auto-fixing: Updating to deterministic mode...
  ✓ Fixed: Set deterministic_nonces = true

Checking ChaCha20Rng configuration...
✓ CSPRNG properly initialized
✓ Entropy source: getrandom
✓ Reseeding interval: 64KB

Test Vector Validation...
✓ ML-KEM test vectors: 50/50 passed
✓ ML-DSA test vectors: 30/30 passed
✓ HQC test vectors: 25/25 passed

Summary:
- Checks performed: 12
- Issues found: 1
- Auto-fixed: 1
- Manual fixes required: 0
- Risk level: LOW

Configuration saved to: /workspaces/QuDAG/.claude/contexts/security_context.md
```

## Related Commands
- `/security-audit`: Full security audit of the system
- `/review-security`: Manual security code review
- `/crypto-validate`: Validate cryptographic implementations
- `/deploy-validate`: Validate deployment security settings

## Workflow Integration
This command is part of the Security Workflow and:
- Follows: `/implement-feature` (check configs after changes)
- Precedes: `/deploy-validate` (ensure security before deployment)
- Can be run in parallel with: `/performance-benchmark` (different focus areas)

## Agent Coordination
- **Primary Agent**: Security Agent (performs diagnostics)
- **Supporting Agents**: 
  - Crypto Agent: Validates cryptographic configurations
  - Network Agent: Checks network security settings
  - Integration Agent: Verifies system-wide settings

## Notes
- Run regularly to catch configuration drift
- Auto-fix only handles safe, deterministic changes
- Critical issues always require manual intervention
- Verbose mode provides detailed diagnostic information
- JSON output useful for CI/CD integration and monitoring
- Some checks require elevated permissions (e.g., ASLR verification)