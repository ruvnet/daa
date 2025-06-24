# /security-audit

## Purpose
Perform comprehensive security analysis and testing of the QuDAG protocol implementation, validating quantum-resistant cryptography, memory safety, and network security against known attack vectors.

## Parameters
- `[module]`: Optional module to audit specifically (crypto, network, dag, protocol, all) - defaults to "all"
- `[depth]`: Audit depth level (quick, standard, comprehensive) - defaults to "comprehensive"
- `[focus]`: Optional array of security aspects to focus on (timing, memory, crypto, network, input-validation, side-channel)

## Prerequisites
- [ ] All code compiles without errors
- [ ] Test suite passes with current implementation
- [ ] Security tools installed (cargo-audit, cargo-deny, clippy)
- [ ] Access to test vectors for cryptographic validation

## Execution Steps

### 1. Validation Phase
- Validate module parameter is one of: crypto, network, dag, protocol, or all
- Check that security analysis tools are available and up to date
- Verify access to NIST PQC test vectors at `/workspaces/QuDAG/core/crypto/tests/.test_vectors/`
- Confirm audit scope and depth settings match available time and resources

### 2. Planning Phase
- Analyze the security workflow requirements at `/workspaces/QuDAG/.claude/commands/workflow/security_workflow.md`
- Create audit plan based on specified module and focus areas
- Identify critical security boundaries and trust assumptions
- Prepare security context for tracking findings

### 3. Implementation Phase
- Step 3.1: Static Security Analysis
  ```bash
  # Run clippy with security lints
  cargo clippy --all-features -- -D warnings -W clippy::all
  # Check for unsafe blocks
  rg "unsafe" --type rust -A 5 -B 5
  # Validate error handling patterns
  cargo check --all-features
  ```
  
- Step 3.2: Cryptographic Validation
  - ML-KEM-768 (FIPS 203):
    - Verify constant-time matrix operations
    - Validate parameter set (k=3, n=256, q=3329)
    - Test encapsulation/decapsulation against NIST vectors
    - Check for timing side-channels in polynomial arithmetic
  - ML-DSA-65 (FIPS 204):
    - Verify constant-time polynomial operations
    - Validate deterministic signing process
    - Test signing/verification against NIST vectors
    - Check rejection sampling security bounds
  - HQC (Round 4):
    - Verify constant-time decoding operations
    - Validate error vector generation
    - Test encryption/decryption functionality
    - Check syndrome computation correctness

- Step 3.3: Memory Safety Verification
  ```bash
  # Check for memory leaks with valgrind
  cargo test --release -- --test-threads=1
  # Verify zeroization of sensitive data
  rg "zeroize|drop|clear_on_drop" --type rust
  # Analyze stack usage and allocation patterns
  cargo +nightly miri test
  ```

- Step 3.4: Network Security Assessment
  - Review protocol message validation
  - Check authentication and authorization flows
  - Verify message integrity mechanisms
  - Assess resistance to replay attacks
  - Validate rate limiting and DoS protections

- Step 3.5: Integration Security Analysis
  - Map component interaction boundaries
  - Identify and validate trust assumptions
  - Assess overall attack surface
  - Review access control implementations
  - Check for TOCTOU vulnerabilities

### 4. Verification Phase
- Run security-specific test suites
  ```bash
  cargo test --features security-tests
  cargo test --features quantum-resistant
  ```
- Execute timing attack resistance tests
- Verify all cryptographic operations complete in constant time
- Validate secure random number generation with ChaCha20Rng
- Check compliance with NIST PQC standards

### 5. Documentation Phase
- Generate comprehensive security audit report at `/workspaces/QuDAG/reports/security_audit_report.md`
- Create vulnerability findings document at `/workspaces/QuDAG/reports/vulnerability_findings.json`
- Develop remediation plan at `/workspaces/QuDAG/reports/remediation_plan.md`
- Update security context at `/workspaces/QuDAG/.claude/contexts/security_context.md`
- Generate compliance matrix at `/workspaces/QuDAG/reports/compliance_matrix.csv`

## Success Criteria
- [ ] No critical vulnerabilities discovered
- [ ] All cryptographic operations verified constant-time
- [ ] Memory safety confirmed with no leaks or use-after-free
- [ ] Quantum-resistant algorithms correctly implemented per NIST standards
- [ ] Network protocols resist common attack vectors
- [ ] All unsafe code blocks justified and documented
- [ ] Security audit report generated with actionable findings

## Error Handling
- **Security Scan Failure**: Check tool installation with `cargo install cargo-audit cargo-deny` and retry with verbose output
- **Invalid Module**: Ensure module name is one of: crypto, network, dag, protocol, all
- **Audit Timeout**: Partial results will be saved - retry with `--depth quick` or specific focus areas
- **Critical Finding**: Stop all deployment activities immediately and implement remediation plan before proceeding
- **Tool Missing**: Install required security tools and update PATH configuration

## Output
- **Success**: "Security audit completed successfully. No critical vulnerabilities found. Report saved to /workspaces/QuDAG/reports/security_audit_report.md"
- **Failure**: "Security audit identified {count} critical issues requiring immediate attention. See /workspaces/QuDAG/reports/vulnerability_findings.json"
- **Reports**: 
  - Security audit report with executive summary and detailed findings
  - Vulnerability findings in JSON format for tracking
  - Remediation plan with prioritized actions
  - Compliance matrix showing NIST PQC adherence

## Example Usage
```
/security-audit
/security-audit crypto comprehensive
/security-audit network --focus timing,side-channel
/security-audit --module all --depth quick
```

### Example Scenario
Running a comprehensive crypto module audit:
```
/security-audit crypto comprehensive

Starting security audit of crypto module...
✓ Static analysis completed - 0 unsafe blocks found
✓ ML-KEM-768 validation passed all NIST test vectors
✓ ML-DSA-65 validation passed all NIST test vectors
✓ HQC implementation validated against reference
✓ Constant-time operations verified
✓ Memory zeroization confirmed
✓ No timing vulnerabilities detected

Security audit completed successfully.
Reports generated:
- /workspaces/QuDAG/reports/security_audit_crypto_2024-01-15.md
- /workspaces/QuDAG/reports/compliance_matrix_crypto.csv
```

## Related Commands
- `/review-security`: Focused security code review for specific modules
- `/debug-security`: Debug and validate security configurations
- `/crypto-validate`: Deep validation of cryptographic implementations
- `/fuzz-test`: Execute fuzzing campaigns against security-critical code

## Workflow Integration
This command is part of the Security Workflow and:
- Follows: `/integration-test` (ensure system is stable before auditing)
- Precedes: `/deploy-validate` (security must pass before deployment)
- Can be run in parallel with: `/performance-benchmark` (if resources allow)

## Agent Coordination
- **Primary Agent**: Security Agent (leads the audit process)
- **Supporting Agents**: 
  - Crypto Agent: Validates quantum-resistant implementations
  - Network Agent: Assesses communication security
  - Integration Agent: Verifies component boundaries

## Notes
- Comprehensive audits can take 30-60 minutes depending on codebase size
- Always run with latest security tool versions for current vulnerability data
- Critical findings must be addressed before any deployment
- Quantum-resistant crypto validation requires access to NIST test vectors
- Consider running quick audits in CI/CD and comprehensive audits before releases