# QuDAG Command Reference Directory

This directory contains command references optimized for QuDAG quantum-resistant distributed ledger development. Each subdirectory focuses on specific aspects of the QuDAG ecosystem.

## Directory Structure

```
.claude/commands/
├── README.md              # This file - overview and quick reference
├── build/                 # Build system commands
│   ├── cargo.txt         # Rust/Cargo build commands
│   ├── wasm-pack.txt     # WebAssembly packaging
│   └── npm.txt           # Node.js package management
├── crypto/               # Cryptographic operations
│   ├── ml-dsa.txt        # ML-DSA (FIPS 204) signature operations
│   ├── ml-kem.txt        # ML-KEM (FIPS 203) key encapsulation
│   └── hqc.txt           # HQC code-based cryptography
├── network/              # Network and P2P operations
│   ├── peers.txt         # Peer discovery and management
│   ├── routing.txt       # Network routing commands
│   └── gossip.txt        # Gossip protocol operations
├── dag/                  # DAG consensus and validation
│   ├── consensus.txt     # Consensus mechanism commands
│   ├── validation.txt    # Transaction validation
│   └── finality.txt      # Finality confirmation
├── dark/                 # Dark domain system
│   ├── addressing.txt    # Dark addressing schemes
│   ├── domains.txt       # Domain management
│   └── fingerprints.txt  # Identity fingerprinting
├── vault/                # Vault operations
│   ├── passwords.txt     # Password management
│   ├── secrets.txt       # Secret storage
│   └── backup.txt        # Backup and recovery
├── test/                 # Testing commands
│   ├── security.txt      # Security testing
│   ├── performance.txt   # Performance benchmarks
│   ├── integration.txt   # Integration tests
│   └── crypto-timing.txt # Cryptographic timing tests
├── deploy/               # Deployment operations
│   ├── publishing.txt    # Crate publishing
│   ├── releases.txt      # Release management
│   └── testnet.txt       # Testnet deployment
└── dev/                  # Development utilities
    ├── debugging.txt     # Debugging tools
    ├── profiling.txt     # Performance profiling
    └── monitoring.txt    # System monitoring
```

## QuDAG Optimization Approach

This command structure is designed around QuDAG's core principles:

### 1. **Quantum-Resistant Security**
- Commands prioritize post-quantum cryptographic operations
- All security commands assume quantum threat models
- Emphasis on ML-DSA, ML-KEM, and HQC implementations

### 2. **DAG-Based Consensus**
- Commands support parallel transaction processing
- Focus on asynchronous finality mechanisms
- Optimized for high-throughput distributed operations

### 3. **Dark Domain Integration**
- Specialized commands for privacy-preserving operations
- Support for anonymous addressing and domain management
- Identity protection through cryptographic fingerprinting

### 4. **Cross-Platform Support**
- Native Rust commands for core operations
- WebAssembly commands for browser integration
- Node.js commands for JavaScript ecosystems

## Quick Reference

### Most Common Operations

```bash
# Build the entire project
cargo build --release

# Run security tests
cargo test security_tests

# Start testnet deployment
./scripts/deploy-testnet.sh

# Generate ML-DSA keypair
cargo run --bin keygen -- --algorithm ml-dsa

# Validate DAG structure
cargo run --bin dag-validator
```

### Development Workflow

1. **Setup**: Use `build/` commands to configure development environment
2. **Security**: Reference `crypto/` commands for cryptographic operations
3. **Testing**: Apply `test/` commands for comprehensive validation
4. **Deployment**: Execute `deploy/` commands for release management

### Integration Points

- **Claude-Flow**: All commands integrate with `./claude-flow` orchestration
- **Memory System**: Commands store results in persistent memory
- **Batch Operations**: Commands support parallel execution patterns
- **SPARC Modes**: Commands align with SPARC development methodologies

## Usage Guidelines

1. **Command Files**: Each `.txt` file contains specific commands for that category
2. **Context Awareness**: Commands include QuDAG-specific parameters and flags
3. **Security First**: All commands prioritize security and quantum resistance
4. **Performance Optimized**: Commands focus on high-throughput operations
5. **Developer Experience**: Commands are optimized for Claude Code integration

## Integration with Claude-Flow

These commands are designed to work seamlessly with the Claude-Flow orchestration system:

```bash
# Use specific command category
./claude-flow sparc run coder "Implement ML-DSA signing using crypto/ml-dsa.txt commands"

# Coordinate multiple categories
./claude-flow swarm "Deploy QuDAG testnet" --strategy development --mode hierarchical
```

---

**Note**: This directory structure will be populated by specialized agents, each focusing on their domain expertise within the QuDAG ecosystem.

## Legacy Commands and Advanced Workflows

The following sections contain the existing command system for reference:

### Primary Development Commands

Core development commands following Test-Driven Development (TDD) methodology.

