# /test/wasm

## Purpose
Execute comprehensive WASM testing across browser and Node.js environments, including unit tests, integration tests, performance benchmarks, and cross-platform compatibility validation.

## Parameters
- `<environment>`: Test environment - browser|node|all (default: all)
- `[test-type]`: Test type - unit|integration|benchmark|compatibility (default: all)
- `[browser]`: Browser for testing - chrome|firefox|safari|edge (default: chrome)
- `[headless]`: Run headless - true|false (default: true)

## Prerequisites
- [ ] WASM builds available in `/workspaces/QuDAG/qudag-wasm/pkg*/`
- [ ] wasm-bindgen-test installed
- [ ] Browser drivers installed (chromedriver, geckodriver)
- [ ] Node.js 16+ for Node.js testing
- [ ] Test fixtures in `/workspaces/QuDAG/qudag-wasm/tests/`

## Execution Steps

### 1. Validation Phase
- Verify WASM builds exist
- Check test environment setup
- Validate browser driver availability
- Confirm test dependencies

### 2. Environment Setup
- Install wasm-bindgen-test if missing
  ```bash
  cargo install wasm-bindgen-cli
  ```
- Set test environment variables
  ```bash
  export CHROMEDRIVER=/usr/local/bin/chromedriver
  export GECKODRIVER=/usr/local/bin/geckodriver
  ```

### 3. Unit Tests Execution
- Step 3.1: Run Node.js unit tests
  ```bash
  cd /workspaces/QuDAG/qudag-wasm
  wasm-pack test --node --release
  ```
- Step 3.2: Run browser unit tests
  ```bash
  wasm-pack test --chrome --headless --release
  wasm-pack test --firefox --headless --release
  ```
- Step 3.3: Run specific test modules
  ```bash
  wasm-pack test --node --release -- --test crypto_tests
  wasm-pack test --chrome --headless --release -- --test dag_tests
  ```

### 4. Integration Tests
- Step 4.1: Cross-environment integration
  ```bash
  # Test Node.js + Browser interaction
  node test-nodejs.mjs
  ```
- Step 4.2: API compatibility tests
  ```bash
  # Test different WASM targets
  node tests/api-compatibility.mjs
  ```
- Step 4.3: Real-world usage scenarios
  ```bash
  # Run example applications
  cd examples && node unified_crypto_example.js
  ```

### 5. Performance Benchmarks
- Step 5.1: Crypto performance benchmarks
  ```bash
  wasm-pack test --node --release -- --bench crypto_benchmarks
  ```
- Step 5.2: Memory usage analysis
  ```bash
  node tests/memory-benchmark.mjs
  ```
- Step 5.3: Load time analysis
  ```bash
  node tests/load-time-benchmark.mjs
  ```

### 6. Cross-Platform Compatibility
- Step 6.1: Browser compatibility matrix
  ```bash
  # Test across different browsers
  for browser in chrome firefox safari edge; do
    wasm-pack test --$browser --headless --release || echo "$browser failed"
  done
  ```
- Step 6.2: Node.js version compatibility
  ```bash
  # Test across Node.js versions
  for version in 16 18 20 latest; do
    docker run -v $(pwd):/app node:$version \
      sh -c "cd /app && npm test" || echo "Node $version failed"
  done
  ```

### 7. Web Integration Tests
- Step 7.1: HTML integration test
  ```bash
  # Serve test HTML and run automated tests
  python3 -m http.server 8000 &
  SERVER_PID=$!
  node tests/web-integration-test.mjs
  kill $SERVER_PID
  ```
- Step 7.2: Module bundler tests
  ```bash
  # Test with different bundlers
  cd tests/bundler-tests
  npm run test:webpack
  npm run test:rollup
  npm run test:vite
  ```

### 8. Security Tests
- Step 8.1: Memory safety validation
  ```bash
  wasm-pack test --node --release -- --test security_tests
  ```
- Step 8.2: Side-channel resistance
  ```bash
  node tests/timing-attack-resistance.mjs
  ```

## Success Criteria
- [ ] All unit tests pass in Node.js and browser environments
- [ ] Integration tests demonstrate cross-platform compatibility
- [ ] Performance benchmarks meet minimum thresholds
- [ ] No memory leaks detected in long-running tests
- [ ] Security tests pass with no vulnerabilities
- [ ] WASM modules load within 100ms in browser

