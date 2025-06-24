# /review-security

## Purpose
Perform security-focused code review of QuDAG modules, examining cryptographic implementations, memory safety patterns, and potential attack vectors with emphasis on quantum-resistant security requirements.

## Parameters
- `<module>`: Module to review (crypto, network, dag, protocol) - required
- `[file_pattern]`: Optional file pattern to focus review (e.g., '**/*.rs', 'src/transport.rs')
- `[checklist]`: Optional array of custom security checklist items to verify

## Prerequisites
- [ ] Module exists and compiles successfully
- [ ] Security context is up to date at `/workspaces/QuDAG/.claude/contexts/security_context.md`
- [ ] Access to module source code and tests
- [ ] Understanding of quantum-resistant cryptographic requirements

## Execution Steps

### 1. Validation Phase
- Validate module parameter is one of: crypto, network, dag, protocol
- Check module exists at `/workspaces/QuDAG/core/{module}/`
- Verify file pattern matches existing files if provided
- Load security workflow from `/workspaces/QuDAG/.claude/commands/workflow/security_workflow.md`

### 2. Planning Phase
- Analyze module structure and identify security-critical components
- Create review checklist combining standard items with any custom additions
- Identify trust boundaries and external interfaces
- Map potential attack vectors specific to the module

### 3. Implementation Phase
- Step 3.1: Code Security Analysis
  ```bash
  # Scan for unsafe code blocks
  rg "unsafe" /workspaces/QuDAG/core/${module} --type rust -C 5
  # Check error handling patterns
  rg "unwrap\(\)|expect\(" /workspaces/QuDAG/core/${module} --type rust
  # Review panic usage
  rg "panic!|unreachable!" /workspaces/QuDAG/core/${module} --type rust
  ```

- Step 3.2: Cryptographic Security Review (if applicable)
  - Verify constant-time operations using `subtle` crate
  - Check proper key derivation and management
  - Validate secure random number generation with ChaCha20Rng
  - Ensure proper memory cleanup with `zeroize`
  - Review quantum-resistant algorithm implementations:
    - ML-KEM: Matrix operations, polynomial arithmetic
    - ML-DSA: Signature generation, rejection sampling
    - HQC: Error correction, syndrome decoding

- Step 3.3: Memory Safety Verification
  - Check for potential buffer overflows
  - Verify bounds checking on all array/vector access
  - Review unsafe transmutations
  - Validate proper lifetime management
  - Ensure secrets are zeroized on drop

- Step 3.4: Trust Boundary Analysis
  - Identify all external inputs to the module
  - Verify input validation at trust boundaries
  - Check for TOCTOU vulnerabilities
  - Review serialization/deserialization security
  - Validate access control implementations

- Step 3.5: Attack Vector Assessment
  - Timing attacks: Review operations on secret data
  - Side-channel attacks: Check for data-dependent operations
  - Resource exhaustion: Verify rate limiting and bounds
  - Injection attacks: Validate all external data parsing
  - Replay attacks: Check nonce/timestamp usage

### 4. Verification Phase
- Run security-specific lints
  ```bash
  cargo clippy -p qudag-${module} -- -W clippy::all -D warnings
  ```
- Execute module security tests
  ```bash
  cargo test -p qudag-${module} --features security-tests
  ```
- Validate against security checklist items
- Cross-reference with known vulnerability patterns
- Verify compliance with security requirements

### 5. Documentation Phase
- Generate security review report at `/workspaces/QuDAG/reports/security_review_{module}.md`
- Create vulnerability report at `/workspaces/QuDAG/reports/vulnerability_report_{module}.json`
- Build risk matrix at `/workspaces/QuDAG/reports/risk_matrix_{module}.csv`
- Update security context with findings
- Document any false positives or accepted risks

## Success Criteria
- [ ] No unsafe code without proper justification and review comments
- [ ] All external inputs validated at module boundaries
- [ ] Cryptographic operations follow constant-time principles
- [ ] Memory containing secrets properly zeroized
- [ ] No high-risk vulnerabilities identified
- [ ] All checklist items verified or documented as N/A
- [ ] Review report generated with actionable findings

## Error Handling
- **Invalid Module**: Verify module name is one of: crypto, network, dag, protocol
- **Scan Failure**: Check module path exists and retry with `--verbose` flag
- **High Risk Finding**: Stop any deployment plans, notify team lead, and prioritize remediation
- **Checklist Parse Error**: Verify checklist items are properly formatted strings
- **Missing Dependencies**: Install required analysis tools with cargo

## Output
- **Success**: "Security review completed for {module}. {passed}/{total} checks passed. Report: /workspaces/QuDAG/reports/security_review_{module}.md"
- **Failure**: "Security review identified {count} high-risk issues in {module}. Immediate action required. See: /workspaces/QuDAG/reports/vulnerability_report_{module}.json"
- **Reports**:
  - Detailed security review with findings and recommendations
  - Vulnerability report with severity ratings and remediation steps
  - Risk matrix showing likelihood vs impact assessment

## Example Usage
```
/review-security crypto
/review-security network --file_pattern "src/transport.rs"
/review-security protocol --checklist "Check message authentication" "Verify replay protection"
```

### Example Scenario
Reviewing the crypto module for security issues:
```
/review-security crypto

Starting security review of crypto module...

Code Analysis:
✓ No unsafe blocks found without justification
✓ Error handling uses Result types consistently
✓ No direct unwrap() calls on Results

Cryptographic Review:
✓ ML-KEM operations use constant-time arithmetic
✓ ML-DSA uses deterministic signing
✓ ChaCha20Rng properly seeded from OS entropy
✓ All keys implement Zeroize trait

Memory Safety:
✓ No buffer overflow risks identified
✓ Bounds checking on all polynomial operations
✓ Secrets cleared on drop

Security Checklist Results:
✓ 18/20 checks passed
⚠ 2 items need attention (see report)

Review completed. Report saved to:
/workspaces/QuDAG/reports/security_review_crypto_2024-01-15.md
```

## Related Commands
- `/security-audit`: Comprehensive security analysis of entire system
- `/debug-security`: Debug security configurations and settings
- `/crypto-validate`: Deep validation of cryptographic correctness
- `/fuzz-test`: Fuzz testing for security vulnerabilities

## Workflow Integration
This command is part of the Security Workflow and:
- Follows: `/implement-feature` (review security after implementation)
- Precedes: `/security-audit` (focused review before comprehensive audit)
- Can be run in parallel with: `/refactor-optimize` (coordinate changes)

## Agent Coordination
- **Primary Agent**: Security Agent (conducts the review)
- **Supporting Agents**: 
  - Crypto Agent: Assists with cryptographic analysis
  - Network Agent: Reviews network security aspects
  - Integration Agent: Validates module interactions

## Notes
- Always review cryptographic code with extra scrutiny
- Focus on data flow from untrusted sources
- Consider both implementation bugs and design flaws
- Quantum-resistant algorithms require specialized knowledge
- Document accepted risks with clear justification
- Security reviews should be performed by someone other than the original author when possible