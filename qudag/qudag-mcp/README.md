# QuDAG MCP Server

[![Crates.io](https://img.shields.io/crates/v/qudag-mcp.svg)](https://crates.io/crates/qudag-mcp)
[![Documentation](https://docs.rs/qudag-mcp/badge.svg)](https://docs.rs/qudag-mcp)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/qudag-mcp.svg)](#license)

**QuDAG MCP Server** is a [Model Context Protocol (MCP)](https://spec.modelcontextprotocol.io/) server implementation that provides secure, quantum-resistant access to QuDAG's distributed system capabilities.

## 🚀 Features

- **🔒 Quantum-Resistant Security**: Built on post-quantum cryptographic algorithms (ML-KEM, ML-DSA, HQC)
- **📊 DAG Operations**: Full access to QuDAG's Directed Acyclic Graph consensus and storage
- **🔐 Secure Vault Integration**: Encrypted secret management with DAG-based storage
- **🌐 Network Management**: P2P networking, dark addressing, and onion routing capabilities
- **🔧 Comprehensive Tools**: Rich set of MCP tools for all QuDAG operations
- **📦 Resource Access**: Structured access to system state and metrics
- **⚡ High Performance**: Optimized for production workloads with comprehensive benchmarks
- **🔄 Real-time Updates**: Live synchronization with WebSocket and notification support

## 📋 Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Tools & Resources](#tools--resources)
- [Transport Options](#transport-options)
- [Examples](#examples)
- [Configuration](#configuration)
- [Security](#security)
- [Performance](#performance)
- [Contributing](#contributing)
- [License](#license)

## 🛠️ Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qudag-mcp = "1.0.0"
```

Or install via cargo:

```bash
cargo add qudag-mcp
```

## 🚀 Quick Start

### Basic Server

```rust
use qudag_mcp::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server configuration
    let config = ServerConfig::new()
        .with_server_info("My QuDAG MCP Server", "1.0.0")
        .with_transport(transport::TransportFactory::stdio());

    // Create and run the server
    let mut server = QuDAGMCPServer::new(config).await?;
    server.run().await?;
    
    Ok(())
}
```

### Basic Client

```rust
use qudag_mcp::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client configuration
    let config = ClientConfig::new()
        .with_client_info("My QuDAG MCP Client", "1.0.0")
        .with_transport(transport::TransportFactory::stdio())
        .with_timeout(Duration::from_secs(30));

    // Create and connect the client
    let mut client = QuDAGMCPClient::new(config).await?;
    client.connect().await?;

    // List available tools
    let tools = client.list_tools().await?;
    println!("Available tools: {}", tools.len());

    // Execute a tool
    let result = client.call_tool("dag_get_tips", None).await?;
    println!("DAG tips: {:?}", result);

    Ok(())
}
```

## 🏗️ Architecture

QuDAG MCP Server exposes QuDAG's capabilities through the standardized MCP protocol:

```
┌─────────────────────────────────────┐
│           MCP Clients               │
│     (Claude, IDEs, Applications)    │
├─────────────────────────────────────┤
│         MCP Protocol Layer          │
│      (JSON-RPC 2.0 over stdio/     │
│        HTTP/WebSocket)              │
├─────────────────────────────────────┤
│        QuDAG MCP Server             │
│                                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐│
│  │   DAG   │ │ Crypto  │ │ Network ││
│  │  Tools  │ │  Tools  │ │  Tools  ││
│  └─────────┘ └─────────┘ └─────────┘│
│  ┌─────────┐ ┌─────────┐ ┌─────────┐│
│  │   DAG   │ │ Crypto  │ │ Vault   ││
│  │Resources│ │Resources│ │Resources││
│  └─────────┘ └─────────┘ └─────────┘│
├─────────────────────────────────────┤
│         QuDAG Core System           │
│    (Consensus, Storage, Network)    │
└─────────────────────────────────────┘
```

## 🔧 Tools & Resources

### Available Tools

**DAG Operations:**
- `dag_add_vertex` - Add new vertex to the DAG
- `dag_get_vertex` - Retrieve vertex information
- `dag_get_tips` - Get current DAG tips
- `dag_get_order` - Get total order of vertices
- `dag_get_confidence` - Get vertex confidence level

**Cryptographic Operations:**
- `crypto_generate_keypair` - Generate quantum-resistant key pairs
- `crypto_sign` - Digital signature creation
- `crypto_verify` - Signature verification
- `crypto_encrypt` - Data encryption with ML-KEM
- `crypto_decrypt` - Data decryption

**Network Operations:**
- `network_connect_peer` - Connect to network peers
- `network_register_dark` - Register dark addresses
- `network_resolve` - Resolve network addresses
- `network_create_shadow` - Create temporary shadow addresses

**Vault Operations:**
- `vault_add_secret` - Store encrypted secrets
- `vault_get_secret` - Retrieve secrets
- `vault_list_secrets` - List available secrets
- `vault_generate_password` - Generate secure passwords

### Available Resources

**DAG Resources:**
- `dag://vertices/all` - All DAG vertices
- `dag://tips/current` - Current tip vertices
- `dag://consensus/status` - Consensus status
- `dag://order/global` - Global vertex ordering
- `dag://stats/summary` - DAG statistics

**Crypto Resources:**
- `crypto://algorithms/supported` - Supported algorithms
- `crypto://keys/public` - Public key information
- `crypto://stats/performance` - Performance metrics

**Vault Resources:**
- `vault://entries/count` - Entry count statistics
- `vault://categories/list` - Available categories
- `vault://stats/usage` - Usage statistics
- `vault://health/status` - System health

**Network Resources:**
- `network://peers/connected` - Connected peers
- `network://stats/traffic` - Network traffic
- `network://routes/active` - Active routes
- `network://dark/addresses` - Dark addresses

## 🚀 Transport Options

QuDAG MCP Server supports multiple transport mechanisms:

### Standard I/O (stdio)
```rust
let config = ServerConfig::new()
    .with_transport(transport::TransportFactory::stdio());
```

### HTTP Transport
```rust
let config = ServerConfig::new()
    .with_transport(transport::TransportFactory::http("http://localhost:8080"));
```

### WebSocket Transport
```rust
let config = ServerConfig::new()
    .with_transport(transport::TransportFactory::websocket("ws://localhost:8080/mcp"));
```

## 📖 Examples

### Running Examples

```bash
# Basic server example
cargo run --example basic_server

# Vault integration example
cargo run --example with_vault
```

### DAG Operations Example

```rust
use qudag_mcp::*;

// Add a new vertex to the DAG
let args = serde_json::json!({
    "id": "vertex_123",
    "payload": "Important data",
    "parents": ["parent_vertex_1", "parent_vertex_2"]
});

let result = client.call_tool("dag_add_vertex", Some(args)).await?;
```

### Crypto Operations Example

```rust
// Generate quantum-resistant key pair
let args = serde_json::json!({
    "algorithm": "ml-kem",
    "security_level": 3
});

let keypair = client.call_tool("crypto_generate_keypair", Some(args)).await?;

// Sign data
let sign_args = serde_json::json!({
    "data": base64::encode("Hello, QuDAG!"),
    "private_key": keypair_result["private_key"]
});

let signature = client.call_tool("crypto_sign", Some(sign_args)).await?;
```

### Resource Access Example

```rust
// Read DAG statistics
let dag_stats = client.read_resource("dag://stats/summary").await?;

// Read crypto algorithms
let algorithms = client.read_resource("crypto://algorithms/supported").await?;

// Read vault health
let vault_health = client.read_resource("vault://health/status").await?;
```

## ⚙️ Configuration

### Server Configuration

```rust
let config = ServerConfig::new()
    .with_server_info("Production QuDAG Server", "1.0.0")
    .with_transport(transport::TransportFactory::websocket("ws://0.0.0.0:8080/mcp"))
    .with_log_level("info");
```

### Client Configuration

```rust
let config = ClientConfig::new()
    .with_client_info("Production Client", "1.0.0")
    .with_transport(transport::TransportFactory::websocket("ws://server:8080/mcp"))
    .with_timeout(Duration::from_secs(60))
    .with_capability("authentication", serde_json::json!({
        "required": true,
        "methods": ["oauth2", "api_key"]
    }));
```

### Environment Variables

```bash
# Server configuration
QUDAG_MCP_LOG_LEVEL=info
QUDAG_MCP_SERVER_PORT=8080
QUDAG_MCP_TRANSPORT=websocket

# Client configuration  
QUDAG_MCP_CLIENT_TIMEOUT=30
QUDAG_MCP_SERVER_URL=ws://localhost:8080/mcp
```

## 🔒 Security

### Quantum-Resistant Cryptography

QuDAG MCP Server uses post-quantum cryptographic algorithms:

- **ML-KEM (Module-Lattice-Based Key Encapsulation)**: For secure key exchange
- **ML-DSA (Module-Lattice-Based Digital Signature Algorithm)**: For digital signatures
- **HQC (Hamming Quasi-Cyclic)**: For additional encryption capabilities

### Authentication & Authorization

```rust
// Configure authentication
let mut capabilities = HashMap::new();
capabilities.insert("authentication".to_string(), serde_json::json!({
    "required": true,
    "methods": ["oauth2", "vault_token"],
    "secure_channels_only": true
}));

let config = ClientConfig::new()
    .with_capability("authentication", capabilities["authentication"].clone());
```

### Secure Transport

Always use secure transports in production:

```rust
// Use WSS for WebSocket
let config = ServerConfig::new()
    .with_transport(transport::TransportFactory::websocket("wss://secure-server:443/mcp"));

// Use HTTPS for HTTP transport
let config = ServerConfig::new()
    .with_transport(transport::TransportFactory::http("https://secure-server:443"));
```

## ⚡ Performance

### Benchmarks

Run performance benchmarks:

```bash
cargo bench
```

### Optimization Tips

1. **Connection Pooling**: Reuse connections for multiple operations
2. **Batch Operations**: Group multiple tool calls when possible
3. **Resource Caching**: Cache frequently accessed resources
4. **Compression**: Enable compression for large payloads

### Performance Metrics

Typical performance characteristics:

- **Tool Execution**: ~1-5ms per operation
- **Resource Access**: ~0.5-2ms per read
- **Throughput**: 1000+ operations/second
- **Concurrent Connections**: 100+ simultaneous clients

## 🧪 Testing

Run the complete test suite:

```bash
# Unit and integration tests
cargo test

# Protocol compliance tests
cargo test --test protocol_tests

# Security tests
cargo test --test auth_tests

# Performance benchmarks
cargo bench
```

### Test Coverage

The library includes comprehensive tests:

- ✅ **Integration Tests**: End-to-end MCP protocol compliance
- ✅ **Protocol Tests**: Message handling and transport layers
- ✅ **Security Tests**: Authentication and cryptographic operations
- ✅ **Performance Tests**: Benchmarks and stress tests
- ✅ **Error Handling**: Comprehensive error scenarios

## 🏗️ Development

### Prerequisites

- Rust 1.70+ (MSRV)
- Tokio runtime
- OpenSSL (for cryptographic operations)

### Building from Source

```bash
git clone https://github.com/qudag/qudag.git
cd qudag/qudag-mcp
cargo build --release
```

### Running Tests

```bash
# All tests
cargo test

# Specific test suites
cargo test --test integration_tests
cargo test --test protocol_tests
cargo test --test auth_tests

# With logging
RUST_LOG=debug cargo test
```

## 📚 Documentation

- [API Documentation](https://docs.rs/qudag-mcp)
- [MCP Protocol Specification](https://spec.modelcontextprotocol.io/)
- [QuDAG System Documentation](../README.md)
- [Security Guide](./docs/security.md)
- [Performance Guide](./docs/performance.md)

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](../CONTRIBUTING.md) for details on:

- Code of Conduct
- Development Process
- Pull Request Process
- Issue Reporting

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## 🐛 Troubleshooting

### Common Issues

**Connection Failures:**
```bash
# Check if server is running
netstat -tlnp | grep 8080

# Test with telnet
telnet localhost 8080
```

**Authentication Errors:**
```rust
// Verify client capabilities
let capabilities = client.server_capabilities().await?;
println!("Server auth: {:?}", capabilities.experimental);
```

**Performance Issues:**
```bash
# Enable debug logging
RUST_LOG=qudag_mcp=debug cargo run

# Check system resources
htop
iostat -x 1
```

## 📄 License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([LICENSE-APACHE](LICENSE-APACHE))
- [MIT License](https://opensource.org/licenses/MIT) ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🔗 Links

- [QuDAG Main Repository](https://github.com/qudag/qudag)
- [Model Context Protocol](https://spec.modelcontextprotocol.io/)
- [Crates.io Package](https://crates.io/crates/qudag-mcp)
- [Documentation](https://docs.rs/qudag-mcp)
- [Issue Tracker](https://github.com/qudag/qudag/issues)

---

<div align="center">

**Built with ❤️ for the quantum-resistant future**

[🌟 Star us on GitHub](https://github.com/qudag/qudag) | [📖 Read the Docs](https://docs.rs/qudag-mcp) | [💬 Join Discussions](https://github.com/qudag/qudag/discussions)

</div>