# /fuzz-test

## Purpose
Execute comprehensive fuzzing campaigns against QuDAG components with focus on quantum-resistant cryptographic implementations, using coverage-guided fuzzing to discover security vulnerabilities, crashes, and edge cases.

## Parameters
- `<target>`: Component to fuzz test (required) - one of:
  - `crypto-ml-kem`: ML-KEM encapsulation/decapsulation
  - `crypto-ml-dsa`: ML-DSA signing/verification  
  - `crypto-hqc`: HQC encryption/decryption
  - `crypto-input`: General crypto input validation
  - `network-messages`: Network protocol messages
  - `dag-consensus`: DAG consensus state transitions
  - `protocol-state`: Protocol state machine
- `[duration]`: Fuzzing duration (default: "1h") - format: `<number>[h|m|s]`
- `[--corpus]`: Path to seed corpus (default: `/workspaces/QuDAG/fuzz/corpus/{target}`)
- `[--max-len]`: Maximum input length in bytes (default: 10240)
- `[--sanitizers]`: Enable sanitizers - comma-separated: `address,memory,undefined,thread` (default: address,memory)

## Prerequisites
- [ ] Rust nightly toolchain installed (`rustup install nightly`)
- [ ] cargo-fuzz installed (`cargo install cargo-fuzz`)
- [ ] Fuzz targets implemented in `/workspaces/QuDAG/fuzz/fuzz_targets/`
- [ ] Seed corpus available or will be generated
- [ ] Sufficient disk space for crash artifacts (>1GB recommended)

## Execution Steps

### 1. Validation Phase
- Verify target is valid and fuzz harness exists
- Check nightly toolchain availability
- Validate duration format and sanitizer options
- Ensure output directories exist:
  ```bash
  mkdir -p /workspaces/QuDAG/fuzz/corpus/${target}
  mkdir -p /workspaces/QuDAG/fuzz/artifacts/${target}
  ```

### 2. Planning Phase
- Load security agent context from `/workspaces/QuDAG/.claude/commands/agents/security_agent.md`
- For crypto targets, also load crypto agent context
- Identify specific vulnerabilities to target based on component type
- Set up sanitizer flags and fuzzing parameters

### 3. Implementation Phase
- Step 3.1: Build fuzz target with sanitizers
  ```bash
  cd /workspaces/QuDAG
  cargo +nightly fuzz build ${target} --sanitizer ${sanitizers}
  ```
  
- Step 3.2: Initialize or verify seed corpus
  ```bash
  # For crypto targets, generate cryptographic test inputs
  if [[ ${target} == crypto-* ]]; then
    # Generate valid key pairs, ciphertexts, signatures as seeds
    cargo run --bin generate-fuzz-corpus -- ${target}
  fi
  ```
  
- Step 3.3: Execute fuzzing campaign
  ```bash
  # Run fuzzer with specified parameters
  cargo +nightly fuzz run ${target} -- \
    -max_len=${max_len} \
    -max_total_time=${duration_seconds} \
    -print_final_stats=1 \
    -detect_leaks=1 \
    ${corpus_path}
  ```
  
- Step 3.4: Monitor for crashes and interesting inputs
  - Track unique crashes in real-time
  - Save interesting inputs that increase coverage
  - Monitor memory usage and performance

### 4. Verification Phase
- Analyze crash artifacts in `/workspaces/QuDAG/fuzz/artifacts/${target}/`
- Minimize crashing inputs to smallest reproducers
- Verify crashes are genuine vulnerabilities:
  ```bash
  # Reproduce each crash
  for crash in fuzz/artifacts/${target}/crash-*; do
    cargo +nightly fuzz run ${target} < ${crash}
  done
  ```
- Generate coverage report

### 5. Documentation Phase
- Create detailed fuzzing report
- Document all discovered vulnerabilities
- Generate reproducer test cases
- Update security documentation

## Success Criteria
- [ ] Fuzzing campaign completes full duration without errors
- [ ] Code coverage exceeds 80% for target component
- [ ] All discovered crashes are analyzed and documented
- [ ] Memory leaks and undefined behavior detected by sanitizers
- [ ] Security vulnerabilities properly categorized by severity

