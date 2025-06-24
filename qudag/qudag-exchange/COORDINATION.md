# QuDAG Exchange - Agent Coordination Document

## Project Structure Overview

### Current Status
- **Branch**: qudag-exchange
- **Root Directory**: /workspaces/QuDAG/qudag-exchange/
- **Workspace Type**: Multi-crate Rust workspace (being established)

### Existing Components
1. **Main Package**: qudag-exchange (main package with existing structure)
2. **Sub-crates in crates/ directory**:
   - crates/core - Core exchange logic
   - crates/cli - Command-line interface  
   - crates/wasm - WASM bindings
   - crates/api - API server (to be implemented)
3. **New standalone crates created**:
   - qudag-exchange-core - Enhanced core library
   - qudag-exchange-cli - Enhanced CLI
   - qudag-exchange-wasm - WASM module
   - qudag-exchange-server - HTTP API server
   - qudag-exchange-sim - Simulation tools

### rUv Token Specification
- **Full Name**: Resource Utilization Voucher
- **Symbol**: rUv
- **Purpose**: Quantum-secure computational resource exchange credits
- **Initial Balance**: 1000 rUv per new account
- **Minimum Transaction**: 1 rUv
- **Basic Transaction Fee**: 1 rUv

### Core Dependencies
- qudag-crypto: ../core/crypto (ML-DSA, ML-KEM, HQC)
- qudag-dag: ../core/dag (QR-Avalanche consensus)
- qudag-vault: ../core/vault (Secure key management)
- qudag-network: ../core/network (P2P networking)

### Development Approach
- **Methodology**: Test-Driven Development (TDD)
- **Safety**: No unsafe code allowed
- **Security**: Quantum-resistant cryptography throughout
- **Concurrency**: Rust's fearless concurrency with DashMap, parking_lot

### Module Organization

#### Core Library (qudag-exchange-core)
- `ledger.rs`: rUv token accounting and balance management
- `transaction.rs`: Transaction structures and verification
- `metering.rs`: Resource usage tracking and cost calculation
- `consensus.rs`: Integration with QuDAG DAG consensus
- `crypto.rs`: Cryptographic operations and vault integration
- `identity.rs`: User identity and key management
- `types.rs`: Core types and constants
- `error.rs`: Error handling
- `utils.rs`: Utility functions

#### CLI (qudag-exchange-cli)
- Commands: create-account, balance, transfer, node, network
- Configuration file support
- Interactive vault unlocking

#### WASM Module (qudag-exchange-wasm)
- Browser-compatible builds
- No filesystem dependencies
- Memory-only storage in WASM mode

#### API Server (qudag-exchange-server)
- RESTful endpoints
- JWT authentication
- WebSocket support for real-time updates

### Agent Task Assignments

1. **Test Agent**: Write comprehensive tests for all modules
2. **Core Implementation Agent**: Implement ledger, transaction, and consensus logic
3. **Interface Agent**: Build CLI commands and API endpoints
4. **Security Agent**: Audit crypto operations and ensure quantum resistance
5. **Optimization Agent**: Profile and optimize performance bottlenecks
6. **Documentation Agent**: Generate API docs and user guides
7. **Verification Agent**: Formal verification of consensus properties
8. **Integration Agent**: Ensure all modules work together
9. **DevOps Agent**: Set up CI/CD and testing infrastructure

### Current Tasks (Updated by Coordinator)
1. ✅ Set up workspace structure
2. ✅ Create core library scaffold
3. ⏳ Implement TDD tests for ledger
4. ⏳ Implement resource metering
5. ⏳ Integrate QuDAG consensus
6. ⏳ Build CLI interface
7. ⏳ Create WASM bindings
8. ⏳ Develop API server

### Communication Protocol
- All agents should read this file for coordination
- Update progress in respective module directories
- Use TODO comments for handoff points
- Commit frequently to qudag-exchange branch

### Testing Strategy
1. Unit tests in each module
2. Integration tests in tests/ directory
3. Property-based testing with proptest
4. Fuzzing for security-critical components
5. Timing attack resistance tests
6. Multi-node consensus testing

### Security Considerations
- All cryptographic operations through qudag-crypto
- Keys stored only in QuDAG Vault
- Constant-time operations for sensitive code
- No secret data in logs or error messages
- Zero-knowledge proofs for transaction privacy

---
Last Updated: 2024-12-20 by Coordinator Agent