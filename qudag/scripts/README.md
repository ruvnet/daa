# QuDAG Compatibility Testing Scripts

This directory contains comprehensive compatibility testing tools for the QuDAG Protocol implementation.

## Overview

The compatibility testing system ensures that QuDAG works correctly across:
- Different Rust versions (MSRV compliance)
- Multiple target platforms (Linux, Windows, macOS, embedded)
- Various feature combinations (std/no_std, minimal/full features)
- Different compilation settings (optimization levels, security flags)

## Scripts

### `compatibility_test.sh`
**Primary comprehensive testing script**

```bash
# Run all compatibility tests
./compatibility_test.sh

# Run specific test categories
./compatibility_test.sh features    # Feature combinations
./compatibility_test.sh versions    # Rust version testing
./compatibility_test.sh cross      # Cross-compilation
./compatibility_test.sh nostd      # no_std compatibility
./compatibility_test.sh security   # Security-hardened flags
```

**Features:**
- Multi-Rust version testing (1.70.0 to nightly)
- Cross-compilation for 6+ target platforms
- Feature flag matrix testing
- no_std compatibility verification
- Security compiler flag testing
- Documentation generation testing
- Benchmark compilation verification

### `basic_compatibility_test.sh`
**Focused testing for core functionality**

```bash
# Run basic compatibility tests
./basic_compatibility_test.sh

# Run specific test suites
./basic_compatibility_test.sh features      # Basic feature tests
./basic_compatibility_test.sh functionality # Core functionality
./basic_compatibility_test.sh docs         # Documentation
./basic_compatibility_test.sh compiler     # Compiler flags
```

**Features:**
- Essential build and test verification
- Core functionality testing
- Documentation generation
- Basic cross-platform compatibility
- Faster execution for quick validation

### `test_matrix.py`
**Advanced Python-based matrix testing**

```bash
# Run comprehensive matrix testing
python3 test_matrix.py

# Run with custom worker count
python3 test_matrix.py 8

# Get help
python3 test_matrix.py --help
```

**Features:**
- Parallel test execution
- Comprehensive test matrix generation
- Detailed JSON reporting
- Configuration-driven testing
- Performance metrics collection

## Configuration

### `.compatibility.toml`
Main configuration file defining:

```toml
[rust_versions]
msrv = "1.70.0"
supported = ["1.70.0", "1.75.0", "1.80.0", "stable", "beta"]

[target_platforms]
tier1 = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "x86_64-apple-darwin"]
tier2 = ["x86_64-unknown-linux-musl", "aarch64-unknown-linux-gnu"]
experimental = ["wasm32-unknown-unknown", "riscv64gc-unknown-linux-gnu"]

[features]
combinations = [
    { name = "default", flags = [] },
    { name = "minimal", flags = ["--no-default-features"] },
    { name = "full", flags = ["--all-features"] }
]
```

## Test Categories

### Rust Version Compatibility
- **MSRV Testing**: Ensures code works with minimum supported Rust version
- **Stable/Beta/Nightly**: Tests against different Rust channels
- **Version Matrix**: Comprehensive testing across multiple versions

### Platform Compatibility
- **Tier 1 Platforms**: Must work (Linux, Windows, macOS)
- **Tier 2 Platforms**: Should work (musl, ARM64)
- **Experimental**: Nice to have (WASM, RISC-V)

### Feature Combinations
- **Default Features**: Standard configuration
- **Minimal Features**: `--no-default-features`
- **All Features**: `--all-features`
- **Custom Combinations**: Per-crate specific features

### Security Testing
- **Overflow Checks**: `-C overflow-checks=on`
- **Debug Assertions**: `-C debug-assertions=on`
- **Warnings as Errors**: `-D warnings`
- **Unsafe Code Denial**: `-D unsafe_code`

### no_std Compatibility
- **Embedded Targets**: ARM Cortex-M, RISC-V
- **Core Library Only**: No standard library dependencies
- **Cryptographic Primitives**: Constant-time operations

## Output and Reporting

