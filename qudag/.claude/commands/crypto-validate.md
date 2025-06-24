# crypto-validate

Validate quantum-resistant cryptographic implementation against test vectors and security requirements

## Usage

```
/crypto-validate <algorithm> [options]
```

## Parameters

### algorithm (required)
- **Type**: string
- **Description**: Quantum-resistant cryptographic algorithm to validate
- **Allowed values**: ml-kem, ml-dsa, hqc, ml-kem-768, ml-dsa-65

### options
- **Type**: object
- **Description**: Validation options
  - `test_vectors` (boolean, default: true): Validate against NIST test vectors
  - `timing_analysis` (boolean, default: true): Perform constant-time analysis
  - `memory_safety` (boolean, default: true): Check secure memory handling

## Examples

```
/crypto-validate ml-kem
/crypto-validate ml-dsa --timing-analysis --memory-safety
/crypto-validate hqc --test-vectors
/crypto-validate ml-kem-768 --all
```

## Validation Steps

1. **Test Vector Validation**
   - Validate against NIST PQC test vectors
   - Files: `core/crypto/tests/ml_kem_tests.rs`, `core/crypto/tests/ml_dsa_tests.rs`

2. **Constant-time Analysis**
   - Verify operations are constant-time
   - Tools: dudect, ct-verif
   - Files: `core/crypto/tests/security/timing_analysis_tests.rs`

3. **Memory Safety Check**
   - Ensure secure memory handling and zeroization
   - Files: `core/crypto/tests/security/memory_tests.rs`

4. **Known Attack Resistance**
   - Test against known quantum and classical attacks
   - Attacks: side-channel, timing, fault-injection, quantum

5. **Compliance Report**
   - Verify NIST PQC compliance
   - Standards: FIPS 203, FIPS 204, FIPS 205

## Test Vectors

- **ml-kem**: `core/crypto/tests/.test_vectors/mlkem768_*.txt`
- **ml-dsa**: `core/crypto/tests/.test_vectors/mldsa65_*.txt`
- **hqc**: `core/crypto/tests/.test_vectors/hqc_*.txt`

## Output Format

```
1. Test Vector Validation Results
   - NIST vector compliance: PASS/FAIL
   - Custom vector tests: X/Y passed
2. Constant-time Analysis
   - Timing variance: < 0.01%
   - Statistical tests: PASS/FAIL
3. Memory Safety Check
   - Zeroization: VERIFIED
   - No memory leaks: CONFIRMED
4. Known Attack Resistance
   - Side-channel: RESISTANT
   - Quantum attacks: RESISTANT
5. Compliance Report
   - NIST PQC: COMPLIANT
   - Security level: X bits
```

## Security Requirements

- **Constant Time**: All operations must be constant-time
- **Memory Safety**: Secrets must be zeroized after use
- **Side Channel**: Implementation must resist timing attacks
- **Quantum Resistance**: Meet NIST PQC security levels

## Error Handling

- **invalid_algorithm**: Supported algorithms: ml-kem, ml-dsa, hqc
- **validation_failure**: Detailed validation error with specific test case
- **security_concern**: Critical security issue requiring immediate attention
- **test_vector_missing**: Required test vectors not found at expected path

## Agent Context

- **Primary Agent**: `agents/crypto_agent.md`
- **Workflow**: `workflow/security_workflow.md`
- **Security Context**: `contexts/security_context.md`