| Command | Description | Example |
|---------|-------------|---------|
| `/tdd-cycle` | Execute complete TDD cycle for a feature | `/tdd-cycle crypto ml_kem_implementation` |
| `/security-audit` | Comprehensive security analysis and testing | `/security-audit crypto --depth comprehensive` |
| `/performance-benchmark` | Run all benchmarks and generate reports | `/performance-benchmark network --criteria throughput` |
| `/integration-test` | Execute full integration test suite | `/integration-test crypto` |
| `/deploy-validate` | Validate deployment configuration and test | `/deploy-validate --environment staging` |

### Development Workflow Commands

Supporting development workflow commands for feature implementation.

| Command | Description | Example |
|---------|-------------|---------|
| `/create-test` | Generate test skeleton for new feature | `/create-test tests/crypto/ml_kem_test.rs 'ML-KEM key generation'` |
| `/implement-feature` | Implement feature to pass specified tests | `/implement-feature tests/crypto/ml_kem_test.rs` |
| `/refactor-optimize` | Refactor module while maintaining test coverage | `/refactor-optimize crypto --focus performance` |
| `/review-security` | Security-focused code review | `/review-security crypto` |
| `/update-docs` | Update documentation for module changes | `/update-docs network --sections api,examples` |

### Specialized Commands

Module-specific commands for advanced operations.

| Command | Description | Example |
|---------|-------------|---------|
| `/crypto-validate` | Validate cryptographic implementation | `/crypto-validate ml-kem --compliance` |
| `/network-simulate` | Run network simulation scenarios | `/network-simulate high-latency --nodes 1000` |
| `/dag-visualize` | Generate DAG state visualization | `/dag-visualize --format png` |
| `/fuzz-test` | Execute fuzzing campaign against target | `/fuzz-test crypto_parser --duration 1h` |

### Debug Commands

Debugging and diagnostics commands for troubleshooting.

| Command | Description | Example |
|---------|-------------|---------|
| `/debug-network` | Diagnose networking issues | `/debug-network` |
| `/debug-consensus` | Analyze consensus state | `/debug-consensus` |
| `/debug-performance` | Profile performance bottlenecks | `/debug-performance` |
| `/debug-security` | Check security configurations | `/debug-security` |

## Command Details

### `/tdd-cycle <module> <feature>`

Execute a complete Test-Driven Development cycle for a feature.

**Parameters:**
- `module` (required): Target module (crypto|dag|network|protocol)
- `feature` (required): Feature name to implement

**Workflow:** `workflow/tdd_workflow.md`

**Agents:**
- crypto → `agents/crypto_agent.md`
- network → `agents/network_agent.md`
- dag → `agents/consensus_agent.md`
- protocol → `agents/integration_agent.md`

### `/security-audit [module] [--depth]`

Perform comprehensive security analysis and testing.

**Parameters:**
- `module` (optional): Module to audit (crypto|network|dag|protocol|all) - default: all
- `depth` (optional): Audit depth (quick|standard|comprehensive) - default: comprehensive

**Workflow:** `workflow/security_workflow.md`

**Agent:** `agents/security_agent.md`

**Contexts:**
- `contexts/security_context.md`
- `contexts/test_status.md`

### `/performance-benchmark [module] [--criteria] [--baseline]`

Run benchmarks and generate performance reports.

**Parameters:**
- `module` (optional): Module to benchmark (crypto|dag|network|protocol|all) - default: all
- `criteria` (optional): Performance criteria (throughput|latency|memory|scalability)
- `baseline` (optional): Compare against baseline - default: true

**Workflow:** `workflow/performance_workflow.md`

**Agent:** `agents/performance_agent.md`

**Contexts:**
- `contexts/performance_context.md`
- `contexts/test_status.md`

## Quick Reference

### Development Flow
1. `/create-test` - Write failing tests
2. `/tdd-cycle` - Execute TDD cycle
3. `/implement-feature` - Implement to pass tests
4. `/integration-test` - Run integration tests
5. `/security-audit` - Security validation
6. `/performance-benchmark` - Performance testing
7. `/refactor-optimize` - Optimize implementation
8. `/deploy-validate` - Validate deployment

### Module-Agent Mapping
- **Crypto Module** → `crypto_agent`
- **Network Module** → `network_agent`
- **DAG/Consensus** → `consensus_agent`
- **Protocol** → `integration_agent`
- **Security** → `security_agent`
- **Performance** → `performance_agent`

### Available Workflows
- **TDD Workflow** → Development cycle implementation
- **Security Workflow** → Security audit and validation
- **Performance Workflow** → Benchmarking and optimization
- **Deployment Workflow** → Deployment validation

## Command Categories Explained

### 1. Development Workflow
Commands that support the core TDD development cycle:
- `tdd-cycle`: Full TDD implementation (RED-GREEN-REFACTOR)
- `create-test`: Test scaffolding for new features
- `implement-feature`: Feature implementation to pass tests
- `refactor-optimize`: Code improvement while maintaining tests