### Console Output
```
==============================================
QuDAG Protocol Compatibility Testing
==============================================
Current Rust version: rustc 1.87.0
Date: Mon Jun 16 2025

[INFO] Testing feature combinations with current Rust version
[INFO] Running test: Features '' for core/crypto
[SUCCESS] Features '' for core/crypto
[INFO] Running test: Features '--no-default-features' for core/crypto
[SUCCESS] Features '--no-default-features' for core/crypto

==============================================
COMPATIBILITY TEST SUMMARY
==============================================
Total tests: 45
Passed: 42
Failed: 3
```

### JSON Reports
Detailed test results saved to `compatibility_report.json`:

```json
{
  "timestamp": 1703097600,
  "total_tests": 45,
  "passed_tests": 42,
  "failed_tests": 3,
  "results": [
    {
      "rust_version": "stable",
      "target_platform": "x86_64-unknown-linux-gnu",
      "crate": "qudag-crypto",
      "features": "default",
      "success": true,
      "duration": 12.34
    }
  ]
}
```

## Integration with CI/CD

### GitHub Actions
The compatibility tests are integrated into GitHub Actions workflow:

```yaml
name: Compatibility Testing
on: [push, pull_request, schedule]

jobs:
  basic-compatibility:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
```

### Local Development
```bash
# Quick compatibility check before commit
./scripts/basic_compatibility_test.sh

# Full compatibility verification before release
./scripts/compatibility_test.sh

# Generate comprehensive report
python3 scripts/test_matrix.py
```

## Requirements

### System Requirements
- **Rust**: 1.70.0 or later
- **Python**: 3.8+ (for matrix testing)
- **Git**: For version control integration
- **Disk Space**: ~2GB for all toolchains and targets

### Optional Dependencies
- **rustup**: For multi-version testing
- **cross**: For cross-compilation (alternative to rustup targets)
- **docker**: For containerized testing environments

## Troubleshooting

### Common Issues

1. **Rust Version Installation Failures**
   ```bash
   # Install specific Rust version
   rustup toolchain install 1.70.0
   rustup default 1.70.0
   ```

2. **Target Platform Installation**
   ```bash
   # Add cross-compilation targets
   rustup target add x86_64-unknown-linux-musl
   rustup target add wasm32-unknown-unknown
   ```

3. **Permission Errors**
   ```bash
   # Make scripts executable
   chmod +x scripts/*.sh
   ```

4. **Dependency Lock Issues**
   ```bash
   # Clear cargo cache and rebuild
   cargo clean
   rm -rf target/
   cargo build
   ```

### Debug Mode
Run tests with debug output:
```bash
# Enable debug logging
RUST_LOG=debug ./scripts/compatibility_test.sh

# Verbose output
./scripts/compatibility_test.sh 2>&1 | tee compatibility.log
```

## Contributing

### Adding New Tests
1. Update `.compatibility.toml` with new configurations
2. Modify test scripts to include new scenarios
3. Update documentation and examples
4. Test changes with existing test suite

### Adding New Platforms
1. Define platform in `.compatibility.toml`
2. Add platform-specific test logic
3. Update CI/CD configuration
4. Document platform-specific requirements

### Reporting Issues
Include the following when reporting compatibility issues:
- Operating system and version
- Rust version (`rustc --version`)
- Target platform
- Feature configuration
- Full error output
- Compatibility test results

## Best Practices

1. **Run Basic Tests First**: Use `basic_compatibility_test.sh` for quick validation
2. **Full Testing Before Releases**: Run comprehensive tests before major releases
3. **Regular CI Testing**: Ensure CI tests run on all supported platforms
4. **Update MSRV Carefully**: Only increase MSRV when necessary
5. **Document Breaking Changes**: Update compatibility documentation for breaking changes

## Future Enhancements

- [ ] WebAssembly runtime testing
- [ ] Performance regression detection
- [ ] Automated dependency updates
- [ ] Security vulnerability scanning
- [ ] Memory usage profiling
- [ ] Fuzzing integration
- [ ] Custom test environments