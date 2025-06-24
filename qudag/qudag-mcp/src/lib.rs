//! # QuDAG MCP (Model Context Protocol) Server
//!
//! This crate provides a Model Context Protocol (MCP) server implementation for QuDAG
//! distributed systems. It enables AI models to securely interact with QuDAG's
//! distributed ledger, vault, and network capabilities through the standardized MCP protocol.
//!
//! ## Features
//!
//! - **Multi-transport support**: STDIO, HTTP, and WebSocket transports
//! - **Vault integration**: Secure access to QuDAG vault operations
//! - **Resource management**: Expose QuDAG resources through MCP
//! - **Tool integration**: Provide QuDAG tools to AI models
//! - **Event streaming**: Real-time updates from QuDAG network
//! - **Authentication**: Secure access control and permissions
//!
//! ## Quick Start
//!
//! ```rust
//! use qudag_mcp::{McpServer, Config, Transport};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::builder()
//!         .transport(Transport::Http { port: 8080 })
//!         .vault_path("/path/to/vault")
//!         .build()?;
//!     
//!     let server = McpServer::new(config).await?;
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(missing_docs)] // TODO: Re-enable after adding docs
#![warn(clippy::all)]

// Re-export important types from dependencies
pub use qudag_crypto as crypto;
pub use qudag_dag as dag;
pub use qudag_network as network;
pub use qudag_vault_core as vault;

// Public modules
pub mod auth;
pub mod config;
pub mod defaults;
pub mod error;
pub mod events;
pub mod server;
pub mod transport;
pub mod types;

// Internal modules
mod protocol;
mod resources;
mod tools;

// Public re-exports
pub use config::{
    AuditConfig, AuthConfig, BackupConfig, ConfigBuilder, McpConfig, RateLimitConfig,
    SecurityConfig, StorageConfig,
};
pub use error::{Error, Result};
pub use server::{QuDAGMCPServer, ServerConfig, ServerState, ServerStats};
pub use transport::TransportConfig;
pub use types::*;

/// Current version of the QuDAG MCP library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocol version supported by this implementation
pub const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

/// Convenience function to create a new MCP server with default configuration
pub async fn create_server() -> Result<QuDAGMCPServer> {
    QuDAGMCPServer::new(ServerConfig::default()).await
}

/// Convenience function to create a new MCP server with custom configuration
pub async fn create_server_with_config(config: ServerConfig) -> Result<QuDAGMCPServer> {
    QuDAGMCPServer::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_protocol_version() {
        assert_eq!(MCP_PROTOCOL_VERSION, "2025-03-26");
    }

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        // Basic smoke test - actual validation happens in config module
        assert!(matches!(
            config.transport,
            crate::transport::TransportConfig::Stdio
        ));
    }
}