## Error Handling
- **Invalid Target**: Show valid targets: crypto-ml-kem, crypto-ml-dsa, crypto-hqc, crypto-input, network-messages, dag-consensus, protocol-state
- **Build Failure**: Check nightly toolchain and run `cargo clean`
- **Out of Memory**: Reduce max_len or use smaller corpus
- **Sanitizer Crash**: May indicate real vulnerability - save artifact immediately
- **Corpus Missing**: Auto-generate minimal corpus or use empty directory

## Output
- **Success**: 
  ```
  Fuzzing Campaign Report: crypto-ml-kem
  =====================================
  1. Campaign Summary
     - Target: crypto-ml-kem  
     - Duration: 1h 00m 00s
     - Total executions: 12,450,000
     - Executions/sec: 3,458
  
  2. Code Coverage
     - Line coverage: 87.3%
     - Branch coverage: 82.1%
     - New edges found: 234
  
  3. Found Issues
     - Crashes: 3
     - Timeouts: 0
     - Memory leaks: 1
     - Security issues: 2
  
  4. Crash Analysis
     - Unique crashes: 2
     - CVE-worthy: 1 (buffer overflow in ciphertext parsing)
     - Artifacts: /workspaces/QuDAG/fuzz/artifacts/crypto-ml-kem/
  
  5. Security Analysis
     - Timing variations: Found in keygen (non-constant time)
     - Memory safety: Use-after-free in error path
     - Input validation: Missing bounds check on public key size
  
  6. Recommendations
     - Priority: Fix buffer overflow in ml_kem_decapsulate()
     - Add input validation for ciphertext length
     - Make keygen timing constant
  ```

- **Failure**: Error details with fuzzer output and diagnostics
- **Reports**: 
  - Summary: `/workspaces/QuDAG/reports/fuzz-${target}-${timestamp}.md`
  - Crashes: `/workspaces/QuDAG/fuzz/artifacts/${target}/`
  - Coverage: `/workspaces/QuDAG/fuzz/coverage/${target}.html`

## Example Usage
```
/fuzz-test crypto-ml-kem
/fuzz-test crypto-ml-dsa --duration 2h
/fuzz-test network-messages --duration 30m --sanitizers address,thread
/fuzz-test crypto-input --corpus /workspaces/QuDAG/fuzz/seed-corpus/crypto --max-len 50000
```

### Example Scenario
Fuzzing ML-KEM implementation to find vulnerabilities:
```
/fuzz-test crypto-ml-kem --duration 4h --sanitizers address,memory,undefined
# This will:
# 1. Build ML-KEM fuzz harness with all sanitizers
# 2. Generate corpus with valid/invalid keys and ciphertexts
# 3. Run 4-hour fuzzing campaign targeting:
#    - Malformed public/private keys
#    - Invalid ciphertext formats
#    - Edge cases in polynomial arithmetic
#    - Memory safety violations
# 4. Produce actionable security report with reproducers
```

## Related Commands
- `/crypto-validate`: Validate crypto implementation against test vectors
- `/security-audit`: Comprehensive security analysis
- `/debug-security`: Debug specific security issues
- `/create-test`: Create tests from fuzzer discoveries

## Workflow Integration
This command is part of the Security Workflow and:
- Follows: `/implement-feature` for new components
- Precedes: `/security-audit` for comprehensive analysis
- Can be run in parallel with: `/performance-benchmark`

## Agent Coordination
- **Primary Agent**: Security Agent - manages fuzzing campaigns
- **Supporting Agents**: 
  - Crypto Agent: Provides crypto-specific fuzzing guidance
  - Performance Agent: Analyzes performance impact of fixes

## Notes
- Fuzzing is CPU-intensive - use dedicated resources when possible
- Longer campaigns (>4h) significantly increase bug discovery rate
- Address sanitizer may 2-3x slowdown but catches more bugs
- Save all crashing inputs for regression testing
- Consider AFL++ for advanced fuzzing features
- For crypto fuzzing, focus on:
  - Malformed keys and ciphertexts
  - Integer overflows in size calculations  
  - Timing variations revealing secrets
  - Memory corruption in error paths
- Network fuzzing should test protocol violations
- Consensus fuzzing must check Byzantine behaviors