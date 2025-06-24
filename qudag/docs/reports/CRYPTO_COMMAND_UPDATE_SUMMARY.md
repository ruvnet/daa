# Crypto Command Updates Summary

## Updated Files

### 1. crypto-validate.json
- Added `agent_context` section referencing:
  - Primary: `agents/crypto_agent.md`
  - Workflow: `workflow/security_workflow.md`
  - Security context: `contexts/security_context.md`
- Enhanced parameter definitions with specific quantum-resistant algorithms
- Added `test_vectors` section pointing to actual test vector locations
- Included detailed `validation_steps` with specific files and tools
- Added `security_requirements` section for crypto-specific requirements
- Enhanced output format with detailed validation results

### 2. fuzz-test.json
- Added `agent_context` section referencing:
  - Primary: `agents/security_agent.md`
  - Crypto: `agents/crypto_agent.md`
  - Workflow: `workflow/security_workflow.md`
  - Security context: `contexts/security_context.md`
- Added specific crypto fuzzing targets:
  - `crypto-ml-kem`: ML-KEM encapsulation/decapsulation fuzzing
  - `crypto-ml-dsa`: ML-DSA signing/verification fuzzing
  - `crypto-hqc`: HQC encryption/decryption fuzzing
  - `crypto-input`: General crypto input validation
- Added `crypto_targets` section with harness locations and focus areas
- Enhanced output format with security analysis section
- Added `security_focus` section for crypto-specific vulnerabilities

### 3. security-audit.json
- Enhanced crypto review prompts with specific algorithms:
  - ML-KEM validation against NIST FIPS 203
  - ML-DSA verification against NIST FIPS 204
  - HQC implementation checking
  - Test vector validation
- Added `crypto_specific` section with:
  - ML-KEM: FIPS 203, Level 3 security, specific checks
  - ML-DSA: FIPS 204, Level 3 security, specific checks
  - HQC: NIST Round 4, Level 1 security, specific checks
- Updated validation criteria to include quantum-resistant requirements
- Added NIST PQC compliance verification to reporting

### 4. debug-security.json
- Enhanced crypto settings check with specific parameters:
  - ML-KEM-768 parameter validation (k=3, n=256, q=3329)
  - ML-DSA-65 signature parameters
  - HQC error correction parameters
  - ChaCha20Rng CSPRNG seeding
- Added crypto library version checking:
  - pqcrypto-mlkem768
  - pqcrypto-mldsa65
  - hqc-rust
- Added `crypto_diagnostics` section with algorithm-specific checks
- Included test vector paths for each algorithm

## Key Improvements

1. **Algorithm-Specific Context**: Each command now has detailed knowledge of ML-KEM, ML-DSA, and HQC requirements
2. **Test Vector Integration**: Commands reference actual test vector locations
3. **Security Workflow Integration**: All commands reference the security workflow for consistent processes
4. **Agent Coordination**: Commands specify which agents should handle crypto-related tasks
5. **Compliance Focus**: NIST PQC compliance verification is integrated throughout
6. **Timing Attack Prevention**: Constant-time operation verification is emphasized
7. **Memory Safety**: Secure memory handling and zeroization checks are included

## Usage Examples

```bash
# Validate ML-KEM implementation
/crypto-validate ml-kem

# Fuzz test ML-DSA with extended duration
/fuzz-test crypto-ml-dsa --duration 2h

# Run comprehensive crypto security audit
/security-audit crypto --depth comprehensive

# Debug crypto configuration with verbose output
/debug-security --scope crypto --verbose
```

These updates ensure that all crypto-related commands have proper context for quantum-resistant cryptographic validation and testing.