### 2. Security & Compliance
Commands focused on security validation:
- `security-audit`: Comprehensive vulnerability analysis
- `review-security`: Module-specific security review
- `crypto-validate`: Cryptographic standard compliance
- `fuzz-test`: Automated vulnerability discovery

### 3. Performance & Optimization
Commands for performance management:
- `performance-benchmark`: Throughput and latency measurement
- `refactor-optimize`: Performance-focused refactoring
- `debug-performance`: Bottleneck identification

### 4. Testing & Validation
Commands for various testing needs:
- `integration-test`: Cross-module system testing
- `network-simulate`: Protocol behavior under various conditions
- `fuzz-test`: Edge case and robustness testing

### 5. Debugging & Diagnostics
Commands for troubleshooting:
- `debug-network`: Network connectivity issues
- `debug-consensus`: DAG consensus problems
- `debug-performance`: Performance degradation
- `debug-security`: Security configuration issues

## Practical Usage Examples

### Example 1: Implementing a New Cryptographic Feature

```bash
# 1. Start by creating comprehensive tests
Execute create-test for crypto/test_hqc.rs "HQC encryption and decryption"

# 2. Run the full TDD cycle
Execute tdd-cycle for crypto hqc_implementation

# 3. Validate cryptographic compliance
Execute crypto-validate for HQC

# 4. Perform security audit
Execute security-audit for crypto

# 5. Benchmark performance
Execute performance-benchmark for crypto
```

### Example 2: Debugging Network Issues

```bash
# 1. Start with diagnostics
Execute debug-network

# 2. Simulate problematic scenario
Execute network-simulate for latency_test

# 3. Run integration tests
Execute integration-test for network_flow

# 4. Security review
Execute review-security for network
```

### Example 3: Complete Feature Development

```bash
# 1. Test specification
Execute create-test for dag/test_consensus.rs "QR-Avalanche consensus algorithm"

# 2. TDD implementation
Execute tdd-cycle for dag qr_avalanche

# 3. Integration testing
Execute integration-test

# 4. Security validation
Execute security-audit for dag

# 5. Performance optimization
Execute refactor-optimize for dag

# 6. Final benchmarks
Execute performance-benchmark

# 7. Documentation update
Execute update-docs for dag
```

## Creating New Commands

To create a new command:

1. Copy `TEMPLATE.md` to `your-command.md`
2. Fill in all required sections:
   - Command metadata
   - Clear objectives
   - Input parameters
   - Step-by-step workflow
   - Validation criteria
   - Examples
3. Test the command thoroughly
4. Update this README with the new command

See `TEMPLATE.md` for the complete command template structure.

## Best Practices

1. **Always start with tests**: Use `create-test` before implementing any feature
2. **Follow TDD strictly**: Use `tdd-cycle` for all new implementations
3. **Validate security early**: Run `security-audit` after significant changes
4. **Benchmark regularly**: Use `performance-benchmark` to catch regressions
5. **Document changes**: Use `update-docs` to keep documentation current
6. **Debug systematically**: Use appropriate debug commands for issues

## Command Execution Flow

```
User Request → Command Parser → Workflow Execution → Result Validation → Report Generation
                                        ↓
                                  Agent Execution
                                        ↓
                                  Context Updates
```

## Integration with CI/CD

These commands integrate with the QuDAG CI/CD pipeline:

- **Pre-commit hooks**: `review-security`, `create-test`
- **CI build stage**: `integration-test`, `security-audit`
- **Performance gates**: `performance-benchmark`
- **Deployment stage**: `deploy-validate`

## Troubleshooting Command Failures

If a command fails:

1. **Check prerequisites**: Ensure dependencies and test data exist
2. **Verify parameters**: Confirm module names and paths are correct
3. **Review test status**: Ensure all tests pass before running commands
4. **Check error output**: Look for specific error messages in output
5. **Use debug commands**: Run appropriate debug command for investigation

## Tips for Effective Command Usage

- **Batch related commands**: Group security and performance tests together
- **Use command options**: Leverage parameters for targeted execution
- **Monitor progress**: Check context files for command status
- **Review outputs**: Examine generated reports and logs
- **Iterate quickly**: Use quick validation before comprehensive tests

## Related Files
- `TEMPLATE.md` - Template for creating new commands
- `tdd-cycle.md` - TDD cycle command implementation
- `create-test.md` - Test creation command
- `contexts/` - Shared context files for inter-command communication
- `../CLAUDE.md` - Main project instructions and guidelines

## Additional Resources

- Project documentation: `/docs/`
- Architecture details: `/docs/architecture/`
- Security guidelines: `/docs/security/`
- Performance targets: `/benchmarks/README.md`
- Testing strategy: `/tests/README.md`

---

For questions or to report issues with commands, please check the project's issue tracker or create a new command to address specific needs.