## Error Handling
- **Browser driver not found**: Install appropriate driver (chromedriver, geckodriver)
- **WASM load failures**: Check WASM binary integrity and browser compatibility
- **Test timeouts**: Increase timeout values or optimize WASM performance
- **Memory issues**: Use smaller test datasets or implement streaming
- **Cross-origin errors**: Serve tests from HTTP server, not file:// protocol
- **Node.js import errors**: Verify Node.js WASM import syntax compatibility

## Output
- **Success**: Test results with pass/fail counts and performance metrics
- **Failure**: Detailed error messages with stack traces
- **Reports**: 
  - `/workspaces/QuDAG/qudag-wasm/test-results.json`: Comprehensive test results
  - `/workspaces/QuDAG/qudag-wasm/performance-report.html`: Performance analysis
  - `/workspaces/QuDAG/qudag-wasm/compatibility-matrix.md`: Browser/Node.js compatibility

## Example Usage
```bash
# Run all tests in all environments
/test/wasm all all chrome true

# Run only browser unit tests
/test/wasm browser unit chrome false

# Run Node.js integration tests
/test/wasm node integration
```

### Example Output
```
Running QuDAG WASM Tests...

Node.js Tests:
✓ crypto_tests (45ms)
✓ dag_tests (32ms)
✓ integration_tests (78ms)
✓ performance_tests (156ms)

Browser Tests (Chrome):
✓ crypto_tests (89ms)
✓ dag_tests (67ms)
✓ ui_integration_tests (234ms)

Performance Benchmarks:
- Key generation: 12.3ms (target: <20ms) ✓
- Signature: 8.7ms (target: <15ms) ✓
- Verification: 4.2ms (target: <10ms) ✓
- WASM load time: 45ms (target: <100ms) ✓

Compatibility Matrix:
- Chrome 118+: ✓ All tests pass
- Firefox 118+: ✓ All tests pass
- Safari 16+: ✓ All tests pass
- Node.js 16+: ✓ All tests pass
```

## Related Commands
- `/build/wasm`: Build WASM binaries before testing
- `/dev/tools`: Development server with live reload
- `/deploy/npm`: Deploy tested WASM package

## Workflow Integration
This command is part of the WASM Development workflow and:
- Follows: `/build/wasm` to build WASM binaries
- Precedes: `/deploy/npm` for publication
- Can be run in parallel with: Native Rust testing

## Agent Coordination
- **Primary Agent**: WASM Test Agent
- **Supporting Agents**: 
  - Browser Agent: Manages browser-specific testing
  - Performance Agent: Analyzes benchmark results
  - Security Agent: Validates security test outcomes

## Notes
- Browser testing requires display server or headless mode
- Some browsers may not support latest WASM features
- Performance tests should run on consistent hardware
- Memory usage tests need longer observation periods
- Cross-origin policies may affect browser testing

---

## Advanced Testing Scenarios

### Custom Test Configurations
```javascript
// tests/config.js
export const testConfig = {
  browsers: ['chrome', 'firefox', 'safari'],
  nodeVersions: ['16', '18', '20'],
  wasmFeatures: ['crypto-only', 'full'],
  performanceThresholds: {
    keyGen: 20, // ms
    signature: 15, // ms
    verification: 10 // ms
  }
};
```

### Automated Browser Testing
```bash
# Using Selenium for comprehensive browser testing
npm install selenium-webdriver
node tests/selenium-test-runner.mjs
```

### Memory Leak Detection
```javascript
// tests/memory-leak-test.mjs
const runMemoryTest = async () => {
  const initialMemory = process.memoryUsage();
  
  for (let i = 0; i < 1000; i++) {
    const wasm = await import('../pkg-node/qudag_wasm.js');
    await wasm.generate_keypair();
  }
  
  const finalMemory = process.memoryUsage();
  const memoryIncrease = finalMemory.heapUsed - initialMemory.heapUsed;
  
  console.log(`Memory increase: ${memoryIncrease / 1024 / 1024} MB`);
};
```

### Load Testing
```bash
# Concurrent WASM instance testing
node tests/concurrent-load-test.mjs --instances 100 --duration 60s
```

### CI/CD Integration
```yaml
# .github/workflows/wasm-test.yml
- name: Test WASM
  run: |
    /test/wasm all all chrome true
    /test/wasm node benchmark
```